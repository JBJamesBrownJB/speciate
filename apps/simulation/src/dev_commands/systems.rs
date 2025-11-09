//! ECS systems for processing dev commands
//!
//! These systems run in the Bevy schedule and process dev commands
//! received from the DevCommandListener.

use super::commands::DevCommand;
use super::listener::DevCommandListener;
use crate::simulation::components::*;
use crate::simulation::core::components::DeltaTime;
use crate::simulation::creatures::builder::CritBuilder;
use crate::simulation::creatures::systems::{EntityIdMap, NextCreatureId};
use bevy_ecs::prelude::*;
use log::{info, warn};

/// Resource to track next CritId for dev spawns
#[derive(Resource)]
pub struct DevSpawnIdCounter {
    next_id: u32,
}

impl Default for DevSpawnIdCounter {
    fn default() -> Self {
        // Start at a high number to avoid conflicts with normal spawns
        Self { next_id: 900_000 }
    }
}

/// Process dev commands from the listener
///
/// This system runs early in the Bevy schedule (before behavior systems)
/// to process dev commands received from NATS.
pub fn process_dev_commands_system(
    mut commands: Commands,
    listener: Res<DevCommandListener>,
    mut next_id: ResMut<NextCreatureId>,
    mut entity_map: ResMut<EntityIdMap>,
    mut delta_time: ResMut<DeltaTime>,
    query: Query<Entity, With<CritId>>,
) {
    // Receive all pending commands
    let dev_commands = listener.recv_all();

    for command in dev_commands {
        match command {
            DevCommand::Spawn {
                x,
                y,
                behavior,
                target_x,
                target_y,
                energy,
                max_speed,
            } => {
                handle_spawn(
                    &mut commands,
                    &mut next_id,
                    &mut entity_map,
                    x,
                    y,
                    &behavior,
                    target_x,
                    target_y,
                    energy,
                    max_speed,
                );
            }
            DevCommand::Clear => {
                handle_clear(&mut commands, &mut entity_map, &query);
            }
            DevCommand::Speed { multiplier } => {
                handle_speed(&mut delta_time, multiplier);
            }
        }
    }
}

/// Handle spawn command
///
/// Spawns directly using Commands and registers in EntityIdMap.
/// This is the same pattern as Simulation::spawn_crit() - all spawns go through
/// the same resources (NextCreatureId, EntityIdMap) ensuring consistency.
fn handle_spawn(
    commands: &mut Commands,
    next_id: &mut NextCreatureId,
    entity_map: &mut EntityIdMap,
    x: f32,
    y: f32,
    behavior: &str,
    target_x: Option<f32>,
    target_y: Option<f32>,
    energy: Option<f32>,
    max_speed: Option<f32>,
) {
    // Parse behavior mode
    let behavior_mode = match behavior.to_lowercase().as_str() {
        "seeking" => BehaviorMode::Seeking,
        "wandering" => BehaviorMode::Wandering,
        "catatonic" => BehaviorMode::Catatonic,
        _ => {
            warn!("[DEV] Unknown behavior '{}', defaulting to Catatonic", behavior);
            BehaviorMode::Catatonic
        }
    };

    // Build creature with CritBuilder
    let mut builder = CritBuilder::new()
        .at(x, y)
        .with_all_capabilities()
        .in_behavior(behavior_mode);

    // Apply optional parameters
    if let Some(e) = energy {
        builder = builder.with_energy(e);
    }
    if let Some(s) = max_speed {
        builder = builder.with_max_speed(s);
    }

    // Set target for seeking behavior
    if behavior_mode == BehaviorMode::Seeking {
        if let (Some(tx), Some(ty)) = (target_x, target_y) {
            builder = builder.with_target(tx, ty);
        } else {
            warn!("[DEV] Seeking behavior requires target_x and target_y, spawning at current position as target");
            builder = builder.with_target(x, y);
        }
    }

    // Assign unique ID from shared resource (same as Simulation::spawn_crit)
    let id = next_id.next();

    // Spawn entity with Commands (deferred until end of stage)
    let entity = commands.spawn(builder.build(id)).id();

    // Register in entity map (same as Simulation::spawn_crit)
    entity_map.insert(entity, id);

    info!("[DEV] Spawned creature #{} at ({}, {}) with behavior {:?}", id, x, y, behavior_mode);
}

/// Handle clear command
fn handle_clear(commands: &mut Commands, entity_map: &mut EntityIdMap, query: &Query<Entity, With<CritId>>) {
    let count = query.iter().count();
    for entity in query.iter() {
        commands.entity(entity).despawn();
        entity_map.remove(&entity); // Remove from tracking map
    }
    info!("[DEV] Cleared {} creatures", count);
}

/// Handle speed command
fn handle_speed(delta_time: &mut DeltaTime, multiplier: f32) {
    // Clamp multiplier to reasonable range
    let clamped = multiplier.clamp(0.1, 10.0);

    // Base delta time is 0.05 (20 Hz)
    let base_dt = 0.05;
    delta_time.0 = base_dt * clamped;

    info!("[DEV] Speed multiplier set to {:.2}x (delta time: {:.4}s)", clamped, delta_time.0);
}
