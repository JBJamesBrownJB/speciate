//! Main simulation game loop
//!
//! This module contains the core game loop logic that runs the simulation,
//! broadcasts state to clients, and manages timing.

use crate::config::TimingConfig;
use crate::network::AppState;
use crate::simulation::timing::TickTimer;
use crate::simulation::Simulation;
use log::{info, warn};
use serde::Serialize;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Game state message sent to clients
#[derive(Serialize)]
struct GameState {
    tick: u64,
    creatures: Vec<CreatureState>,
    server_time: u64,
}

/// Creature state for network serialization
#[derive(Serialize)]
struct CreatureState {
    id: u32,
    x: f32,
    y: f32,
    rotation: f32,
    width: f32,
    height: f32,
}

/// Run the main simulation loop
///
/// This function runs indefinitely, updating the simulation at the configured tick rate
/// and broadcasting state to connected WebSocket clients.
pub async fn run_game_loop(
    mut simulation: Simulation,
    ws_state: Arc<AppState>,
    timing_config: TimingConfig,
) -> ! {
    let mut tick: u64 = 0;
    let tick_duration = Duration::from_secs_f64(1.0 / timing_config.target_tick_rate as f64);
    let mut last_time = Instant::now();

    // Tick timing instrumentation
    let mut tick_timer = TickTimer::new(
        timing_config.timing_window_size,
        timing_config.timing_report_interval,
    );
    let mut last_log_time = Instant::now();

    info!(
        "⏱️  Starting simulation loop ({} Hz target, delta time based)...\n",
        timing_config.target_tick_rate
    );

    loop {
        let tick_start = Instant::now();

        // Calculate delta time
        let now = Instant::now();
        let delta = now.duration_since(last_time);
        let delta_time = delta.as_secs_f32();
        last_time = now;

        // Run simulation tick with actual elapsed time
        simulation.update(delta_time);

        // Prepare state for broadcasting
        let creature_data = simulation.get_creatures();
        let mut creatures = Vec::with_capacity(creature_data.len());

        for creature in creature_data {
            creatures.push(CreatureState {
                id: creature.id,
                x: creature.x,
                y: creature.y,
                rotation: creature.rotation,
                width: creature.width,
                height: creature.height,
            });
        }

        let state_msg = GameState {
            tick,
            creatures,
            server_time: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("Time went backwards")
                .as_millis() as u64,
        };

        // Broadcast state to connected clients
        match ws_state.broadcast(state_msg) {
            Ok(count) if count > 0 => {
                // Successfully sent to subscribers
            }
            Ok(_) => {
                // No subscribers (this is OK, just waiting for client connection)
            }
            Err(e) => {
                eprintln!("❌ Broadcast error: {}", e);
            }
        }

        tick += 1;

        // Record tick duration for performance monitoring
        let tick_elapsed = tick_start.elapsed();
        if tick_timer.record_tick(tick_elapsed) {
            info!("{}", tick_timer.format_stats());
        }

        // Log creature count periodically (based on real time, not tick count)
        if last_log_time.elapsed()
            >= Duration::from_secs(timing_config.creature_count_log_interval_secs)
        {
            let count = simulation.creature_count();
            info!("Tick {}: {} creatures active", tick, count);
            last_log_time = Instant::now();
        }

        // Sleep for remaining time to maintain target tick rate
        if tick_elapsed < tick_duration {
            tokio::time::sleep(tick_duration - tick_elapsed).await;
        } else {
            // Tick took longer than target - log warning
            warn!(
                "Simulation tick took {:.2}ms (target: {:.2}ms)",
                tick_elapsed.as_secs_f64() * 1000.0,
                tick_duration.as_secs_f64() * 1000.0
            );
        }
    }
}
