//! Speciate - Unified Simulation Server
//!
//! Server-authoritative simulation with WebSocket broadcasting

mod simulation;
mod network;
mod state;

use axum::{routing::get, Router};
use log::info;
use simulation::{Health, Position, Velocity, Simulation};
use network::{ws_handler, AppState, EntityState, SimulationStateMessage};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

async fn health_check() -> &'static str {
    "OK"
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    info!("🦀 Speciate Simulation Server Starting...");
    info!("Starting server...\n");

    // Initialize simulation
    let mut simulation = Simulation::new();

    // Spawn a test entity
    info!("Spawning demo entity...");
    simulation.spawn_entity(
        Position::new(0.0, 0.0),
        Velocity::new(0.1, 0.05),
        Health::new(100.0),
    );

    // Initialize WebSocket state
    let ws_state = Arc::new(AppState::new());
    let ws_state_clone = ws_state.clone();

    // Create HTTP router with CORS and WebSocket support
    let app = Router::new()
        .route("/health", get(health_check))
        .route(
            "/ws",
            get(|ws, state| async move { ws_handler(ws, state).await }),
        )
        .layer(CorsLayer::permissive())
        .with_state(ws_state);

    // Spawn WebSocket server task
    tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
            .await
            .expect("Failed to bind to port 8080");
        info!("🌐 WebSocket server listening on ws://localhost:8080/ws");
        info!("❤️  Health check available at http://localhost:8080/health\n");
        axum::serve(listener, app)
            .await
            .expect("Server failed");
    });

    // Main simulation loop
    let mut tick: u64 = 0;
    let tick_duration = std::time::Duration::from_millis(100); // 10 TPS

    info!("⏱️  Starting simulation loop (10 TPS)...\n");

    loop {
        // Run simulation tick
        simulation.update();

        // Broadcast state to connected clients
        if let Some(entity) = simulation.get_entities().next() {
            let state_msg = SimulationStateMessage {
                tick,
                entity: EntityState {
                    x: entity.position.x,
                    y: entity.position.y,
                    z: 0.0, // 2D for now
                },
                server_time: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_millis() as u64,
            };

            // Broadcast (ignore error if no clients connected)
            let _ = ws_state_clone.broadcast(state_msg);
        }

        tick += 1;
        if tick % 10 == 0 {
            info!(
                "Tick {}: Position ({:.2}, {:.2})",
                tick,
                simulation.get_entities().next().unwrap().position.x,
                simulation.get_entities().next().unwrap().position.y
            );
        }

        tokio::time::sleep(tick_duration).await;
    }
}
