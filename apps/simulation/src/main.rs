//! Speciate - Unified Simulation Server
//!
//! Server-authoritative simulation with WebSocket broadcasting

mod config;
mod game_loop;
mod network;
mod simulation;
mod spawner;
mod state;

use axum::{routing::get, Router};
use config::WorldConfig;
use game_loop::run_game_loop;
use log::info;
use network::{ws_handler, AppState};
use simulation::Simulation;
use spawner::spawn_initial_creatures;
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

    // Load configuration
    let config = WorldConfig::new();

    // Initialize simulation with ECS
    let mut simulation = Simulation::new();
    simulation.set_boundaries(config.world.width, config.world.height);

    // Spawn initial creatures
    spawn_initial_creatures(&mut simulation, &config.spawning);

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
    let bind_addr = config.ws_bind_address();
    let ws_url = config.ws_url();
    let health_url = config.health_url();

    tokio::spawn(async move {
        let listener = match tokio::net::TcpListener::bind(&bind_addr).await {
            Ok(l) => {
                info!("🌐 WebSocket server listening on {}", ws_url);
                info!("❤️  Health check available at {}\n", health_url);
                l
            }
            Err(e) => {
                eprintln!("❌ Failed to bind to {}: {}", bind_addr, e);
                eprintln!("   Make sure no other process is using the port");
                eprintln!("   Run: pkill -f 'cargo run' or kill the process using the port");
                return;
            }
        };

        if let Err(e) = axum::serve(listener, app).await {
            eprintln!("❌ Server error: {}", e);
        }
    });

    // Run the main game loop (never returns)
    run_game_loop(simulation, ws_state_clone, config.timing).await;
}
