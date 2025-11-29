use super::constants::{AVOIDANCE_FORCE, PANIC_FORCE, SEEKING_PERSONAL_SPACE_BUFFER};
use crate::simulation::core::components::*;
use crate::simulation::math::{clamp_force, magnitude_sq};
use crate::simulation::perception::constants::PANIC_THRESHOLD_RATIO;
use crate::simulation::queries::AvoidanceQuery;
use bevy_ecs::prelude::*;

pub fn avoidance_system(
    mut query: AvoidanceQuery,
    others: Query<(&Position, &BodySize)>,
    #[cfg(feature = "dev-tools")] timings: bevy_ecs::system::Res<
        crate::instrumentation::SystemTimings,
    >,
) {
    #[cfg(feature = "dev-tools")]
    crate::time_system!(timings, "avoidance");

    for (entity, position, size, mut acceleration, perception, avoidance, state) in query.iter_mut() {
        if !perception.has_neighbors() {
            continue;
        }

        // Seeking creatures tolerate very close proximity (body + buffer)
        // Non-seeking creatures use energy-based personal space
        let effective_space = if state.behavior == crate::simulation::creatures::components::BehaviorMode::Seeking {
            size.radius() + SEEKING_PERSONAL_SPACE_BUFFER
        } else {
            let energy_fraction = state.energy / 100.0;
            avoidance.effective_personal_space(energy_fraction)
        };

        let panic_threshold = effective_space * PANIC_THRESHOLD_RATIO;
        let self_radius = size.radius();

        let mut total_repulsion_x = 0.0;
        let mut total_repulsion_y = 0.0;

        for other_entity in perception.iter_neighbors() {
            if other_entity == entity {
                continue;
            }

            let Ok((other_pos, other_size)) = others.get(other_entity) else {
                continue;
            };

            let away_x = position.x - other_pos.x;
            let away_y = position.y - other_pos.y;
            let center_distance_sq = magnitude_sq(away_x, away_y);

            let other_radius = other_size.radius();
            let max_combined_radius = self_radius + other_radius;
            let max_interaction_distance = effective_space + max_combined_radius;
            let max_interaction_distance_sq = max_interaction_distance * max_interaction_distance;

            if center_distance_sq > max_interaction_distance_sq {
                continue;
            }

            let center_distance = center_distance_sq.sqrt();

            if center_distance < 0.001 {
                continue;
            }

            let edge_distance = center_distance - self_radius - other_radius;
            let safe_distance = edge_distance.max(0.01);

            if safe_distance < effective_space {
                let ratio = effective_space / safe_distance;
                let mut force_magnitude = AVOIDANCE_FORCE * ratio * ratio;

                if safe_distance < panic_threshold {
                    force_magnitude = force_magnitude.min(PANIC_FORCE);
                }

                let force_x = (away_x / center_distance) * force_magnitude;
                let force_y = (away_y / center_distance) * force_magnitude;

                total_repulsion_x += force_x;
                total_repulsion_y += force_y;
            }
        }

        let (clamped_x, clamped_y) = clamp_force(total_repulsion_x, total_repulsion_y, avoidance.max_force);
        acceleration.ax += clamped_x;
        acceleration.ay += clamped_y;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::IntoSystem;
    use crate::simulation::components::*;
    use crate::simulation::perception::{AvoidanceBehavior, Perception};

    fn run_system(world: &mut World) {
        #[cfg(feature = "dev-tools")]
        world.insert_resource(crate::instrumentation::SystemTimings::new());

        let mut system = IntoSystem::into_system(avoidance_system);
        system.initialize(world);
        system.run((), world);
        system.apply_deferred(world);
    }

    #[test]
    fn test_no_avoidance_when_no_neighbors() {
        let mut world = World::new();

        let crit = world
            .spawn((
                Position { x: 0.0, y: 0.0 },
                Velocity::default(),
                Acceleration::default(),
                Perception::new(10.0),
                AvoidanceBehavior::new(2.5, 15.0),
                BodySize::default(),
                CreatureState::default(),
                CanAvoidObstacles,
            ))
            .id();

        run_system(&mut world);

        let accel = world.get::<Acceleration>(crit).unwrap();
        assert_eq!(accel.ax, 0.0);
        assert_eq!(accel.ay, 0.0);
    }

    #[test]
    fn test_avoidance_repulsion_within_personal_space() {
        let mut world = World::new();

        let crit1 = world
            .spawn((
                Position { x: 0.0, y: 0.0 },
                Velocity::default(),
                Acceleration::default(),
                Perception::new(10.0),
                AvoidanceBehavior::new(2.5, 15.0),
                BodySize::default(),
                CreatureState::default(),
                CanAvoidObstacles,
            ))
            .id();

        let crit2 = world
            .spawn((Position { x: 1.0, y: 0.0 }, BodySize::default()))
            .id();

        if let Some(mut p) = world.get_mut::<Perception>(crit1) {
            p.add_neighbor(crit2);
        }

        run_system(&mut world);

        let accel = world.get::<Acceleration>(crit1).unwrap();
        assert!(accel.ax < 0.0, "Should be repelled in -X direction");
        assert_eq!(accel.ay, 0.0, "No Y component");
        assert!(accel.ax.abs() > 0.0);
    }

    #[test]
    fn test_no_avoidance_outside_personal_space() {
        let mut world = World::new();

        let crit1 = world
            .spawn((
                Position { x: 0.0, y: 0.0 },
                Velocity::default(),
                Acceleration::default(),
                Perception::new(10.0),
                AvoidanceBehavior::new(2.5, 15.0),
                BodySize::default(),
                CreatureState::default(),
                CanAvoidObstacles,
            ))
            .id();

        let crit2 = world
            .spawn((Position { x: 5.0, y: 0.0 }, BodySize::default()))
            .id();

        if let Some(mut p) = world.get_mut::<Perception>(crit1) {
            p.add_neighbor(crit2);
        }

        run_system(&mut world);

        let accel = world.get::<Acceleration>(crit1).unwrap();
        assert_eq!(accel.ax, 0.0, "No force outside personal space");
        assert_eq!(accel.ay, 0.0);
    }

    #[test]
    fn test_panic_force_cap() {
        let mut world = World::new();

        let crit1 = world
            .spawn((
                Position { x: 0.0, y: 0.0 },
                Velocity::default(),
                Acceleration::default(),
                Perception::new(10.0),
                AvoidanceBehavior::new(2.5, 15.0),
                BodySize::default(),
                CreatureState::default(),
                CanAvoidObstacles,
            ))
            .id();

        let crit2 = world
            .spawn((Position { x: 0.5, y: 0.0 }, BodySize::default()))
            .id();

        if let Some(mut p) = world.get_mut::<Perception>(crit1) {
            p.add_neighbor(crit2);
        }

        run_system(&mut world);

        let accel = world.get::<Acceleration>(crit1).unwrap();
        let force_mag = (accel.ax * accel.ax + accel.ay * accel.ay).sqrt();

        assert!(
            force_mag <= PANIC_FORCE,
            "Force should be capped at {}N, got: {:.2}N",
            PANIC_FORCE,
            force_mag
        );
        assert!(force_mag > 0.0);
    }

    #[test]
    fn test_inverse_square_scaling() {
        let avoidance = AvoidanceBehavior::new(2.5, 15.0);
        let base_force = AVOIDANCE_FORCE;

        let test_cases = vec![
            (2.0_f32, base_force * (2.5_f32 / 2.0_f32).powi(2)),
            (1.5_f32, base_force * (2.5_f32 / 1.5_f32).powi(2)),
            (1.0_f32, base_force * (2.5_f32 / 1.0_f32).powi(2)),
        ];

        for (distance, expected_force) in test_cases {
            let ratio = avoidance.personal_space / distance;
            let calculated_force = base_force * ratio * ratio;

            let final_force = if distance < avoidance.panic_threshold() {
                calculated_force.min(PANIC_FORCE)
            } else {
                calculated_force
            };

            let expected_final = if distance < avoidance.panic_threshold() {
                expected_force.min(PANIC_FORCE)
            } else {
                expected_force
            };

            assert!(
                (final_force - expected_final).abs() < 0.01,
                "Force at {:.1}m: expected {:.2}N, got {:.2}N",
                distance,
                expected_final,
                final_force
            );
        }
    }

    #[test]
    fn test_multiple_obstacles_accumulation() {
        let mut world = World::new();

        let crit1 = world
            .spawn((
                Position { x: 0.0, y: 0.0 },
                Velocity::default(),
                Acceleration::default(),
                Perception::new(10.0),
                AvoidanceBehavior::new(2.5, 15.0),
                BodySize::default(),
                CreatureState::default(),
                CanAvoidObstacles,
            ))
            .id();

        let crit2 = world
            .spawn((Position { x: 1.0, y: 0.0 }, BodySize::default()))
            .id();
        let crit3 = world
            .spawn((Position { x: 0.0, y: 1.0 }, BodySize::default()))
            .id();

        if let Some(mut p) = world.get_mut::<Perception>(crit1) {
            p.add_neighbor(crit2);
            p.add_neighbor(crit3);
        }

        run_system(&mut world);

        let accel = world.get::<Acceleration>(crit1).unwrap();

        assert!(accel.ax < 0.0, "Should be repelled in -X");
        assert!(accel.ay < 0.0, "Should be repelled in -Y");

        assert!(
            (accel.ax.abs() - accel.ay.abs()).abs() < 0.01,
            "Forces from equidistant obstacles should be equal"
        );
    }

    #[test]
    fn test_hungry_creatures_reduce_personal_space() {
        let avoidance = AvoidanceBehavior::new(10.0, 35.0);

        let full_energy_space = avoidance.effective_personal_space(1.0);
        let half_energy_space = avoidance.effective_personal_space(0.5);
        let zero_energy_space = avoidance.effective_personal_space(0.0);

        assert!(half_energy_space < full_energy_space, "Hungry creatures should have reduced space");
        assert!(zero_energy_space < half_energy_space, "Starving creatures should have even less space");
        assert!((zero_energy_space - 4.0).abs() < 0.001, "Should be 40% of base at 0 energy");
    }

    #[test]
    fn test_low_energy_tolerates_closer_proximity() {
        let mut world = World::new();

        let high_energy_crit = world
            .spawn((
                Position { x: 0.0, y: 0.0 },
                Velocity::default(),
                Acceleration::default(),
                Perception::new(10.0),
                AvoidanceBehavior::new(2.5, 15.0),
                BodySize::default(),
                CreatureState { energy: 100.0, ..Default::default() },
                CanAvoidObstacles,
            ))
            .id();

        let low_energy_crit = world
            .spawn((
                Position { x: 10.0, y: 0.0 },
                Velocity::default(),
                Acceleration::default(),
                Perception::new(10.0),
                AvoidanceBehavior::new(2.5, 15.0),
                BodySize::default(),
                CreatureState { energy: 10.0, ..Default::default() },
                CanAvoidObstacles,
            ))
            .id();

        let obstacle = world
            .spawn((Position { x: 1.0, y: 0.0 }, BodySize::default()))
            .id();

        if let Some(mut p) = world.get_mut::<Perception>(high_energy_crit) {
            p.add_neighbor(obstacle);
        }
        if let Some(mut p) = world.get_mut::<Perception>(low_energy_crit) {
            p.add_neighbor(obstacle);
        }

        run_system(&mut world);

        let high_energy_accel = world.get::<Acceleration>(high_energy_crit).unwrap();
        let low_energy_accel = world.get::<Acceleration>(low_energy_crit).unwrap();

        let high_energy_force = high_energy_accel.ax.abs();
        let low_energy_force = low_energy_accel.ax.abs();

        assert!(
            low_energy_force < high_energy_force,
            "Low energy creature should experience less repulsion (tolerates closer proximity). High: {:.2}, Low: {:.2}",
            high_energy_force,
            low_energy_force
        );
    }

    #[test]
    fn test_seeking_overrides_personal_space() {
        let mut world = World::new();

        // Seeker with 0.5 radius at full energy - should tolerate very close proximity
        let seeker = world
            .spawn((
                Position { x: 0.0, y: 0.0 },
                Velocity::default(),
                Acceleration::default(),
                Perception::new(10.0),
                AvoidanceBehavior::new(2.5, 15.0),
                BodySize::new(1.0),  // radius = 0.5
                CreatureState {
                    behavior: BehaviorMode::Seeking,
                    energy: 100.0,
                    ..Default::default()
                },
                CanAvoidObstacles,
            ))
            .id();

        // Wanderer with same config - uses energy-based space
        let wanderer = world
            .spawn((
                Position { x: 10.0, y: 0.0 },
                Velocity::default(),
                Acceleration::default(),
                Perception::new(10.0),
                AvoidanceBehavior::new(2.5, 15.0),
                BodySize::new(1.0),
                CreatureState {
                    behavior: BehaviorMode::Wandering,
                    energy: 100.0,
                    ..Default::default()
                },
                CanAvoidObstacles,
            ))
            .id();

        // Both creatures 2m from their respective obstacles (edge distance = 1m)
        // This is outside seeker's comfort (0.6m) but inside wanderer's (2.5m)
        let seeker_obstacle = world
            .spawn((Position { x: 2.0, y: 0.0 }, BodySize::new(1.0)))
            .id();

        let wanderer_obstacle = world
            .spawn((Position { x: 12.0, y: 0.0 }, BodySize::new(1.0)))
            .id();

        // Add obstacles to respective perceptions
        if let Some(mut p) = world.get_mut::<Perception>(seeker) {
            p.add_neighbor(seeker_obstacle);
        }
        if let Some(mut p) = world.get_mut::<Perception>(wanderer) {
            p.add_neighbor(wanderer_obstacle);
        }

        run_system(&mut world);

        let seeker_accel = world.get::<Acceleration>(seeker).unwrap();
        let wanderer_accel = world.get::<Acceleration>(wanderer).unwrap();

        let seeker_force = seeker_accel.ax.abs();
        let wanderer_force = wanderer_accel.ax.abs();

        // Seeker uses body + 0.1 = 0.6m effective space (edge_distance 1m > 0.6m = no force)
        // Wanderer uses energy-based = 2.5m effective space (edge_distance 1m < 2.5m = force)
        // Seeker should have 0 or very low force, wanderer should have significant force
        assert!(
            seeker_force < 0.1,
            "Seeking creature should experience minimal repulsion at 1m edge distance. Seeker: {:.2}",
            seeker_force
        );
        assert!(
            wanderer_force > 1.0,
            "Wandering creature should experience significant repulsion at 1m edge distance. Wanderer: {:.2}",
            wanderer_force
        );
    }
}
