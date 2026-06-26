use crate::config::MovementConfig;
#[cfg(feature = "dev-tools")]
use crate::instrumentation::SystemTimings;
use crate::simulation::core::components::{
    Acceleration, BodySize, DeltaTime, PhysicsTick, Position, Rotation, Velocity,
};
use crate::simulation::creatures::components::{BehaviorMode, CreatureState};
use crate::simulation::creatures::constants::{
    DRAG_COEFFICIENT, MAX_TURN_RATE, MAX_TURN_RATE_DEG, MIN_TURN_RATE_DEG,
    NOISE_SPEED_THRESHOLD_SQ, STOPPED_THRESHOLD, TURN_RATE_SIZE_EXPONENT, TURN_RATE_SPEED_PENALTY,
};
use crate::simulation::math::{fast_atan2, normalize_angle};
use crate::simulation::movement::noise::NoiseTable;
use bevy_ecs::prelude::*;
pub fn integrate_motion_system(
    mut query: Query<(
        Entity,
        &BodySize,
        &mut Position,
        &mut Velocity,
        &mut Acceleration,
        &CreatureState,
        &mut Rotation,
    )>,
    delta_time: Res<DeltaTime>,
    physics_tick: Res<PhysicsTick>,
    world_bounds: Res<crate::simulation::core::WorldBounds>,
    movement_config: Res<MovementConfig>,
    noise_table: Res<NoiseTable>,
    #[cfg(feature = "dev-tools")] timings: Res<SystemTimings>,
) {
    #[cfg(feature = "dev-tools")]
    crate::time_system!(timings, "movement");

    let dt = delta_time.0;
    let tick = physics_tick.get();

    // Time-based drag: v *= exp(-drag * dt) is frame-rate independent
    let drag_factor = (-DRAG_COEFFICIENT * dt).exp();
    let noise_base = movement_config.locomotion_noise_base;
    let noise_time_scale = movement_config.noise_time_scale;

    // Capture bounds for parallel access
    let min_x = world_bounds.min_x;
    let max_x = world_bounds.max_x;
    let min_y = world_bounds.min_y;
    let max_y = world_bounds.max_y;

    // Get reference to noise table for parallel access
    let noise_ref = &*noise_table;

    let stopped_threshold_sq = STOPPED_THRESHOLD * STOPPED_THRESHOLD;

    // Native Bevy par_iter_mut — eliminates the per-tick 1M-entity Vec allocation.
    // ComputeTaskPool is initialised once in SimulationBuilder::new().
    query.par_iter_mut().for_each(
        |(entity, size, mut position, mut velocity, mut acceleration, creature_state, mut rotation)| {
            if creature_state.behavior == BehaviorMode::Catatonic {
                acceleration.ax = 0.0;
                acceleration.ay = 0.0;

                let speed_sq = velocity.vx * velocity.vx + velocity.vy * velocity.vy;
                if speed_sq < stopped_threshold_sq {
                    if velocity.vx != 0.0 || velocity.vy != 0.0 {
                        velocity.vx = 0.0;
                        velocity.vy = 0.0;
                    }
                    return;
                }

                velocity.vx *= drag_factor;
                velocity.vy *= drag_factor;

                position.x += velocity.vx * dt;
                position.y += velocity.vy * dt;

                // Boundary enforcement for coasting catatonic creatures
                if position.x < min_x {
                    position.x = min_x;
                    velocity.vx = velocity.vx.max(0.0);
                } else if position.x > max_x {
                    position.x = max_x;
                    velocity.vx = velocity.vx.min(0.0);
                }
                if position.y < min_y {
                    position.y = min_y;
                    velocity.vy = velocity.vy.max(0.0);
                } else if position.y > max_y {
                    position.y = max_y;
                    velocity.vy = velocity.vy.min(0.0);
                }

                return;
            }

            // Capture old heading before velocity changes (fast_atan2: ~5x faster)
            let old_speed_sq = velocity.vx * velocity.vx + velocity.vy * velocity.vy;
            let old_angle = if old_speed_sq > stopped_threshold_sq {
                fast_atan2(velocity.vy, velocity.vx)
            } else {
                f32::NAN
            };

            velocity.vx += acceleration.ax * dt;
            velocity.vy += acceleration.ay * dt;
            velocity.vx *= drag_factor;
            velocity.vy *= drag_factor;

            // Size-based speed limit for this creature
            let max_speed = size.max_speed();
            let max_speed_sq = max_speed * max_speed;

            // Track speed for reuse (avoid redundant sqrt)
            let mut speed_sq = velocity.vx * velocity.vx + velocity.vy * velocity.vy;
            let mut current_speed = 0.0_f32;
            let mut speed_computed = false;

            if speed_sq > NOISE_SPEED_THRESHOLD_SQ {
                current_speed = speed_sq.sqrt();
                let speed_ratio = current_speed / max_speed;
                let size_factor = size.inv_sqrt_length;
                let noise_magnitude = noise_base * speed_ratio * speed_ratio * size_factor;

                let noise_x = noise_ref.get(entity.index(), tick, 0, noise_time_scale);
                let noise_y = noise_ref.get(entity.index(), tick, 1, noise_time_scale);

                let inv_speed = 1.0 / current_speed;
                let perpendicular_x = -velocity.vy * inv_speed;
                let perpendicular_y = velocity.vx * inv_speed;

                velocity.vx += perpendicular_x * noise_x * noise_magnitude * dt;
                velocity.vy += perpendicular_y * noise_y * noise_magnitude * dt;

                // Recalculate after noise modification
                speed_sq = velocity.vx * velocity.vx + velocity.vy * velocity.vy;
                speed_computed = false; // Speed changed, need fresh sqrt if used
            }

            // Speed clamping
            let was_clamped = speed_sq > max_speed_sq;
            if was_clamped {
                if !speed_computed {
                    current_speed = speed_sq.sqrt();
                }
                let scale = max_speed / current_speed;
                velocity.vx *= scale;
                velocity.vy *= scale;
                current_speed = max_speed; // After clamping, speed is exactly max_speed
                speed_sq = max_speed_sq;
                // Note: speed_computed not set - if turn rate limiting needs speed,
                // sqrt(max_speed_sq) = max_speed = current_speed, so result is same
            }

            // Size-dependent turn rate limiting
            // Biological basis: turn_rate ∝ 1/size^1.33 (moment of inertia vs muscle torque)
            if old_angle.is_finite() && speed_sq > stopped_threshold_sq {
                // Calculate size-dependent base turn rate (deg/s)
                let base_turn_rate_deg = (MAX_TURN_RATE
                    / size.length.powf(TURN_RATE_SIZE_EXPONENT))
                .clamp(MIN_TURN_RATE_DEG, MAX_TURN_RATE_DEG);

                // Apply speed penalty: faster movement = less agile turning
                // At max speed, creatures retain (1 - PENALTY) of their turn ability
                let current_speed_for_penalty = if speed_computed || was_clamped {
                    current_speed
                } else {
                    speed_sq.sqrt()
                };
                let normalized_speed = (current_speed_for_penalty / max_speed).min(1.0);
                let speed_factor =
                    1.0 - TURN_RATE_SPEED_PENALTY * normalized_speed * normalized_speed;
                let effective_turn_rate_deg = base_turn_rate_deg * speed_factor;

                // Convert to radians and apply dt
                let max_delta = effective_turn_rate_deg.to_radians() * dt;

                let new_angle = fast_atan2(velocity.vy, velocity.vx);
                let delta = normalize_angle(new_angle - old_angle);

                if delta.abs() > max_delta {
                    let clamped_delta = delta.clamp(-max_delta, max_delta);
                    let final_angle = old_angle + clamped_delta;
                    // Reuse speed from penalty calculation
                    let new_speed = current_speed_for_penalty;
                    velocity.vx = new_speed * final_angle.cos();
                    velocity.vy = new_speed * final_angle.sin();
                }
            }

            acceleration.ax = 0.0;
            acceleration.ay = 0.0;

            position.x += velocity.vx * dt;
            position.y += velocity.vy * dt;

            // Boundary enforcement (merged into main loop)
            if position.x < min_x {
                position.x = min_x;
                velocity.vx = velocity.vx.max(0.0);
            } else if position.x > max_x {
                position.x = max_x;
                velocity.vx = velocity.vx.min(0.0);
            }
            if position.y < min_y {
                position.y = min_y;
                velocity.vy = velocity.vy.max(0.0);
            } else if position.y > max_y {
                position.y = max_y;
                velocity.vy = velocity.vy.min(0.0);
            }

            // Rotation update (fused for parallelization - vx/vy already in cache)
            rotation.set_from_velocity(velocity.vx, velocity.vy);
        },
    );
}

pub fn update_body_size_cache(mut query: Query<&mut BodySize, Changed<BodySize>>) {
    for mut size in query.iter_mut() {
        size.inv_sqrt_length = 1.0 / size.length.sqrt();
    }
}
