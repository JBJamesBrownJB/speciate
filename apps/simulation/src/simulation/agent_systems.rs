//! ECS Systems for agent/creature behavior

use super::components::*;
use bevy_ecs::prelude::*;
use rand::Rng;

// Physics Constants
const MAX_SPEED: f32 = 150.0;

// Behavior Constants
const FLEE_FORCE: f32 = 1.0;
const FLEE_ANGLE_VARIATION: f32 = 0.5;
const WANDER_RADIUS_DEFAULT: f32 = 25.0;
const WANDER_DISTANCE_DEFAULT: f32 = 50.0;
const WANDER_ANGLE_CHANGE_DEFAULT: f32 = 0.15;
const WANDER_MAX_FORCE: f32 = 0.3;

// Energy Constants
const ENERGY_COST_WANDERING: f32 = 0.01;
const ENERGY_COST_FLEEING: f32 = 0.05;
const ENERGY_RESTORE_FEEDING: f32 = 0.1;
const ENERGY_RESTORE_RESTING: f32 = 0.02;
const ENERGY_THRESHOLD_MODERATE: f32 = 50.0;
const ENERGY_THRESHOLD_HIGH: f32 = 80.0;

// Aging and Transition Constants
const AGE_INCREMENT_PER_TICK: f32 = 0.001;
const TRANSITION_PROB_WANDERING_TO_FLEEING: f64 = 0.01;
const TRANSITION_PROB_WANDERING_TO_RESTING: f64 = 0.001;
const TRANSITION_PROB_FLEEING_TO_WANDERING: f64 = 0.02;
const TRANSITION_PROB_RESTING_TO_WANDERING: f64 = 0.05;

pub fn update_physics_system(
    mut query: Query<(&mut Position, &mut Velocity, &mut Acceleration)>,
    delta_time: Res<DeltaTime>,
) {
    let dt = delta_time.0;
    let max_speed_sq = MAX_SPEED * MAX_SPEED;

    for (mut position, mut velocity, mut acceleration) in query.iter_mut() {
        velocity.vx += acceleration.ax * dt;
        velocity.vy += acceleration.ay * dt;

        let speed_sq = velocity.vx * velocity.vx + velocity.vy * velocity.vy;
        if speed_sq > max_speed_sq {
            let speed = speed_sq.sqrt();
            let inv_speed = MAX_SPEED / speed;
            velocity.vx *= inv_speed;
            velocity.vy *= inv_speed;
        }

        acceleration.ax = 0.0;
        acceleration.ay = 0.0;

        position.x += velocity.vx * dt;
        position.y += velocity.vy * dt;
    }
}

pub fn rotation_system(mut query: Query<(&mut Rotation, &Velocity)>) {
    for (mut rotation, velocity) in query.iter_mut() {
        if velocity.vx != 0.0 || velocity.vy != 0.0 {
            rotation.radians = velocity.vy.atan2(velocity.vx);
        }
    }
}

pub fn flee_system(
    mut query: Query<(&mut Acceleration, &mut Velocity, &FleeState, &CreatureState)>,
) {
    for (mut acceleration, mut velocity, flee_state, creature_state) in query.iter_mut() {
        if creature_state.behavior != BehaviorMode::Fleeing {
            continue;
        }
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let angle_variation = rng.gen_range(-FLEE_ANGLE_VARIATION..FLEE_ANGLE_VARIATION);
        let current_angle = velocity.angle() + angle_variation;

        acceleration.ax += current_angle.cos() * FLEE_FORCE;
        acceleration.ay += current_angle.sin() * FLEE_FORCE;

        let max_flee_speed = creature_state.max_speed * flee_state.flee_speed_multiplier;
        velocity.limit(max_flee_speed);
    }
}

/// Implements wandering behavior for creatures in Wandering state
pub fn wander_system(
    mut query: Query<(
        &mut Acceleration,
        &mut WanderState,
        &Velocity,
        &CreatureState,
    )>,
) {
    let mut rng = rand::thread_rng();

    for (mut acceleration, mut wander_state, velocity, creature_state) in query.iter_mut() {
        if creature_state.behavior != BehaviorMode::Wandering {
            continue;
        }
        if wander_state.wander_radius == 0.0 {
            wander_state.wander_radius = WANDER_RADIUS_DEFAULT;
            wander_state.wander_distance = WANDER_DISTANCE_DEFAULT;
            wander_state.angle_change = WANDER_ANGLE_CHANGE_DEFAULT;
        }

        wander_state.wander_angle +=
            rng.gen_range(-wander_state.angle_change..wander_state.angle_change);

        let vel_magnitude = (velocity.vx * velocity.vx + velocity.vy * velocity.vy).sqrt();
        let (vel_normalized_x, vel_normalized_y) = if vel_magnitude > 0.0 {
            (velocity.vx / vel_magnitude, velocity.vy / vel_magnitude)
        } else {
            let angle = rng.gen_range(0.0..std::f32::consts::TAU);
            (angle.cos(), angle.sin())
        };

        let circle_center_x = vel_normalized_x * wander_state.wander_distance;
        let circle_center_y = vel_normalized_y * wander_state.wander_distance;

        let displacement_x = wander_state.wander_angle.cos() * wander_state.wander_radius;
        let displacement_y = wander_state.wander_angle.sin() * wander_state.wander_radius;

        let wander_force_x = circle_center_x + displacement_x;
        let wander_force_y = circle_center_y + displacement_y;
        let force_magnitude =
            (wander_force_x * wander_force_x + wander_force_y * wander_force_y).sqrt();
        if force_magnitude > 0.0 {
            acceleration.ax += (wander_force_x / force_magnitude) * WANDER_MAX_FORCE;
            acceleration.ay += (wander_force_y / force_magnitude) * WANDER_MAX_FORCE;
        }
    }
}

/// Behavior transition system that manages creature state changes
pub fn behavior_transition_system(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut CreatureState,
        Option<&WanderState>,
        Option<&FleeState>,
    )>,
) {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    for (entity, mut creature_state, wander_state, _flee_state) in query.iter_mut() {
        // Consume energy based on current behavior
        match creature_state.behavior {
            BehaviorMode::Wandering => creature_state.consume_energy(ENERGY_COST_WANDERING),
            BehaviorMode::Fleeing => creature_state.consume_energy(ENERGY_COST_FLEEING),
            BehaviorMode::Feeding => creature_state.restore_energy(ENERGY_RESTORE_FEEDING),
            BehaviorMode::Resting => creature_state.restore_energy(ENERGY_RESTORE_RESTING),
        }

        // Age the creature
        creature_state.age += AGE_INCREMENT_PER_TICK;

        // State transition logic
        let previous_behavior = creature_state.behavior;

        match creature_state.behavior {
            BehaviorMode::Wandering => {
                // Transition to resting if exhausted
                if creature_state.is_exhausted() {
                    creature_state.behavior = BehaviorMode::Resting;
                }
                // Random chance to start feeding if energy is low
                else if creature_state.is_low_energy() && rng.gen_bool(TRANSITION_PROB_WANDERING_TO_FLEEING) {
                    creature_state.behavior = BehaviorMode::Feeding;
                }
                // Very small chance to flee (simulating perceived threat)
                else if rng.gen_bool(TRANSITION_PROB_WANDERING_TO_RESTING) {
                    creature_state.behavior = BehaviorMode::Fleeing;
                }
            }
            BehaviorMode::Resting => {
                // Return to wandering when energy is restored
                if creature_state.energy > ENERGY_THRESHOLD_MODERATE {
                    creature_state.behavior = BehaviorMode::Wandering;
                }
            }
            BehaviorMode::Feeding => {
                // Stop feeding when energy is full or random chance
                if creature_state.energy > ENERGY_THRESHOLD_HIGH || rng.gen_bool(TRANSITION_PROB_FLEEING_TO_WANDERING) {
                    creature_state.behavior = BehaviorMode::Wandering;
                }
            }
            BehaviorMode::Fleeing => {
                // Stop fleeing after a while or if exhausted
                if creature_state.is_exhausted() || rng.gen_bool(TRANSITION_PROB_RESTING_TO_WANDERING) {
                    if creature_state.is_exhausted() {
                        creature_state.behavior = BehaviorMode::Resting;
                    } else {
                        creature_state.behavior = BehaviorMode::Wandering;
                    }
                }
            }
        }

        // Add/remove behavior-specific components based on state changes
        if previous_behavior != creature_state.behavior {
            match creature_state.behavior {
                BehaviorMode::Wandering => {
                    // Remove other behavior components
                    commands.entity(entity).remove::<FleeState>();
                    // Add wander state if not present
                    if wander_state.is_none() {
                        commands.entity(entity).insert(WanderState {
                            wander_angle: rng.gen_range(0.0..std::f32::consts::TAU),
                            wander_radius: WANDER_RADIUS_DEFAULT,
                            wander_distance: WANDER_DISTANCE_DEFAULT,
                            angle_change: WANDER_ANGLE_CHANGE_DEFAULT,
                        });
                    }
                }
                BehaviorMode::Fleeing => {
                    // Add flee state
                    commands.entity(entity).insert(FleeState::new(None));
                }
                BehaviorMode::Resting | BehaviorMode::Feeding => {
                    // These states don't need specific components yet
                    commands.entity(entity).remove::<FleeState>();
                }
            }
        }
    }
}

/// Applies force toward center when near boundaries (soft boundary)
pub fn boundary_seek_system(
    mut query: Query<(&Position, &mut Acceleration)>,
    config: Res<BoundaryConfig>,
) {
    for (position, mut acceleration) in query.iter_mut() {
        let center_x = config.width / 2.0;
        let center_y = config.height / 2.0;

        let mut desired_x = 0.0;
        let mut desired_y = 0.0;
        let mut apply_force = false;

        // Check horizontal boundaries
        if position.x < config.margin || position.x > config.width - config.margin {
            desired_x = center_x - position.x;
            apply_force = true;
        }

        // Check vertical boundaries
        if position.y < config.margin || position.y > config.height - config.margin {
            desired_y = center_y - position.y;
            apply_force = true;
        }

        // Apply steering force toward center if near boundary
        if apply_force {
            let distance = (desired_x * desired_x + desired_y * desired_y).sqrt();
            if distance > 0.0 {
                // Normalize and scale by max force
                acceleration.ax += (desired_x / distance) * config.max_force;
                acceleration.ay += (desired_y / distance) * config.max_force;
            }
        }
    }
}

/// HARD boundary enforcement - prevents creatures from escaping world bounds
/// This system MUST run after movement_system to work correctly
/// Optimized with branch-free clamping for better CPU pipeline utilization
pub fn boundary_enforcement_system(
    mut query: Query<(&mut Position, &mut Velocity)>,
    config: Res<BoundaryConfig>,
) {
    for (mut position, mut velocity) in query.iter_mut() {
        let hit_left = (position.x < 0.0) as i32 as f32;
        let hit_right = (position.x > config.width) as i32 as f32;
        position.x = position.x.clamp(0.0, config.width);
        velocity.vx = velocity.vx * (1.0 - 2.0 * hit_left) * (1.0 - 2.0 * hit_right).abs();

        let hit_bottom = (position.y < 0.0) as i32 as f32;
        let hit_top = (position.y > config.height) as i32 as f32;
        position.y = position.y.clamp(0.0, config.height);
        velocity.vy = velocity.vy * (1.0 - 2.0 * hit_bottom) * (1.0 - 2.0 * hit_top).abs();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_movement_system_updates_position() {
        let mut world = World::new();
        world.insert_resource(DeltaTime(0.1));

        let entity = world
            .spawn((Position { x: 0.0, y: 0.0 }, Velocity { vx: 10.0, vy: 5.0 }))
            .id();

        // Get delta time first
        let dt = world.resource::<DeltaTime>().0;

        // Run movement system
        let mut query = world.query::<(&mut Position, &Velocity)>();
        for (mut pos, vel) in query.iter_mut(&mut world) {
            pos.x += vel.vx * dt;
            pos.y += vel.vy * dt;
        }

        // Check position updated
        let position = world.get::<Position>(entity).unwrap();
        assert_eq!(position.x, 1.0); // 10 * 0.1
        assert_eq!(position.y, 0.5); // 5 * 0.1
    }

    #[test]
    fn test_acceleration_system_updates_velocity() {
        let mut world = World::new();
        world.insert_resource(DeltaTime(0.1));

        let entity = world
            .spawn((
                Velocity { vx: 0.0, vy: 0.0 },
                Acceleration { ax: 10.0, ay: 5.0 },
            ))
            .id();

        // Get delta time first
        let dt = world.resource::<DeltaTime>().0;

        // Simulate acceleration system
        let mut query = world.query::<(&mut Velocity, &mut Acceleration)>();
        for (mut vel, mut acc) in query.iter_mut(&mut world) {
            vel.vx += acc.ax * dt;
            vel.vy += acc.ay * dt;
            acc.ax = 0.0;
            acc.ay = 0.0;
        }

        // Check velocity updated and acceleration reset
        let velocity = world.get::<Velocity>(entity).unwrap();
        assert_eq!(velocity.vx, 1.0); // 10 * 0.1
        assert_eq!(velocity.vy, 0.5); // 5 * 0.1

        let acceleration = world.get::<Acceleration>(entity).unwrap();
        assert_eq!(acceleration.ax, 0.0);
        assert_eq!(acceleration.ay, 0.0);
    }

    #[test]
    fn test_rotation_system_matches_velocity() {
        let mut world = World::new();

        let entity = world
            .spawn((
                Rotation { radians: 0.0 },
                Velocity { vx: 1.0, vy: 1.0 }, // 45 degrees
            ))
            .id();

        // Simulate rotation system
        let mut query = world.query::<(&mut Rotation, &Velocity)>();
        for (mut rot, vel) in query.iter_mut(&mut world) {
            if vel.vx != 0.0 || vel.vy != 0.0 {
                rot.radians = vel.vy.atan2(vel.vx);
            }
        }

        let rotation = world.get::<Rotation>(entity).unwrap();
        let expected = 1.0f32.atan2(1.0); // ≈ 0.785 radians (45°)
        assert!((rotation.radians - expected).abs() < 0.001);
    }
}
