use super::components::*;
use crate::simulation::components::Rotation;
use crate::simulation::core::components::{BodySize, Position};
use crate::simulation::creatures::components::CreatureState;
use crate::simulation::spatial::{DoubleBufferedSpatialGrid, SpatialGrid};
#[cfg(feature = "dev-tools")]
use crate::simulation::components::CritId;
#[cfg(feature = "dev-tools")]
use crate::instrumentation::SystemTimings;
use bevy_ecs::prelude::*;
use rayon::prelude::*;
use std::cell::RefCell;

const MAX_OTHER_RADIUS: f32 = 5.0;
const CELL_HALF_DIAGONAL: f32 = crate::simulation::spatial::constants::CELL_SIZE * 0.7072; // sqrt(2)/2

// Thread-local scratch buffer for sorted cell indices (avoids allocation per creature)
// Format: (distance_sq, cell_index)
thread_local! {
    static CELL_SCRATCH: RefCell<Vec<(f32, usize)>> = RefCell::new(Vec::with_capacity(64));
}

pub fn update_perception_system(
    grid: Res<DoubleBufferedSpatialGrid>,
    mut query: Query<(Entity, &Position, &Rotation, &BodySize, &mut Perception, &CreatureState)>,
    #[cfg(feature = "dev-tools")] timings: Res<SystemTimings>,
    #[cfg(feature = "dev-tools")] debug_target: Res<PerceptionDebugTarget>,
    #[cfg(feature = "dev-tools")] mut debug_snapshot: ResMut<PerceptionDebugSnapshot>,
    #[cfg(feature = "dev-tools")] crit_ids: Query<&CritId>,
) {
    #[cfg(feature = "dev-tools")]
    crate::time_system!(timings, "perception");

    // Read from FRONT buffer (back buffer is being rebuilt in parallel)
    let grid_ref = grid.read_grid();

    // Phase 1: Process debug target FIRST with cell tracking (dev-tools only)
    // This captures REAL queried/skipped cells from actual perception execution
    #[cfg(feature = "dev-tools")]
    let debug_target_entity = debug_target.get();

    #[cfg(feature = "dev-tools")]
    {
        if let Some(target_entity) = debug_target_entity {
            if let Ok((_, pos, rotation, size, mut perception, state)) = query.get_mut(target_entity) {
                perception.clear();

                let entity_id = crit_ids.get(target_entity)
                    .map(|id| id.0)
                    .unwrap_or(0);

                if state.behavior.is_active() {
                    let x = pos.x;
                    let y = pos.y;
                    let self_radius = size.radius();
                    let range = perception.range;
                    let cos_half_fov_sq = perception.cos_half_fov_sq;
                    let facing_x = rotation.radians.cos();
                    let facing_y = rotation.radians.sin();
                    let query_radius = range + self_radius + MAX_OTHER_RADIUS;

                    // Run perception WITH cell tracking - this is the REAL execution
                    let (neighbors, queried_cells, skipped_cells) = run_perception_with_tracking(
                        target_entity,
                        x, y,
                        self_radius,
                        range,
                        cos_half_fov_sq,
                        facing_x, facing_y,
                        query_radius,
                        grid_ref,
                    );

                    // Note: Don't add neighbors here - Phase 3 will clear() and re-add anyway
                    // We only need the neighbors Vec for building neighbor_debug below

                    // Capture values before releasing the borrow
                    let fov_angle = perception.fov_angle;
                    let rotation_radians = rotation.radians;
                    let (creature_cx, creature_cy) = grid_ref.world_to_cell(x, y);

                    // Build neighbor debug info directly from captured positions (O(n), not O(n²))
                    let neighbor_debug: Vec<NeighborDebugInfo> = neighbors.iter()
                        .filter_map(|neighbor| {
                            let neighbor_id = crit_ids.get(neighbor.entity).ok()?.0;
                            Some(NeighborDebugInfo {
                                id: neighbor_id,
                                x: neighbor.x,
                                y: neighbor.y,
                            })
                        })
                        .collect();

                    *debug_snapshot = PerceptionDebugSnapshot {
                        entity_id,
                        x,
                        y,
                        perception_range: range,
                        fov_angle,
                        rotation: rotation_radians,
                        neighbors: neighbor_debug,
                        queried_cells,              // Green: cells we actually examined
                        checked_cells: skipped_cells, // Orange: cells skipped due to early break
                        creature_cell: QueriedCell { x: creature_cx, y: creature_cy },
                    };
                } else {
                    // Inactive creature - empty debug snapshot
                    let (creature_cx, creature_cy) = grid_ref.world_to_cell(pos.x, pos.y);
                    *debug_snapshot = PerceptionDebugSnapshot {
                        entity_id,
                        x: pos.x,
                        y: pos.y,
                        perception_range: perception.range,
                        fov_angle: perception.fov_angle,
                        rotation: rotation.radians,
                        neighbors: vec![],
                        queried_cells: vec![],
                        checked_cells: vec![],
                        creature_cell: QueriedCell { x: creature_cx, y: creature_cy },
                    };
                }
            } else {
                *debug_snapshot = PerceptionDebugSnapshot::default();
            }
        } else {
            *debug_snapshot = PerceptionDebugSnapshot::default();
        }
    }

    // Phase 2: Collect ALL entities for parallel processing
    // Debug target runs again (cheap: 1 creature) - much faster than 150K filter calls
    // Its debug snapshot was already captured in Phase 1
    let mut entities: Vec<_> = query.iter_mut().collect();

    // Phase 3: Parallel perception using Rayon (unchanged hot path)
    entities.par_iter_mut().for_each(|(entity, pos, rot, size, perception, state)| {
        perception.clear();

        if !state.behavior.is_active() {
            return;
        }

        let x = pos.x;
        let y = pos.y;
        let self_radius = size.radius();
        let range = perception.range;
        let cos_half_fov_sq = perception.cos_half_fov_sq;
        let facing_x = rot.radians.cos();
        let facing_y = rot.radians.sin();
        let query_radius = range + self_radius + MAX_OTHER_RADIUS;

        // Fixed-size buffer for top-k closest neighbors (no heap allocation)
        const CAPACITY: usize = super::constants::MAX_PERCEIVED_NEIGHBORS;
        let mut closest: [(Entity, f32); CAPACITY] = [(Entity::PLACEHOLDER, f32::MAX); CAPACITY];
        let mut count = 0usize;

        // Get cells sorted by distance (closest first) for faster buffer filling
        CELL_SCRATCH.with(|scratch| {
            let mut cells = scratch.borrow_mut();
            grid_ref.collect_cells_sorted(x, y, query_radius, facing_x, facing_y, &mut cells);

            // Iterate cells in distance order
            for &(cell_dist_sq, cell_idx) in cells.iter() {
                // Early break: if buffer full and cell is definitely farther than 8th closest, stop
                if count == CAPACITY {
                    let cell_dist = cell_dist_sq.sqrt();
                    let min_proxy_dist = cell_dist - CELL_HALF_DIAGONAL;
                    if min_proxy_dist * min_proxy_dist > closest[CAPACITY - 1].1 {
                        break; // All remaining cells are farther, done
                    }
                }

                for proxy in grid_ref.get_cell_proxies(cell_idx) {
                    if *entity == proxy.entity {
                        continue;
                    }

                    let dx = proxy.x - x;
                    let dy = proxy.y - y;
                    let center_dist_sq = dx * dx + dy * dy;

                    // Distance check (cheaper)
                    let max_dist = range + self_radius + proxy.radius;
                    if center_dist_sq > max_dist * max_dist {
                        continue;
                    }

                    // Quick rejection: skip if farther than our 8th closest (avoids FOV check)
                    if center_dist_sq >= closest[CAPACITY - 1].1 {
                        continue;
                    }

                    // Early-exit: skip entities clearly behind
                    let rough_dot = dx * facing_x + dy * facing_y;
                    if rough_dot <= 0.0 {
                        continue;
                    }

                    // FOV check using squared comparison (no sqrt, no division)
                    if rough_dot * rough_dot >= cos_half_fov_sq * center_dist_sq {
                        // Insert into sorted buffer (closest first, max 8 items)
                        let insert_limit = if count < CAPACITY { count } else { CAPACITY - 1 };
                        let mut i = insert_limit;
                        while i > 0 && closest[i - 1].1 > center_dist_sq {
                            closest[i] = closest[i - 1];
                            i -= 1;
                        }
                        closest[i] = (proxy.entity, center_dist_sq);
                        if count < CAPACITY {
                            count += 1;
                        }
                    }
                }
            }
        });

        // Add sorted neighbors to perception
        for i in 0..count {
            perception.add_neighbor(closest[i].0);
        }
    });
}

/// Neighbor info captured during perception (avoids re-querying grid)
#[cfg(feature = "dev-tools")]
struct NeighborInfo {
    entity: Entity,
    x: f32,
    y: f32,
}

/// Run ACTUAL perception for debug target with cell tracking.
/// Returns: (neighbors_with_positions, queried_cells, skipped_cells)
/// - neighbors_with_positions: entities AND their positions (captured during perception)
/// - queried_cells: cells we actually examined
/// - skipped_cells: cells we would have examined but skipped due to early break
#[cfg(feature = "dev-tools")]
fn run_perception_with_tracking(
    self_entity: Entity,
    x: f32,
    y: f32,
    self_radius: f32,
    range: f32,
    cos_half_fov_sq: f32,
    facing_x: f32,
    facing_y: f32,
    query_radius: f32,
    grid: &SpatialGrid,
) -> (Vec<NeighborInfo>, Vec<QueriedCell>, Vec<QueriedCell>) {
    const CAPACITY: usize = super::constants::MAX_PERCEIVED_NEIGHBORS;
    // Store entity, distance, AND position (avoid re-query later)
    let mut closest: [(Entity, f32, f32, f32); CAPACITY] = [(Entity::PLACEHOLDER, f32::MAX, 0.0, 0.0); CAPACITY];
    let mut count = 0usize;
    let mut queried_cells = Vec::new();
    let mut skipped_cells = Vec::new();

    // Get cells sorted by distance (same as main loop)
    let mut cells: Vec<(f32, usize)> = Vec::with_capacity(64);
    grid.collect_cells_sorted(x, y, query_radius, facing_x, facing_y, &mut cells);

    let mut early_break_index: Option<usize> = None;

    // Iterate cells in distance order, tracking which we check
    for (i, &(cell_dist_sq, cell_idx)) in cells.iter().enumerate() {
        // Early break: if buffer full and cell is definitely farther than 8th closest, stop
        if count == CAPACITY {
            let cell_dist = cell_dist_sq.sqrt();
            let min_proxy_dist = cell_dist - CELL_HALF_DIAGONAL;
            if min_proxy_dist * min_proxy_dist > closest[CAPACITY - 1].1 {
                early_break_index = Some(i);
                break; // All remaining cells are farther, done
            }
        }

        // This cell was queried (not skipped by early break)
        let (cx, cy) = grid.get_cell_coords_by_index(cell_idx);
        queried_cells.push(QueriedCell { x: cx, y: cy });

        // Process proxies (same logic as main loop)
        for proxy in grid.get_cell_proxies(cell_idx) {
            if self_entity == proxy.entity {
                continue;
            }

            let dx = proxy.x - x;
            let dy = proxy.y - y;
            let center_dist_sq = dx * dx + dy * dy;

            let max_dist = range + self_radius + proxy.radius;
            if center_dist_sq > max_dist * max_dist {
                continue;
            }

            if center_dist_sq >= closest[CAPACITY - 1].1 {
                continue;
            }

            let rough_dot = dx * facing_x + dy * facing_y;
            if rough_dot <= 0.0 {
                continue;
            }

            if rough_dot * rough_dot >= cos_half_fov_sq * center_dist_sq {
                let insert_limit = if count < CAPACITY { count } else { CAPACITY - 1 };
                let mut i = insert_limit;
                while i > 0 && closest[i - 1].1 > center_dist_sq {
                    closest[i] = closest[i - 1];
                    i -= 1;
                }
                // Store position along with entity and distance
                closest[i] = (proxy.entity, center_dist_sq, proxy.x, proxy.y);
                if count < CAPACITY {
                    count += 1;
                }
            }
        }
    }

    // Collect skipped cells (remaining cells after early break)
    if let Some(break_idx) = early_break_index {
        for &(_, cell_idx) in &cells[break_idx..] {
            let (cx, cy) = grid.get_cell_coords_by_index(cell_idx);
            skipped_cells.push(QueriedCell { x: cx, y: cy });
        }
    }

    // Extract neighbors with positions from closest buffer
    let neighbors: Vec<NeighborInfo> = closest[..count]
        .iter()
        .map(|(e, _, px, py)| NeighborInfo { entity: *e, x: *px, y: *py })
        .collect();

    (neighbors, queried_cells, skipped_cells)
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

    fn is_in_fov(dx: f32, dy: f32, facing: f32, half_fov: f32) -> bool {
        let dist = (dx * dx + dy * dy).sqrt();
        let dir_x = dx / dist;
        let dir_y = dy / dist;
        let facing_x = facing.cos();
        let facing_y = facing.sin();
        let dot = dir_x * facing_x + dir_y * facing_y;
        dot >= half_fov.cos()
    }

    #[test]
    fn test_fov_detects_target_in_front() {
        let perception = Perception::new(180.0, 1.0);
        let facing = 0.0_f32;
        let half_fov = perception.half_fov();

        let dx = 5.0_f32;
        let dy = 0.0_f32;

        assert!(
            is_in_fov(dx, dy, facing, half_fov),
            "Target directly in front should be in FOV"
        );
    }

    #[test]
    fn test_fov_detects_target_at_edge() {
        let perception = Perception::new(90.0, 1.0);
        let facing = 0.0_f32;
        let half_fov = perception.half_fov();

        let angle_44 = 44.0_f32.to_radians();
        let dx = angle_44.cos() * 5.0;
        let dy = angle_44.sin() * 5.0;

        assert!(
            is_in_fov(dx, dy, facing, half_fov),
            "Target at 44° should be in 90° FOV (±45°)"
        );
    }

    #[test]
    fn test_fov_misses_target_outside_cone() {
        let perception = Perception::new(90.0, 1.0);
        let facing = 0.0_f32;
        let half_fov = perception.half_fov();

        let angle_60 = 60.0_f32.to_radians();
        let dx = angle_60.cos() * 5.0;
        let dy = angle_60.sin() * 5.0;

        assert!(
            !is_in_fov(dx, dy, facing, half_fov),
            "Target at 60° should NOT be in 90° FOV (±45°)"
        );
    }

    #[test]
    fn test_fov_misses_target_behind() {
        let perception = Perception::new(120.0, 1.0);
        let facing = 0.0_f32;
        let half_fov = perception.half_fov();

        let dx = -5.0_f32;
        let dy = 0.0_f32;

        assert!(
            !is_in_fov(dx, dy, facing, half_fov),
            "Target behind should NOT be in 120° FOV"
        );
    }

    #[test]
    fn test_wide_fov_sees_almost_everywhere() {
        let perception = Perception::new(320.0, 1.0);
        let facing = 0.0_f32;
        let half_fov = perception.half_fov();

        let angle_150 = 150.0_f32.to_radians();
        let dx = angle_150.cos() * 5.0;
        let dy = angle_150.sin() * 5.0;

        assert!(
            is_in_fov(dx, dy, facing, half_fov),
            "Target at 150° should be in 320° FOV"
        );

        let dx_behind = -5.0_f32;
        let dy_behind = 0.0_f32;

        assert!(
            !is_in_fov(dx_behind, dy_behind, facing, half_fov),
            "Target at 180° should be in blind spot of 320° FOV"
        );
    }
}
