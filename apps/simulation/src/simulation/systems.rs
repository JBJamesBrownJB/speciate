//! ECS Systems using Bevy ECS

use bevy_ecs::prelude::*;
use rand::Rng;
use super::components::*;

/// Applies acceleration to velocity and resets acceleration
pub fn apply_acceleration_system(
    mut query: Query<(&mut Velocity, &mut Acceleration)>,
    delta_time: Res<DeltaTime>,
) {
    for (mut velocity, mut acceleration) in query.iter_mut() {
        // Apply acceleration to velocity
        velocity.vx += acceleration.ax * delta_time.0;
        velocity.vy += acceleration.ay * delta_time.0;

        // Limit velocity to max speed
        let max_speed = 150.0;
        let speed = (velocity.vx * velocity.vx + velocity.vy * velocity.vy).sqrt();
        if speed > max_speed {
            velocity.vx = (velocity.vx / speed) * max_speed;
            velocity.vy = (velocity.vy / speed) * max_speed;
        }

        // Reset acceleration for next frame (steering behaviors accumulate forces)
        acceleration.ax = 0.0;
        acceleration.ay = 0.0;
    }
}

/// Updates position based on velocity
pub fn movement_system(
    mut query: Query<(&mut Position, &Velocity)>,
    delta_time: Res<DeltaTime>,
) {
    for (mut position, velocity) in query.iter_mut() {
        position.x += velocity.vx * delta_time.0;
        position.y += velocity.vy * delta_time.0;
    }
}

/// Updates rotation to match velocity direction
pub fn rotation_system(
    mut query: Query<(&mut Rotation, &Velocity)>,
) {
    for (mut rotation, velocity) in query.iter_mut() {
        if velocity.vx != 0.0 || velocity.vy != 0.0 {
            rotation.radians = velocity.vy.atan2(velocity.vx);
        }
    }
}

/// Implements fleeing behavior for creatures in Flee state
pub fn flee_system(
    mut query: Query<(&mut Acceleration, &mut Velocity, &FleeState, &CreatureState)>,
) {
    for (mut acceleration, mut velocity, flee_state, creature_state) in query.iter_mut() {
        // Only apply fleeing if creature is in Fleeing behavior mode
        if creature_state.behavior != BehaviorMode::Fleeing {
            continue;
        }

        // Apply a strong random force to simulate panic
        use rand::Rng;
        let mut rng = rand::thread_rng();

        // Increase speed for fleeing
        let flee_force = 1.0;
        let angle_variation = rng.gen_range(-0.5..0.5);
        let current_angle = velocity.angle() + angle_variation;

        acceleration.ax += current_angle.cos() * flee_force;
        acceleration.ay += current_angle.sin() * flee_force;

        // Apply speed multiplier to velocity
        let max_flee_speed = creature_state.max_speed * flee_state.flee_speed_multiplier;
        velocity.limit(max_flee_speed);
    }
}

/// Implements wandering behavior for creatures in Wandering state
pub fn wander_system(
    mut query: Query<(&mut Acceleration, &mut WanderState, &Velocity, &CreatureState)>,
) {
    let mut rng = rand::thread_rng();

    for (mut acceleration, mut wander_state, velocity, creature_state) in query.iter_mut() {
        // Only apply wandering if creature is in Wandering behavior mode
        if creature_state.behavior != BehaviorMode::Wandering {
            continue;
        }
        // Initialize wander state if needed
        if wander_state.wander_radius == 0.0 {
            wander_state.wander_radius = 25.0;
            wander_state.wander_distance = 50.0;
            wander_state.angle_change = 0.15;
        }

        // Update wander angle with random change
        wander_state.wander_angle += rng.gen_range(-wander_state.angle_change..wander_state.angle_change);

        // Calculate current velocity direction
        let vel_magnitude = (velocity.vx * velocity.vx + velocity.vy * velocity.vy).sqrt();
        let (vel_normalized_x, vel_normalized_y) = if vel_magnitude > 0.0 {
            (velocity.vx / vel_magnitude, velocity.vy / vel_magnitude)
        } else {
            // If stationary, use a random direction
            let angle = rng.gen_range(0.0..std::f32::consts::TAU);
            (angle.cos(), angle.sin())
        };

        // Project circle in front of the agent
        let circle_center_x = vel_normalized_x * wander_state.wander_distance;
        let circle_center_y = vel_normalized_y * wander_state.wander_distance;

        // Calculate displacement force on the circle
        let displacement_x = wander_state.wander_angle.cos() * wander_state.wander_radius;
        let displacement_y = wander_state.wander_angle.sin() * wander_state.wander_radius;

        // Calculate wander force
        let wander_force_x = circle_center_x + displacement_x;
        let wander_force_y = circle_center_y + displacement_y;

        // Normalize and apply wander force
        let force_magnitude = (wander_force_x * wander_force_x + wander_force_y * wander_force_y).sqrt();
        if force_magnitude > 0.0 {
            let max_force = 0.3;
            acceleration.ax += (wander_force_x / force_magnitude) * max_force;
            acceleration.ay += (wander_force_y / force_magnitude) * max_force;
        }
    }
}

/// Behavior transition system that manages creature state changes
pub fn behavior_transition_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut CreatureState, Option<&WanderState>, Option<&FleeState>)>,
) {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    for (entity, mut creature_state, wander_state, _flee_state) in query.iter_mut() {
        // Consume energy based on current behavior
        match creature_state.behavior {
            BehaviorMode::Wandering => creature_state.consume_energy(0.01),
            BehaviorMode::Fleeing => creature_state.consume_energy(0.05), // Fleeing costs more energy
            BehaviorMode::Feeding => creature_state.restore_energy(0.1),  // Feeding restores energy
            BehaviorMode::Resting => creature_state.restore_energy(0.02), // Resting slowly restores energy
        }

        // Age the creature
        creature_state.age += 0.001;

        // State transition logic
        let previous_behavior = creature_state.behavior;

        match creature_state.behavior {
            BehaviorMode::Wandering => {
                // Transition to resting if exhausted
                if creature_state.is_exhausted() {
                    creature_state.behavior = BehaviorMode::Resting;
                }
                // Random chance to start feeding if energy is low
                else if creature_state.is_low_energy() && rng.gen_bool(0.01) {
                    creature_state.behavior = BehaviorMode::Feeding;
                }
                // Very small chance to flee (simulating perceived threat)
                else if rng.gen_bool(0.001) {
                    creature_state.behavior = BehaviorMode::Fleeing;
                }
            },
            BehaviorMode::Resting => {
                // Return to wandering when energy is restored
                if creature_state.energy > 50.0 {
                    creature_state.behavior = BehaviorMode::Wandering;
                }
            },
            BehaviorMode::Feeding => {
                // Stop feeding when energy is full or random chance
                if creature_state.energy > 80.0 || rng.gen_bool(0.02) {
                    creature_state.behavior = BehaviorMode::Wandering;
                }
            },
            BehaviorMode::Fleeing => {
                // Stop fleeing after a while or if exhausted
                if creature_state.is_exhausted() || rng.gen_bool(0.05) {
                    if creature_state.is_exhausted() {
                        creature_state.behavior = BehaviorMode::Resting;
                    } else {
                        creature_state.behavior = BehaviorMode::Wandering;
                    }
                }
            },
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
                            wander_radius: 25.0,
                            wander_distance: 50.0,
                            angle_change: 0.15,
                        });
                    }
                },
                BehaviorMode::Fleeing => {
                    // Add flee state
                    commands.entity(entity).insert(FleeState::new(None));
                },
                BehaviorMode::Resting | BehaviorMode::Feeding => {
                    // These states don't need specific components yet
                    commands.entity(entity).remove::<FleeState>();
                },
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
pub fn boundary_enforcement_system(
    mut query: Query<(&mut Position, &mut Velocity)>,
    config: Res<BoundaryConfig>,
) {
    for (mut position, mut velocity) in query.iter_mut() {
        // Hard clamp X boundary with velocity reversal
        if position.x < 0.0 {
            position.x = 0.0;
            velocity.vx = velocity.vx.abs(); // Bounce right
        } else if position.x > config.width {
            position.x = config.width;
            velocity.vx = -velocity.vx.abs(); // Bounce left
        }

        // Hard clamp Y boundary with velocity reversal
        if position.y < 0.0 {
            position.y = 0.0;
            velocity.vy = velocity.vy.abs(); // Bounce up
        } else if position.y > config.height {
            position.y = config.height;
            velocity.vy = -velocity.vy.abs(); // Bounce down
        }
    }
}

/// Main simulation struct managing the ECS World and Schedule
pub struct Simulation {
    world: World,
    schedule: Schedule,
    next_id: u32,
    entity_id_map: std::collections::HashMap<bevy_ecs::entity::Entity, u32>,
}

impl Simulation {
    /// Creates a new simulation with ECS world and scheduled systems
    pub fn new() -> Self {
        let mut world = World::new();
        let mut schedule = Schedule::default();

        // Add systems with proper ordering
        // Behavior transition should run first to update states
        // Force systems (wander, flee, boundary) must run before applying acceleration
        // boundary_enforcement_system MUST run after movement_system to prevent escapes
        schedule.add_systems((
            behavior_transition_system.before(wander_system),
            wander_system.before(apply_acceleration_system),
            flee_system.before(apply_acceleration_system),
            boundary_seek_system.before(apply_acceleration_system),
            apply_acceleration_system,
            movement_system.after(apply_acceleration_system),
            boundary_enforcement_system.after(movement_system), // CRITICAL: Hard boundary after movement
            rotation_system.after(boundary_enforcement_system),
        ));

        // Insert resources
        world.insert_resource(DeltaTime::default());
        world.insert_resource(BoundaryConfig::default());

        Self {
            world,
            schedule,
            next_id: 0,
            entity_id_map: std::collections::HashMap::new(),
        }
    }

    /// Sets the boundary configuration
    pub fn set_boundaries(&mut self, width: f32, height: f32) {
        self.world.insert_resource(BoundaryConfig {
            width,
            height,
            margin: 20.0,
            max_force: 1.0,
        });
    }

    /// Spawns a new creature entity
    pub fn spawn_creature(&mut self, x: f32, y: f32, width: f32, height: f32) -> u32 {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let angle = rng.gen_range(0.0..std::f32::consts::TAU);
        let speed = rng.gen_range(30.0..60.0);

        // Species ID (0 = default species for now)
        let species_id = 0;

        let entity = self.world.spawn((
            Position { x, y },
            Velocity {
                vx: angle.cos() * speed,
                vy: angle.sin() * speed,
            },
            Acceleration { ax: 0.0, ay: 0.0 },
            Rotation { radians: angle },
            Size { width, height },
            CreatureState::new(species_id),  // General state (required)
            WanderState {  // Behavior-specific state (only for wandering creatures)
                wander_angle: rng.gen_range(0.0..std::f32::consts::TAU),
                wander_radius: 25.0,
                wander_distance: 50.0,
                angle_change: 0.15,
            },
        )).id();

        let id = self.next_id;
        self.next_id += 1;
        self.entity_id_map.insert(entity, id);
        id
    }

    /// Updates the simulation by one step
    pub fn update(&mut self, delta_time: f32) {
        self.world.insert_resource(DeltaTime(delta_time));
        self.schedule.run(&mut self.world);
    }

    /// Returns creature data for network serialization
    pub fn get_creatures(&mut self) -> Vec<CreatureData> {
        let mut creatures = Vec::new();
        let mut query = self.world.query::<(
            bevy_ecs::entity::Entity,
            &Position,
            &Rotation,
            &Size,
            &CreatureState,
        )>();

        for (entity, position, rotation, size, creature_state) in query.iter(&self.world) {
            if let Some(&id) = self.entity_id_map.get(&entity) {
                creatures.push(CreatureData {
                    id,
                    x: position.x,
                    y: position.y,
                    rotation: rotation.radians,
                    width: size.width,
                    height: size.height,
                    behavior: creature_state.behavior,
                    energy: creature_state.energy,
                    species_id: creature_state.species_id,
                });
            }
        }

        creatures
    }

    /// Returns the number of active creatures
    pub fn creature_count(&self) -> usize {
        self.entity_id_map.len()
    }
}

impl Default for Simulation {
    fn default() -> Self {
        Self::new()
    }
}
