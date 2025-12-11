use crate::simulation::creatures::constants::WANDER_FORCE_MULT;
use crate::simulation::creatures::components::BehaviorMode;
use crate::simulation::math::{clamp_force, magnitude_sq, normalize};
use crate::simulation::queries::WanderQuery;
use rand::Rng;
use rayon::prelude::*;

/// Pure Reynolds wander steering behavior.
///
/// Wander creates smooth, organic-looking exploration by projecting a "wander circle"
/// ahead of the creature and picking a random point on it as the steering target.
/// The wander angle changes gradually, creating continuous curved paths.
///
/// NOTE: No "homeward force" - this was biologically incorrect. Animals don't feel
/// pulled home; they DECIDE to return based on internal state (hunger, fatigue).
/// Territory bounds are enforced through navigation decisions, not invisible forces.
pub fn territory_wandering_system(
    mut query: WanderQuery,
    #[cfg(feature = "dev-tools")] timings: bevy_ecs::system::Res<
        crate::instrumentation::SystemTimings,
    >,
) {
    #[cfg(feature = "dev-tools")]
    crate::time_system!(timings, "wander");

    let mut entities: Vec<_> = query.iter_mut().collect();

    entities.par_iter_mut().for_each(|(_entity, acceleration, wander_state, velocity, _position, _home, creature_state, size)| {
        if creature_state.behavior != BehaviorMode::Wandering {
            return;
        }

        let mut rng = rand::thread_rng();

        let speed_sq = magnitude_sq(velocity.vx, velocity.vy);

        // Use current heading, or wander angle if stationary
        let (heading_x, heading_y) = if speed_sq < 0.0001 {
            let (sin_a, cos_a) = wander_state.wander_angle.sin_cos();
            (cos_a, sin_a)
        } else {
            normalize(velocity.vx, velocity.vy)
        };

        // Project wander circle ahead of creature
        let circle_center_x = heading_x * wander_state.wander_distance;
        let circle_center_y = heading_y * wander_state.wander_distance;

        // Randomly adjust wander angle (creates smooth direction changes)
        let angle_change = rng.gen_range(-wander_state.angle_change..wander_state.angle_change);
        wander_state.wander_angle += angle_change.to_radians();
        wander_state.wander_angle = wander_state.wander_angle.rem_euclid(std::f32::consts::TAU);

        // Pick target point on wander circle
        let (sin_wander, cos_wander) = wander_state.wander_angle.sin_cos();
        let target_x = circle_center_x + wander_state.wander_radius * cos_wander;
        let target_y = circle_center_y + wander_state.wander_radius * sin_wander;

        // Calculate steering toward target
        let (norm_desired_x, norm_desired_y) = normalize(target_x, target_y);
        let max_speed = creature_state.max_speed;
        let scaled_desired_x = norm_desired_x * max_speed;
        let scaled_desired_y = norm_desired_y * max_speed;

        let steer_x = scaled_desired_x - velocity.vx;
        let steer_y = scaled_desired_y - velocity.vy;

        // Apply wander force (low effort exploration)
        let wander_force_limit = size.max_force() * WANDER_FORCE_MULT.get();
        let (force_x, force_y) = clamp_force(steer_x, steer_y, wander_force_limit);

        if force_x.is_finite() && force_y.is_finite() {
            acceleration.ax += force_x;
            acceleration.ay += force_y;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::core::components::{Acceleration, BodySize, Position, Velocity};
    use crate::simulation::creatures::components::{CreatureState, HomePosition, WanderState, CanWander};
    use bevy_ecs::prelude::*;

    fn run_wander_system(world: &mut World) {
        // Use proper Bevy system invocation for tests
        use bevy_ecs::system::{IntoSystem, System};

        #[cfg(feature = "dev-tools")]
        world.insert_resource(crate::instrumentation::SystemTimings::new());

        let mut system = IntoSystem::into_system(territory_wandering_system);
        system.initialize(world);
        system.run((), world);
        system.apply_deferred(world);
    }

    #[test]
    fn test_wander_produces_force() {
        let mut world = World::new();

        world.spawn((
            Position { x: 0.0, y: 0.0 },
            Velocity { vx: 1.0, vy: 0.0 },
            Acceleration::default(),
            WanderState {
                wander_angle: 0.0,
                wander_radius: 10.0,
                wander_distance: 50.0,
                angle_change: 4.5,
            },
            HomePosition::new(0.0, 0.0),
            CreatureState {
                behavior: BehaviorMode::Wandering,
                ..Default::default()
            },
            BodySize::default(),
            CanWander,
        ));

        run_wander_system(&mut world);

        let accel = world.query::<&Acceleration>().single(&world);
        let force_mag = (accel.ax * accel.ax + accel.ay * accel.ay).sqrt();

        assert!(force_mag > 0.0, "Wander should produce some steering force");
    }

    #[test]
    fn test_wander_respects_force_limit() {
        let mut world = World::new();

        let size = BodySize::default();
        let max_wander_force = size.max_force() * WANDER_FORCE_MULT.get();

        world.spawn((
            Position { x: 0.0, y: 0.0 },
            Velocity { vx: 0.0, vy: 0.0 },
            Acceleration::default(),
            WanderState {
                wander_angle: 0.0,
                wander_radius: 10.0,
                wander_distance: 50.0,
                angle_change: 4.5,
            },
            HomePosition::new(0.0, 0.0),
            CreatureState {
                behavior: BehaviorMode::Wandering,
                ..Default::default()
            },
            size,
            CanWander,
        ));

        run_wander_system(&mut world);

        let accel = world.query::<&Acceleration>().single(&world);
        let force_mag = (accel.ax * accel.ax + accel.ay * accel.ay).sqrt();

        assert!(
            force_mag <= max_wander_force + 0.01,
            "Wander force ({:.2}) should not exceed limit ({:.2})",
            force_mag,
            max_wander_force
        );
    }

    #[test]
    fn test_non_wandering_creatures_ignored() {
        let mut world = World::new();

        world.spawn((
            Position { x: 0.0, y: 0.0 },
            Velocity { vx: 1.0, vy: 0.0 },
            Acceleration::default(),
            WanderState::default(),
            HomePosition::new(0.0, 0.0),
            CreatureState {
                behavior: BehaviorMode::Seeking, // Not wandering!
                ..Default::default()
            },
            BodySize::default(),
            CanWander,
        ));

        run_wander_system(&mut world);

        let accel = world.query::<&Acceleration>().single(&world);
        assert_eq!(accel.ax, 0.0, "Non-wandering creature should have no wander force");
        assert_eq!(accel.ay, 0.0, "Non-wandering creature should have no wander force");
    }
}
