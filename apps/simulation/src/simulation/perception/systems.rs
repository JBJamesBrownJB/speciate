use super::components::*;
use crate::simulation::core::components::{BodySize, Position};
#[cfg(feature = "dev-tools")]
use crate::instrumentation::SystemTimings;
use bevy_ecs::prelude::*;
#[cfg(feature = "dev-tools")]
use bevy_ecs::system::Res;

pub fn update_perception_system(
    mut query: Query<(Entity, &Position, &BodySize, &mut Perception)>,
    #[cfg(feature = "dev-tools")] timings: Res<SystemTimings>,
) {
    #[cfg(feature = "dev-tools")]
    crate::time_system!(timings, "perception");

    let creatures: Vec<(Entity, Position, BodySize)> = query
        .iter()
        .map(|(entity, pos, size, _)| (entity, *pos, *size))
        .collect();

    for (entity, pos, size, mut perception) in query.iter_mut() {
        perception.clear();

        let self_radius = size.radius();

        for (other_entity, other_pos, other_size) in &creatures {
            if entity == *other_entity {
                continue;
            }

            let dx = other_pos.x - pos.x;
            let dy = other_pos.y - pos.y;
            let center_dist_sq = dx * dx + dy * dy;

            let other_radius = other_size.radius();
            let combined_radii = self_radius + other_radius;

            if center_dist_sq > (perception.range + combined_radii).powi(2) {
                continue;
            }

            let center_dist = center_dist_sq.sqrt();
            let edge_dist = center_dist - combined_radii;

            if edge_dist <= perception.range {
                perception.add_neighbor(*other_entity);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perception_detects_nearby_entities() {
        let mut world = World::new();

        let crit1 = world
            .spawn((
                Position { x: 0.0, y: 0.0 },
                Perception::new(10.0),
            ))
            .id();

        let crit2 = world
            .spawn((
                Position { x: 5.0, y: 0.0 },
                Perception::new(10.0),
            ))
            .id();

        let crit3 = world
            .spawn((
                Position { x: 20.0, y: 0.0 },
                Perception::new(10.0),
            ))
            .id();

        let positions: Vec<(Entity, Position)> = world
            .query::<(Entity, &Position)>()
            .iter(&world)
            .map(|(e, p)| (e, *p))
            .collect();

        let mut query = world.query::<(Entity, &Position, &mut Perception)>();
        for (entity, pos, mut perception) in query.iter_mut(&mut world) {
            perception.clear();
            let range_sq = perception.range * perception.range;

            for (other_entity, other_pos) in &positions {
                if entity == *other_entity {
                    continue;
                }
                let dx = other_pos.x - pos.x;
                let dy = other_pos.y - pos.y;
                let dist_sq = dx * dx + dy * dy;
                if dist_sq <= range_sq {
                    perception.add_neighbor(*other_entity);
                }
            }
        }

        let perception1 = world.get::<Perception>(crit1).unwrap();
        assert_eq!(perception1.neighbor_count(), 1);
        assert!(perception1.nearby.contains(&crit2));
        assert!(!perception1.nearby.contains(&crit3));

        let perception2 = world.get::<Perception>(crit2).unwrap();
        assert_eq!(perception2.neighbor_count(), 1);
        assert!(perception2.nearby.contains(&crit1));
        assert!(!perception2.nearby.contains(&crit3));

        let perception3 = world.get::<Perception>(crit3).unwrap();
        assert_eq!(perception3.neighbor_count(), 0);
    }

    #[test]
    fn test_perception_does_not_detect_self() {
        let mut world = World::new();

        let crit = world
            .spawn((Position { x: 0.0, y: 0.0 }, Perception::new(10.0)))
            .id();

        let positions: Vec<(Entity, Position)> = world
            .query::<(Entity, &Position)>()
            .iter(&world)
            .map(|(e, p)| (e, *p))
            .collect();

        let mut query = world.query::<(Entity, &Position, &mut Perception)>();
        for (entity, pos, mut perception) in query.iter_mut(&mut world) {
            perception.clear();
            let range_sq = perception.range * perception.range;

            for (other_entity, other_pos) in &positions {
                if entity == *other_entity {
                    continue;
                }
                let dx = other_pos.x - pos.x;
                let dy = other_pos.y - pos.y;
                let dist_sq = dx * dx + dy * dy;
                if dist_sq <= range_sq {
                    perception.add_neighbor(*other_entity);
                }
            }
        }

        let perception = world.get::<Perception>(crit).unwrap();
        assert_eq!(perception.neighbor_count(), 0);
        assert!(!perception.nearby.contains(&crit));
    }

    #[test]
    fn test_perception_clears_previous_neighbors() {
        let mut world = World::new();

        let crit1 = world
            .spawn((Position { x: 0.0, y: 0.0 }, Perception::new(10.0)))
            .id();

        let crit2 = world
            .spawn((Position { x: 5.0, y: 0.0 }, Perception::new(10.0)))
            .id();

        let positions: Vec<(Entity, Position)> = world
            .query::<(Entity, &Position)>()
            .iter(&world)
            .map(|(e, p)| (e, *p))
            .collect();

        let mut query = world.query::<(Entity, &Position, &mut Perception)>();
        for (entity, pos, mut perception) in query.iter_mut(&mut world) {
            perception.clear();
            let range_sq = perception.range * perception.range;

            for (other_entity, other_pos) in &positions {
                if entity == *other_entity {
                    continue;
                }
                let dx = other_pos.x - pos.x;
                let dy = other_pos.y - pos.y;
                let dist_sq = dx * dx + dy * dy;
                if dist_sq <= range_sq {
                    perception.add_neighbor(*other_entity);
                }
            }
        }

        let perception1 = world.get::<Perception>(crit1).unwrap();
        assert_eq!(perception1.neighbor_count(), 1);

        if let Some(mut pos2) = world.get_mut::<Position>(crit2) {
            pos2.x = 50.0;
        }

        let positions: Vec<(Entity, Position)> = world
            .query::<(Entity, &Position)>()
            .iter(&world)
            .map(|(e, p)| (e, *p))
            .collect();

        let mut query = world.query::<(Entity, &Position, &mut Perception)>();
        for (entity, pos, mut perception) in query.iter_mut(&mut world) {
            perception.clear();
            let range_sq = perception.range * perception.range;

            for (other_entity, other_pos) in &positions {
                if entity == *other_entity {
                    continue;
                }
                let dx = other_pos.x - pos.x;
                let dy = other_pos.y - pos.y;
                let dist_sq = dx * dx + dy * dy;
                if dist_sq <= range_sq {
                    perception.add_neighbor(*other_entity);
                }
            }
        }

        let perception1 = world.get::<Perception>(crit1).unwrap();
        assert_eq!(perception1.neighbor_count(), 0);
    }

    #[test]
    fn test_perception_respects_range() {
        let mut world = World::new();

        let crit1 = world
            .spawn((
                Position { x: 0.0, y: 0.0 },
                Perception::new(5.0),
            ))
            .id();

        let crit2 = world
            .spawn((
                Position { x: 0.0, y: 0.0 },
                Perception::new(20.0),
            ))
            .id();

        let crit3 = world
            .spawn((
                Position { x: 10.0, y: 0.0 },
                Perception::new(10.0),
            ))
            .id();

        let positions: Vec<(Entity, Position)> = world
            .query::<(Entity, &Position)>()
            .iter(&world)
            .map(|(e, p)| (e, *p))
            .collect();

        let mut query = world.query::<(Entity, &Position, &mut Perception)>();
        for (entity, pos, mut perception) in query.iter_mut(&mut world) {
            perception.clear();
            let range_sq = perception.range * perception.range;

            for (other_entity, other_pos) in &positions {
                if entity == *other_entity {
                    continue;
                }
                let dx = other_pos.x - pos.x;
                let dy = other_pos.y - pos.y;
                let dist_sq = dx * dx + dy * dy;
                if dist_sq <= range_sq {
                    perception.add_neighbor(*other_entity);
                }
            }
        }

        let perception1 = world.get::<Perception>(crit1).unwrap();
        assert!(!perception1.nearby.contains(&crit3));

        let perception2 = world.get::<Perception>(crit2).unwrap();
        assert!(perception2.nearby.contains(&crit3));
    }

    #[test]
    fn test_perception_performance_baseline() {
        let mut world = World::new();

        for i in 0..100 {
            let x = (i % 10) as f32 * 10.0;
            let y = (i / 10) as f32 * 10.0;

            world.spawn((Position { x, y }, Perception::new(10.0)));
        }

        let start = std::time::Instant::now();

        let positions: Vec<(Entity, Position)> = world
            .query::<(Entity, &Position)>()
            .iter(&world)
            .map(|(e, p)| (e, *p))
            .collect();

        let mut query = world.query::<(Entity, &Position, &mut Perception)>();
        for (entity, pos, mut perception) in query.iter_mut(&mut world) {
            perception.clear();
            let range_sq = perception.range * perception.range;

            for (other_entity, other_pos) in &positions {
                if entity == *other_entity {
                    continue;
                }
                let dx = other_pos.x - pos.x;
                let dy = other_pos.y - pos.y;
                let dist_sq = dx * dx + dy * dy;
                if dist_sq <= range_sq {
                    perception.add_neighbor(*other_entity);
                }
            }
        }

        let duration = start.elapsed();

        println!("Perception update (100 crits, naive O(n²)): {:?}", duration);

        let mut total_neighbors = 0;
        for perception in world.query::<&Perception>().iter(&world) {
            total_neighbors += perception.neighbor_count();
        }

        println!("Total neighbor detections: {}", total_neighbors);
        assert!(total_neighbors > 0, "Perception should detect some neighbors");
    }
}
