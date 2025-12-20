pub const CELL_SIZE: f32 = 10.0;

/// Half-diagonal of a cell (distance from center to corner / √2)
/// Used for conservative distance bounds in spatial queries.
pub const CELL_HALF_DIAGONAL: f32 = CELL_SIZE * 0.7071068; // CELL_SIZE * √2 / 2

/// Priority offset ensures adjacent cells sort before all non-adjacent cells in spatial queries.
/// Max possible dist_sq in a reasonable world is ~1e8, so 1e9 is safe.
pub const NON_ADJACENT_OFFSET: f32 = 1e9;

/// Precomputed cos(15 degrees) for cell FOV safety margin.
/// Conservative margin accounts for cell corners inside FOV when center is outside.
pub const COS_SAFETY_MARGIN: f32 = 0.9659;

/// Precomputed sin(15 degrees) for cell FOV safety margin.
pub const SIN_SAFETY_MARGIN: f32 = 0.2588;
