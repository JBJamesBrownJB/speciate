use crate::simulation::creatures::constants::{EMERGENCY_BRAKE_DISTANCE, SEEKING_SPACE_REDUCTION};
use crate::simulation::math::magnitude_sq;
use crate::simulation::queries::AvoidanceQuery;
use rayon::prelude::*;

// Minimum speed² below which we allow full avoidance (can't define "forward" when stationary)
const MIN_SPEED_SQ_FOR_STEERING: f32 = 0.01;

pub fn avoidance_system(
    mut query: AvoidanceQuery,
    #[cfg(feature = "dev-tools")] timings: bevy_ecs::system::Res<
        crate::instrumentation::SystemTimings,
    >,
) {
    #[cfg(feature = "dev-tools")]
    crate::time_system!(timings, "avoidance");

    // Collect entities for parallel processing
    let mut entities: Vec<_> = query.iter_mut().collect();

    entities.par_iter_mut().for_each(|(entity, position, velocity, size, acceleration, _perception, neighbor_cache, avoidance, state)| {
        if !neighbor_cache.has_neighbors() {
            return;
        }

        // Seeking creatures tolerate closer proximity (reduced personal space)
        // Non-seeking creatures use energy-based personal space
        let effective_space = if state.behavior == crate::simulation::creatures::components::BehaviorMode::Seeking {
            avoidance.personal_space() * SEEKING_SPACE_REDUCTION
        } else {
            let energy_fraction = state.energy / 100.0;
            avoidance.effective_personal_space(energy_fraction)
        };

        // Emergency brake: apply max force when very close (simple fixed threshold)
        let self_radius = size.radius();
        let max_force = size.max_force();
        let mass = size.mass();

        // CRITICAL: F=ma conversion - max_accel = max_force / mass
        // Without this, a 65kg creature would experience 390 m/s² instead of 6 m/s²!
        let max_accel = max_force / mass;

        let mut total_repulsion_x = 0.0;
        let mut total_repulsion_y = 0.0;

        // Pre-compute self_radius contribution (constant across all neighbors)
        let base_interaction = effective_space + self_radius;

        // Read neighbor positions directly from NeighborCache (cached during perception phase)
        for neighbor in neighbor_cache.iter_neighbors() {
            if neighbor.entity == *entity {
                continue;
            }

            let away_x = position.x - neighbor.x;
            let away_y = position.y - neighbor.y;
            let center_distance_sq = magnitude_sq(away_x, away_y);

            // Early exit: degenerate case (squared space - avoid sqrt)
            if center_distance_sq < 0.000001 {
                continue;
            }

            let max_interaction_distance = base_interaction + neighbor.radius;
            let max_interaction_distance_sq = max_interaction_distance * max_interaction_distance;

            // Early exit: outside interaction range (squared space)
            if center_distance_sq > max_interaction_distance_sq {
                continue;
            }

            // Compute distance and inverse for direction normalization
            let center_distance = center_distance_sq.sqrt();
            let inv_distance = 1.0 / center_distance;

            let edge_distance = center_distance - self_radius - neighbor.radius;
            let safe_distance = edge_distance.max(0.01);

            // Note: Removed redundant `if safe_distance < effective_space` check
            // because max_interaction_distance filter already guarantees this condition

            // Urgency scales with inverse square of distance
            let ratio = effective_space / safe_distance;
            let urgency = ratio * ratio;

            // Emergency brake: max ACCELERATION when very close, otherwise scale by urgency
            // Note: We use max_accel (m/s²), NOT max_force (Newtons)!
            let accel_magnitude = if safe_distance < EMERGENCY_BRAKE_DISTANCE {
                max_accel
            } else {
                (max_accel * urgency).min(max_accel)
            };

            // Use pre-computed inv_distance for direction (avoids division)
            let accel_x = away_x * inv_distance * accel_magnitude;
            let accel_y = away_y * inv_distance * accel_magnitude;

            total_repulsion_x += accel_x;
            total_repulsion_y += accel_y;
        }

        // Avoidance = BRAKING + STEERING, never forward acceleration
        // - Braking: slow down when heading toward obstacle (dot < 0)
        // - Steering: lateral deflection (perpendicular component)
        // - NO forward thrust: never speed up from avoidance (remove dot > 0 component)
        let speed_sq = magnitude_sq(velocity.vx, velocity.vy);
        let (steer_x, steer_y) = if speed_sq > MIN_SPEED_SQ_FOR_STEERING {
            // dot > 0: avoidance pushes same direction as velocity (FORWARD - bad!)
            // dot < 0: avoidance pushes opposite to velocity (BRAKING - good!)
            let dot = total_repulsion_x * velocity.vx + total_repulsion_y * velocity.vy;

            if dot > 0.0 {
                // Obstacle behind us - remove forward component, keep only lateral steering
                let parallel_factor = dot / speed_sq;
                (
                    total_repulsion_x - parallel_factor * velocity.vx,
                    total_repulsion_y - parallel_factor * velocity.vy,
                )
            } else {
                // Obstacle ahead/side - keep full force (braking + steering)
                (total_repulsion_x, total_repulsion_y)
            }
        } else {
            // Stationary: allow full avoidance
            (total_repulsion_x, total_repulsion_y)
        };

        // Add avoidance force to accumulated acceleration
        // Final capping is done by cap_accumulated_steering_system after all behaviors
        acceleration.ax += steer_x;
        acceleration.ay += steer_y;
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use bevy_ecs::system::{IntoSystem, System};
    use crate::simulation::core::components::{Acceleration, BodySize, Position, Velocity};
    use crate::simulation::creatures::components::{BehaviorMode, CanAvoidObstacles, CreatureState};
    use crate::simulation::perception::{AvoidanceBehavior, NeighborCache, NeighborData, Perception};

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
                Perception::from_body_size(1.0),
                NeighborCache::new(),
                AvoidanceBehavior::new(1.25),
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
                Perception::from_body_size(1.0),
                NeighborCache::new(),
                AvoidanceBehavior::new(1.25),
                BodySize::default(),
                CreatureState::default(),
                CanAvoidObstacles,
            ))
            .id();

        let crit2 = world
            .spawn((Position { x: 1.0, y: 0.0 }, BodySize::default()))
            .id();

        if let Some(mut c) = world.get_mut::<NeighborCache>(crit1) {
            c.add_neighbor(NeighborData { entity: crit2, x: 1.0, y: 0.0, radius: 0.5 });
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
                Perception::from_body_size(1.0),
                NeighborCache::new(),
                AvoidanceBehavior::new(1.25),
                BodySize::default(),
                CreatureState::default(),
                CanAvoidObstacles,
            ))
            .id();

        let crit2 = world
            .spawn((Position { x: 5.0, y: 0.0 }, BodySize::default()))
            .id();

        if let Some(mut c) = world.get_mut::<NeighborCache>(crit1) {
            c.add_neighbor(NeighborData { entity: crit2, x: 5.0, y: 0.0, radius: 0.5 });
        }

        run_system(&mut world);

        let accel = world.get::<Acceleration>(crit1).unwrap();
        assert_eq!(accel.ax, 0.0, "No force outside personal space");
        assert_eq!(accel.ay, 0.0);
    }

    #[test]
    fn test_acceleration_capped_at_max_accel() {
        let mut world = World::new();

        let size = BodySize::default();
        let max_force = size.max_force();
        let mass = size.mass();
        let max_accel = max_force / mass; // F=ma → a=F/m

        let crit1 = world
            .spawn((
                Position { x: 0.0, y: 0.0 },
                Velocity::default(),
                Acceleration::default(),
                Perception::from_body_size(1.0),
                NeighborCache::new(),
                AvoidanceBehavior::new(1.25),
                size,
                CreatureState::default(),
                CanAvoidObstacles,
            ))
            .id();

        let crit2 = world
            .spawn((Position { x: 0.5, y: 0.0 }, BodySize::default()))
            .id();

        if let Some(mut c) = world.get_mut::<NeighborCache>(crit1) {
            c.add_neighbor(NeighborData { entity: crit2, x: 0.5, y: 0.0, radius: 0.5 });
        }

        run_system(&mut world);

        let accel = world.get::<Acceleration>(crit1).unwrap();
        let accel_mag = (accel.ax * accel.ax + accel.ay * accel.ay).sqrt();

        // Should be capped at max_accel (F/m), NOT max_force
        assert!(
            accel_mag <= max_accel * 1.01, // 1% tolerance for float rounding
            "Acceleration should be capped at max_accel ({:.2} m/s² = {:.0}N / {:.0}kg), got: {:.2} m/s²",
            max_accel,
            max_force,
            mass,
            accel_mag
        );
        assert!(accel_mag > 0.0);
    }

    #[test]
    fn test_inverse_square_urgency_scaling() {
        // Verify urgency scales with inverse square of distance
        let body_radius = 1.25; // Gives personal_space = 1.25 × 2.0 = 2.5
        let avoidance = AvoidanceBehavior::new(body_radius);
        let personal_space = avoidance.personal_space();
        let size = BodySize::default();
        let max_force = size.max_force();
        let mass = size.mass();
        let max_accel = max_force / mass; // F=ma conversion!

        // Test urgency calculation at various distances
        let test_cases = vec![
            (2.0_f32, (personal_space / 2.0_f32).powi(2)),   // ratio=1.25, urgency=1.5625
            (1.5_f32, (personal_space / 1.5_f32).powi(2)),   // ratio=1.67, urgency=2.78
            (1.0_f32, (personal_space / 1.0_f32).powi(2)),   // ratio=2.5, urgency=6.25
        ];

        for (distance, expected_urgency) in test_cases {
            let ratio = personal_space / distance;
            let urgency = ratio * ratio;

            // Acceleration = min(max_accel * urgency, max_accel) - capped at max_accel
            let expected_accel = (max_accel * expected_urgency).min(max_accel);
            let calculated_accel = (max_accel * urgency).min(max_accel);

            assert!(
                (calculated_accel - expected_accel).abs() < 0.01,
                "Accel at {:.1}m: expected {:.2} m/s², got {:.2} m/s² (urgency={:.2})",
                distance,
                expected_accel,
                calculated_accel,
                urgency
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
                Perception::from_body_size(1.0),
                NeighborCache::new(),
                AvoidanceBehavior::new(1.25),
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

        if let Some(mut c) = world.get_mut::<NeighborCache>(crit1) {
            c.add_neighbor(NeighborData { entity: crit2, x: 1.0, y: 0.0, radius: 0.5 });
            c.add_neighbor(NeighborData { entity: crit3, x: 0.0, y: 1.0, radius: 0.5 });
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
        use crate::simulation::creatures::constants::ENERGY_MODIFIER;

        // body_radius = 5.0 → personal_space = 5.0 × 2.0 = 10.0
        let body_radius = 5.0;
        let avoidance = AvoidanceBehavior::new(body_radius);
        let personal_space = avoidance.personal_space();

        let full_energy_space = avoidance.effective_personal_space(1.0);
        let half_energy_space = avoidance.effective_personal_space(0.5);
        let zero_energy_space = avoidance.effective_personal_space(0.0);

        assert!(half_energy_space < full_energy_space, "Hungry creatures should have reduced space");
        assert!(zero_energy_space < half_energy_space, "Starving creatures should have even less space");
        let expected_min = personal_space * ENERGY_MODIFIER.min_modifier;
        assert!((zero_energy_space - expected_min).abs() < 0.001,
            "Should be {}% of base at 0 energy", ENERGY_MODIFIER.min_modifier * 100.0);
    }

    #[test]
    fn test_low_energy_tolerates_closer_proximity() {
        let mut world = World::new();

        let high_energy_crit = world
            .spawn((
                Position { x: 0.0, y: 0.0 },
                Velocity::default(),
                Acceleration::default(),
                Perception::from_body_size(1.0),
                NeighborCache::new(),
                AvoidanceBehavior::new(1.25),
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
                Perception::from_body_size(1.0),
                NeighborCache::new(),
                AvoidanceBehavior::new(1.25),
                BodySize::default(),
                CreatureState { energy: 10.0, ..Default::default() },
                CanAvoidObstacles,
            ))
            .id();

        let obstacle = world
            .spawn((Position { x: 1.0, y: 0.0 }, BodySize::default()))
            .id();

        if let Some(mut c) = world.get_mut::<NeighborCache>(high_energy_crit) {
            c.add_neighbor(NeighborData { entity: obstacle, x: 1.0, y: 0.0, radius: 0.5 });
        }
        if let Some(mut c) = world.get_mut::<NeighborCache>(low_energy_crit) {
            c.add_neighbor(NeighborData { entity: obstacle, x: 1.0, y: 0.0, radius: 0.5 });
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
                Perception::from_body_size(1.0),
                NeighborCache::new(),
                AvoidanceBehavior::new(1.25),
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
                Perception::from_body_size(1.0),
                NeighborCache::new(),
                AvoidanceBehavior::new(1.25),
                BodySize::new(1.0),
                CreatureState {
                    behavior: BehaviorMode::Wandering,
                    energy: 100.0,
                    ..Default::default()
                },
                CanAvoidObstacles,
            ))
            .id();

        // New model: seeker_space = personal_space × 0.5 = 1.25m, wanderer_space = 2.5m
        // Place seeker obstacle OUTSIDE seeker's space (edge_distance > 1.25m)
        // Place wanderer obstacle INSIDE wanderer's space (edge_distance < 2.5m)
        let seeker_obstacle = world
            .spawn((Position { x: 3.0, y: 0.0 }, BodySize::new(1.0))) // edge_dist = 3.0 - 0.5 - 0.5 = 2.0m > 1.25m
            .id();

        let wanderer_obstacle = world
            .spawn((Position { x: 12.5, y: 0.0 }, BodySize::new(1.0))) // edge_dist = 2.5 - 1.0 = 1.5m < 2.5m
            .id();

        // Add obstacles to respective neighbor caches
        if let Some(mut c) = world.get_mut::<NeighborCache>(seeker) {
            c.add_neighbor(NeighborData { entity: seeker_obstacle, x: 3.0, y: 0.0, radius: 0.5 });
        }
        if let Some(mut c) = world.get_mut::<NeighborCache>(wanderer) {
            c.add_neighbor(NeighborData { entity: wanderer_obstacle, x: 12.5, y: 0.0, radius: 0.5 });
        }

        run_system(&mut world);

        let seeker_accel = world.get::<Acceleration>(seeker).unwrap();
        let wanderer_accel = world.get::<Acceleration>(wanderer).unwrap();

        let seeker_force = seeker_accel.ax.abs();
        let wanderer_force = wanderer_accel.ax.abs();

        // Seeker effective_space = 2.5 × 0.5 = 1.25m, edge_distance = 2.0m > 1.25m = no force
        // Wanderer effective_space = 2.5m, edge_distance = 1.5m < 2.5m = force
        assert!(
            seeker_force < 0.1,
            "Seeking creature should tolerate closer proximity. Seeker: {:.2} (edge_dist 2.0m > space 1.25m)",
            seeker_force
        );
        assert!(
            wanderer_force > 1.0,
            "Wandering creature should experience repulsion. Wanderer: {:.2} (edge_dist 1.5m < space 2.5m)",
            wanderer_force
        );
    }

    #[test]
    fn test_avoidance_is_perpendicular_to_velocity() {
        // Avoidance should be PURE STEERING - perpendicular to velocity only
        // An obstacle directly behind a moving creature should NOT push it forward
        let mut world = World::new();

        // Creature moving in +X direction
        let crit = world
            .spawn((
                Position { x: 0.0, y: 0.0 },
                Velocity { vx: 10.0, vy: 0.0 },  // Moving right
                Acceleration::default(),
                Perception::from_body_size(1.0),
                NeighborCache::new(),
                AvoidanceBehavior::new(1.25),
                BodySize::default(),
                CreatureState::default(),
                CanAvoidObstacles,
            ))
            .id();

        // Obstacle directly BEHIND the creature (in -X direction)
        let obstacle = world
            .spawn((Position { x: -1.5, y: 0.0 }, BodySize::default()))
            .id();

        if let Some(mut c) = world.get_mut::<NeighborCache>(crit) {
            c.add_neighbor(NeighborData { entity: obstacle, x: -1.5, y: 0.0, radius: 0.5 });
        }

        run_system(&mut world);

        let accel = world.get::<Acceleration>(crit).unwrap();

        // Obstacle behind should NOT produce forward acceleration
        // The "away" direction is +X, but since we're moving +X, this is parallel to velocity
        // Pure steering means NO forward component - only perpendicular allowed
        assert!(
            accel.ax.abs() < 1.0,  // Should be near-zero (allow small epsilon)
            "Avoidance should NOT accelerate forward. Obstacle behind moving creature produced ax={:.2}",
            accel.ax
        );
    }

    #[test]
    fn test_avoidance_braking_for_obstacle_directly_ahead() {
        // Obstacle directly ahead - should produce BRAKING force (slowing down)
        // This is the correct behavior: slow down when heading toward obstacle
        let mut world = World::new();

        // Creature moving in +X direction
        let crit = world
            .spawn((
                Position { x: 0.0, y: 0.0 },
                Velocity { vx: 10.0, vy: 0.0 },  // Moving right
                Acceleration::default(),
                Perception::from_body_size(1.0),
                NeighborCache::new(),
                AvoidanceBehavior::new(1.25),
                BodySize::default(),
                CreatureState::default(),
                CanAvoidObstacles,
            ))
            .id();

        // Obstacle directly AHEAD of the creature (in +X direction)
        let obstacle = world
            .spawn((Position { x: 1.5, y: 0.0 }, BodySize::default()))
            .id();

        if let Some(mut c) = world.get_mut::<NeighborCache>(crit) {
            c.add_neighbor(NeighborData { entity: obstacle, x: 1.5, y: 0.0, radius: 0.5 });
        }

        run_system(&mut world);

        let accel = world.get::<Acceleration>(crit).unwrap();

        // Obstacle directly ahead - should produce braking force (negative X)
        // Braking force opposes velocity direction
        assert!(
            accel.ax < -1.0,
            "Obstacle directly ahead should produce braking force. Got ax={:.2}",
            accel.ax
        );
        // Lateral force should be near-zero (obstacle is on the axis of travel)
        assert!(
            accel.ay.abs() < 0.1,
            "Obstacle directly ahead should produce no lateral force. Got ay={:.2}",
            accel.ay
        );
    }

    #[test]
    fn test_avoidance_lateral_force_for_side_obstacle() {
        // Obstacle to the side should produce lateral (perpendicular) avoidance
        let mut world = World::new();

        // Creature moving in +X direction
        let crit = world
            .spawn((
                Position { x: 0.0, y: 0.0 },
                Velocity { vx: 10.0, vy: 0.0 },  // Moving right
                Acceleration::default(),
                Perception::from_body_size(1.0),
                NeighborCache::new(),
                AvoidanceBehavior::new(1.25),
                BodySize::default(),
                CreatureState::default(),
                CanAvoidObstacles,
            ))
            .id();

        // Obstacle to the LEFT of the creature (in -Y direction relative to travel)
        let obstacle = world
            .spawn((Position { x: 0.0, y: 1.5 }, BodySize::default()))
            .id();

        if let Some(mut c) = world.get_mut::<NeighborCache>(crit) {
            c.add_neighbor(NeighborData { entity: obstacle, x: 0.0, y: 1.5, radius: 0.5 });
        }

        run_system(&mut world);

        let accel = world.get::<Acceleration>(crit).unwrap();

        // Obstacle to the side should produce LATERAL avoidance (perpendicular to velocity)
        // Moving +X, obstacle at +Y, so avoidance should push -Y (perpendicular steering)
        assert!(
            accel.ay < -1.0,  // Should have significant -Y component
            "Side obstacle should produce lateral avoidance. Expected ay < -1.0, got ay={:.2}",
            accel.ay
        );
        assert!(
            accel.ax.abs() < accel.ay.abs() * 0.1,  // X component should be negligible
            "Lateral avoidance should have minimal forward component. ax={:.2}, ay={:.2}",
            accel.ax, accel.ay
        );
    }

    #[test]
    fn test_panic_zone_uses_max_accel_when_stationary() {
        // When stationary (no velocity), panic zone uses full max_accel (F/m)
        // (can't project perpendicular without velocity)
        let mut world = World::new();

        let size = BodySize::new(1.0);
        let max_force = size.max_force();
        let mass = size.mass();
        let max_accel = max_force / mass; // F=ma conversion!

        // STATIONARY creature at origin with personal_space = 2.5
        let crit = world
            .spawn((
                Position { x: 0.0, y: 0.0 },
                Velocity::default(),  // Stationary!
                Acceleration::default(),
                Perception::from_body_size(1.0),
                NeighborCache::new(),
                AvoidanceBehavior::new(1.25),
                size,
                CreatureState::default(),
                CanAvoidObstacles,
            ))
            .id();

        // Place neighbor very close - inside panic zone
        let neighbor = world
            .spawn((Position { x: 1.1, y: 0.0 }, BodySize::new(1.0)))
            .id();

        if let Some(mut c) = world.get_mut::<NeighborCache>(crit) {
            c.add_neighbor(NeighborData { entity: neighbor, x: 1.1, y: 0.0, radius: 0.5 });
        }

        run_system(&mut world);

        let accel = world.get::<Acceleration>(crit).unwrap();
        let accel_mag = (accel.ax * accel.ax + accel.ay * accel.ay).sqrt();

        // Stationary creature in panic zone gets full avoidance acceleration
        assert!(
            accel_mag >= max_accel * 0.9,
            "Stationary creature in panic zone should get full accel. Expected ~{:.2} m/s², got {:.2} m/s²",
            max_accel,
            accel_mag
        );
    }

    #[test]
    fn test_panic_zone_lateral_only_when_moving() {
        // When moving, even in panic zone, only get lateral (perpendicular) force
        let mut world = World::new();

        let size = BodySize::new(1.0);

        // Moving creature
        let crit = world
            .spawn((
                Position { x: 0.0, y: 0.0 },
                Velocity { vx: 10.0, vy: 0.0 },  // Moving right
                Acceleration::default(),
                Perception::from_body_size(1.0),
                NeighborCache::new(),
                AvoidanceBehavior::new(1.25),
                size,
                CreatureState::default(),
                CanAvoidObstacles,
            ))
            .id();

        // Neighbor to the side and slightly behind (will produce lateral force)
        let neighbor = world
            .spawn((Position { x: -0.5, y: 1.1 }, BodySize::new(1.0)))
            .id();

        if let Some(mut c) = world.get_mut::<NeighborCache>(crit) {
            c.add_neighbor(NeighborData { entity: neighbor, x: -0.5, y: 1.1, radius: 0.5 });
        }

        run_system(&mut world);

        let accel = world.get::<Acceleration>(crit).unwrap();

        // Should have lateral force (Y) but minimal forward force (X)
        assert!(
            accel.ay.abs() > 1.0,
            "Should have lateral avoidance. Got ay={:.2}",
            accel.ay
        );
        assert!(
            accel.ax.abs() < accel.ay.abs() * 0.3,
            "Forward component should be much smaller than lateral. ax={:.2}, ay={:.2}",
            accel.ax, accel.ay
        );
    }
}
