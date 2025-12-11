use super::components::*;
use crate::simulation::creatures::constants::MAX_PERCEIVED_NEIGHBORS;
use crate::simulation::core::components::{BodySize, Position, Rotation};
#[cfg(feature = "dev-tools")]
use crate::simulation::core::components::Acceleration;
use crate::simulation::creatures::components::CreatureState;
#[cfg(feature = "dev-tools")]
use crate::simulation::creatures::components::CritId;
use crate::simulation::spatial::DoubleBufferedSpatialGrid;
#[cfg(feature = "dev-tools")]
use crate::simulation::spatial::NON_ADJACENT_OFFSET;
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
    grid: Res<DoubleBufferedSpatialGrid>,
    mut query: Query<(Entity, &Position, &Rotation, &BodySize, &mut Perception, &CreatureState)>,
    #[cfg(feature = "dev-tools")] timings: Res<SystemTimings>,
    #[cfg(feature = "dev-tools")] debug_target: Res<PerceptionDebugTarget>,
    #[cfg(feature = "dev-tools")] mut debug_snapshot: ResMut<PerceptionDebugSnapshot>,
    #[cfg(feature = "dev-tools")] crit_ids: Query<&CritId>,
    #[cfg(feature = "dev-tools")] accel_query: Query<&Acceleration>,
) {
    #[cfg(feature = "dev-tools")]
    crate::time_system!(timings, "perception");

    let grid_ref = grid.read_grid();

    // Get debug target (dev-tools only) - used for visualization AFTER perception runs
    #[cfg(feature = "dev-tools")]
    let debug_target_entity = debug_target.get();

    // Collect ALL entities for parallel processing
    let mut entities: Vec<_> = query.iter_mut().collect();

    // ============================================================
    // SINGLE PERCEPTION PASS - identical in dev and production
    // ============================================================
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

        // Topological neighbor selection: collect ALL neighbors, sort by distance, keep K closest
        // This ensures creatures always perceive their CLOSEST neighbors, regardless of cell
        CELL_SCRATCH.with(|scratch| {
            NEIGHBOR_CANDIDATES.with(|candidates_cell| {
                let mut cells = scratch.borrow_mut();
                let mut candidates = candidates_cell.borrow_mut();
                candidates.clear();

                grid_ref.collect_cells_sorted(x, y, query_radius, facing_x, facing_y, &mut cells);

                // Collect ALL neighbors in range
                for &(_, cell_idx) in cells.iter() {
                    for proxy in grid_ref.get_cell_proxies(cell_idx) {
                        if *entity == proxy.entity {
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
                        if rough_dot <= 0.0 {
                            continue;
                        }

                        if rough_dot * rough_dot >= cos_half_fov_sq * center_dist_sq {
                            candidates.push((center_dist_sq, NeighborData {
                                entity: proxy.entity,
                                x: proxy.x,
                                y: proxy.y,
                                radius: proxy.radius,
                            }));
                        }
                    }
                }

                // Partial sort: get K closest without fully sorting all candidates
                // select_nth_unstable is O(n) average vs O(n log n) for full sort
                let k = MAX_PERCEIVED_NEIGHBORS.min(candidates.len());
                if k > 0 {
                    if candidates.len() > k {
                        // Partition so first K elements are the K smallest (unordered among themselves)
                        candidates.select_nth_unstable_by(k - 1, |a, b| {
                            a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal)
                        });
                    }
                    // Add the K closest neighbors
                    for (_, neighbor) in candidates.iter().take(k) {
                        perception.add_neighbor(*neighbor);
                    }
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
        if let Some(target_entity) = debug_target_entity {
            // Find the debug target in our entities list and capture its state
            if let Some((_, pos, rot, size, perception, state)) = entities
                .iter()
                .find(|(e, _, _, _, _, _)| *e == target_entity)
            {
                let entity_id = crit_ids.get(target_entity).map(|id| id.0).unwrap_or(0);

                // Query acceleration for force visualization
                let (ax, ay) = accel_query
                    .get(target_entity)
                    .map(|a| (a.ax, a.ay))
                    .unwrap_or((0.0, 0.0));

                if state.behavior.is_active() {
                    let x = pos.x;
                    let y = pos.y;
                    let self_radius = size.radius();
                    let range = perception.range;
                    let facing_x = rot.radians.cos();
                    let facing_y = rot.radians.sin();
                    let query_radius = range + self_radius + MAX_OTHER_RADIUS;

                    // Compute which cells would be queried/skipped (for visualization only)
                    let (queried_cells, skipped_cells) = compute_cell_visualization(
                        x, y, query_radius, facing_x, facing_y, perception.neighbor_count(), grid_ref,
                    );

                    // Build neighbor debug info from the ACTUAL perception results
                    let neighbor_debug: Vec<NeighborDebugInfo> = perception
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
                        range,
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
            } else {
                debug_snapshot.clear();
            }
        } else {
            // No debug target set - this is normal
            debug_snapshot.clear();
        }
    }
}

/// Compute which cells would be queried vs skipped for visualization.
/// This is called AFTER perception runs, purely for debug display.
#[cfg(feature = "dev-tools")]
fn compute_cell_visualization(
    x: f32,
    y: f32,
    query_radius: f32,
    facing_x: f32,
    facing_y: f32,
    neighbor_count: usize,
    grid: &crate::simulation::spatial::SpatialGrid,
) -> (Vec<QueriedCell>, Vec<QueriedCell>) {
    let mut queried = Vec::with_capacity(64);
    let mut skipped = Vec::with_capacity(64);
    let mut cells: Vec<(f32, usize)> = Vec::with_capacity(64);

    grid.collect_cells_sorted(x, y, query_radius, facing_x, facing_y, &mut cells);

    // Simulate the cell examination logic to determine which would be queried/skipped
    let capacity_reached = neighbor_count >= MAX_PERCEIVED_NEIGHBORS;

    for &(sort_key, cell_idx) in cells.iter() {
        let is_adjacent = sort_key < NON_ADJACENT_OFFSET;
        let (cx, cy) = grid.get_cell_coords_by_index(cell_idx);

        if !is_adjacent && capacity_reached {
            skipped.push(QueriedCell { x: cx, y: cy });
        } else {
            queried.push(QueriedCell { x: cx, y: cy });
        }
    }

    (queried, skipped)
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

        let mut query = if check_behavior {
            world.query::<(Entity, &Position, &BodySize, &mut Perception, &CreatureState)>()
        } else {
            world.query::<(Entity, &Position, &BodySize, &mut Perception, &CreatureState)>()
        };

        for (entity, pos, size, mut perception, state) in query.iter_mut(world) {
            perception.clear();

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
                    perception.add_neighbor(NeighborData {
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

        let mut query = world.query::<(Entity, &Position, &mut Perception)>();
        for (entity, pos, mut perception) in query.iter_mut(world) {
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
                    perception.add_neighbor(NeighborData {
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

        run_naive_perception(&mut world, true);

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
        use crate::simulation::creatures::constants::PERCEPTION_MULTIPLIER;

        let mut world = World::new();

        // Body size 1.0 → range = PERCEPTION_MULTIPLIER (100.0)
        // Place crit1 and crit2 close together, crit3 far away
        let crit1 = world
            .spawn((Position { x: 0.0, y: 0.0 }, Perception::from_body_size(1.0)))
            .id();

        let crit2 = world
            .spawn((Position { x: 5.0, y: 0.0 }, Perception::from_body_size(1.0)))
            .id();

        // crit3 is beyond perception range (PERCEPTION_MULTIPLIER + some buffer)
        let crit3 = world
            .spawn((Position { x: PERCEPTION_MULTIPLIER + 50.0, y: 0.0 }, Perception::from_body_size(1.0)))
            .id();

        run_simple_perception(&mut world);

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

        run_simple_perception(&mut world);

        let perception = world.get::<Perception>(crit).unwrap();
        assert_eq!(perception.neighbor_count(), 0);
        assert!(!perception.contains(crit));
    }

    #[test]
    fn test_perception_clears_previous_neighbors() {
        use crate::simulation::creatures::constants::PERCEPTION_MULTIPLIER;

        let mut world = World::new();

        // Body size 1.0 → range = PERCEPTION_MULTIPLIER (100.0)
        let crit1 = world
            .spawn((Position { x: 0.0, y: 0.0 }, Perception::from_body_size(1.0)))
            .id();

        let crit2 = world
            .spawn((Position { x: 5.0, y: 0.0 }, Perception::from_body_size(1.0)))
            .id();

        run_simple_perception(&mut world);

        let perception1 = world.get::<Perception>(crit1).unwrap();
        assert_eq!(perception1.neighbor_count(), 1);

        // Move crit2 beyond perception range
        world.get_mut::<Position>(crit2).unwrap().x = PERCEPTION_MULTIPLIER + 50.0;

        run_simple_perception(&mut world);

        let perception1 = world.get::<Perception>(crit1).unwrap();
        assert_eq!(perception1.neighbor_count(), 0);
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
            .spawn((Position { x: 0.0, y: 0.0 }, small_perception))
            .id();

        // crit2: large range (4x small range since body size is 4x)
        let crit2 = world
            .spawn((Position { x: 0.0, y: 0.0 }, large_perception))
            .id();

        // crit3 at distance midway: outside small range but inside large range
        let midpoint_distance = (small_range + large_range) / 2.0;
        let crit3 = world
            .spawn((Position { x: midpoint_distance, y: 0.0 }, Perception::new(180.0, 1.0)))
            .id();

        run_simple_perception(&mut world);

        let perception1 = world.get::<Perception>(crit1).unwrap();
        assert!(!perception1.contains(crit3), "small creature should NOT see crit3 beyond its range");

        let perception2 = world.get::<Perception>(crit2).unwrap();
        assert!(perception2.contains(crit3), "large creature SHOULD see crit3 within its range");
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
        run_simple_perception(&mut world);
        let duration = start.elapsed();

        println!("Perception update (100 crits, naive O(n²)): {:?}", duration);

        let total_neighbors: usize = world
            .query::<&Perception>()
            .iter(&world)
            .map(|p| p.neighbor_count())
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

        let mut sim = SimulationBuilder::new().build();
        sim.set_boundaries(100.0, 100.0);

        // Spawn a creature and get its CritId
        let crit_id = sim.spawn_crit(
            CritBuilder::new()
                .at(50.0, 50.0)
                .with_all_capabilities()
                .with_wandering(),
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

        // Run simulation to trigger perception system
        sim.update(0.016);

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
                    .with_all_capabilities()
                    .with_wandering(),
            );
        }

        // Run several updates for creatures to wander and potentially turn toward each other
        for _ in 0..10 {
            sim.update(0.016);
        }

        // Check perception worked - some creatures should have neighbors
        let world = sim.world_mut();
        let mut total_neighbors = 0;

        for perception in world.query::<&Perception>().iter(world) {
            total_neighbors += perception.neighbor_count();
        }

        // With 8 creatures in a 2D cluster, at least some should perceive neighbors
        // regardless of FOV angle (narrow FOV means some miss, but with enough creatures some hit)
        assert!(
            total_neighbors > 0,
            "At least some creatures should perceive neighbors after simulation runs"
        );

        // Verify neighbor count is bounded correctly
        for perception in world.query::<&Perception>().iter(world) {
            assert!(
                perception.neighbor_count() <= MAX_PERCEIVED_NEIGHBORS,
                "Neighbor count should not exceed MAX_PERCEIVED_NEIGHBORS"
            );
        }
    }
}
