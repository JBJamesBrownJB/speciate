use super::classification::{classify_l1_cell, L1Classification};
use super::components::*;
#[cfg(feature = "dev-tools")]
use super::debug::*;
use super::entity_filter::should_perceive_entity;
use super::fov_patterns;
#[cfg(feature = "dev-tools")]
use crate::instrumentation::SystemTimings;
use crate::simulation::core::components::{BodySize, FreqConfig, PhysicsTick, Position, Rotation};
use crate::simulation::core::FrequencyThrottle;
use crate::simulation::creatures::components::CreatureState;
#[cfg(feature = "dev-tools")]
use crate::simulation::creatures::components::CritId;
use crate::simulation::creatures::constants::MAX_PERCEIVED_NEIGHBORS;
use crate::simulation::spatial::constants::CELL_SIZE;
use crate::simulation::spatial::{BioSignature, HierarchicalGrid};
use bevy_ecs::prelude::*;
use rayon::prelude::*;
use std::cell::RefCell;

/// L0 scan radius: Always query 9 adjacent cells only.
/// This is FIXED regardless of creature's perception range - L1 provides long-range awareness.
/// Math: ceil(radius / cell_size) determines cells_radius in collect_cells_sorted_fov()
///   - CELL_SIZE * 1.0 = 10m → ceil(1.0) = 1 → 3×3 = 9 cells ✓
///   - CELL_SIZE * 1.5 = 15m → ceil(1.5) = 2 → 5×5 = 25 cells ✗
pub(crate) const L0_SCAN_RADIUS: f32 = CELL_SIZE * 1.0; // 10m → ceil(1.0) = 1 → 3×3 = 9 cells

/// How many cells out we query (derived from L0_SCAN_RADIUS).
/// IMPORTANT: If you change L0_SCAN_RADIUS, update this! Rust const doesn't support ceil().
#[allow(dead_code)]
pub(crate) const L0_CELLS_RADIUS: f32 = 1.0; // ceil(L0_SCAN_RADIUS / CELL_SIZE) = ceil(1.0) = 1

/// Actual L0 visible range - the furthest distance at which entities can be perceived via L0.
/// This is the distance to the corner of the furthest queried cell.
/// Formula: sqrt(2) × (cells_radius + 0.5) × CELL_SIZE
/// Entities entering this sphere CAN become neighbors (assuming not size-culled).
#[allow(dead_code)]
pub(crate) const L0_VISIBLE_RANGE: f32 = 1.41421356 * (L0_CELLS_RADIUS + 0.5) * CELL_SIZE;

// Thread-local scratch buffer for sorted cell indices (avoids allocation per creature)
thread_local! {
    pub(crate) static CELL_SCRATCH: RefCell<Vec<(f32, usize)>> = RefCell::new(Vec::with_capacity(256));
}

// Thread-local scratch buffer for topological sorting (collects all neighbors, then sorts)
thread_local! {
    pub(crate) static NEIGHBOR_CANDIDATES: RefCell<Vec<(f32, NeighborData)>> = RefCell::new(Vec::with_capacity(256));
}

/// Check if a target is within the field of view.
/// Uses squared comparisons for both narrow and wide FOV (no sqrt).
#[inline]
pub(crate) fn is_in_fov(rough_dot: f32, center_dist_sq: f32, cos_half_fov: f32, cos_half_fov_sq: f32) -> bool {
    if cos_half_fov >= 0.0 {
        // Narrow FOV (≤180°): target must be in front
        rough_dot > 0.0 && rough_dot * rough_dot >= cos_half_fov_sq * center_dist_sq
    } else {
        // Wide FOV (>180°): cos_half_fov is negative
        if rough_dot >= 0.0 {
            // In front/side: always visible (positive >= negative*positive)
            true
        } else {
            // Behind: compare squared magnitudes (inequality flips for negatives)
            rough_dot * rough_dot <= cos_half_fov_sq * center_dist_sq
        }
    }
}

pub fn update_perception_system(
    physics_tick: Res<PhysicsTick>,
    freq: Res<FreqConfig>,
    grid: Res<HierarchicalGrid>,
    mut query: Query<(
        Entity,
        &Position,
        &Rotation,
        &BodySize,
        &Perception,
        &mut NeighborCache,
        &mut L1Vision,
        &CreatureState,
    )>,
    #[cfg(feature = "dev-tools")] timings: Res<SystemTimings>,
    #[cfg(feature = "dev-tools")] debug_target: Res<PerceptionDebugTarget>,
    #[cfg(feature = "dev-tools")] mut debug_snapshot: ResMut<PerceptionDebugSnapshot>,
    #[cfg(feature = "dev-tools")] crit_ids: Query<&CritId>,
) {
    #[cfg(feature = "dev-tools")]
    crate::time_system!(timings, "perception");

    let grid_ref = grid.l0.read_grid();

    // Get debug target (dev-tools only) - used for visualization AFTER perception runs
    #[cfg(feature = "dev-tools")]
    let debug_target_entity = debug_target.get();

    // Collect all entities for parallel processing
    let mut entities: Vec<_> = query.iter_mut().collect();

    // DEV-TOOLS: Mutex to capture ACTUAL cell data during perception for debug target
    #[cfg(feature = "dev-tools")]
    let debug_cell_capture: std::sync::Mutex<Option<(Vec<(i32, i32)>, Vec<(i32, i32)>)>> =
        std::sync::Mutex::new(None);

    // Get L1 grid reference for early-exit optimization
    let l1_grid_ref = &grid.l1;

    // Frequency throttling: entity-ID bucketing with power-of-2 optimization
    let throttle = FrequencyThrottle::new(freq.perception_divisor, physics_tick.get());

    // ============================================================
    // SINGLE PERCEPTION PASS - identical in dev and production
    // ============================================================
    // Perception: Heavy, variable workload - smaller chunks for load balancing
    entities.par_iter_mut().with_min_len(128).for_each(
        |(entity, pos, rot, size, perception, neighbor_cache, l1_vision, state)| {
            // Check if this entity is the debug target (dev-tools only)
            #[cfg(feature = "dev-tools")]
            let is_debug_target = debug_target_entity.map_or(false, |t| *entity == t);

            // Frequency throttling: skip if not in current bucket
            // IMPORTANT: Do NOT clear neighbor_cache when skipping - keep stale data
            // EXCEPTION: Always process debug target to prevent visualization flashing
            #[cfg(feature = "dev-tools")]
            let bypass_throttle = is_debug_target;
            #[cfg(not(feature = "dev-tools"))]
            let bypass_throttle = false;

            if !bypass_throttle && !throttle.should_process(entity.index()) {
                return;
            }

            if !state.behavior.is_active() {
                return;
            }

            // Clear caches only when we're actually updating perception
            neighbor_cache.clear();
            l1_vision.clear();

            let x = pos.x;
            let y = pos.y;
            let self_radius = size.radius();
            // NOTE: L1 early-exit happens PER L0 CELL (not per-creature).
            // Each L0 cell's parent L1 is classified; if Empty, the entire L0 cell is skipped.
            // This handles creatures near L1 boundaries where adjacent L0 cells may have
            // different L1 parents with different classifications.
            let range = perception.range;
            let range_sq = range * range;
            let threshold = perception.threshold;
            let cos_half_fov_sq = perception.cos_half_fov_sq;
            let cos_half_fov = perception.cos_half_fov;
            let fov_angle = perception.fov_angle;
            // Use cached cos/sin from rotation (avoids 400K trig calls per tick)
            let facing_x = rot.cos_radians;
            let facing_y = rot.sin_radians;
            // L0 scan: ALWAYS 9 adjacent cells only (fixed radius, not perception range)
            // L1 provides long-range awareness via L1Vision component
            let query_radius = L0_SCAN_RADIUS;

            // Topological neighbor selection with smart early-exit:
            // 1. Always scan adjacent cells, tracking max distance seen
            // 2. After adjacent cells, if we have K+ candidates, use max distance as cutoff
            // 3. Skip non-adjacent cells whose nearest edge is beyond cutoff
            // This ensures correctness (always K closest) while being fast in dense crowds
            CELL_SCRATCH.with(|scratch| {
                NEIGHBOR_CANDIDATES.with(|candidates_cell| {
                    use crate::simulation::spatial::constants::{
                        CELL_HALF_DIAGONAL, NON_ADJACENT_OFFSET,
                    };

                    let mut cells = scratch.borrow_mut();
                    let mut candidates = candidates_cell.borrow_mut();
                    candidates.clear();

                    // Get creature's cell coordinates for FOV cell culling
                    let (creature_cx, creature_cy) = grid_ref.world_to_cell(x, y);

                    // Compute FOV cell pattern once per creature (precomputed lookup table)
                    let cell_pattern = fov_patterns::get_cell_pattern(fov_angle, facing_x, facing_y);

                    // Use pre-computed cos_half_fov from perception component for grid-level FOV culling
                    grid_ref.collect_cells_sorted_fov(
                        x,
                        y,
                        query_radius,
                        range, // perception_range: cull cells beyond creature's perception
                        facing_x,
                        facing_y,
                        cos_half_fov,
                        &mut cells,
                    );

                    // Instrumentation: count cells queried for performance tracking
                    #[cfg(feature = "dev-tools")]
                    timings
                        .cells_queried_total
                        .fetch_add(cells.len() as u64, std::sync::atomic::Ordering::Relaxed);

                    // Pre-compute for L1 classification (size domination optimization)
                    let my_mass = BioSignature::mass_from_radius(self_radius);
                    let my_l1_cell_idx = l1_grid_ref.position_to_cell_index(x, y);
                    let l0_width = grid_ref.width();

                    // L1 classification cache: avoid redundant classify_l1_cell calls.
                    // A 3×3 L0 neighborhood has at most 4 unique L1 parents.
                    let mut l1_cache_count = 0usize;
                    let mut l1_cache: [(usize, L1Classification); 4] =
                        [(usize::MAX, L1Classification::Empty); 4];

                    // Helper: get or compute L1 classification with caching
                    let get_l1_classification = |l1_idx: usize,
                                                 cache: &mut [(usize, L1Classification); 4],
                                                 cache_count: &mut usize|
                     -> L1Classification {
                        // Check cache first
                        for i in 0..*cache_count {
                            if cache[i].0 == l1_idx {
                                return cache[i].1;
                            }
                        }
                        // Not cached - compute and cache
                        let biosig = l1_grid_ref.get_biosignature(l1_idx);
                        let is_my_cell = l1_idx == my_l1_cell_idx;
                        let classification =
                            classify_l1_cell(biosig, my_mass, self_radius, is_my_cell);
                        if *cache_count < 4 {
                            cache[*cache_count] = (l1_idx, classification);
                            *cache_count += 1;
                        }
                        classification
                    };

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
                    let mut debug_queried: Vec<(i32, i32)> = if is_debug_target {
                        Vec::with_capacity(64)
                    } else {
                        Vec::new()
                    };
                    #[cfg(feature = "dev-tools")]
                    let mut debug_skipped: Vec<(i32, i32)> = if is_debug_target {
                        Vec::with_capacity(32)
                    } else {
                        Vec::new()
                    };

                    // Get FOV tier for extended cell pattern lookup
                    let fov_tier = perception.fov_tier;

                    for &(sort_key, cell_idx) in cells.iter() {
                        // Detect transition from adjacent to non-adjacent cells.
                        // Adjacent cells: sort_key = distance² (typically < 500 for ~22m diagonal)
                        // Non-adjacent: sort_key = distance² + NON_ADJACENT_OFFSET (1e9)
                        // The 0.5 multiplier creates a safe threshold (5e8) between these ranges.
                        let is_non_adjacent = sort_key >= NON_ADJACENT_OFFSET * 0.5;

                        if is_non_adjacent && !processed_adjacent {
                            processed_adjacent = true;
                            // Pre-compute expanded cutoff: (sqrt(max_dist) + half_diag)²
                            // This moves sqrt from per-cell to once-per-creature
                            if candidates.len() >= MAX_PERCEIVED_NEIGHBORS {
                                let cutoff_dist = max_adjacent_dist_sq.sqrt();
                                expanded_cutoff_dist_sq =
                                    (cutoff_dist + CELL_HALF_DIAGONAL).powi(2);
                            }
                        }

                        // For non-adjacent cells, check if cell center is beyond expanded cutoff
                        // If center² > (cutoff + half_diag)², the nearest edge must be beyond cutoff
                        if is_non_adjacent && expanded_cutoff_dist_sq < f32::MAX {
                            let cell_center_dist_sq = sort_key - NON_ADJACENT_OFFSET;
                            if cell_center_dist_sq > expanded_cutoff_dist_sq {
                                // DEV-TOOLS: Capture this skipped cell before breaking
                                // (remaining cells are beyond cutoff - no need to enumerate all)
                                #[cfg(feature = "dev-tools")]
                                if is_debug_target {
                                    let (cx, cy) = grid_ref.get_cell_coords_by_index(cell_idx);
                                    debug_skipped.push((cx, cy));
                                }
                                // ALWAYS break - identical behavior with/without dev-tools
                                break;
                            }
                        }

                        // FOV CELL CULLING: Skip cells geometrically outside creature's FOV cone.
                        // Uses precomputed lookup table with 50° safety margin (45° cell corner + 5° variance).
                        let (cell_cx, cell_cy) = grid_ref.get_cell_coords_by_index(cell_idx);
                        let cell_dx = cell_cx - creature_cx;
                        let cell_dy = cell_cy - creature_cy;

                        if !fov_patterns::should_query_cell(cell_dx, cell_dy, cell_pattern) {
                            #[cfg(feature = "dev-tools")]
                            if is_debug_target {
                                debug_skipped.push((cell_cx, cell_cy));
                            }
                            continue;
                        }

                        // DEV-TOOLS: Capture queried cell for debug target
                        #[cfg(feature = "dev-tools")]
                        if is_debug_target {
                            debug_queried.push((cell_cx, cell_cy));
                        }

                        // L1 CLASSIFICATION CHECK (size domination optimization):
                        // Check if this L0 cell's parent L1 cell has any creatures worth perceiving.
                        // If Empty, skip the entire L0 cell scan - no entities above our threshold.
                        // Uses cache since 3×3 L0 neighborhood has at most 4 unique L1 parents.
                        let parent_l1_idx = l1_grid_ref.l0_to_l1_cell_index(cell_idx, l0_width);
                        let classification = get_l1_classification(
                            parent_l1_idx,
                            &mut l1_cache,
                            &mut l1_cache_count,
                        );

                        if classification == L1Classification::Empty {
                            // Skip this L0 cell - nothing above our perception threshold
                            continue;
                        }

                        // L1 VISION: Record non-Empty L1 cells discovered during L0 scan.
                        // The L0 cell passed FOV culling, so PART of this L1 cell is in FOV.
                        // Don't check if L1 center is in FOV - creature may be at cell edge.
                        if !l1_vision.contains_cell(parent_l1_idx as u32) {
                            let (l1_center_x, l1_center_y) =
                                l1_grid_ref.cell_center_from_index(parent_l1_idx);
                            let l1_dx = l1_center_x - x;
                            let l1_dy = l1_center_y - y;
                            let l1_dist = (l1_dx * l1_dx + l1_dy * l1_dy).sqrt().max(0.001);
                            l1_vision.push(L1VisionEntry {
                                cell_idx: parent_l1_idx as u32,
                                classification,
                                _pad: [0; 3],
                                direction_x: l1_dx / l1_dist,
                                direction_y: l1_dy / l1_dist,
                            });
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
                            let in_fov =
                                is_in_fov(rough_dot, center_dist_sq, cos_half_fov, cos_half_fov_sq);

                            if in_fov {
                                // SIZE DOMINATION FILTER (Phase A):
                                // Large creatures ignore small entities below their perception threshold.
                                // This creates asymmetric perception: mice see giants, giants ignore mice.
                                let target_mass = BioSignature::mass_from_radius(proxy.radius);
                                if !should_perceive_entity(
                                    threshold,
                                    target_mass,
                                    center_dist_sq,
                                    range_sq,
                                    in_fov,
                                ) {
                                    continue; // Target too small for this perceiver
                                }

                                candidates.push((
                                    center_dist_sq,
                                    NeighborData {
                                        entity: proxy.entity,
                                        x: proxy.x,
                                        y: proxy.y,
                                        vx: proxy.vx,
                                        vy: proxy.vy,
                                        radius: proxy.radius,
                                    },
                                ));

                                // Track max distance in adjacent cells (O(1) per candidate)
                                if !is_non_adjacent && center_dist_sq > max_adjacent_dist_sq {
                                    max_adjacent_dist_sq = center_dist_sq;
                                }
                            }
                        }
                    }

                    // =================================================================
                    // FOV-TIER EXTENDED CELLS: Query extra cells for Narrow/Wide FOV
                    // =================================================================
                    // Narrow FOV (<120°): +2 cells FRONT (predator depth hunting)
                    // Wide FOV (>200°): +2 cells SIDES (prey panoramic awareness)
                    // Medium FOV (120-200°): No extra cells (generalist)
                    //
                    // GOLDEN ZONE: Generalists query fewer cells = cheaper AND biologically accurate
                    if let Some(extra_offsets) =
                        fov_patterns::get_extra_cells(fov_tier, facing_x, facing_y)
                    {
                        for (dx, dy) in extra_offsets {
                            let extra_cx = creature_cx + dx as i32;
                            let extra_cy = creature_cy + dy as i32;

                            // Get cell index (None if outside grid bounds)
                            let Some(extra_cell_idx) =
                                grid_ref.get_cell_index_by_coords(extra_cx, extra_cy)
                            else {
                                continue;
                            };

                            // DEV-TOOLS: Track extra cells for debug visualization
                            #[cfg(feature = "dev-tools")]
                            if is_debug_target {
                                debug_queried.push((extra_cx, extra_cy));
                            }

                            // L1 classification check (same as base cells)
                            let parent_l1_idx =
                                l1_grid_ref.l0_to_l1_cell_index(extra_cell_idx, l0_width);
                            let classification = get_l1_classification(
                                parent_l1_idx,
                                &mut l1_cache,
                                &mut l1_cache_count,
                            );

                            if classification == L1Classification::Empty {
                                continue;
                            }

                            // L1 VISION: Record non-Empty L1 cells from extra cells too.
                            if !l1_vision.contains_cell(parent_l1_idx as u32) {
                                let (l1_center_x, l1_center_y) =
                                    l1_grid_ref.cell_center_from_index(parent_l1_idx);
                                let l1_dx = l1_center_x - x;
                                let l1_dy = l1_center_y - y;
                                let l1_dist = (l1_dx * l1_dx + l1_dy * l1_dy).sqrt().max(0.001);
                                l1_vision.push(L1VisionEntry {
                                    cell_idx: parent_l1_idx as u32,
                                    classification,
                                    _pad: [0; 3],
                                    direction_x: l1_dx / l1_dist,
                                    direction_y: l1_dy / l1_dist,
                                });
                            }

                            // Process entities in this extra cell
                            for proxy in grid_ref.get_cell_proxies(extra_cell_idx) {
                                if *entity == proxy.entity {
                                    continue;
                                }

                                let dx = proxy.x - x;
                                let dy = proxy.y - y;
                                let center_dist_sq = dx * dx + dy * dy;

                                // Range check
                                let max_dist = base_dist + proxy.radius;
                                if center_dist_sq > max_dist * max_dist {
                                    continue;
                                }

                                // FOV check
                                let rough_dot = dx * facing_x + dy * facing_y;
                                let in_fov = is_in_fov(
                                    rough_dot,
                                    center_dist_sq,
                                    cos_half_fov,
                                    cos_half_fov_sq,
                                );

                                if in_fov {
                                    // Size domination filter
                                    let target_mass = BioSignature::mass_from_radius(proxy.radius);
                                    if !should_perceive_entity(
                                        threshold,
                                        target_mass,
                                        center_dist_sq,
                                        range_sq,
                                        in_fov,
                                    ) {
                                        continue;
                                    }

                                    candidates.push((
                                        center_dist_sq,
                                        NeighborData {
                                            entity: proxy.entity,
                                            x: proxy.x,
                                            y: proxy.y,
                                            vx: proxy.vx,
                                            vy: proxy.vy,
                                            radius: proxy.radius,
                                        },
                                    ));
                                }
                            }
                        }
                    }

                    // Final selection: get K closest using partial sort
                    // select_nth_unstable_by(k-1) partitions so [0..k-1] are <= element at k-1
                    // Then truncate(k) keeps [0..k), which are the k smallest
                    let k = MAX_PERCEIVED_NEIGHBORS.min(candidates.len());
                    if k > 0 {
                        if candidates.len() > k {
                            candidates.select_nth_unstable_by(k - 1, |a, b| {
                                a.0.total_cmp(&b.0)
                            });
                            candidates.truncate(k);
                        }

                        for (_, neighbor) in candidates.drain(..) {
                            neighbor_cache.add_neighbor(neighbor);
                        }
                    }

                    // DEV-TOOLS: Store captured cell data for debug target
                    #[cfg(feature = "dev-tools")]
                    if is_debug_target {
                        *debug_cell_capture.lock().unwrap() = Some((debug_queried, debug_skipped));
                    }
                });
            });
        },
    );

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
            if let Some((_, pos, rot, _size, perception, neighbor_cache, l1_vision, state)) =
                entities
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
                                queried
                                    .into_iter()
                                    .map(|(cx, cy)| QueriedCell { x: cx, y: cy })
                                    .collect(),
                                skipped
                                    .into_iter()
                                    .map(|(cx, cy)| QueriedCell { x: cx, y: cy })
                                    .collect(),
                            )
                        } else {
                            (Vec::new(), Vec::new())
                        };

                    // Build neighbor debug info from the ACTUAL perception results
                    let neighbor_debug: Vec<NeighborDebugInfo> = neighbor_cache
                        .iter_neighbors()
                        .filter_map(|n| {
                            let neighbor_id = crit_ids.get(n.entity).ok()?.0;
                            Some(NeighborDebugInfo {
                                id: neighbor_id,
                                x: n.x,
                                y: n.y,
                            })
                        })
                        .collect();

                    // Build L1Vision debug entries with cell centers
                    let l1_vision_debug: Vec<L1VisionDebugEntry> = l1_vision
                        .iter()
                        .map(|entry| {
                            let (center_x, center_y) =
                                l1_grid_ref.cell_center_from_index(entry.cell_idx as usize);
                            L1VisionDebugEntry {
                                cell_idx: entry.cell_idx,
                                classification: entry.classification as u8,
                                center_x,
                                center_y,
                                direction_x: entry.direction_x,
                                direction_y: entry.direction_y,
                            }
                        })
                        .collect();

                    let (creature_cx, creature_cy) = grid_ref.world_to_cell(x, y);

                    debug_snapshot.update(
                        entity_id,
                        x,
                        y,
                        perception.range,
                        L0_VISIBLE_RANGE, // Actual visible range, not query radius
                        perception.fov_angle,
                        rot.radians,
                        ax,
                        ay,
                        neighbor_debug,
                        queried_cells,
                        skipped_cells,
                        QueriedCell {
                            x: creature_cx,
                            y: creature_cy,
                        },
                        l1_vision_debug,
                    );
                } else {
                    let (creature_cx, creature_cy) = grid_ref.world_to_cell(pos.x, pos.y);
                    debug_snapshot.update(
                        entity_id,
                        pos.x,
                        pos.y,
                        perception.range,
                        L0_VISIBLE_RANGE, // Actual visible range, not query radius
                        perception.fov_angle,
                        rot.radians,
                        ax,
                        ay,
                        std::iter::empty(),
                        std::iter::empty(),
                        std::iter::empty(),
                        QueriedCell {
                            x: creature_cx,
                            y: creature_cy,
                        },
                        std::iter::empty::<L1VisionDebugEntry>(),
                    );
                }
            }
            // else: Entity not found in query results (may lack required components)
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
