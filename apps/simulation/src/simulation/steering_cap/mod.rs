use crate::simulation::core::components::{Acceleration, BodySize};
use bevy_ecs::prelude::*;

/// Caps accumulated steering forces to the creature's physical maximum.
///
/// This system runs AFTER all behavior systems (wander, seek, flee, avoidance)
/// have added their forces to the Acceleration component, but BEFORE physics
/// integration. It ensures no creature can exceed its muscular force capacity,
/// regardless of how many motivations are driving it.
///
/// External forces (wind, collisions) should be applied AFTER this system,
/// as they're not limited by the creature's self-generated force capacity.
pub fn cap_accumulated_steering_system(
    mut query: Query<(&mut Acceleration, &BodySize)>,
    #[cfg(feature = "dev-tools")] timings: bevy_ecs::system::Res<
        crate::instrumentation::SystemTimings,
    >,
) {
    #[cfg(feature = "dev-tools")]
    crate::time_system!(timings, "steering_cap");

    for (mut accel, size) in query.iter_mut() {
        let max_accel = size.max_force() / size.mass();
        let mag_sq = accel.ax * accel.ax + accel.ay * accel.ay;
        let max_sq = max_accel * max_accel;

        if mag_sq > max_sq && mag_sq > 0.0001 {
            let scale = max_accel / mag_sq.sqrt();
            accel.ax *= scale;
            accel.ay *= scale;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::core::components::BodySize;
    use bevy_ecs::system::{IntoSystem, System};

    fn run_cap_system(world: &mut World) {
        #[cfg(feature = "dev-tools")]
        world.insert_resource(crate::instrumentation::SystemTimings::new());

        let mut system = IntoSystem::into_system(cap_accumulated_steering_system);
        system.initialize(world);
        system.run((), world);
        system.apply_deferred(world);
    }

    #[test]
    fn forces_under_max_pass_through_unchanged() {
        let mut world = World::new();

        // Default BodySize: mass=65kg, max_force=390N → max_accel=6 m/s²
        let entity = world
            .spawn((Acceleration { ax: 3.0, ay: 4.0 }, BodySize::default()))
            .id();

        // mag = 5.0, which is under max_accel of 6.0
        run_cap_system(&mut world);

        let accel = world.get::<Acceleration>(entity).unwrap();
        assert!(
            (accel.ax - 3.0).abs() < 0.001,
            "ax should be unchanged: {}",
            accel.ax
        );
        assert!(
            (accel.ay - 4.0).abs() < 0.001,
            "ay should be unchanged: {}",
            accel.ay
        );
    }

    #[test]
    fn forces_exceeding_max_get_capped() {
        let mut world = World::new();

        let body = BodySize::default();
        let max_accel = body.max_force() / body.mass();

        // Set acceleration to 166% of max (3:4 ratio for direction test)
        let over_accel = max_accel * 1.66;
        let ax = over_accel * 0.6; // 3/5
        let ay = over_accel * 0.8; // 4/5

        let entity = world
            .spawn((Acceleration { ax, ay }, body))
            .id();

        run_cap_system(&mut world);

        let accel = world.get::<Acceleration>(entity).unwrap();
        let mag = (accel.ax * accel.ax + accel.ay * accel.ay).sqrt();

        assert!(
            (mag - max_accel).abs() < 0.01,
            "magnitude {} should be capped to max_accel {}",
            mag,
            max_accel
        );

        // Direction should be preserved (3:4 ratio)
        let ratio = accel.ax / accel.ay;
        assert!(
            (ratio - 0.75).abs() < 0.01,
            "direction should be preserved: ax/ay = {} (expected 0.75)",
            ratio
        );
    }

    #[test]
    fn zero_acceleration_unchanged() {
        let mut world = World::new();

        let entity = world
            .spawn((Acceleration { ax: 0.0, ay: 0.0 }, BodySize::default()))
            .id();

        run_cap_system(&mut world);

        let accel = world.get::<Acceleration>(entity).unwrap();
        assert_eq!(accel.ax, 0.0);
        assert_eq!(accel.ay, 0.0);
    }

    #[test]
    fn combined_forces_at_150_percent_capped_to_100() {
        let mut world = World::new();

        let body = BodySize::default();
        let max_accel = body.max_force() / body.mass();

        // Simulate combined forces at 150% of max (e.g., wander + avoidance)
        let over_accel = max_accel * 1.5;
        let entity = world
            .spawn((Acceleration { ax: over_accel, ay: 0.0 }, body))
            .id();

        run_cap_system(&mut world);

        let accel = world.get::<Acceleration>(entity).unwrap();

        assert!(
            (accel.ax - max_accel).abs() < 0.01,
            "ax {} should be capped to max_accel {}",
            accel.ax,
            max_accel
        );
        assert!(
            accel.ay.abs() < 0.001,
            "ay should remain 0: {}",
            accel.ay
        );
    }

    #[test]
    fn different_body_sizes_have_different_caps() {
        let mut world = World::new();

        // Small creature: length=0.5 → radius=0.25m
        let small = world
            .spawn((
                Acceleration { ax: 10.0, ay: 0.0 },
                BodySize::new(0.5),
            ))
            .id();

        // Large creature: length=4.0 → radius=2.0m
        let large = world
            .spawn((
                Acceleration { ax: 10.0, ay: 0.0 },
                BodySize::new(4.0),
            ))
            .id();

        run_cap_system(&mut world);

        let small_accel = world.get::<Acceleration>(small).unwrap();
        let large_accel = world.get::<Acceleration>(large).unwrap();

        let small_size = BodySize::new(0.5);
        let large_size = BodySize::new(4.0);

        let small_max = small_size.max_force() / small_size.mass();
        let large_max = large_size.max_force() / large_size.mass();

        // Both should be capped to their respective max_accel values
        assert!(
            (small_accel.ax - small_max).abs() < 0.1,
            "small creature ax {} should be ~{}",
            small_accel.ax,
            small_max
        );
        assert!(
            (large_accel.ax - large_max).abs() < 0.1,
            "large creature ax {} should be ~{}",
            large_accel.ax,
            large_max
        );
    }
}
