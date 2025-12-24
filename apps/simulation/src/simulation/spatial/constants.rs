pub const CELL_SIZE: f32 = 10.0;

/// Ratio of L1 to L0 cell size (L1 = L0_TO_L1_RATIO × L0).
pub const L0_TO_L1_RATIO: usize = 3; /// DO NOT CHANGE, it is already assumed to be 3 in pre-computed grid pattern tables!

/// L1 (coarse) grid cell size in world units (derived from L0 × ratio).
pub const L1_CELL_SIZE: f32 = CELL_SIZE * L0_TO_L1_RATIO as f32;

/// Half-diagonal of a cell (distance from center to corner / √2)
/// Used for conservative distance bounds in spatial queries.
pub const CELL_HALF_DIAGONAL: f32 = CELL_SIZE * 0.7071068; // CELL_SIZE * √2 / 2

/// Priority offset ensures adjacent cells sort before all non-adjacent cells in spatial queries.
/// Max possible dist_sq in a reasonable world is ~1e8, so 1e9 is safe.
pub const NON_ADJACENT_OFFSET: f32 = 1e9;

/// Skip sorting for small cell sets (adjacent 3x3 grid = 9 cells max)
pub const SMALL_SORT_THRESHOLD: usize = 9;

/// Epsilon for distance comparisons (avoid division by zero)
pub const DISTANCE_EPSILON: f32 = 0.001;

/// FOV threshold for disabling culling (cos(150°) ≈ -0.866, i.e., >= 300° total FOV)
pub const WIDE_FOV_THRESHOLD: f32 = -0.866;

/// Precomputed trigonometry for 15° safety margin in FOV calculations
pub const COS_15_DEG: f32 = 0.9659;
pub const SIN_15_DEG: f32 = 0.2588;
