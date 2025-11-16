use crate::simulation::components::*;
use crate::simulation::core::components::*;
use crate::simulation::movement::STEERING;
use crate::simulation::perception::*;
use bevy_ecs::prelude::*;

#[allow(clippy::type_complexity)]
pub fn avoidance_system(
    mut query: Query<
        (
            Entity,
            &Position,
            &BodySize,
            &mut Acceleration,
            &Perception,
            &AvoidanceBehavior,
        ),
        With<CanAvoidObstacles>,
    >,
    others: Query<(&Position, &BodySize)>,
    #[cfg(feature = "dev-tools")] timings: bevy_ecs::system::Res<
        crate::instrumentation::SystemTimings,
    >,
) {
    #[cfg(feature = "dev-tools")]
    crate::time_system!(timings, "avoidance");

    for (entity, position, size, mut acceleration, perception, avoidance) in query.iter_mut() {
        if !perception.has_neighbors() {
            continue;
        }

        let panic_threshold = avoidance.panic_threshold();
        let self_radius = size.radius();

        let mut total_repulsion_x = 0.0;
        let mut total_repulsion_y = 0.0;

        for &other_entity in &perception.nearby {
            if other_entity == entity {
                continue;
            }

            let Ok((other_pos, other_size)) = others.get(other_entity) else {
                continue;
            };

            let away_x = position.x - other_pos.x;
            let away_y = position.y - other_pos.y;
            let center_distance = (away_x * away_x + away_y * away_y).sqrt();

            if center_distance < 0.001 {
                continue;
            }

            let edge_distance = center_distance - self_radius - other_size.radius();
            let safe_distance = edge_distance.max(0.01);

            if safe_distance < avoidance.personal_space {
                let ratio = avoidance.personal_space / safe_distance;
                let mut force_magnitude = STEERING.avoidance_force * ratio * ratio;

                if safe_distance < panic_threshold {
                    force_magnitude = force_magnitude.min(STEERING.panic_force);
                }

                let force_x = (away_x / center_distance) * force_magnitude;
                let force_y = (away_y / center_distance) * force_magnitude;

                total_repulsion_x += force_x;
                total_repulsion_y += force_y;
            }
        }

        let total_mag_sq = total_repulsion_x * total_repulsion_x
            + total_repulsion_y * total_repulsion_y;
        let max_force = avoidance.max_force;
        let max_force_sq = max_force * max_force;

        if total_mag_sq > max_force_sq {
            let total_mag = total_mag_sq.sqrt();
            let scale = max_force / total_mag;
            acceleration.ax += total_repulsion_x * scale;
            acceleration.ay += total_repulsion_y * scale;
        } else {
            acceleration.ax += total_repulsion_x;
            acceleration.ay += total_repulsion_y;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_avoidance_test(world: &mut World) {
        let all_positions: Vec<(Entity, Position)> = world
            .query::<(Entity, &Position)>()
            .iter(world)
            .map(|(e, p)| (e, *p))
            .collect();

        let mut query = world.query_filtered::<(
            Entity,
            &Position,
            &mut Acceleration,
            &Perception,
            &AvoidanceBehavior,
        ), With<CanAvoidObstacles>>();

        for (entity, position, mut acceleration, perception, avoidance) in query.iter_mut(world) {
            if !perception.has_neighbors() {
                continue;
            }

            let panic_threshold = avoidance.panic_threshold();
            let mut total_repulsion_x = 0.0;
            let mut total_repulsion_y = 0.0;

            for &other_entity in &perception.nearby {
                if other_entity == entity {
                    continue;
                }

                let Some((_, other_pos)) = all_positions.iter().find(|(e, _)| *e == other_entity) else {
                    continue;
                };

                let away_x = position.x - other_pos.x;
                let away_y = position.y - other_pos.y;
                let distance = (away_x * away_x + away_y * away_y).sqrt();

                if distance < 0.01 {
                    continue;
                }

                if distance < avoidance.personal_space {
                    let ratio = avoidance.personal_space / distance;
                    let mut force_magnitude = STEERING.avoidance_force * ratio * ratio;

                    if distance < panic_threshold {
                        force_magnitude = force_magnitude.min(STEERING.panic_force);
                    }

                    let force_x = (away_x / distance) * force_magnitude;
                    let force_y = (away_y / distance) * force_magnitude;
                    total_repulsion_x += force_x;
                    total_repulsion_y += force_y;
                }
            }

            let total_mag_sq = total_repulsion_x * total_repulsion_x + total_repulsion_y * total_repulsion_y;
            let max_force = avoidance.max_force;
            let max_force_sq = max_force * max_force;

            if total_mag_sq > max_force_sq {
                let total_mag = total_mag_sq.sqrt();
                let scale = max_force / total_mag;
                acceleration.ax += total_repulsion_x * scale;
                acceleration.ay += total_repulsion_y * scale;
            } else {
                acceleration.ax += total_repulsion_x;
                acceleration.ay += total_repulsion_y;
            }
        }
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
                CanAvoidObstacles,
            ))
            .id();

        run_avoidance_test(&mut world);

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
                CanAvoidObstacles,
            ))
            .id();

        let crit2 = world.spawn((Position { x: 1.0, y: 0.0 },)).id();

        if let Some(mut p) = world.get_mut::<Perception>(crit1) {
            p.add_neighbor(crit2);
        }

        run_avoidance_test(&mut world);

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
                CanAvoidObstacles,
            ))
            .id();

        let crit2 = world.spawn((Position { x: 5.0, y: 0.0 },)).id();

        if let Some(mut p) = world.get_mut::<Perception>(crit1) {
            p.add_neighbor(crit2);
        }

        run_avoidance_test(&mut world);

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
                CanAvoidObstacles,
            ))
            .id();

        let crit2 = world.spawn((Position { x: 0.5, y: 0.0 },)).id();

        if let Some(mut p) = world.get_mut::<Perception>(crit1) {
            p.add_neighbor(crit2);
        }

        run_avoidance_test(&mut world);

        let accel = world.get::<Acceleration>(crit1).unwrap();
        let force_mag = (accel.ax * accel.ax + accel.ay * accel.ay).sqrt();

        assert!(
            force_mag <= STEERING.panic_force,
            "Force should be capped at {}N, got: {:.2}N",
            STEERING.panic_force,
            force_mag
        );
        assert!(force_mag > 0.0);
    }

    #[test]
    fn test_inverse_square_scaling() {
        let avoidance = AvoidanceBehavior::new(2.5, 15.0);
        let base_force = STEERING.avoidance_force;

        let test_cases = vec![
            (2.0_f32, base_force * (2.5_f32 / 2.0_f32).powi(2)),
            (1.5_f32, base_force * (2.5_f32 / 1.5_f32).powi(2)),
            (1.0_f32, base_force * (2.5_f32 / 1.0_f32).powi(2)),
        ];

        for (distance, expected_force) in test_cases {
            let ratio = avoidance.personal_space / distance;
            let calculated_force = base_force * ratio * ratio;

            let final_force = if distance < avoidance.panic_threshold() {
                calculated_force.min(STEERING.panic_force)
            } else {
                calculated_force
            };

            let expected_final = if distance < avoidance.panic_threshold() {
                expected_force.min(STEERING.panic_force)
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
                CanAvoidObstacles,
            ))
            .id();

        let crit2 = world.spawn((Position { x: 1.0, y: 0.0 },)).id();
        let crit3 = world.spawn((Position { x: 0.0, y: 1.0 },)).id();

        if let Some(mut p) = world.get_mut::<Perception>(crit1) {
            p.add_neighbor(crit2);
            p.add_neighbor(crit3);
        }

        run_avoidance_test(&mut world);

        let accel = world.get::<Acceleration>(crit1).unwrap();

        assert!(accel.ax < 0.0, "Should be repelled in -X");
        assert!(accel.ay < 0.0, "Should be repelled in -Y");

        assert!(
            (accel.ax.abs() - accel.ay.abs()).abs() < 0.01,
            "Forces from equidistant obstacles should be equal"
        );
    }
}
