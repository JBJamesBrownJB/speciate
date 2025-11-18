
use crate::runner::RunnerHooks;
use crate::simulation::core::timing::TickTimer;
use crate::simulation::core::components::{ActualTickRate, PhysicsTick, BodySize};
use crate::simulation::components::*;
use crate::simulation::creatures::components::*;
use crate::ipc::snapshot_queue::{CreatureSnapshot, GameState};
use crate::Simulation;
use crossbeam_channel::{bounded, Sender, TrySendError};
use serde::Serialize;
use std::io::{self, Write};
#[cfg(feature = "dev-tools")]
use std::sync::atomic::AtomicU64;
#[cfg(feature = "dev-tools")]
use std::sync::atomic::Ordering;
#[cfg(feature = "dev-tools")]
use std::sync::Arc;
use std::thread;
use std::time::Duration;
#[cfg(feature = "dev-tools")]
use std::time::Instant;

pub struct StdioHooks {
    send_tx: Sender<Vec<u8>>,
    #[cfg(feature = "dev-tools")]
    writer_thread_timing: Arc<AtomicU64>,
}

impl Default for StdioHooks {
    fn default() -> Self {
        Self::new()
    }
}

impl StdioHooks {
    pub fn new() -> Self {
        let (send_tx, send_rx) = bounded::<Vec<u8>>(2);
        #[cfg(feature = "dev-tools")]
        let writer_thread_timing = Arc::new(AtomicU64::new(0));
        #[cfg(feature = "dev-tools")]
        let writer_thread_timing_clone = writer_thread_timing.clone();

        thread::Builder::new()
            .name("ipc-writer".to_string())
            .spawn(move || {
                let mut stdout = io::stdout().lock();

                while let Ok(buf) = send_rx.recv() {
                    #[cfg(feature = "dev-tools")]
                    let write_start = Instant::now();

                    let len = buf.len() as u32;
                    if let Err(e) = stdout
                        .write_all(&len.to_be_bytes())
                        .and_then(|_| stdout.write_all(&buf))
                        .and_then(|_| stdout.flush())
                    {
                        eprintln!("[ipc-writer] Write error: {}", e);
                        break;
                    }

                    #[cfg(feature = "dev-tools")]
                    {
                        let write_elapsed_us = write_start.elapsed().as_micros() as u64;
                        writer_thread_timing_clone.store(write_elapsed_us, Ordering::Relaxed);
                    }
                }
                eprintln!("[ipc-writer] Writer thread exiting");
            })
            .expect("Failed to spawn IPC writer thread");

        Self {
            send_tx,
            #[cfg(feature = "dev-tools")]
            writer_thread_timing,
        }
    }
}

impl RunnerHooks for StdioHooks {
    fn on_tick(&mut self, _tick: u64, _tick_elapsed: Duration, simulation: &mut Simulation) {
        match serialize_snapshot_frame(simulation) {
            Ok(buf) => {
                #[cfg(feature = "dev-tools")]
                let write_start = Instant::now();

                #[cfg(feature = "dev-tools")]
                use std::sync::atomic::Ordering;

                match self.send_tx.try_send(buf) {
                    Ok(()) => {},
                    Err(TrySendError::Full(_)) => {
                        eprintln!("[stdio] Frame dropped: writer thread falling behind");
                        #[cfg(feature = "dev-tools")]
                        {
                            let timings = simulation.world.resource::<crate::instrumentation::SystemTimings>();
                            timings.ipc_frame_drops_total.fetch_add(1, Ordering::Relaxed);
                        }
                    },
                    Err(TrySendError::Disconnected(_)) => {
                        eprintln!("[stdio] ERROR: Writer thread disconnected");
                    },
                }

                #[cfg(feature = "dev-tools")]
                {
                    let write_elapsed_us = write_start.elapsed().as_micros() as u64;
                    let timings = simulation.world.resource::<crate::instrumentation::SystemTimings>();
                    timings.ipc_write_us.store(write_elapsed_us, Ordering::Relaxed);

                    let channel_len = self.send_tx.len();
                    let channel_capacity = self.send_tx.capacity().unwrap_or(2);
                    let utilization_pct = (channel_len * 100) / channel_capacity.max(1);
                    timings.ipc_channel_utilization_pct.store(utilization_pct as u64, Ordering::Relaxed);

                    let writer_thread_us = self.writer_thread_timing.load(Ordering::Relaxed);
                    timings.ipc_writer_thread_us.store(writer_thread_us, Ordering::Relaxed);
                }
            },
            Err(e) => {
                eprintln!("[stdio] Failed to serialize snapshot: {}", e);
            },
        }
    }

    fn on_stats_interval(
        &mut self,
        _tick: u64,
        _simulation: &Simulation,
        _tick_timer: &TickTimer,
        _tick_duration: Duration,
    ) {

    }

    fn on_shutdown(&mut self, tick: u64, simulation: &mut Simulation) {
        eprintln!("[stdio] Simulation stopped at tick {}", tick);


        let snapshots_dir = std::path::Path::new("snapshots");
        if let Err(e) = std::fs::create_dir_all(snapshots_dir) {
            eprintln!("[stdio] Failed to create snapshots directory: {}", e);
            return;
        }


        let snapshot = simulation.to_snapshot();


        let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
        let timestamped_filename = format!("simulation_{}.msgpack", timestamp);
        let timestamped_path = snapshots_dir.join(&timestamped_filename);

        match snapshot.save_to_file(&timestamped_path) {
            Ok(_) => {
                eprintln!(
                    "[stdio] Snapshot saved: {} ({} creatures)",
                    timestamped_path.display(),
                    snapshot.metadata.creature_count
                );
            }
            Err(e) => {
                eprintln!("[stdio] Failed to save snapshot: {}", e);
            }
        }
    }
}

fn serialize_snapshot_frame(simulation: &mut Simulation) -> io::Result<Vec<u8>> {
    const PROTOCOL_VERSION: u8 = 1;

    let world = &mut simulation.world;

    #[cfg(feature = "dev-tools")]
    let query_start = Instant::now();

    let mut query = world.query::<(
        &Position,
        &Velocity,
        &Rotation,
        &CreatureState,
        &BodySize,
        &CritId,
    )>();

    let creatures: Vec<CreatureSnapshot> = query.iter(world)
        .map(|(pos, _vel, rot, _state, body_size, crit_id)| {
            CreatureSnapshot {
                id: crit_id.0,
                x: pos.x,
                y: pos.y,
                rotation: rot.radians,
                size: body_size.length,
            }
        })
        .collect();

    #[cfg(feature = "dev-tools")]
    let query_elapsed_us = query_start.elapsed().as_micros() as u64;

    let entity_count = creatures.len();

    let tick = world.resource::<PhysicsTick>().0;
    let tick_rate = world.resource::<ActualTickRate>().0;

    #[cfg(feature = "dev-tools")]
    {
        use std::sync::atomic::Ordering;
        let timings = world.resource::<crate::instrumentation::SystemTimings>();
        timings.ipc_query_us.store(query_elapsed_us, Ordering::Relaxed);
    }

    #[cfg(feature = "dev-tools")]
    let system_timings_us = world
        .resource::<crate::instrumentation::SystemTimings>()
        .snapshot();

    let state = GameState {
        protocol_version: PROTOCOL_VERSION,
        tick,
        tick_rate_hz: tick_rate,
        creatures,
        #[cfg(feature = "dev-tools")]
        entity_count,
        #[cfg(feature = "dev-tools")]
        system_timings_us,
    };

    #[cfg(feature = "dev-tools")]
    let serialize_start = Instant::now();

    let mut buf = Vec::with_capacity(entity_count * 70 + 200);
    let mut serializer = rmp_serde::Serializer::new(&mut buf).with_struct_map();
    state
        .serialize(&mut serializer)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    #[cfg(feature = "dev-tools")]
    {
        let serialize_elapsed_us = serialize_start.elapsed().as_micros() as u64;
        use std::sync::atomic::Ordering;
        let timings = world.resource::<crate::instrumentation::SystemTimings>();
        timings.ipc_serialize_us.store(serialize_elapsed_us, Ordering::Relaxed);
    }

    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::SimulationBuilder;
    use crate::CritBuilder;
    use std::fs;
    use std::path::Path;
    use std::time::Instant;

    fn cleanup_test_snapshots() {
        let snapshots_dir = Path::new("snapshots");
        if snapshots_dir.exists() {
            if let Ok(entries) = fs::read_dir(snapshots_dir) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("msgpack") {
                        fs::remove_file(path).ok();
                    }
                }
            }
        }
    }

    #[test]
    fn test_stdio_hooks_creation() {
        let hooks = StdioHooks::new();

        drop(hooks);
    }

    #[test]
    fn test_stdio_hooks_shutdown_creates_only_timestamped_file() {
        cleanup_test_snapshots();


        let mut simulation = SimulationBuilder::new().build();
        let mut hooks = StdioHooks::new();


        hooks.on_shutdown(100, &mut simulation);


        let latest_path = Path::new("snapshots/latest.msgpack");
        assert!(
            !latest_path.exists(),
            "Should NOT create latest.msgpack (timestamped-only approach)"
        );


        let snapshots_dir = Path::new("snapshots");
        let timestamped_files: Vec<_> = fs::read_dir(snapshots_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                let path = e.path();
                path.is_file()
                    && path.extension().and_then(|s| s.to_str()) == Some("msgpack")
            })
            .collect();

        assert_eq!(
            timestamped_files.len(),
            1,
            "Should create exactly one timestamped snapshot file"
        );


        let timestamped_path = timestamped_files[0].path();
        let metadata = fs::metadata(&timestamped_path).unwrap();
        assert!(
            metadata.len() > 0,
            "Timestamped snapshot file should not be empty"
        );

        cleanup_test_snapshots();
    }

    #[test]
    fn test_stdio_hooks_shutdown_timestamped_file_format() {
        cleanup_test_snapshots();


        let mut simulation = SimulationBuilder::new().build();
        let mut hooks = StdioHooks::new();


        hooks.on_shutdown(100, &mut simulation);


        let latest_path = Path::new("snapshots/latest.msgpack");
        assert!(
            !latest_path.exists(),
            "Should NOT create latest.msgpack"
        );


        let snapshots_dir = Path::new("snapshots");
        let entries: Vec<_> = fs::read_dir(snapshots_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                let path = e.path();
                path.is_file()
                    && path.extension().and_then(|s| s.to_str()) == Some("msgpack")
            })
            .collect();

        assert_eq!(
            entries.len(),
            1,
            "Should create exactly one timestamped snapshot file (found {} files)",
            entries.len()
        );


        let timestamped_file = &entries[0];
        let filename = timestamped_file.file_name();
        let filename_str = filename.to_str().unwrap();

        assert!(
            filename_str.starts_with("simulation_"),
            "Timestamped file should start with 'simulation_', got: {}",
            filename_str
        );
        assert!(
            filename_str.ends_with(".msgpack"),
            "Timestamped file should end with '.msgpack', got: {}",
            filename_str
        );



        let dash_count = filename_str.matches('-').count();
        assert!(
            dash_count >= 4,
            "Timestamped file should have date-time format with at least 4 dashes, got: {}",
            filename_str
        );

        cleanup_test_snapshots();
    }

    #[test]
    fn test_multiple_shutdowns_create_multiple_timestamped_files() {
        cleanup_test_snapshots();

        // Ensure cleanup completed and wait for filesystem
        std::thread::sleep(std::time::Duration::from_millis(100));

        let snapshots_dir = Path::new("snapshots");

        // Record filenames at start to compare against (not just count)
        let initial_files: std::collections::HashSet<String> = fs::read_dir(snapshots_dir)
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .filter_map(|e| {
                        let path = e.path();
                        if path.is_file()
                            && path.extension().and_then(|s| s.to_str()) == Some("msgpack")
                        {
                            path.file_name()
                                .and_then(|s| s.to_str())
                                .map(|s| s.to_string())
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();


        for i in 0..2 {
            let mut simulation = SimulationBuilder::new().build();
            let mut hooks = StdioHooks::new();
            hooks.on_shutdown(100 + i, &mut simulation);

            // Sleep for >1 second to ensure different timestamps (format: %Y-%m-%d_%H-%M-%S)
            // The timestamp has 1-second granularity, so we need at least 1001ms between calls
            if i < 1 {
                std::thread::sleep(std::time::Duration::from_millis(1100));
            }
        }

        // Give filesystem time to flush
        std::thread::sleep(std::time::Duration::from_millis(200));

        // Count NEW files that weren't present initially
        let final_files: Vec<String> = fs::read_dir(snapshots_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter_map(|e| {
                let path = e.path();
                if path.is_file()
                    && path.extension().and_then(|s| s.to_str()) == Some("msgpack")
                {
                    path.file_name()
                        .and_then(|s| s.to_str())
                        .map(|s| s.to_string())
                } else {
                    None
                }
            })
            .filter(|filename| !initial_files.contains(filename))
            .collect();

        assert_eq!(
            final_files.len(),
            2,
            "Two shutdowns should create two new timestamped snapshot files. Created: {:?}",
            final_files
        );

        cleanup_test_snapshots();
    }

    #[test]
    fn test_background_writer_reduces_main_thread_overhead() {
        let mut simulation = SimulationBuilder::new().build();

        for i in 0..100 {
            let builder = CritBuilder::new()
                .at((i as f32) * 10.0, (i as f32) * 10.0)
                .with_all_capabilities();
            simulation.spawn_crit(builder);
        }

        let mut hooks = StdioHooks::new();

        let start = Instant::now();
        for _ in 0..10 {
            hooks.on_tick(1, Duration::from_millis(16), &mut simulation);
        }
        let elapsed = start.elapsed();

        let avg_per_tick = elapsed.as_micros() / 10;

        #[cfg(feature = "dev-tools")]
        {
            let timings = simulation.world.resource::<crate::instrumentation::SystemTimings>();
            let ipc_write_us = timings.ipc_write_us.load(std::sync::atomic::Ordering::Relaxed);

            assert!(
                ipc_write_us < 1000,
                "Background writer should reduce IPC write overhead to <1ms, got {}μs",
                ipc_write_us
            );
        }

        assert!(
            avg_per_tick < 5000,
            "Average tick overhead should be <5ms with background writer, got {}μs",
            avg_per_tick
        );
    }
}
