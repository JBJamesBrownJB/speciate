pub const CELL_SIZE: f32 = 20.0;

/// L1 (coarse) grid cell size in world units (L1 = 3×3 L0 cells, hardcoded in fov_patterns.rs).
pub const L1_CELL_SIZE: f32 = CELL_SIZE * 3.0;

/// L2 (strategic) grid cell size in world units (L2 = 3×3 L1 cells).
/// Used for long-range strategic perception (180m+ range band).
pub const L2_CELL_SIZE: f32 = L1_CELL_SIZE * 3.0;

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
