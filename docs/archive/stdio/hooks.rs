
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
use std::thread;
use std::time::Duration;

pub struct StdioHooks {
    send_tx: Sender<Vec<u8>>,
}

impl Default for StdioHooks {
    fn default() -> Self {
        Self::new()
    }
}

impl StdioHooks {
    pub fn new() -> Self {
        let (send_tx, send_rx) = bounded::<Vec<u8>>(2);

        thread::Builder::new()
            .name("ipc-writer".to_string())
            .spawn(move || {
                let mut stdout = io::stdout().lock();

                while let Ok(buf) = send_rx.recv() {
                    let len = buf.len() as u32;
                    if let Err(e) = stdout
                        .write_all(&len.to_be_bytes())
                        .and_then(|_| stdout.write_all(&buf))
                        .and_then(|_| stdout.flush())
                    {
                        eprintln!("[ipc-writer] Write error: {}", e);
                        break;
                    }
                }
                eprintln!("[ipc-writer] Writer thread exiting");
            })
            .expect("Failed to spawn IPC writer thread");

        Self {
            send_tx,
        }
    }
}

impl RunnerHooks for StdioHooks {
    fn on_tick(&mut self, _tick: u64, _tick_elapsed: Duration, simulation: &mut Simulation) {
        match serialize_snapshot_frame(simulation) {
            Ok(buf) => {
                match self.send_tx.try_send(buf) {
                    Ok(()) => {},
                    Err(TrySendError::Full(_)) => {
                        eprintln!("[stdio] Frame dropped: writer thread falling behind");
                    },
                    Err(TrySendError::Disconnected(_)) => {
                        eprintln!("[stdio] ERROR: Writer thread disconnected");
                    },
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


        let snapshots_dir = std::path::Path::new("save-states");
        if let Err(e) = std::fs::create_dir_all(snapshots_dir) {
            eprintln!("[stdio] Failed to create save-states directory: {}", e);
            return;
        }


        let save_state = simulation.to_save_state();


        let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
        let timestamped_filename = format!("{}.msgpack", timestamp);
        let timestamped_path = snapshots_dir.join(&timestamped_filename);

        match save_state.save_to_file(&timestamped_path) {
            Ok(_) => {
                eprintln!(
                    "[stdio] Save state saved: {} ({} creatures)",
                    timestamped_path.display(),
                    save_state.metadata.creature_count
                );
            }
            Err(e) => {
                eprintln!("[stdio] Failed to save world state: {}", e);
            }
        }
    }
}

fn serialize_snapshot_frame(simulation: &mut Simulation) -> io::Result<Vec<u8>> {
    const PROTOCOL_VERSION: u8 = 1;

    let world = &mut simulation.world;

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

    let entity_count = creatures.len();

    let tick = world.resource::<PhysicsTick>().0;
    let tick_rate = world.resource::<ActualTickRate>().0;


    #[cfg(feature = "dev-tools")]
    let system_timings_us = {
        use crate::instrumentation::extract_ecs_metrics;
        let (archetype_count, entity_count_u64) = extract_ecs_metrics(world);
        let mut timings = world
            .resource::<crate::instrumentation::SystemTimings>()
            .snapshot();
        timings.archetype_count = archetype_count;
        timings.entity_count = entity_count_u64;
        timings
    };

    let state = GameState {
        protocol_version: PROTOCOL_VERSION,
        tick,
        tick_rate_hz: tick_rate,
        creatures,
        #[cfg(feature = "dev-tools")]
        entity_count,
        #[cfg(feature = "dev-tools")]
        system_timings_us,
        #[cfg(feature = "dev-tools")]
        hardware_metrics: world.resource::<crate::instrumentation::HardwareSnapshotResource>().0.clone(),
        #[cfg(feature = "dev-tools")]
        parallelization_metrics: {
            let mut para_metrics = world.resource_mut::<crate::instrumentation::ParallelizationMetrics>();
            Some(para_metrics.read())
        },
    };

    let mut buf = Vec::with_capacity(entity_count * 70 + 200);
    let mut serializer = rmp_serde::Serializer::new(&mut buf).with_struct_map();
    state
        .serialize(&mut serializer)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;


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
        let snapshots_dir = Path::new("save-states");
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

        // Wait for filesystem after cleanup
        std::thread::sleep(std::time::Duration::from_millis(100));

        let mut simulation = SimulationBuilder::new().build();
        let mut hooks = StdioHooks::new();

        hooks.on_shutdown(100, &mut simulation);

        // Wait for file write to complete (stdio hooks doesn't use background worker)
        std::thread::sleep(std::time::Duration::from_millis(500));

        let latest_path = Path::new("save-states/latest.msgpack");
        assert!(
            !latest_path.exists(),
            "Should NOT create latest.msgpack (timestamped-only approach)"
        );


        let snapshots_dir = Path::new("save-states");
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

        // Wait for filesystem after cleanup
        std::thread::sleep(std::time::Duration::from_millis(100));

        let mut simulation = SimulationBuilder::new().build();
        let mut hooks = StdioHooks::new();

        hooks.on_shutdown(100, &mut simulation);

        // Wait for file write to complete (stdio hooks doesn't use background worker)
        std::thread::sleep(std::time::Duration::from_millis(500));

        let latest_path = Path::new("save-states/latest.msgpack");
        assert!(
            !latest_path.exists(),
            "Should NOT create latest.msgpack"
        );


        let snapshots_dir = Path::new("save-states");
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
            !filename_str.starts_with("simulation_"),
            "Timestamped file should NOT have 'simulation_' prefix, got: {}",
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

        let snapshots_dir = Path::new("save-states");

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
            // Sleep AFTER each save to ensure next one has different timestamp
            std::thread::sleep(std::time::Duration::from_millis(1100));
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


        assert!(
            avg_per_tick < 5000,
            "Average tick overhead should be <5ms with background writer, got {}μs",
            avg_per_tick
        );
    }
}
