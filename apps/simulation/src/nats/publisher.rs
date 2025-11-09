//! NATS publisher worker thread
//!
//! Manages the background thread that publishes simulation frames to NATS,
//! including connection management and automatic reconnection.

use super::frame::SimulationFrame;
use crossbeam_channel::Receiver;
use log::{error, info, warn};
use std::thread;
use std::time::Duration;

const NATS_SUBJECT: &str = "speciate.crits.transform";
const RECONNECT_DELAY_MS: u64 = 1000;
const MAX_RECONNECT_DELAY_MS: u64 = 5000;

/// Spawn a dedicated thread for NATS publishing
///
/// This function creates a separate OS thread with its own Tokio runtime
/// to handle NATS publishing asynchronously without blocking the simulation.
///
/// # Arguments
/// * `rx` - Receiver for SimulationFrame messages from the main simulation loop
/// * `nats_url` - NATS server URL (e.g., "nats://nats:4222")
///
/// # Returns
/// A JoinHandle for the publisher thread
pub fn spawn_nats_publisher(
    rx: Receiver<SimulationFrame>,
    nats_url: String,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        info!("[NATS] Publisher thread starting...");

        // Create a single-threaded Tokio runtime for this thread
        let rt = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(rt) => rt,
            Err(e) => {
                error!("[NATS] Failed to create Tokio runtime: {}", e);
                return;
            }
        };

        // Run the async publisher loop
        rt.block_on(async move {
            publisher_loop(rx, nats_url).await;
        });

        info!("[NATS] Publisher thread stopped");
    })
}

/// Main publisher loop with reconnection logic
async fn publisher_loop(rx: Receiver<SimulationFrame>, nats_url: String) {
    let mut reconnect_delay = RECONNECT_DELAY_MS;
    let mut consecutive_errors = 0;

    loop {
        info!("[NATS] Connecting to {}...", nats_url);

        match async_nats::connect(&nats_url).await {
            Ok(client) => {
                info!("[NATS] Connected successfully");
                consecutive_errors = 0;
                reconnect_delay = RECONNECT_DELAY_MS;

                // Run the publishing loop until it fails
                if let Err(e) = publish_frames(&client, &rx).await {
                    error!("[NATS] Publishing error: {}", e);
                    consecutive_errors += 1;
                }
            }
            Err(e) => {
                warn!("[NATS] Connection failed: {}", e);
                consecutive_errors += 1;
            }
        }

        // Circuit breaker: after 10 consecutive failures, pause for longer
        if consecutive_errors >= 10 {
            warn!(
                "[NATS] Too many consecutive errors ({}), pausing reconnection for 30s",
                consecutive_errors
            );
            tokio::time::sleep(Duration::from_secs(30)).await;
            consecutive_errors = 0;
            continue;
        }

        // Exponential backoff for reconnection
        info!("[NATS] Reconnecting in {}ms...", reconnect_delay);
        tokio::time::sleep(Duration::from_millis(reconnect_delay)).await;

        reconnect_delay = (reconnect_delay * 2).min(MAX_RECONNECT_DELAY_MS);
    }
}

/// Publish frames from the channel to NATS
async fn publish_frames(
    client: &async_nats::Client,
    rx: &Receiver<SimulationFrame>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut frames_published = 0u64;
    let mut frames_dropped = 0u64;

    // Pre-allocate buffer for MessagePack binary serialization
    let mut buffer = Vec::with_capacity(64 * 1024); // 64KB

    loop {
        // Receive frame from channel (blocking)
        let frame = match rx.recv() {
            Ok(frame) => frame,
            Err(_) => {
                info!("[NATS] Channel closed, stopping publisher");
                return Ok(());
            }
        };

        // Serialize to MessagePack with field names (not arrays)
        // This ensures TypeScript can decode with named properties
        buffer.clear();
        let mut serializer = rmp_serde::Serializer::new(&mut buffer).with_struct_map();
        if let Err(e) = serde::Serialize::serialize(&frame, &mut serializer) {
            error!("[NATS] MessagePack serialization failed: {}", e);
            frames_dropped += 1;
            continue;
        }

        // Publish to NATS
        match client.publish(NATS_SUBJECT, buffer.clone().into()).await {
            Ok(_) => {
                frames_published += 1;

                // Flush to ensure message is sent to server
                if let Err(e) = client.flush().await {
                    warn!("[NATS] Flush failed: {}", e);
                    frames_dropped += 1;
                    return Err(Box::new(e));
                }

                // Log stats every 1000 frames
                if frames_published % 1000 == 0 {
                    info!(
                        "[NATS] Published {} frames, dropped {}",
                        frames_published, frames_dropped
                    );
                }
            }
            Err(e) => {
                warn!("[NATS] Publish failed: {}", e);
                frames_dropped += 1;
                return Err(Box::new(e));
            }
        }
    }
}
