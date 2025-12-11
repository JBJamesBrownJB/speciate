pub const CELL_SIZE: f32 = 10.0;

/// Priority offset ensures adjacent cells sort before all non-adjacent cells in spatial queries.
/// Max possible dist_sq in a reasonable world is ~1e8, so 1e9 is safe.
pub const NON_ADJACENT_OFFSET: f32 = 1e9;
