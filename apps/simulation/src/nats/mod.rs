//! NATS streaming integration
//!
//! This module handles publishing simulation frames to NATS for consumption
//! by the Broadcaster service, which streams to connected clients.

pub mod frame;
pub mod publisher;
pub mod systems;

// Re-export commonly used types
pub use frame::{CritTransform, SimulationFrame};
pub use systems::SimulationTick;

use bevy_ecs::system::Resource;
use crossbeam_channel::{bounded, Sender};
use std::thread;

/// Bevy ECS resource for NATS publishing
///
/// This resource holds a channel sender that allows the simulation
/// to send frames to the NATS publisher thread without blocking.
#[derive(Resource)]
pub struct NatsPublisher {
    sender: Sender<SimulationFrame>,
}

impl NatsPublisher {
    /// Create a new NATS publisher with a dedicated thread
    ///
    /// # Arguments
    /// * `nats_url` - NATS server URL (e.g., "nats://nats:4222")
    /// * `channel_capacity` - Bounded channel capacity (default: 4)
    ///
    /// # Returns
    /// A tuple of (NatsPublisher resource, thread JoinHandle)
    pub fn new(nats_url: String, channel_capacity: usize) -> (Self, thread::JoinHandle<()>) {
        let (tx, rx) = bounded(channel_capacity);
        let handle = publisher::spawn_nats_publisher(rx, nats_url);

        (Self { sender: tx }, handle)
    }

    /// Attempt to send a frame without blocking
    ///
    /// If the channel is full (NATS is slow), the frame is dropped.
    /// This ensures the simulation never blocks on NATS.
    ///
    /// # Returns
    /// `Ok(())` if sent successfully, `Err(frame)` if dropped
    pub fn try_send(&self, frame: SimulationFrame) -> Result<(), SimulationFrame> {
        self.sender.try_send(frame).map_err(|e| e.into_inner())
    }
}
