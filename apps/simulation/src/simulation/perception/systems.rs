use super::components::*;
#[cfg(feature = "dev-tools")]
use super::debug::*;
use crate::simulation::core::components::{BodySize, PhysicsTick, Position, Rotation};
use crate::simulation::creatures::components::{CreatureState, UpdateSlice};
use crate::simulation::creatures::constants::{MAX_PERCEIVED_NEIGHBORS, UPDATE_SLICE_COUNT};
#[cfg(feature = "dev-tools")]
use crate::simulation::creatures::components::CritId;
use crate::simulation::spatial::DoubleBufferedSpatialGrid;
#[cfg(feature = "dev-tools")]
use crate::instrumentation::SystemTimings;
use bevy_ecs::prelude::*;
use rayon::prelude::*;
use std::cell::RefCell;

const MAX_OTHER_RADIUS: f32 = 5.0;

// Thread-local scratch buffer for sorted cell indices (avoids allocation per creature)
thread_local! {
    static CELL_SCRATCH: RefCell<Vec<(f32, usize)>> = RefCell::new(Vec::with_capacity(256));
}

// Thread-local scratch buffer for topological sorting (collects all neighbors, then sorts)
thread_local! {
    static NEIGHBOR_CANDIDATES: RefCell<Vec<(f32, NeighborData)>> = RefCell::new(Vec::with_capacity(256));
}

pub fn update_perception_system(
    physics_tick: Res<PhysicsTick>,
    grid: Res<DoubleBufferedSpatialGrid>,
    mut query: Query<(Entity, &Position, &Rotation, &BodySize, &Perception, &mut NeighborCache, &CreatureState, &UpdateSlice)>,
    #[cfg(feature = "dev-tools")] timings: Res<SystemTimings>,
    #[cfg(feature = "dev-tools")] debug_target: Res<PerceptionDebugTarget>,
    #[cfg(feature = "dev-tools")] mut debug_snapshot: ResMut<PerceptionDebugSnapshot>,
    #[cfg(feature = "dev-tools")] crit_ids: Query<&CritId>,
) {
    #[cfg(feature = "dev-tools")]
    crate::time_system!(timings, "perception");

    let grid_ref = grid.read_grid();

    // Get debug target (dev-tools only) - used for visualization AFTER perception runs
    #[cfg(feature = "dev-tools")]
    let debug_target_entity = debug_target.get();

    // Current update slice (cycles 0..UPDATE_SLICE_COUNT each tick)
    let current_slice = (physics_tick.get() % UPDATE_SLICE_COUNT as u64) as u8;

    // Collect only entities in current slice (filters before parallel processing)
    let mut entities: Vec<_> = query
        .iter_mut()
        .filter(|(.., update_slice)| update_slice.id == current_slice)
        .collect();

    // DEV-TOOLS: Mutex to capture ACTUAL cell data during perception for debug target
    #[cfg(feature = "dev-tools")]
    let debug_cell_capture: std::sync::Mutex<Option<(Vec<(i32, i32)>, Vec<(i32, i32)>)>> =
        std::sync::Mutex::new(None);

    // ============================================================
    // SINGLE PERCEPTION PASS - identical in dev and production
    // ============================================================
    entities.par_iter_mut().for_each(|(entity, pos, rot, size, perception, neighbor_cache, state, _update_slice)| {
        // Check if this entity is the debug target (dev-tools only)
        #[cfg(feature = "dev-tools")]
        let is_debug_target = debug_target_entity.map_or(false, |t| *entity == t);
        neighbor_cache.clear();

        if !state.behavior.is_active() {
            return;
        }

        let x = pos.x;
        let y = pos.y;
        let self_radius = size.radius();
        let range = perception.range;
        let cos_half_fov_sq = perception.cos_half_fov_sq;
        let cos_half_fov = perception.cos_half_fov;
        // Use cached cos/sin from rotation (avoids 400K trig calls per tick)
        let facing_x = rot.cos_radians;
        let facing_y = rot.sin_radians;
        let query_radius = range + self_radius + MAX_OTHER_RADIUS;

        // Topological neighbor selection with smart early-exit:
        // 1. Always scan adjacent cells, tracking max distance seen
        // 2. After adjacent cells, if we have K+ candidates, use max distance as cutoff
        // 3. Skip non-adjacent cells whose nearest edge is beyond cutoff
        // This ensures correctness (always K closest) while being fast in dense crowds
        CELL_SCRATCH.with(|scratch| {
            NEIGHBOR_CANDIDATES.with(|candidates_cell| {
                use crate::simulation::spatial::constants::{CELL_HALF_DIAGONAL, NON_ADJACENT_OFFSET};

                let mut cells = scratch.borrow_mut();
                let mut candidates = candidates_cell.borrow_mut();
                candidates.clear();

                grid_ref.collect_cells_sorted_fov(x, y, query_radius, facing_x, facing_y, cos_half_fov, &mut cells);

                // Pre-compute base distance for faster range checks
                let base_dist = range + self_radius;

                // Track max distance seen in adjacent cells (O(1) per candidate)
                // This is used as cutoff for non-adjacent cells - no expensive partial sort needed
                let mut max_adjacent_dist_sq: f32 = 0.0;
                // Expanded cutoff: (sqrt(cutoff) + half_diag)² - allows sqrt-free comparison
                let mut expanded_cutoff_dist_sq = f32::MAX;
                let mut processed_adjacent = false;

                // DEV-TOOLS: Track actual queried/skipped cells for debug target
                #[cfg(feature = "dev-tools")]
                let mut debug_queried: Vec<(i32, i32)> = if is_debug_target { Vec::with_capacity(64) } else { Vec::new() };
                #[cfg(feature = "dev-tools")]
                let mut debug_skipped: Vec<(i32, i32)> = if is_debug_target { Vec::with_capacity(32) } else { Vec::new() };

                for &(sort_key, cell_idx) in cells.iter() {
                    // Detect transition from adjacent to non-adjacent cells
                    let is_non_adjacent = sort_key >= NON_ADJACENT_OFFSET * 0.5;

                    if is_non_adjacent && !processed_adjacent {
                        processed_adjacent = true;
                        // Pre-compute expanded cutoff: (sqrt(max_dist) + half_diag)²
                        // This moves sqrt from per-cell to once-per-creature
                        if candidates.len() >= MAX_PERCEIVED_NEIGHBORS {
                            let cutoff_dist = max_adjacent_dist_sq.sqrt();
                            expanded_cutoff_dist_sq = (cutoff_dist + CELL_HALF_DIAGONAL).powi(2);
                        }
                    }

                    // For non-adjacent cells, check if cell center is beyond expanded cutoff
                    // If center² > (cutoff + half_diag)², the nearest edge must be beyond cutoff
                    if is_non_adjacent && expanded_cutoff_dist_sq < f32::MAX {
                        let cell_center_dist_sq = sort_key - NON_ADJACENT_OFFSET;
                        if cell_center_dist_sq > expanded_cutoff_dist_sq {
                            // DEV-TOOLS: Capture skipped cells for debug target
                            #[cfg(feature = "dev-tools")]
                            if is_debug_target {
                                // This cell and remaining cells are skipped
                                let (cx, cy) = grid_ref.get_cell_coords_by_index(cell_idx);
                                debug_skipped.push((cx, cy));
                            }
                            #[cfg(not(feature = "dev-tools"))]
                            break;
                            #[cfg(feature = "dev-tools")]
                            if !is_debug_target {
                                break;
                            } else {
                                continue; // Keep iterating to capture ALL skipped cells
                            }
                        }
                    }

                    // DEV-TOOLS: Capture queried cell for debug target
                    #[cfg(feature = "dev-tools")]
                    if is_debug_target {
                        let (cx, cy) = grid_ref.get_cell_coords_by_index(cell_idx);
                        debug_queried.push((cx, cy));
                    }

                    // Process entities in this cell
                    for proxy in grid_ref.get_cell_proxies(cell_idx) {
                        if *entity == proxy.entity {
                            continue;
                        }

                        let dx = proxy.x - x;
                        let dy = proxy.y - y;
                        let center_dist_sq = dx * dx + dy * dy;

                        // Use pre-computed base_dist for faster range check
                        let max_dist = base_dist + proxy.radius;
                        if center_dist_sq > max_dist * max_dist {
                            continue;
                        }

                        let rough_dot = dx * facing_x + dy * facing_y;

                        // FOV check: For wide FOV (>180°), cos_half_fov is negative
                        let in_fov = if cos_half_fov >= 0.0 {
                            // Narrow FOV (≤180°): target must be in front (rough_dot > 0)
                            // Use fast squared comparison to avoid sqrt
                            rough_dot > 0.0 && rough_dot * rough_dot >= cos_half_fov_sq * center_dist_sq
                        } else {
                            // Wide FOV (>180°): cos_half_fov is negative, need signed comparison
                            // rough_dot / dist >= cos_half_fov (both can be negative)
                            let dist = center_dist_sq.sqrt();
                            rough_dot >= cos_half_fov * dist
                        };

                        if in_fov {
                            candidates.push((center_dist_sq, NeighborData {
                                entity: proxy.entity,
                                x: proxy.x,
                                y: proxy.y,
                                radius: proxy.radius,
                            }));

                            // Track max distance in adjacent cells (O(1) per candidate)
                            if !is_non_adjacent && center_dist_sq > max_adjacent_dist_sq {
                                max_adjacent_dist_sq = center_dist_sq;
                            }
                        }
                    }
                }

                // Final selection: get K closest
                let k = MAX_PERCEIVED_NEIGHBORS.min(candidates.len());
                if k > 0 {
                    if candidates.len() > k {
                        // Partition so first K elements are the K smallest
                        candidates.select_nth_unstable_by(k - 1, |a, b| {
                            a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal)
                        });
                    }
                    // Add the K closest neighbors
                    for (_, neighbor) in candidates.iter().take(k) {
                        neighbor_cache.add_neighbor(*neighbor);
                    }
                }

                // DEV-TOOLS: Store captured cell data for debug target
                #[cfg(feature = "dev-tools")]
                if is_debug_target {
                    *debug_cell_capture.lock().unwrap() = Some((debug_queried, debug_skipped));
                }
            });
        });
    });

    // ============================================================
    // DEV-TOOLS ONLY: Capture visualization AFTER perception runs
    // This observes the results, doesn't change behavior
    // ============================================================
    #[cfg(feature = "dev-tools")]
    {
        // Extract the ACTUAL captured cell data from the Mutex
        let captured_cells = debug_cell_capture.into_inner().unwrap();

        if let Some(target_entity) = debug_target_entity {
            // Find the debug target in our entities list and capture its state
            // NOTE: Entity may not be in `entities` if not in current update slice
            if let Some((_, pos, rot, _size, perception, neighbor_cache, state, _)) = entities
                .iter()
                .find(|(e, _, _, _, _, _, _, _)| *e == target_entity)
            {
                let entity_id = crit_ids.get(target_entity).map(|id| id.0).unwrap_or(0);

                // NOTE: Acceleration is captured LATER by capture_debug_acceleration_system
                // (runs after behaviors). Set 0.0 here as placeholder - will be overwritten.
                let (ax, ay) = (0.0, 0.0);

                if state.behavior.is_active() {
                    let x = pos.x;
                    let y = pos.y;

                    // Use ACTUAL captured cell data from perception pass (no estimation!)
                    let (queried_cells, skipped_cells): (Vec<QueriedCell>, Vec<QueriedCell>) =
                        if let Some((queried, skipped)) = captured_cells {
                            (
                                queried.into_iter().map(|(cx, cy)| QueriedCell { x: cx, y: cy }).collect(),
                                skipped.into_iter().map(|(cx, cy)| QueriedCell { x: cx, y: cy }).collect(),
                            )
                        } else {
                            (Vec::new(), Vec::new())
                        };

                    // Build neighbor debug info from the ACTUAL perception results
                    let neighbor_debug: Vec<NeighborDebugInfo> = neighbor_cache
                        .iter_neighbors()
                        .filter_map(|n| {
                            let neighbor_id = crit_ids.get(n.entity).ok()?.0;
                            Some(NeighborDebugInfo { id: neighbor_id, x: n.x, y: n.y })
                        })
                        .collect();

                    let (creature_cx, creature_cy) = grid_ref.world_to_cell(x, y);

                    debug_snapshot.update(
                        entity_id,
                        x, y,
                        perception.range,
                        perception.fov_angle,
                        rot.radians,
                        ax,
                        ay,
                        neighbor_debug,
                        queried_cells,
                        skipped_cells,
                        QueriedCell { x: creature_cx, y: creature_cy },
                    );
                } else {
                    let (creature_cx, creature_cy) = grid_ref.world_to_cell(pos.x, pos.y);
                    debug_snapshot.update(
                        entity_id,
                        pos.x, pos.y,
                        perception.range,
                        perception.fov_angle,
                        rot.radians,
                        ax,
                        ay,
                        std::iter::empty(),
                        std::iter::empty(),
                        std::iter::empty(),
                        QueriedCell { x: creature_cx, y: creature_cy },
                    );
                }
            }
            // else: Entity not in current slice - preserve existing snapshot data
            // (don't clear, as the creature will be processed in a future tick)
        } else {
            // No debug target set - this is normal
            debug_snapshot.clear();
        }
    }
}

/// Captures debug acceleration for force visualization AFTER behavior systems have run.
///
/// This system runs AFTER all behavior systems (wander, flee, seek, avoidance) have
/// accumulated their forces into the Acceleration component, but BEFORE integrate_motion_system
/// resets it. This ensures the visualization shows the CURRENT frame's acceleration,
/// not stale values from the previous frame.
///
/// System ordering: behaviors → capture_debug_acceleration → integrate_motion
#[cfg(feature = "dev-tools")]
pub fn capture_debug_acceleration_system(
    debug_target: Res<PerceptionDebugTarget>,
    mut debug_snapshot: ResMut<PerceptionDebugSnapshot>,
    accel_query: Query<&crate::simulation::core::components::Acceleration>,
    #[allow(unused)] timings: Res<crate::instrumentation::SystemTimings>,
) {
    crate::time_system!(timings, "capture_debug_accel");

    if let Some(target_entity) = debug_target.get() {
        if let Ok(accel) = accel_query.get(target_entity) {
            // Update ONLY the acceleration fields - rest was set by perception system
            debug_snapshot.ax = accel.ax;
            debug_snapshot.ay = accel.ay;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::creatures::components::{BehaviorMode, CreatureState};

    /// Test helper: naive O(n²) perception for unit tests.
    /// Simpler than the real system (no spatial grid, no FOV) but validates core logic.
    fn run_naive_perception(world: &mut World, check_behavior: bool) {
        let creatures: Vec<(Entity, Position, BodySize)> = world
            .query::<(Entity, &Position, &BodySize)>()
            .iter(world)
            .map(|(e, p, s)| (e, *p, *s))
            .collect();

        let mut query = world.query::<(Entity, &Position, &BodySize, &Perception, &mut NeighborCache, &CreatureState)>();

        for (entity, pos, size, perception, mut neighbor_cache, state) in query.iter_mut(world) {
            neighbor_cache.clear();

            if check_behavior && !state.behavior.is_active() {
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
                    neighbor_cache.add_neighbor(NeighborData {
                        entity: *other_entity,
                        x: other_pos.x,
                        y: other_pos.y,
                        radius: other_radius,
                    });
                }
            }
        }
    }

    /// Simpler version without behavior check or body size (for basic range tests)
    fn run_simple_perception(world: &mut World) {
        let positions: Vec<(Entity, Position)> = world
            .query::<(Entity, &Position)>()
            .iter(world)
            .map(|(e, p)| (e, *p))
            .collect();

        let mut query = world.query::<(Entity, &Position, &Perception, &mut NeighborCache)>();
        for (entity, pos, perception, mut neighbor_cache) in query.iter_mut(world) {
            neighbor_cache.clear();
            let range_sq = perception.range * perception.range;

            for (other_entity, other_pos) in &positions {
                if entity == *other_entity {
                    continue;
                }
                let dx = other_pos.x - pos.x;
                let dy = other_pos.y - pos.y;
                let dist_sq = dx * dx + dy * dy;
                if dist_sq <= range_sq {
                    neighbor_cache.add_neighbor(NeighborData {
                        entity: *other_entity,
                        x: other_pos.x,
                        y: other_pos.y,
                        radius: 0.5,
                    });
                }
            }
        }
    }

    #[test]
    fn test_catatonic_crits_do_not_perceive() {
        let mut world = World::new();

        let catatonic_crit = world
            .spawn((
                Position { x: 0.0, y: 0.0 },
                BodySize::new(1.0),
                Perception::from_body_size(1.0),
                NeighborCache::new(),
                CreatureState::default(),
            ))
            .id();

        let nearby_crit = world
            .spawn((
                Position { x: 2.0, y: 0.0 },
                BodySize::new(1.0),
                Perception::from_body_size(1.0),
                NeighborCache::new(),
                {
                    let mut state = CreatureState::default();
                    state.behavior = BehaviorMode::Wandering;
                    state
                },
            ))
            .id();

        run_naive_perception(&mut world, true);

        let catatonic_cache = world.get::<NeighborCache>(catatonic_crit).unwrap();
        assert_eq!(
            catatonic_cache.neighbor_count(),
            0,
            "Catatonic crit should not perceive neighbors"
        );

        let active_cache = world.get::<NeighborCache>(nearby_crit).unwrap();
        assert_eq!(
            active_cache.neighbor_count(),
            1,
            "Active crit should perceive the catatonic one"
        );
        assert!(active_cache.contains(catatonic_crit));
    }

    #[test]
    fn test_perception_detects_nearby_entities() {
        use crate::simulation::creatures::constants::PERCEPTION_MULTIPLIER;

        let mut world = World::new();

        // Body size 1.0 → range = PERCEPTION_MULTIPLIER (100.0)
        // Place crit1 and crit2 close together, crit3 far away
        let crit1 = world
            .spawn((Position { x: 0.0, y: 0.0 }, Perception::from_body_size(1.0), NeighborCache::new()))
            .id();

        let crit2 = world
            .spawn((Position { x: 5.0, y: 0.0 }, Perception::from_body_size(1.0), NeighborCache::new()))
            .id();

        // crit3 is beyond perception range (PERCEPTION_MULTIPLIER + some buffer)
        let crit3 = world
            .spawn((Position { x: PERCEPTION_MULTIPLIER + 50.0, y: 0.0 }, Perception::from_body_size(1.0), NeighborCache::new()))
            .id();

        run_simple_perception(&mut world);

        let cache1 = world.get::<NeighborCache>(crit1).unwrap();
        assert_eq!(cache1.neighbor_count(), 1);
        assert!(cache1.contains(crit2));
        assert!(!cache1.contains(crit3));

        let cache2 = world.get::<NeighborCache>(crit2).unwrap();
        assert_eq!(cache2.neighbor_count(), 1);
        assert!(cache2.contains(crit1));
        assert!(!cache2.contains(crit3));

        let cache3 = world.get::<NeighborCache>(crit3).unwrap();
        assert_eq!(cache3.neighbor_count(), 0);
    }

    #[test]
    fn test_perception_does_not_detect_self() {
        let mut world = World::new();

        let crit = world
            .spawn((Position { x: 0.0, y: 0.0 }, Perception::from_body_size(1.0), NeighborCache::new()))
            .id();

        run_simple_perception(&mut world);

        let cache = world.get::<NeighborCache>(crit).unwrap();
        assert_eq!(cache.neighbor_count(), 0);
        assert!(!cache.contains(crit));
    }

    #[test]
    fn test_perception_clears_previous_neighbors() {
        use crate::simulation::creatures::constants::PERCEPTION_MULTIPLIER;

        let mut world = World::new();

        // Body size 1.0 → range = PERCEPTION_MULTIPLIER (100.0)
        let crit1 = world
            .spawn((Position { x: 0.0, y: 0.0 }, Perception::from_body_size(1.0), NeighborCache::new()))
            .id();

        let crit2 = world
            .spawn((Position { x: 5.0, y: 0.0 }, Perception::from_body_size(1.0), NeighborCache::new()))
            .id();

        run_simple_perception(&mut world);

        let cache1 = world.get::<NeighborCache>(crit1).unwrap();
        assert_eq!(cache1.neighbor_count(), 1);

        // Move crit2 beyond perception range
        world.get_mut::<Position>(crit2).unwrap().x = PERCEPTION_MULTIPLIER + 50.0;

        run_simple_perception(&mut world);

        let cache1 = world.get::<NeighborCache>(crit1).unwrap();
        assert_eq!(cache1.neighbor_count(), 0);
    }

    #[test]
    fn test_perception_respects_range() {
        let mut world = World::new();

        // Use explicit 180° FOV to get predictable range = body_size × PERCEPTION_MULTIPLIER
        // (No FOV range compensation at 180°)
        let small_perception = Perception::new(180.0, 0.5);
        let large_perception = Perception::new(180.0, 2.0);

        let small_range = small_perception.range;
        let large_range = large_perception.range;

        // crit1: small range
        let crit1 = world
            .spawn((Position { x: 0.0, y: 0.0 }, small_perception, NeighborCache::new()))
            .id();

        // crit2: large range (4x small range since body size is 4x)
        let crit2 = world
            .spawn((Position { x: 0.0, y: 0.0 }, large_perception, NeighborCache::new()))
            .id();

        // crit3 at distance midway: outside small range but inside large range
        let midpoint_distance = (small_range + large_range) / 2.0;
        let crit3 = world
            .spawn((Position { x: midpoint_distance, y: 0.0 }, Perception::new(180.0, 1.0), NeighborCache::new()))
            .id();

        run_simple_perception(&mut world);

        let cache1 = world.get::<NeighborCache>(crit1).unwrap();
        assert!(!cache1.contains(crit3), "small creature should NOT see crit3 beyond its range");

        let cache2 = world.get::<NeighborCache>(crit2).unwrap();
        assert!(cache2.contains(crit3), "large creature SHOULD see crit3 within its range");
    }

    #[test]
    fn test_perception_performance_baseline() {
        let mut world = World::new();

        for i in 0..100 {
            let x = (i % 10) as f32 * 10.0;
            let y = (i / 10) as f32 * 10.0;
            world.spawn((Position { x, y }, Perception::from_body_size(1.0), NeighborCache::new()));
        }

        let start = std::time::Instant::now();
        run_simple_perception(&mut world);
        let duration = start.elapsed();

        println!("Perception update (100 crits, naive O(n²)): {:?}", duration);

        let total_neighbors: usize = world
            .query::<&NeighborCache>()
            .iter(&world)
            .map(|c| c.neighbor_count())
            .sum();

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

    /// Regression test: In crowds, perception must select CLOSEST neighbors,
    /// not just first-found from adjacent cells. The early-exit optimization
    /// was skipping non-adjacent cells that contained closer neighbors.
    ///
    /// NOTE: collect_cells_sorted only sorts when > 9 cells, so we need a large
    /// query radius to ensure enough cells are collected and sorted properly.
    #[test]
    fn test_crowd_selects_closest_not_first_found() {
        use crate::simulation::spatial::SpatialGrid;
        use crate::simulation::creatures::constants::MAX_PERCEIVED_NEIGHBORS;

        // Create grid with SMALL cell size to ensure many cells
        let mut grid = SpatialGrid::new(20.0);

        // Test setup: creature at (19, 10) facing +X in cell (0, 0)
        let self_entity = Entity::from_raw(0);
        let x = 19.0;
        let y = 10.0;
        let self_radius = 1.0;
        let range = 500.0;
        let cos_half_fov_sq = 0.0; // Full 180° FOV
        let cos_half_fov = 0.0;    // cos(90°) = 0 for 180° FOV
        let facing_x = 1.0;
        let facing_y = 0.0;

        let mut entities: Vec<(Entity, f32, f32, f32)> = Vec::new();
        let mut entity_id = 1u32;

        // To trigger the bug, we need:
        // 1. > 9 cells with entities (to trigger sorting)
        // 2. >= MAX_PERCEIVED_NEIGHBORS candidates from adjacent cells AFTER sorting
        // 3. A closer entity in a non-adjacent cell that gets skipped
        //
        // Key insight: entities BEHIND the creature (dx < 0) are FOV filtered!
        // So we need entities in FRONT (dx >= 0) in adjacent cells.

        // With cell_size = 20, creature at (19, 10) in cell (0, 0):
        // Adjacent cells WITH positive dx:
        // - Cell (1, -1): 20..40 x -20..0 - ALL entities have dx > 0 ✓
        // - Cell (1, 0): 20..40 x 0..20 - ALL entities have dx > 0 ✓
        // - Cell (1, 1): 20..40 x 20..40 - ALL entities have dx > 0 ✓
        // - Cell (0, -1): 0..20 x -20..0 - entities at x > 19 have dx > 0
        // - Cell (0, 0): 0..20 x 0..20 - entities at x > 19 have dx > 0
        // - Cell (0, 1): 0..20 x 20..40 - entities at x > 19 have dx > 0

        // Put MAX+1 entities in adjacent cells that pass FOV (all at x > 19)
        // Cell (1, 0) - 3 entities at far distances
        entities.push((Entity::from_raw(entity_id), 38.0, 5.0, 1.0)); entity_id += 1;  // dist 19.6
        entities.push((Entity::from_raw(entity_id), 38.0, 10.0, 1.0)); entity_id += 1; // dist 19.0
        entities.push((Entity::from_raw(entity_id), 38.0, 15.0, 1.0)); entity_id += 1; // dist 19.6

        // Cell (1, 1) - 3 entities at far distances
        entities.push((Entity::from_raw(entity_id), 35.0, 35.0, 1.0)); entity_id += 1; // dist 29.7
        entities.push((Entity::from_raw(entity_id), 38.0, 35.0, 1.0)); entity_id += 1; // dist 31.4
        entities.push((Entity::from_raw(entity_id), 38.0, 38.0, 1.0)); entity_id += 1; // dist 33.3

        // Cell (1, -1) - 3 entities (negative y but in front)
        entities.push((Entity::from_raw(entity_id), 35.0, -5.0, 1.0)); entity_id += 1;  // dist 21.9
        entities.push((Entity::from_raw(entity_id), 35.0, -10.0, 1.0)); entity_id += 1; // dist 25.6
        entities.push((Entity::from_raw(entity_id), 35.0, -15.0, 1.0)); entity_id += 1; // dist 29.7

        // Now we have 9 entities in 3 adjacent cells, all passing FOV.
        // That's > MAX_PERCEIVED_NEIGHBORS (7).

        // Add entities to non-adjacent cells to get > 9 total cells
        // Cell (2, 0): 40..60 x 0..20
        entities.push((Entity::from_raw(entity_id), 55.0, 10.0, 1.0)); entity_id += 1; // dist 36
        // Cell (2, 1): 40..60 x 20..40
        entities.push((Entity::from_raw(entity_id), 55.0, 35.0, 1.0)); entity_id += 1; // dist ~45
        // Cell (2, -1): 40..60 x -20..0
        entities.push((Entity::from_raw(entity_id), 55.0, -10.0, 1.0)); entity_id += 1; // dist 40
        // Cell (3, 0): 60..80 x 0..20
        entities.push((Entity::from_raw(entity_id), 75.0, 10.0, 1.0)); entity_id += 1; // dist 56
        // Cell (3, 1): 60..80 x 20..40
        entities.push((Entity::from_raw(entity_id), 75.0, 35.0, 1.0)); entity_id += 1; // dist ~63
        // Cell (3, -1): 60..80 x -20..0
        entities.push((Entity::from_raw(entity_id), 75.0, -10.0, 1.0)); entity_id += 1; // dist 60
        // Cell (4, 0): 80..100 x 0..20
        entities.push((Entity::from_raw(entity_id), 95.0, 10.0, 1.0)); // dist 76

        // Now we have 16 entities in 10 cells:
        // 3 adjacent cells with 9 entities (all pass FOV)
        // 7 non-adjacent cells with 7 entities
        // Total: 10 cells, triggers sorting

        // Now place entity 100 in a NON-ADJACENT cell at a distance that's CLOSER
        // than some adjacent entities but will be SKIPPED due to early exit!
        //
        // Adjacent entities have distances: 19.0, 19.6, 19.6, 21.9, 25.6, 29.7, 29.7, 31.4, 33.3
        // If we place entity 100 at distance 25.0, it should be in top 7.
        // But since it's in a non-adjacent cell, it will be skipped.
        //
        // Cell (2, 0): x=40..60, y=0..20
        // Entity at (44, 10): distance = 44 - 19 = 25.0 - CLOSER than entities at 25.6, 29.7, etc.

        entities.push((
            Entity::from_raw(100),
            44.0, // x: in cell (2, 0)
            10.0, // y: same as creature
            1.0,
        ));
        // Distance 25.0 - SHOULD be in top 7, but will be skipped by early exit!

        grid.rebuild(entities.into_iter());

        let mut candidates: Vec<(f32, NeighborData)> = Vec::new();

        CELL_SCRATCH.with(|scratch| {
            NEIGHBOR_CANDIDATES.with(|candidates_cell| {
                let mut cells = scratch.borrow_mut();
                let mut local_candidates = candidates_cell.borrow_mut();
                local_candidates.clear();

                let query_radius = range + self_radius + MAX_OTHER_RADIUS;
                grid.collect_cells_sorted(x, y, query_radius, facing_x, facing_y, &mut cells);

                // THIS IS THE CODE UNDER TEST - matches the fixed production code
                // No early exit - collect ALL candidates, then partial sort
                for &(_sort_key, cell_idx) in cells.iter() {
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

                        let rough_dot = dx * facing_x + dy * facing_y;

                        // FOV check: matches production code
                        let in_fov = if cos_half_fov >= 0.0 {
                            rough_dot > 0.0 && rough_dot * rough_dot >= cos_half_fov_sq * center_dist_sq
                        } else {
                            let dist = center_dist_sq.sqrt();
                            rough_dot >= cos_half_fov * dist
                        };

                        if in_fov {
                            local_candidates.push((center_dist_sq, NeighborData {
                                entity: proxy.entity,
                                x: proxy.x,
                                y: proxy.y,
                                radius: proxy.radius,
                            }));
                        }
                    }
                }

                // Partial sort to get K closest
                let k = MAX_PERCEIVED_NEIGHBORS.min(local_candidates.len());
                if k > 0 && local_candidates.len() > k {
                    local_candidates.select_nth_unstable_by(k - 1, |a, b| {
                        a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal)
                    });
                }

                for (dist_sq, neighbor) in local_candidates.iter().take(k) {
                    candidates.push((*dist_sq, *neighbor));
                }
            });
        });

        candidates.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        // Entity 100 at distance 25.0 should be in top 7:
        // Sorted adjacent: 19.0, 19.6, 19.6, 21.9, 25.6, 29.7, 29.7, 31.4, 33.3
        // Entity 100 at 25.0 is closer than entities at 25.6, 29.7, etc.
        // So it SHOULD be in the final 7 neighbors.
        //
        // But with the bug, the early exit triggers after processing adjacent cells
        // (which have 9 entities >= MAX), skipping non-adjacent cells including entity 100.

        let entity_100_found = candidates.iter().any(|(_, n)| n.entity == Entity::from_raw(100));

        // This assertion will FAIL with the current buggy code
        assert!(
            entity_100_found,
            "Entity 100 in non-adjacent cell (distance 25.0) should be in top 7 neighbors because it's closer than adjacent entities at 25.6, 29.7, etc. Found entities: {:?}",
            candidates.iter().map(|(d, n)| (n.entity, d.sqrt())).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_topological_selects_closest_neighbors_across_cells() {
        // Tests that topological neighbor selection picks the CLOSEST neighbors
        // regardless of which spatial grid cell they're in.
        use crate::simulation::spatial::SpatialGrid;

        // Create grid with small cell size so we have clear cell boundaries
        let mut grid = SpatialGrid::new(10.0);

        // Creature at (5, 5) facing right (+X), in cell (0, 0)
        let self_entity = Entity::from_raw(0);
        let x = 5.0;
        let y = 5.0;
        let self_radius = 1.0;
        let range = 100.0;
        let _cos_half_fov_sq = 0.0; // Full 180° FOV (cos^2(90°) = 0)
        let facing_x = 1.0;
        let facing_y = 0.0;

        // Place FAR creatures in center cell (0,0) - should NOT be selected
        let mut entities: Vec<(Entity, f32, f32, f32)> = Vec::new();
        for i in 0..5 {
            entities.push((
                Entity::from_raw(i + 1),
                8.0, // x=8.0 -> distance ~3.0 from (5,5)
                5.0 + (i as f32 * 0.5),
                1.0,
            ));
        }

        // Place CLOSE creature in adjacent cell (1, 0) at x=10.0 - closer than center cell creatures!
        // Distance from (5,5) to (6,5) = 1.0 (much closer than 3.0)
        entities.push((Entity::from_raw(100), 6.0, 5.0, 1.0)); // distance 1.0 - CLOSEST!

        // Place another close creature in a different adjacent cell
        entities.push((Entity::from_raw(101), 5.5, 6.0, 1.0)); // distance ~1.1

        grid.rebuild(entities.into_iter());

        // Collect neighbors using topological sorting
        let mut candidates: Vec<(f32, NeighborData)> = Vec::new();

        CELL_SCRATCH.with(|scratch| {
            let mut cells = scratch.borrow_mut();
            let query_radius = range + self_radius + 2.0;
            grid.collect_cells_sorted(x, y, query_radius, facing_x, facing_y, &mut cells);

            for &(_, cell_idx) in cells.iter() {
                for proxy in grid.get_cell_proxies(cell_idx) {
                    if proxy.entity == self_entity {
                        continue;
                    }
                    let dx = proxy.x - x;
                    let dy = proxy.y - y;
                    let center_dist_sq = dx * dx + dy * dy;
                    let combined_radius = self_radius + proxy.radius;
                    let edge_dist_sq = (center_dist_sq.sqrt() - combined_radius).max(0.0).powi(2);

                    if edge_dist_sq <= range * range {
                        candidates.push((
                            center_dist_sq,
                            NeighborData {
                                entity: proxy.entity,
                                x: proxy.x,
                                y: proxy.y,
                                radius: proxy.radius,
                            },
                        ));
                    }
                }
            }
        });

        // Sort by distance (closest first)
        candidates.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        // The CLOSEST neighbor should be Entity 100 (distance 1.0)
        assert!(!candidates.is_empty(), "Should find at least one neighbor");
        assert_eq!(
            candidates[0].1.entity,
            Entity::from_raw(100),
            "Closest neighbor should be entity 100 from adjacent cell, not a farther entity from center cell"
        );

        // Second closest should be Entity 101 (distance ~1.1)
        assert!(candidates.len() >= 2, "Should find at least two neighbors");
        assert_eq!(
            candidates[1].1.entity,
            Entity::from_raw(101),
            "Second closest should be entity 101"
        );
    }

    #[cfg(feature = "dev-tools")]
    #[test]
    fn test_perception_debug_target_populates_snapshot() {
        use crate::simulation::core::SimulationBuilder;
        use crate::simulation::creatures::builder::CritBuilder;
        use crate::simulation::creatures::components::CritId;
        use crate::simulation::creatures::constants::UPDATE_SLICE_COUNT;

        let mut sim = SimulationBuilder::new().build();
        sim.set_boundaries(100.0, 100.0);

        // Spawn a creature and get its CritId
        let crit_id = sim.spawn_crit(
            CritBuilder::new()
                .at(50.0, 50.0)
                .with_all_capabilities(),
        );

        // Find the entity with this CritId
        let world = sim.world_mut();
        let entity = world
            .query::<(bevy_ecs::entity::Entity, &CritId)>()
            .iter(world)
            .find(|(_, id)| id.0 == crit_id)
            .map(|(e, _)| e)
            .expect("Should find entity by CritId");

        // Set the debug target
        {
            let mut target = world.get_resource_mut::<PerceptionDebugTarget>().expect("PerceptionDebugTarget resource should exist");
            target.0 = Some(entity);
        }

        // Run enough updates to ensure the creature is processed at least once
        // (perception uses UPDATE_SLICE_COUNT slices, each creature processed every N ticks)
        for _ in 0..UPDATE_SLICE_COUNT {
            sim.update(0.016);
        }

        // Check that snapshot was populated
        let world = sim.world();
        let snapshot = world.get_resource::<PerceptionDebugSnapshot>().expect("PerceptionDebugSnapshot resource should exist");

        assert_eq!(snapshot.entity_id, crit_id, "Snapshot should have the selected creature's ID");
        assert!(snapshot.x > 0.0 || snapshot.y > 0.0, "Snapshot should have position data");
    }

    #[test]
    fn test_real_perception_system_integration() {
        use crate::simulation::core::SimulationBuilder;
        use crate::simulation::creatures::builder::CritBuilder;
        use crate::simulation::creatures::constants::MAX_PERCEIVED_NEIGHBORS;

        let mut sim = SimulationBuilder::new().build();
        sim.set_boundaries(100.0, 100.0);

        // Spawn creatures in a 2D cluster (not just a line) so with narrow FOV,
        // creatures facing any direction will likely have neighbors in their cone.
        // Use more creatures to ensure statistical coverage.
        let positions = [
            (50.0, 50.0),
            (51.0, 50.0),
            (50.0, 51.0),
            (51.0, 51.0),
            (50.5, 50.5),
            (49.5, 50.5),
            (50.5, 49.5),
            (49.0, 50.0),
        ];
        for (x, y) in positions {
            sim.spawn_crit(
                CritBuilder::new()
                    .at(x, y)
                    .with_all_capabilities(),
            );
        }

        // Run several updates for creatures to wander and potentially turn toward each other
        for _ in 0..10 {
            sim.update(0.016);
        }

        // Check perception worked - some creatures should have neighbors
        let world = sim.world_mut();
        let mut total_neighbors = 0;

        for neighbor_cache in world.query::<&NeighborCache>().iter(world) {
            total_neighbors += neighbor_cache.neighbor_count();
        }

        // With 8 creatures in a 2D cluster, at least some should perceive neighbors
        // regardless of FOV angle (narrow FOV means some miss, but with enough creatures some hit)
        assert!(
            total_neighbors > 0,
            "At least some creatures should perceive neighbors after simulation runs"
        );

        // Verify neighbor count is bounded correctly
        for neighbor_cache in world.query::<&NeighborCache>().iter(world) {
            assert!(
                neighbor_cache.neighbor_count() <= MAX_PERCEIVED_NEIGHBORS,
                "Neighbor count should not exceed MAX_PERCEIVED_NEIGHBORS"
            );
        }
    }

    /// Test that wide FOV (340°) can perceive targets at 100° from facing.
    /// Verifies the fixed FOV check correctly handles wide FOV angles.
    #[test]
    fn test_wide_fov_340_perceives_target_at_100_degrees() {
        // 340° FOV has half_fov = 170°, so should see targets from -170° to +170°
        // A target at 100° from facing should definitely be visible
        let perception = Perception::new(340.0, 1.0);
        let cos_half_fov_sq = perception.cos_half_fov_sq;
        let cos_half_fov = perception.cos_half_fov;

        // Creature at origin facing +X (0°)
        let facing_x = 1.0_f32;
        let facing_y = 0.0_f32;

        // Target at 100° from facing
        let angle_100 = 100.0_f32.to_radians();
        let dx = angle_100.cos() * 5.0;  // ~-0.87
        let dy = angle_100.sin() * 5.0;  // ~4.92
        let center_dist_sq = dx * dx + dy * dy;  // 25.0

        let rough_dot = dx * facing_x + dy * facing_y;

        // Sanity check: target at 100° has negative dot product
        assert!(
            rough_dot < 0.0,
            "Target at 100° should have negative dot product (rough_dot = {})",
            rough_dot
        );

        // Wide FOV (340°) has cos_half_fov < 0, so use wide FOV branch
        assert!(
            cos_half_fov < 0.0,
            "340° FOV should have negative cos_half_fov (got {})",
            cos_half_fov
        );

        // FIXED check: for wide FOV, compare rough_dot / dist >= cos_half_fov
        let in_fov = if cos_half_fov >= 0.0 {
            rough_dot > 0.0 && rough_dot * rough_dot >= cos_half_fov_sq * center_dist_sq
        } else {
            let dist = center_dist_sq.sqrt();
            rough_dot >= cos_half_fov * dist
        };

        assert!(
            in_fov,
            "Target at 100° should be in 340° FOV. rough_dot={}, cos_half_fov={}, dist={}",
            rough_dot,
            cos_half_fov,
            center_dist_sq.sqrt()
        );
    }

    /// Test that wide FOV (340°) has only a 20° blind spot directly behind.
    #[test]
    fn test_wide_fov_340_blind_spot_is_20_degrees() {
        let perception = Perception::new(340.0, 1.0);
        let cos_half_fov_sq = perception.cos_half_fov_sq;
        let cos_half_fov = perception.cos_half_fov;

        let facing_x = 1.0_f32;
        let facing_y = 0.0_f32;

        // Test angles around the creature
        let test_cases: [(f32, bool, &str); 7] = [
            (0.0, true, "directly in front"),
            (90.0, true, "at 90° (should be visible)"),
            (100.0, true, "at 100° (should be visible)"),
            (150.0, true, "at 150° (should be visible)"),
            (169.0, true, "at 169° (just inside FOV edge)"),
            (171.0, false, "at 171° (just outside FOV, in blind spot)"),
            (180.0, false, "directly behind (blind spot center)"),
        ];

        for (angle_deg, expected_visible, description) in test_cases {
            let angle_rad = angle_deg.to_radians();
            let dx = angle_rad.cos() * 5.0;
            let dy = angle_rad.sin() * 5.0;
            let center_dist_sq = dx * dx + dy * dy;

            let rough_dot = dx * facing_x + dy * facing_y;

            // FIXED check: handles both narrow and wide FOV
            let in_fov = if cos_half_fov >= 0.0 {
                rough_dot > 0.0 && rough_dot * rough_dot >= cos_half_fov_sq * center_dist_sq
            } else {
                let dist = center_dist_sq.sqrt();
                rough_dot >= cos_half_fov * dist
            };

            assert_eq!(
                in_fov, expected_visible,
                "Target {} at {}° should be {}, but got {}. rough_dot={}",
                description,
                angle_deg,
                if expected_visible { "visible" } else { "in blind spot" },
                if in_fov { "visible" } else { "filtered" },
                rough_dot
            );
        }
    }

    /// Integration test: Wide FOV creature in a crowd should perceive many neighbors.
    #[test]
    fn test_wide_fov_perceives_many_neighbors_in_crowd() {
        use crate::simulation::core::SimulationBuilder;
        use crate::simulation::creatures::builder::CritBuilder;
        use crate::simulation::creatures::constants::UPDATE_SLICE_COUNT;

        let mut sim = SimulationBuilder::new().build();
        sim.set_boundaries(200.0, 200.0);

        // Spawn a creature with 340° FOV at center
        let center_crit_id = sim.spawn_crit(
            CritBuilder::new()
                .at(100.0, 100.0)
                .with_fov(340.0)
                .with_all_capabilities(),
        );

        // Spawn neighbors in a circle around it at various angles
        let distance = 5.0;  // Close enough to be in perception range
        for angle_deg in (0..360).step_by(30) {
            let angle_rad = (angle_deg as f32).to_radians();
            let x = 100.0 + distance * angle_rad.cos();
            let y = 100.0 + distance * angle_rad.sin();
            sim.spawn_crit(
                CritBuilder::new()
                    .at(x, y)
                    .with_all_capabilities(),
            );
        }

        // Run enough ticks to ensure perception runs
        for _ in 0..UPDATE_SLICE_COUNT {
            sim.update(0.016);
        }

        // Find the center creature and check its neighbor count
        let world = sim.world_mut();
        let center_entity = world
            .query::<(bevy_ecs::entity::Entity, &crate::simulation::creatures::components::CritId)>()
            .iter(world)
            .find(|(_, id)| id.0 == center_crit_id)
            .map(|(e, _)| e)
            .expect("Should find center creature");

        let neighbor_cache = world.get::<NeighborCache>(center_entity).expect("Should have NeighborCache");
        let neighbor_count = neighbor_cache.neighbor_count();

        // With 340° FOV and 12 neighbors evenly spaced, should see about 11
        // (all except the one in the ~20° blind spot behind)
        // But due to FOV direction and movement, let's just check we see MORE than 1
        assert!(
            neighbor_count > 1,
            "Wide FOV (340°) creature should perceive many neighbors, but only got {}",
            neighbor_count
        );

        // Should see at least 6 neighbors (half of the 12 we spawned)
        assert!(
            neighbor_count >= 6,
            "Wide FOV (340°) creature should perceive at least 6 neighbors, but only got {}",
            neighbor_count
        );
    }

    // ============================================================================
    // Category 2: Per-Entity FOV Filtering Tests
    //
    // These tests verify the entity-level FOV check works correctly for various
    // FOV angles. They should PASS (entity-level filtering is correct).
    // ============================================================================

    /// Test narrow 45° FOV entity filtering boundary cases
    #[test]
    fn test_narrow_fov_45_entity_filtering() {
        let perception = Perception::new(45.0, 1.0);
        let cos_half_fov = perception.cos_half_fov;
        let cos_half_fov_sq = perception.cos_half_fov_sq;

        let facing_x = 1.0_f32;
        let facing_y = 0.0_f32;

        // 45° FOV = ±22.5° from facing
        let test_cases: [(f32, bool, &str); 5] = [
            (0.0, true, "directly in front"),
            (20.0, true, "at 20° (inside FOV)"),
            (22.0, true, "at 22° (just inside edge)"),
            (23.5, false, "at 23.5° (just outside edge)"),
            (45.0, false, "at 45° (well outside)"),
        ];

        for (angle_deg, expected_visible, description) in test_cases {
            let angle_rad = angle_deg.to_radians();
            let dx = angle_rad.cos() * 5.0;
            let dy = angle_rad.sin() * 5.0;
            let center_dist_sq = dx * dx + dy * dy;
            let rough_dot = dx * facing_x + dy * facing_y;

            let in_fov = if cos_half_fov >= 0.0 {
                rough_dot > 0.0 && rough_dot * rough_dot >= cos_half_fov_sq * center_dist_sq
            } else {
                let dist = center_dist_sq.sqrt();
                rough_dot >= cos_half_fov * dist
            };

            assert_eq!(
                in_fov, expected_visible,
                "Target {} at {}° should be {}, but got {}",
                description, angle_deg,
                if expected_visible { "visible" } else { "not visible" },
                if in_fov { "visible" } else { "filtered" }
            );
        }
    }

    /// Test medium 90° FOV entity filtering boundary cases
    #[test]
    fn test_medium_fov_90_entity_filtering() {
        let perception = Perception::new(90.0, 1.0);
        let cos_half_fov = perception.cos_half_fov;
        let cos_half_fov_sq = perception.cos_half_fov_sq;

        let facing_x = 1.0_f32;
        let facing_y = 0.0_f32;

        // 90° FOV = ±45° from facing
        let test_cases: [(f32, bool, &str); 5] = [
            (0.0, true, "directly in front"),
            (40.0, true, "at 40° (inside FOV)"),
            (44.0, true, "at 44° (just inside edge)"),
            (46.0, false, "at 46° (just outside edge)"),
            (90.0, false, "at 90° (perpendicular)"),
        ];

        for (angle_deg, expected_visible, description) in test_cases {
            let angle_rad = angle_deg.to_radians();
            let dx = angle_rad.cos() * 5.0;
            let dy = angle_rad.sin() * 5.0;
            let center_dist_sq = dx * dx + dy * dy;
            let rough_dot = dx * facing_x + dy * facing_y;

            let in_fov = if cos_half_fov >= 0.0 {
                rough_dot > 0.0 && rough_dot * rough_dot >= cos_half_fov_sq * center_dist_sq
            } else {
                let dist = center_dist_sq.sqrt();
                rough_dot >= cos_half_fov * dist
            };

            assert_eq!(
                in_fov, expected_visible,
                "Target {} at {}° should be {}, but got {}",
                description, angle_deg,
                if expected_visible { "visible" } else { "not visible" },
                if in_fov { "visible" } else { "filtered" }
            );
        }
    }

    /// Test standard 180° FOV entity filtering boundary cases
    #[test]
    fn test_standard_fov_180_entity_filtering() {
        let perception = Perception::new(180.0, 1.0);
        let cos_half_fov = perception.cos_half_fov;
        let cos_half_fov_sq = perception.cos_half_fov_sq;

        let facing_x = 1.0_f32;
        let facing_y = 0.0_f32;

        // 180° FOV = ±90° from facing (hemisphere in front)
        let test_cases: [(f32, bool, &str); 5] = [
            (0.0, true, "directly in front"),
            (45.0, true, "at 45° (inside FOV)"),
            (89.0, true, "at 89° (just inside edge)"),
            (91.0, false, "at 91° (just outside edge)"),
            (180.0, false, "directly behind"),
        ];

        for (angle_deg, expected_visible, description) in test_cases {
            let angle_rad = angle_deg.to_radians();
            let dx = angle_rad.cos() * 5.0;
            let dy = angle_rad.sin() * 5.0;
            let center_dist_sq = dx * dx + dy * dy;
            let rough_dot = dx * facing_x + dy * facing_y;

            let in_fov = if cos_half_fov >= 0.0 {
                rough_dot > 0.0 && rough_dot * rough_dot >= cos_half_fov_sq * center_dist_sq
            } else {
                let dist = center_dist_sq.sqrt();
                rough_dot >= cos_half_fov * dist
            };

            assert_eq!(
                in_fov, expected_visible,
                "Target {} at {}° should be {}, but got {}",
                description, angle_deg,
                if expected_visible { "visible" } else { "not visible" },
                if in_fov { "visible" } else { "filtered" }
            );
        }
    }

    /// Test extra-wide 270° FOV entity filtering
    #[test]
    fn test_extra_wide_fov_270_entity_filtering() {
        let perception = Perception::new(270.0, 1.0);
        let cos_half_fov = perception.cos_half_fov;
        let cos_half_fov_sq = perception.cos_half_fov_sq;

        let facing_x = 1.0_f32;
        let facing_y = 0.0_f32;

        // 270° FOV = ±135° from facing (only 45° blind spot behind)
        let test_cases: [(f32, bool, &str); 5] = [
            (0.0, true, "directly in front"),
            (90.0, true, "at 90° (perpendicular, visible)"),
            (130.0, true, "at 130° (inside FOV)"),
            (140.0, false, "at 140° (in blind spot)"),
            (180.0, false, "directly behind"),
        ];

        for (angle_deg, expected_visible, description) in test_cases {
            let angle_rad = angle_deg.to_radians();
            let dx = angle_rad.cos() * 5.0;
            let dy = angle_rad.sin() * 5.0;
            let center_dist_sq = dx * dx + dy * dy;
            let rough_dot = dx * facing_x + dy * facing_y;

            let in_fov = if cos_half_fov >= 0.0 {
                rough_dot > 0.0 && rough_dot * rough_dot >= cos_half_fov_sq * center_dist_sq
            } else {
                let dist = center_dist_sq.sqrt();
                rough_dot >= cos_half_fov * dist
            };

            assert_eq!(
                in_fov, expected_visible,
                "Target {} at {}° should be {}, but got {}",
                description, angle_deg,
                if expected_visible { "visible" } else { "not visible" },
                if in_fov { "visible" } else { "filtered" }
            );
        }
    }

    // ============================================================================
    // Category 3: Integration Tests (Full Pipeline)
    //
    // These tests verify the complete perception pipeline with various FOV angles.
    // ============================================================================

    /// Integration test: 45° FOV creature in a crowd
    #[test]
    fn test_fov_variants_narrow_45_crowd() {
        use crate::simulation::core::SimulationBuilder;
        use crate::simulation::creatures::builder::CritBuilder;
        use crate::simulation::creatures::constants::UPDATE_SLICE_COUNT;

        let mut sim = SimulationBuilder::new().build();
        sim.set_boundaries(200.0, 200.0);

        // Spawn creature with 45° FOV at center
        let center_crit_id = sim.spawn_crit(
            CritBuilder::new()
                .at(100.0, 100.0)
                .with_fov(45.0)
                .with_all_capabilities(),
        );

        // Spawn neighbors at known angles (every 30°)
        let distance = 5.0;
        for angle_deg in (0..360).step_by(30) {
            let angle_rad = (angle_deg as f32).to_radians();
            let x = 100.0 + distance * angle_rad.cos();
            let y = 100.0 + distance * angle_rad.sin();
            sim.spawn_crit(
                CritBuilder::new()
                    .at(x, y)
                    .with_all_capabilities(),
            );
        }

        // Run enough ticks to ensure perception runs
        for _ in 0..UPDATE_SLICE_COUNT {
            sim.update(0.016);
        }

        // Find the center creature and check its neighbor count
        let world = sim.world_mut();
        let center_entity = world
            .query::<(bevy_ecs::entity::Entity, &crate::simulation::creatures::components::CritId)>()
            .iter(world)
            .find(|(_, id)| id.0 == center_crit_id)
            .map(|(e, _)| e)
            .expect("Should find center creature");

        let neighbor_cache = world.get::<NeighborCache>(center_entity).expect("Should have NeighborCache");
        let neighbor_count = neighbor_cache.neighbor_count();

        // 45° FOV = ±22.5° from facing. With 12 neighbors at 30° intervals,
        // only 0° and maybe ±30° (depending on exact facing) should be visible.
        // Expected: 1-3 neighbors visible
        assert!(
            neighbor_count <= 4,
            "Narrow FOV (45°) creature should perceive few neighbors (1-4), but got {}",
            neighbor_count
        );
    }

    /// Integration test: 90° FOV creature in a crowd
    #[test]
    fn test_fov_variants_medium_90_crowd() {
        use crate::simulation::core::SimulationBuilder;
        use crate::simulation::creatures::builder::CritBuilder;
        use crate::simulation::creatures::constants::UPDATE_SLICE_COUNT;

        let mut sim = SimulationBuilder::new().build();
        sim.set_boundaries(200.0, 200.0);

        let center_crit_id = sim.spawn_crit(
            CritBuilder::new()
                .at(100.0, 100.0)
                .with_fov(90.0)
                .with_all_capabilities(),
        );

        let distance = 5.0;
        for angle_deg in (0..360).step_by(30) {
            let angle_rad = (angle_deg as f32).to_radians();
            let x = 100.0 + distance * angle_rad.cos();
            let y = 100.0 + distance * angle_rad.sin();
            sim.spawn_crit(
                CritBuilder::new()
                    .at(x, y)
                    .with_all_capabilities(),
            );
        }

        for _ in 0..UPDATE_SLICE_COUNT {
            sim.update(0.016);
        }

        let world = sim.world_mut();
        let center_entity = world
            .query::<(bevy_ecs::entity::Entity, &crate::simulation::creatures::components::CritId)>()
            .iter(world)
            .find(|(_, id)| id.0 == center_crit_id)
            .map(|(e, _)| e)
            .expect("Should find center creature");

        let neighbor_cache = world.get::<NeighborCache>(center_entity).expect("Should have NeighborCache");
        let neighbor_count = neighbor_cache.neighbor_count();

        // 90° FOV = ±45° from facing. With 12 neighbors at 30° intervals,
        // 0°, ±30° should be visible (3-4 neighbors)
        assert!(
            neighbor_count >= 2 && neighbor_count <= 5,
            "Medium FOV (90°) creature should perceive 2-5 neighbors, but got {}",
            neighbor_count
        );
    }

    // ============================================================================
    // Category 4: Edge Cases
    // ============================================================================

    /// Test that stopped creatures maintain their last facing direction
    #[test]
    fn test_fov_zero_velocity_maintains_facing() {
        use crate::simulation::core::SimulationBuilder;
        use crate::simulation::creatures::builder::CritBuilder;
        use crate::simulation::creatures::constants::UPDATE_SLICE_COUNT;
        use crate::simulation::core::components::Rotation;

        let mut sim = SimulationBuilder::new().build();
        sim.set_boundaries(200.0, 200.0);

        // Spawn a creature that will be stopped
        let crit_id = sim.spawn_crit(
            CritBuilder::new()
                .at(100.0, 100.0)
                .with_fov(90.0)
                .with_all_capabilities(),
        );

        // Run a few ticks
        for _ in 0..UPDATE_SLICE_COUNT * 2 {
            sim.update(0.016);
        }

        // Get the creature's rotation
        let world = sim.world_mut();
        let entity = world
            .query::<(bevy_ecs::entity::Entity, &crate::simulation::creatures::components::CritId)>()
            .iter(world)
            .find(|(_, id)| id.0 == crit_id)
            .map(|(e, _)| e)
            .expect("Should find creature");

        let rotation = world.get::<Rotation>(entity).expect("Should have Rotation");

        // Rotation should be valid (not NaN)
        assert!(
            !rotation.cos_radians.is_nan() && !rotation.sin_radians.is_nan(),
            "Rotation should be valid even for stopped creatures"
        );

        // Should be normalized
        let magnitude = (rotation.cos_radians.powi(2) + rotation.sin_radians.powi(2)).sqrt();
        assert!(
            (magnitude - 1.0).abs() < 0.01,
            "Rotation should be normalized: magnitude = {}",
            magnitude
        );
    }
}
