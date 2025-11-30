use super::components::*;
use crate::simulation::components::Rotation;
use crate::simulation::core::components::{BodySize, Position};
use crate::simulation::creatures::components::CreatureState;
#[cfg(feature = "dev-tools")]
use crate::simulation::components::CritId;
#[cfg(feature = "dev-tools")]
use crate::instrumentation::SystemTimings;
use bevy_ecs::prelude::*;
#[cfg(feature = "dev-tools")]
use bevy_ecs::system::Res;
use std::f32::consts::{PI, TAU};

/// Normalize angle to [-PI, PI] range
fn normalize_angle(angle: f32) -> f32 {
    let mut a = angle;
    while a > PI {
        a -= TAU;
    }
    while a < -PI {
        a += TAU;
    }
    a
}

pub fn update_perception_system(
    mut query: Query<(Entity, &Position, &Rotation, &BodySize, &mut Perception, &CreatureState)>,
    mut scratch: ResMut<PerceptionScratchBuffer>,
    #[cfg(feature = "dev-tools")] timings: Res<SystemTimings>,
    #[cfg(feature = "dev-tools")] debug_target: Res<PerceptionDebugTarget>,
    #[cfg(feature = "dev-tools")] mut debug_snapshot: ResMut<PerceptionDebugSnapshot>,
    #[cfg(feature = "dev-tools")] crit_ids: Query<&CritId>,
) {
    #[cfg(feature = "dev-tools")]
    crate::time_system!(timings, "perception");

    scratch.positions.clear();
    let count_hint = query.iter().size_hint().0;
    scratch.positions.reserve(count_hint);
    for (entity, pos, _, size, _, _) in query.iter() {
        scratch.positions.push((entity, pos.x, pos.y, size.radius()));
    }

    for (entity, pos, rotation, size, mut perception, state) in query.iter_mut() {
        perception.clear();

        if !state.behavior.is_active() {
            continue;
        }

        let self_radius = size.radius();
        let perception_range = perception.range;
        let half_fov = perception.half_fov();
        let facing_direction = rotation.radians;

        for &(other_entity, other_x, other_y, other_radius) in &scratch.positions {
            if entity == other_entity {
                continue;
            }

            let dx = other_x - pos.x;
            let dy = other_y - pos.y;
            let center_dist_sq = dx * dx + dy * dy;

            // Distance check first (cheaper)
            let max_dist = perception_range + self_radius + other_radius;
            if center_dist_sq > max_dist * max_dist {
                continue;
            }

            // FOV angle check - is target within the cone?
            let angle_to_target = dy.atan2(dx);
            let relative_angle = normalize_angle(angle_to_target - facing_direction);

            if relative_angle.abs() <= half_fov {
                perception.add_neighbor(other_entity);
                if perception.is_full() {
                    break;
                }
            }
        }
    }

    // Collect debug data for selected creature (dev-tools only)
    #[cfg(feature = "dev-tools")]
    {
        if let Some(target_entity) = debug_target.get() {
            if let Ok((_, pos, rotation, _, perception, _)) = query.get(target_entity) {
                let entity_id = crit_ids.get(target_entity)
                    .map(|id| id.0)
                    .unwrap_or(0);

                let neighbors: Vec<NeighborDebugInfo> = perception.iter_neighbors()
                    .filter_map(|neighbor_entity| {
                        let neighbor_id = crit_ids.get(neighbor_entity).ok()?.0;
                        let (_, neighbor_pos, _, _, _, _) = query.get(neighbor_entity).ok()?;
                        Some(NeighborDebugInfo {
                            id: neighbor_id,
                            x: neighbor_pos.x,
                            y: neighbor_pos.y,
                        })
                    })
                    .collect();

                *debug_snapshot = PerceptionDebugSnapshot {
                    entity_id,
                    x: pos.x,
                    y: pos.y,
                    perception_range: perception.range,
                    fov_angle: perception.fov_angle,
                    rotation: rotation.radians,
                    neighbors,
                };
            } else {
                // Target entity no longer exists, clear snapshot
                *debug_snapshot = PerceptionDebugSnapshot::default();
            }
        } else {
            // No debug target, clear snapshot
            *debug_snapshot = PerceptionDebugSnapshot::default();
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
                Perception::from_body_size(1.0),
                CreatureState::default(),
            ))
            .id();

        let nearby_crit = world
            .spawn((
                Position { x: 2.0, y: 0.0 },
                BodySize::new(1.0),
                Perception::from_body_size(1.0),
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
                Perception::from_body_size(1.0),
            ))
            .id();

        let crit2 = world
            .spawn((
                Position { x: 5.0, y: 0.0 },
                Perception::from_body_size(1.0),
            ))
            .id();

        let crit3 = world
            .spawn((
                Position { x: 20.0, y: 0.0 },
                Perception::from_body_size(1.0),
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
            .spawn((Position { x: 0.0, y: 0.0 }, Perception::from_body_size(1.0)))
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
            .spawn((Position { x: 0.0, y: 0.0 }, Perception::from_body_size(1.0)))
            .id();

        let crit2 = world
            .spawn((Position { x: 5.0, y: 0.0 }, Perception::from_body_size(1.0)))
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
                Perception::from_body_size(0.5),
            ))
            .id();

        let crit2 = world
            .spawn((
                Position { x: 0.0, y: 0.0 },
                Perception::from_body_size(2.0),
            ))
            .id();

        let crit3 = world
            .spawn((
                Position { x: 10.0, y: 0.0 },
                Perception::from_body_size(1.0),
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

            world.spawn((Position { x, y }, Perception::from_body_size(1.0)));
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

    #[test]
    fn test_fov_detects_target_in_front() {
        // Creature at origin, facing right (0 radians), target directly ahead
        let perception = Perception::new(180.0, 1.0); // 180° FOV
        let facing = 0.0_f32; // facing right
        let half_fov = perception.half_fov();

        // Target at (5, 0) - directly in front
        let dx = 5.0_f32;
        let dy = 0.0_f32;
        let angle_to_target = dy.atan2(dx);
        let relative_angle = normalize_angle(angle_to_target - facing);

        assert!(
            relative_angle.abs() <= half_fov,
            "Target directly in front should be in FOV"
        );
    }

    #[test]
    fn test_fov_detects_target_at_edge() {
        // 90° FOV (±45° from facing direction)
        let perception = Perception::new(90.0, 1.0);
        let facing = 0.0_f32;
        let half_fov = perception.half_fov();

        // Target at 44° - just inside FOV
        let angle_44 = 44.0_f32.to_radians();
        let dx = angle_44.cos() * 5.0;
        let dy = angle_44.sin() * 5.0;
        let angle_to_target = dy.atan2(dx);
        let relative_angle = normalize_angle(angle_to_target - facing);

        assert!(
            relative_angle.abs() <= half_fov,
            "Target at 44° should be in 90° FOV (±45°)"
        );
    }

    #[test]
    fn test_fov_misses_target_outside_cone() {
        // 90° FOV (±45° from facing direction)
        let perception = Perception::new(90.0, 1.0);
        let facing = 0.0_f32;
        let half_fov = perception.half_fov();

        // Target at 60° - outside FOV
        let angle_60 = 60.0_f32.to_radians();
        let dx = angle_60.cos() * 5.0;
        let dy = angle_60.sin() * 5.0;
        let angle_to_target = dy.atan2(dx);
        let relative_angle = normalize_angle(angle_to_target - facing);

        assert!(
            relative_angle.abs() > half_fov,
            "Target at 60° should NOT be in 90° FOV (±45°)"
        );
    }

    #[test]
    fn test_fov_misses_target_behind() {
        // 120° FOV - should not see behind
        let perception = Perception::new(120.0, 1.0);
        let facing = 0.0_f32; // facing right
        let half_fov = perception.half_fov();

        // Target directly behind at 180°
        let dx = -5.0_f32;
        let dy = 0.0_f32;
        let angle_to_target = dy.atan2(dx);
        let relative_angle = normalize_angle(angle_to_target - facing);

        assert!(
            relative_angle.abs() > half_fov,
            "Target behind should NOT be in 120° FOV"
        );
    }

    #[test]
    fn test_wide_fov_sees_almost_everywhere() {
        // 320° FOV - only 40° blind spot behind
        let perception = Perception::new(320.0, 1.0);
        let facing = 0.0_f32;
        let half_fov = perception.half_fov();

        // Target at 150° - should be visible with wide FOV
        let angle_150 = 150.0_f32.to_radians();
        let dx = angle_150.cos() * 5.0;
        let dy = angle_150.sin() * 5.0;
        let angle_to_target = dy.atan2(dx);
        let relative_angle = normalize_angle(angle_to_target - facing);

        assert!(
            relative_angle.abs() <= half_fov,
            "Target at 150° should be in 320° FOV"
        );

        // Target directly behind (180°) - should still be in blind spot
        let angle_180 = std::f32::consts::PI;
        let dx_behind = angle_180.cos() * 5.0;
        let dy_behind = angle_180.sin() * 5.0;
        let angle_to_behind = dy_behind.atan2(dx_behind);
        let relative_behind = normalize_angle(angle_to_behind - facing);

        assert!(
            relative_behind.abs() > half_fov,
            "Target at 180° should be in blind spot of 320° FOV"
        );
    }
}
