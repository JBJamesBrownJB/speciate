use crate::simulation::creatures::constants::WANDER_FORCE_MULT;
use crate::simulation::creatures::components::BehaviorMode;
use crate::simulation::math::SteeringContext;
use crate::simulation::queries::WanderQuery;
use super::steering::{calculate_wander, WanderParams};
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
///
/// FIXED: Now uses proper F=ma physics via `calculate_wander()` pure function.
/// Previously, steering (m/s) was incorrectly treated as force (N).
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

        // Build steering context for pure function
        let ctx = SteeringContext {
            velocity: (velocity.vx, velocity.vy),
            max_speed: creature_state.max_speed,
            max_force: size.max_force(),
            mass: size.mass(),
        };

        // Build wander parameters
        let wander_params = WanderParams {
            wander_angle: wander_state.wander_angle,
            wander_radius: wander_state.wander_radius,
            wander_distance: wander_state.wander_distance,
            force_multiplier: WANDER_FORCE_MULT.get(),
        };

        // Generate random angle change (in radians)
        let angle_change_deg = rng.gen_range(-wander_state.angle_change..wander_state.angle_change);
        let angle_change_rad = angle_change_deg.to_radians();

        // Calculate wander steering using pure function (correct F=ma physics)
        let result = calculate_wander(&ctx, &wander_params, angle_change_rad);

        // Update wander state
        wander_state.wander_angle = result.new_wander_angle;

        // Apply acceleration (already in m/s², not force!)
        let (ax, ay) = result.acceleration;
        if ax.is_finite() && ay.is_finite() {
            acceleration.ax += ax;
            acceleration.ay += ay;
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
    fn test_wander_respects_acceleration_limit() {
        // FIXED: Now tests acceleration limit (m/s²), not force limit (N)
        // The F=ma conversion is: max_accel = (max_force × multiplier) / mass
        let mut world = World::new();

        let size = BodySize::default();
        // Correct physics: max_accel = (max_force × WANDER_FORCE_MULT) / mass
        let max_wander_accel = (size.max_force() * WANDER_FORCE_MULT.get()) / size.mass();

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
        let accel_mag = (accel.ax * accel.ax + accel.ay * accel.ay).sqrt();

        // For default creature: max_force=390N, mass=65kg, mult=0.1
        // max_wander_accel = (390 × 0.1) / 65 = 39 / 65 = 0.6 m/s²
        assert!(
            accel_mag <= max_wander_accel + 0.01,
            "Wander acceleration ({:.2} m/s²) should not exceed limit ({:.2} m/s²). \
             If this is ~39 m/s², the F=ma bug is still present!",
            accel_mag,
            max_wander_accel
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
