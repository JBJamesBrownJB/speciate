//! NATS subscriber for dev commands
//!
//! Manages the background thread that subscribes to dev command messages
//! from NATS and forwards them to the simulation.

use super::commands::DevCommand;
use bevy_ecs::system::Resource;
use crossbeam_channel::{bounded, Receiver, Sender};
use futures::StreamExt;  // For .next() on async_nats::Subscriber
use log::{error, info, warn};
use std::thread;
use std::time::Duration;

const DEV_COMMAND_SUBJECT: &str = "dev.sim.>";
const RECONNECT_DELAY_MS: u64 = 1000;
const MAX_RECONNECT_DELAY_MS: u64 = 5000;

/// Bevy ECS resource for receiving dev commands
///
/// This resource holds a channel receiver that allows the simulation
/// to receive dev commands from the NATS subscriber thread without blocking.
#[derive(Resource)]
pub struct DevCommandListener {
    receiver: Receiver<DevCommand>,
}

impl DevCommandListener {
    /// Create a new dev command listener with a dedicated thread
    ///
    /// # Arguments
    /// * `nats_url` - NATS server URL (e.g., "nats://localhost:4222")
    /// * `channel_capacity` - Bounded channel capacity (default: 16)
    ///
    /// # Returns
    /// A tuple of (DevCommandListener resource, thread JoinHandle)
    pub fn new(nats_url: String, channel_capacity: usize) -> (Self, thread::JoinHandle<()>) {
        let (tx, rx) = bounded(channel_capacity);
        let handle = spawn_dev_command_subscriber(tx, nats_url);

        (Self { receiver: rx }, handle)
    }

    /// Create a test listener with a sender for injecting commands (test helper)
    ///
    /// This constructor is only available for tests and returns both the listener
    /// and a sender, allowing tests to inject dev commands without NATS.
    #[cfg(any(test, feature = "test-helpers"))]
    pub fn new_for_test(channel_capacity: usize) -> (Self, Sender<DevCommand>) {
        let (tx, rx) = bounded(channel_capacity);
        (Self { receiver: rx }, tx)
    }

    /// Try to receive a dev command without blocking
    ///
    /// # Returns
    /// `Some(DevCommand)` if a command is available, `None` if empty
    pub fn try_recv(&self) -> Option<DevCommand> {
        self.receiver.try_recv().ok()
    }

    /// Receive all pending dev commands
    ///
    /// Drains the channel and returns all available commands.
    pub fn recv_all(&self) -> Vec<DevCommand> {
        let mut commands = Vec::new();
        while let Ok(cmd) = self.receiver.try_recv() {
            commands.push(cmd);
        }
        commands
    }
}

/// Spawn a dedicated thread for NATS dev command subscription
///
/// This function creates a separate OS thread with its own Tokio runtime
/// to handle NATS subscription asynchronously without blocking the simulation.
///
/// # Arguments
/// * `tx` - Sender for DevCommand messages to the main simulation loop
/// * `nats_url` - NATS server URL (e.g., "nats://localhost:4222")
///
/// # Returns
/// A JoinHandle for the subscriber thread
fn spawn_dev_command_subscriber(
    tx: Sender<DevCommand>,
    nats_url: String,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        info!("[DEV] Dev command subscriber thread starting...");

        // Create a single-threaded Tokio runtime for this thread
        let rt = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(rt) => rt,
            Err(e) => {
                error!("[DEV] Failed to create Tokio runtime: {}", e);
                return;
            }
        };

        // Run the async subscriber loop
        rt.block_on(async move {
            subscriber_loop(tx, nats_url).await;
        });

        info!("[DEV] Dev command subscriber thread stopped");
    })
}

/// Main subscriber loop with reconnection logic
async fn subscriber_loop(tx: Sender<DevCommand>, nats_url: String) {
    let mut reconnect_delay = RECONNECT_DELAY_MS;
    let mut consecutive_errors = 0;

    loop {
        info!("[DEV] Connecting to {}...", nats_url);

        match async_nats::connect(&nats_url).await {
            Ok(client) => {
                info!("[DEV] Connected successfully, subscribing to {}", DEV_COMMAND_SUBJECT);
                consecutive_errors = 0;
                reconnect_delay = RECONNECT_DELAY_MS;

                // Run the subscription loop until it fails
                if let Err(e) = subscribe_to_commands(&client, &tx).await {
                    error!("[DEV] Subscription error: {}", e);
                    consecutive_errors += 1;
                }
            }
            Err(e) => {
                warn!("[DEV] Connection failed: {}", e);
                consecutive_errors += 1;
            }
        }

        // Circuit breaker: after 10 consecutive failures, pause for longer
        if consecutive_errors >= 10 {
            warn!(
                "[DEV] Too many consecutive errors ({}), pausing reconnection for 30s",
                consecutive_errors
            );
            tokio::time::sleep(Duration::from_secs(30)).await;
            consecutive_errors = 0;
            continue;
        }

        // Exponential backoff for reconnection
        info!("[DEV] Reconnecting in {}ms...", reconnect_delay);
        tokio::time::sleep(Duration::from_millis(reconnect_delay)).await;

        reconnect_delay = (reconnect_delay * 2).min(MAX_RECONNECT_DELAY_MS);
    }
}

/// Subscribe to dev commands and forward to channel
async fn subscribe_to_commands(
    client: &async_nats::Client,
    tx: &Sender<DevCommand>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut subscriber = client.subscribe(DEV_COMMAND_SUBJECT).await?;
    let mut commands_received = 0u64;
    let mut commands_dropped = 0u64;

    info!("[DEV] Subscribed to {}, waiting for commands...", DEV_COMMAND_SUBJECT);

    while let Some(message) = subscriber.next().await {
        // Parse JSON command
        let command = match serde_json::from_slice::<DevCommand>(&message.payload) {
            Ok(cmd) => cmd,
            Err(e) => {
                warn!("[DEV] Failed to parse command: {} - Payload: {:?}", e, String::from_utf8_lossy(&message.payload));
                commands_dropped += 1;
                continue;
            }
        };

        // Send to simulation
        match tx.try_send(command.clone()) {
            Ok(_) => {
                commands_received += 1;
                info!("[DEV] Command received: {:?}", command);

                // Log stats every 100 commands
                if commands_received % 100 == 0 {
                    info!(
                        "[DEV] Received {} commands, dropped {}",
                        commands_received, commands_dropped
                    );
                }
            }
            Err(_) => {
                warn!("[DEV] Command channel full, dropping command");
                commands_dropped += 1;
            }
        }
    }

    Ok(())
}
