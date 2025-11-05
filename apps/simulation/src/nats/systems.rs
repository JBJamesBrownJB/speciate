//! NATS publishing ECS systems
//!
//! Bevy ECS systems that query entities and publish frame data to NATS.

use super::frame::{AgentTransform, SimulationFrame};
use super::NatsPublisher;
use crate::simulation::components::{AgentId, Position, Rotation, Velocity};
use bevy_ecs::prelude::*;
use log::{debug, warn};

/// Tick counter resource for tracking simulation frames
#[derive(Resource, Default)]
pub struct SimulationTick(pub u64);

/// ECS system that publishes simulation frames to NATS
///
/// This system runs at 20 Hz and collects all agent transform data
/// (position, velocity, rotation) then sends it to the NATS publisher
/// thread via a non-blocking channel.
///
/// If the channel is full (NATS is slow), frames are dropped to ensure
/// the simulation never blocks on NATS.
pub fn publish_frame_system(
    query: Query<(&AgentId, &Position, &Velocity, &Rotation)>,
    tick: Res<SimulationTick>,
    publisher: Res<NatsPublisher>,
) {
    // Collect all agent transforms
    let agents: Vec<AgentTransform> = query
        .iter()
        .map(|(agent_id, position, velocity, rotation)| AgentTransform {
            id: agent_id.0,
            x: position.x,
            y: position.y,
            vx: velocity.vx,
            vy: velocity.vy,
            rotation: rotation.radians,
        })
        .collect();

    if tick.0 % 100 == 0 {
        // Log every 100 ticks for visibility
        debug!(
            "[NATS] Tick {}: Publishing frame with {} agents",
            tick.0,
            agents.len()
        );
    }

    // Create simulation frame
    let frame = SimulationFrame::new(tick.0, agents);

    // Try to send without blocking
    if let Err(dropped_frame) = publisher.try_send(frame) {
        // Frame was dropped (channel full)
        warn!(
            "[NATS] Frame dropped (tick {}), NATS publisher can't keep up",
            dropped_frame.tick
        );
    } else {
        // Successfully queued for publishing
        debug!("[NATS] Frame queued for publishing (tick {})", tick.0);
    }
}

/// System to increment the simulation tick counter
///
/// This should run at the end of each frame to keep track of
/// the current simulation tick.
pub fn increment_tick_system(mut tick: ResMut<SimulationTick>) {
    tick.0 += 1;
}
