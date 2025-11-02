//! Speciate - Hello World Simulation
//!
//! A simple demonstration of the Speciate simulation engine running a basic
//! ECS-based simulation with entities moving in 2D space.

use log::info;
use speciate::components::{Health, Position, Velocity};
use speciate::Simulation;
use std::time::Instant;

fn main() {
    // Initialize logging
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    info!("=== Speciate: Hello World Simulation ===");
    info!("Starting simulation engine...\n");

    // Create simulation
    let mut simulation = Simulation::new();

    // Spawn some demo entities
    info!("Spawning 5 demo entities...");
    for i in 0..5 {
        let x = (i as f32) * 10.0;
        let vx = 1.0 + (i as f32) * 0.5;
        let vy = -0.5 - (i as f32) * 0.2;

        let position = Position::new(x, 0.0);
        let velocity = Velocity::new(vx, vy);
        let health = Health::new(100.0);

        let id = simulation.spawn_entity(position, velocity, health);
        info!(
            "  Entity #{} spawned at ({:.1}, {:.1}) with velocity ({:.1}, {:.1})",
            id.0, x, 0.0, vx, vy
        );
    }

    info!("\nRunning simulation for 100 ticks (5 seconds at 20Hz)...\n");

    // Run simulation
    let start = Instant::now();

    for _ in 0..100 {
        simulation.update();

        // Log every 20 ticks (1 second)
        if simulation.tick() % 20 == 0 {
            let elapsed = start.elapsed().as_secs_f32();
            info!(
                "Tick: {:3} | Time: {:.2}s | Entities: {}",
                simulation.tick(),
                elapsed,
                simulation.entity_count()
            );

            // Print sample entity positions
            for (idx, entity) in simulation.get_entities().enumerate() {
                if idx < 2 {
                    info!(
                        "  Entity #{}: pos=({:7.2}, {:7.2}) vel=({:5.2}, {:5.2})",
                        entity.id.0, entity.position.x, entity.position.y, entity.velocity.vx,
                        entity.velocity.vy
                    );
                }
            }
        }
    }

    let elapsed = start.elapsed();
    info!("\n=== Simulation Complete ===");
    info!(
        "Final State: {} ticks executed in {:.3} seconds",
        simulation.tick(),
        elapsed.as_secs_f32()
    );
    info!("Active entities: {}", simulation.entity_count());
    info!("Simulation tick rate: 20 Hz (0.05s per tick)");
    info!("Average wall time per tick: {:.4} ms\n", {
        elapsed.as_secs_f64() / simulation.tick() as f64 * 1000.0
    });

    // Print final entity states
    info!("Final entity positions:");
    for entity in simulation.get_entities() {
        info!(
            "  Entity #{:2}: pos=({:8.2}, {:8.2})",
            entity.id.0, entity.position.x, entity.position.y
        );
    }
}
