
use crate::config::MovementConfig;
#[cfg(feature = "dev-tools")]
use crate::instrumentation::SystemTimings;
use crate::simulation::components::*;
use crate::simulation::core::components::*;
use crate::simulation::movement::constants::{MAX_SPEED, STOPPED_THRESHOLD, VELOCITY_DAMPING};
use crate::simulation::movement::noise::perlin_locomotion_noise;
use bevy_ecs::prelude::*;
pub fn integrate_motion_system(
    mut query: Query<(
        Entity,
        &BodySize,
        &mut Position,
        &mut Velocity,
        &mut Acceleration,
        &CreatureState,
    )>,
    delta_time: Res<DeltaTime>,
    physics_tick: Res<PhysicsTick>,
    world_bounds: Res<crate::simulation::core::WorldBounds>,
    movement_config: Res<MovementConfig>,
    #[cfg(feature = "dev-tools")] timings: Res<SystemTimings>,
) {
    #[cfg(feature = "dev-tools")]
    crate::time_system!(timings, "movement");

    let dt = delta_time.0;
    let max_speed_sq = MAX_SPEED * MAX_SPEED;
    let tick = physics_tick.get();
    for (entity, size, mut position, mut velocity, mut acceleration, creature_state) in query.iter_mut() {
        if creature_state.behavior == BehaviorMode::Catatonic {
            acceleration.ax = 0.0;
            acceleration.ay = 0.0;

            let speed_sq = velocity.vx * velocity.vx + velocity.vy * velocity.vy;
            if speed_sq < STOPPED_THRESHOLD * STOPPED_THRESHOLD {
                velocity.vx = 0.0;
                velocity.vy = 0.0;
                continue;
            }

            velocity.vx *= VELOCITY_DAMPING;
            velocity.vy *= VELOCITY_DAMPING;

            position.x += velocity.vx * dt;
            position.y += velocity.vy * dt;

            continue;
        }
        velocity.vx += acceleration.ax * dt;
        velocity.vy += acceleration.ay * dt;
        velocity.vx *= VELOCITY_DAMPING;
        velocity.vy *= VELOCITY_DAMPING;
        let speed_sq = velocity.vx * velocity.vx + velocity.vy * velocity.vy;
        if speed_sq > 0.01 {
            let speed = speed_sq.sqrt();
            let speed_ratio = speed / MAX_SPEED;
            let size_factor = size.inv_sqrt_length;
            let noise_magnitude = movement_config.locomotion_noise_base * speed_ratio * speed_ratio * size_factor;

            let noise_x = perlin_locomotion_noise(entity.index(), tick, 0, movement_config.noise_time_scale);
            let noise_y = perlin_locomotion_noise(entity.index(), tick, 1, movement_config.noise_time_scale);

            let perpendicular_x = -velocity.vy / speed;
            let perpendicular_y = velocity.vx / speed;

            velocity.vx += perpendicular_x * noise_x * noise_magnitude * dt;
            velocity.vy += perpendicular_y * noise_y * noise_magnitude * dt;
        }
        let speed_sq = velocity.vx * velocity.vx + velocity.vy * velocity.vy;
        if speed_sq > max_speed_sq {
            let scale = (max_speed_sq / speed_sq).sqrt();
            velocity.vx *= scale;
            velocity.vy *= scale;
        }

        acceleration.ax = 0.0;
        acceleration.ay = 0.0;

        position.x += velocity.vx * dt;
        position.y += velocity.vy * dt;
    }

    for (_entity, _size, mut position, mut velocity, _accel, _state) in query.iter_mut() {
        if position.x < world_bounds.min_x {
            position.x = world_bounds.min_x;
            velocity.vx = velocity.vx.max(0.0);
        } else if position.x > world_bounds.max_x {
            position.x = world_bounds.max_x;
            velocity.vx = velocity.vx.min(0.0);
        }
        if position.y < world_bounds.min_y {
            position.y = world_bounds.min_y;
            velocity.vy = velocity.vy.max(0.0);
        } else if position.y > world_bounds.max_y {
            position.y = world_bounds.max_y;
            velocity.vy = velocity.vy.min(0.0);
        }
    }
}

pub fn update_body_size_cache(mut query: Query<&mut BodySize, Changed<BodySize>>) {
    for mut size in query.iter_mut() {
        size.inv_sqrt_length = 1.0 / size.length.sqrt();
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
        let dt = world.resource::<DeltaTime>().0;

        let mut query = world.query::<(&mut Position, &Velocity)>();
        for (mut pos, vel) in query.iter_mut(&mut world) {
            pos.x += vel.vx * dt;
            pos.y += vel.vy * dt;
        }

        let position = world.get::<Position>(entity).unwrap();
        assert_eq!(position.x, 1.0);
        assert_eq!(position.y, 0.5);
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

        let dt = world.resource::<DeltaTime>().0;

        let mut query = world.query::<(&mut Velocity, &mut Acceleration)>();
        for (mut vel, mut acc) in query.iter_mut(&mut world) {
            vel.vx += acc.ax * dt;
            vel.vy += acc.ay * dt;
            acc.ax = 0.0;
            acc.ay = 0.0;
        }

        let velocity = world.get::<Velocity>(entity).unwrap();
        assert_eq!(velocity.vx, 1.0);
        assert_eq!(velocity.vy, 0.5);
        let acceleration = world.get::<Acceleration>(entity).unwrap();
        assert_eq!(acceleration.ax, 0.0);
        assert_eq!(acceleration.ay, 0.0);
    }
}
