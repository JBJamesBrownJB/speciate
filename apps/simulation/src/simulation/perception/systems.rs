use super::components::*;
use crate::simulation::core::components::{BodySize, Position};
use crate::simulation::creatures::components::CreatureState;
#[cfg(feature = "dev-tools")]
use crate::instrumentation::SystemTimings;
use bevy_ecs::prelude::*;
#[cfg(feature = "dev-tools")]
use bevy_ecs::system::Res;

pub fn update_perception_system(
    mut query: Query<(Entity, &Position, &BodySize, &mut Perception, &CreatureState)>,
    mut scratch: ResMut<PerceptionScratchBuffer>,
    #[cfg(feature = "dev-tools")] timings: Res<SystemTimings>,
) {
    #[cfg(feature = "dev-tools")]
    crate::time_system!(timings, "perception");

    scratch.positions.clear();
    for (entity, pos, size, _, _) in query.iter() {
        scratch.positions.push((entity, pos.x, pos.y, size.radius()));
    }

    for (entity, pos, size, mut perception, state) in query.iter_mut() {
        perception.clear();

        if !state.behavior.is_active() {
            continue;
        }

        let self_radius = size.radius();
        let perception_range = perception.range;

        for &(other_entity, other_x, other_y, other_radius) in &scratch.positions {
            if entity == other_entity {
                continue;
            }

            let dx = other_x - pos.x;
            let dy = other_y - pos.y;
            let center_dist_sq = dx * dx + dy * dy;

            let max_dist = perception_range + self_radius + other_radius;
            if center_dist_sq <= max_dist * max_dist {
                perception.add_neighbor(other_entity);
                if perception.is_full() {
                    break;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::creatures::components::{BehaviorMode, CreatureState};

    #[test]
    fn test_catatonic_crits_do_not_perceive() {
        let mut world = World::new();

        let catatonic_crit = world
            .spawn((
                Position { x: 0.0, y: 0.0 },
                BodySize::new(1.0),
                Perception::new(10.0),
                CreatureState::default(),
            ))
            .id();

        let nearby_crit = world
            .spawn((
                Position { x: 2.0, y: 0.0 },
                BodySize::new(1.0),
                Perception::new(10.0),
                {
                    let mut state = CreatureState::default();
                    state.behavior = BehaviorMode::Wandering;
                    state
                },
            ))
            .id();

        let creatures: Vec<(Entity, Position, BodySize)> = world
            .query::<(Entity, &Position, &BodySize)>()
            .iter(&world)
            .map(|(e, p, s)| (e, *p, *s))
            .collect();

        let mut query =
            world.query::<(Entity, &Position, &BodySize, &mut Perception, &CreatureState)>();
        for (entity, pos, size, mut perception, state) in query.iter_mut(&mut world) {
            perception.clear();

            if !state.behavior.is_active() {
                continue;
            }

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

                if center_dist_sq <= (perception.range + combined_radii).powi(2) {
                    let center_dist = center_dist_sq.sqrt();
                    let edge_dist = center_dist - combined_radii;
                    if edge_dist <= perception.range {
                        perception.add_neighbor(*other_entity);
                    }
                }
            }
        }

        let catatonic_perception = world.get::<Perception>(catatonic_crit).unwrap();
        assert_eq!(
            catatonic_perception.neighbor_count(),
            0,
            "Catatonic crit should not perceive neighbors"
        );

        let active_perception = world.get::<Perception>(nearby_crit).unwrap();
        assert_eq!(
            active_perception.neighbor_count(),
            1,
            "Active crit should perceive the catatonic one"
        );
        assert!(active_perception.contains(catatonic_crit));
    }

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
        assert!(perception1.contains(crit2));
        assert!(!perception1.contains(crit3));

        let perception2 = world.get::<Perception>(crit2).unwrap();
        assert_eq!(perception2.neighbor_count(), 1);
        assert!(perception2.contains(crit1));
        assert!(!perception2.contains(crit3));

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
        assert!(!perception.contains(crit));
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
        assert!(!perception1.contains(crit3));

        let perception2 = world.get::<Perception>(crit2).unwrap();
        assert!(perception2.contains(crit3));
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
