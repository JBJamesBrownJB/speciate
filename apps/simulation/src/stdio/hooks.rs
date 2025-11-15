//! Stdio hooks for Electron IPC integration
//!
//! Writes MessagePack-encoded game state frames to stdout for consumption by Electron main process.

use crate::runner::RunnerHooks;
use crate::simulation::core::timing::TickTimer;
use crate::simulation::core::components::{ActualTickRate, PhysicsTick, BodySize};
use crate::simulation::components::*;
use crate::simulation::creatures::components::*;
use crate::ipc::snapshot_queue::{CreatureSnapshot, GameState};
use crate::Simulation;
use serde::Serialize;
use std::io::{self, Write};
use std::time::Duration;

/// Hook implementation that writes MessagePack frames to stdout
///
/// Frame format: [4-byte length (big-endian u32)][MessagePack payload]
pub struct StdioHooks;

impl Default for StdioHooks {
    fn default() -> Self {
        Self::new()
    }
}

impl StdioHooks {
    pub fn new() -> Self {
        Self
    }
}

impl RunnerHooks for StdioHooks {
    fn on_tick(&mut self, _tick: u64, _tick_elapsed: Duration, simulation: &mut Simulation) {
        // Create snapshot and write to stdout
        if let Err(e) = write_snapshot_frame(simulation) {
            eprintln!("[stdio] Failed to write snapshot: {}", e);
        }
    }

    fn on_stats_interval(
        &mut self,
        _tick: u64,
        _simulation: &Simulation,
        _tick_timer: &TickTimer,
        _tick_duration: Duration,
    ) {
        // No-op for stdio mode (stats go to stderr if needed)
    }

    fn on_shutdown(&mut self, tick: u64, simulation: &mut Simulation) {
        eprintln!("[stdio] Simulation stopped at tick {}", tick);

        // Create snapshots directory if it doesn't exist
        let snapshots_dir = std::path::Path::new("snapshots");
        if let Err(e) = std::fs::create_dir_all(snapshots_dir) {
            eprintln!("[stdio] Failed to create snapshots directory: {}", e);
            return;
        }

        // Save shutdown snapshot to latest.msgpack (matches ConsoleHooks behavior)
        let snapshot = simulation.to_snapshot();
        let latest_path = std::path::Path::new("snapshots/latest.msgpack");

        match snapshot.save_to_file(latest_path) {
            Ok(_) => {
                eprintln!(
                    "[stdio] Shutdown snapshot saved: {} ({} creatures)",
                    latest_path.display(),
                    snapshot.metadata.creature_count
                );
            }
            Err(e) => {
                eprintln!("[stdio] Failed to save shutdown snapshot: {}", e);
            }
        }
    }
}

/// Write a MessagePack frame to stdout
fn write_snapshot_frame(simulation: &mut Simulation) -> io::Result<()> {
    // Access the Bevy world (pub(crate) field)
    let world = &mut simulation.world;

    // Query for all creatures with required components
    let mut query = world.query::<(
        &Position,
        &Velocity,
        &Rotation,
        &CreatureState,
        &BodySize,
        &CritId,
    )>();

    let mut creatures = Vec::new();
    for (pos, vel, rot, state, body_size, crit_id) in query.iter(world) {
        creatures.push(CreatureSnapshot {
            id: crit_id.0,
            x: pos.x,
            y: pos.y,
            vx: vel.vx,
            vy: vel.vy,
            rotation: rot.radians,
            width: body_size.length,
            height: body_size.length,
            behavior: format!("{:?}", state.behavior),
            energy: Some(state.energy),
            age: state.age,
        });
    }

    // Get resources
    let tick = world.resource::<PhysicsTick>().0;
    let tick_rate = world.resource::<ActualTickRate>().0;

    // Build game state
    let state = GameState {
        tick,
        tick_rate_hz: tick_rate,
        creatures,
    };

    // Serialize with struct map format
    let mut buf = Vec::new();
    let mut serializer = rmp_serde::Serializer::new(&mut buf).with_struct_map();
    state
        .serialize(&mut serializer)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    // Write length prefix (4-byte big-endian u32)
    let len = buf.len() as u32;
    let mut stdout = io::stdout().lock();
    stdout.write_all(&len.to_be_bytes())?;

    // Write MessagePack payload
    stdout.write_all(&buf)?;
    stdout.flush()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::SimulationBuilder;
    use std::fs;
    use std::path::Path;

    /// Helper to clean up test snapshots
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
        // Just verify it can be created
        drop(hooks);
    }

    #[test]
    fn test_stdio_hooks_shutdown_creates_latest_msgpack() {
        cleanup_test_snapshots();

        // Create simulation
        let mut simulation = SimulationBuilder::new().build();
        let mut hooks = StdioHooks::new();

        // Call shutdown hook
        hooks.on_shutdown(100, &mut simulation);

        // Verify latest.msgpack was created
        let latest_path = Path::new("snapshots/latest.msgpack");
        assert!(
            latest_path.exists(),
            "StdioHooks::on_shutdown() should create snapshots/latest.msgpack"
        );

        // Verify it's a valid snapshot file
        let metadata = fs::metadata(latest_path).unwrap();
        assert!(
            metadata.len() > 0,
            "Snapshot file should not be empty"
        );

        cleanup_test_snapshots();
    }
}
