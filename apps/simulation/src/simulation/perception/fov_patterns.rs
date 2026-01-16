//! Precomputed FOV cell patterns for L0 culling.
//!
//! Safety margins accounted for:
//! - 45° cell corner extent (corners can be ±45° from cell center direction)
//! - 5° position variance within creature's own cell
//! - 22.5° octant quantization (facing direction rounded to nearest 45° octant)
//! Total effective margin: 72.5°
//!
//! Bucket thresholds (accounting for octant quantization):
//! - FOV < 125°: cull 3 cells (rear diagonals at min 112.5° from octant edge)
//! - 125° ≤ FOV < 215°: cull 1 cell (directly behind at min 157.5° from octant edge)
//! - FOV ≥ 215°: query all 9 cells
//!
//! FOV-Tier Extended Cells (beyond base 3×3):
//! - Narrow FOV (<120°): +2 cells FRONT (predator depth hunting)
//! - Medium FOV (120-200°): No extra cells (generalist)
//! - Wide FOV (>200°): +2 cells SIDES (prey panoramic awareness)

use std::f32::consts::{FRAC_PI_4, FRAC_PI_8, TAU};

use crate::simulation::creatures::constants::FovTier;

/// Precomputed bitmasks: [bucket][octant] → 9-bit cell mask
///
/// Bit encoding:
/// - Bit 0: Own cell (always 1)
/// - Bit 1: E  (1,0)   Bit 5: W  (-1,0)
/// - Bit 2: NE (1,1)   Bit 6: SW (-1,-1)
/// - Bit 3: N  (0,1)   Bit 7: S  (0,-1)
/// - Bit 4: NW (-1,1)  Bit 8: SE (1,-1)
const FOV_CELL_PATTERNS: [[u16; 8]; 3] = [
    // Bucket 0: FOV < 125° (cull 3 cells - rear diagonals + behind)
    // Min distance to rear diagonals from octant edge = 112.5°
    // Octants: E, NE, N, NW, W, SW, S, SE
    [0x18F, 0x11F, 0x03F, 0x07D, 0x0F9, 0x1F1, 0x1E3, 0x1C7],
    // Bucket 1: 125° ≤ FOV < 215° (cull 1 cell - directly behind only)
    // Min distance to behind cell from octant edge = 157.5°
    [0x1DF, 0x1BF, 0x17F, 0x0FF, 0x1FD, 0x1FB, 0x1F7, 0x1EF],
    // Bucket 2: FOV ≥ 215° (query all 9)
    [0x1FF, 0x1FF, 0x1FF, 0x1FF, 0x1FF, 0x1FF, 0x1FF, 0x1FF],
];

/// FOV thresholds in radians
/// 125° = boundary for culling rear diagonals (min distance 112.5° with octant edge)
/// 215° = boundary for culling behind cell (min distance 157.5° with octant edge)
const FOV_THRESHOLD_125: f32 = 125.0 * std::f32::consts::PI / 180.0;
const FOV_THRESHOLD_215: f32 = 215.0 * std::f32::consts::PI / 180.0;

/// Quantize FOV (radians) to bucket 0-2 (branchless)
#[inline]
pub fn fov_to_bucket(fov_rad: f32) -> usize {
    (fov_rad >= FOV_THRESHOLD_125) as usize + (fov_rad >= FOV_THRESHOLD_215) as usize
}

/// Quantize facing direction to octant 0-7 (E=0, NE=1, N=2, ...) (branchless)
#[inline]
pub fn facing_to_octant(fx: f32, fy: f32) -> usize {
    let angle = fy.atan2(fx);
    let norm = (angle + TAU).rem_euclid(TAU);
    ((norm + FRAC_PI_8) / FRAC_PI_4) as usize & 7
}

/// Get bitmask of cells to query for given FOV and facing direction
#[inline]
pub fn get_cell_pattern(fov_rad: f32, fx: f32, fy: f32) -> u16 {
    FOV_CELL_PATTERNS[fov_to_bucket(fov_rad)][facing_to_octant(fx, fy)]
}

/// Get bitmask using pre-computed bucket and octant (avoids redundant atan2).
#[inline]
pub fn get_cell_pattern_by_octant(bucket: usize, octant: usize) -> u16 {
    FOV_CELL_PATTERNS[bucket][octant]
}

/// Maps (dx+1)*3 + (dy+1) → bit position in pattern (branchless lookup)
/// Layout: [-1,-1]=0, [-1,0]=1, [-1,1]=2, [0,-1]=3, [0,0]=4, [0,1]=5, [1,-1]=6, [1,0]=7, [1,1]=8
const OFFSET_TO_BIT: [u8; 9] = [
    6, // (-1,-1) → SW
    5, // (-1, 0) → W
    4, // (-1, 1) → NW
    7, // ( 0,-1) → S
    0, // ( 0, 0) → own cell
    3, // ( 0, 1) → N
    8, // ( 1,-1) → SE
    1, // ( 1, 0) → E
    2, // ( 1, 1) → NE
];

/// Check if cell at offset (dx, dy) should be queried given the pattern (branchless)
/// SAFETY: dx and dy must be in [-1, 1]. The perception cell loop guarantees this.
#[inline]
pub fn should_query_cell(dx: i32, dy: i32, pattern: u16) -> bool {
    debug_assert!(
        (-1..=1).contains(&dx) && (-1..=1).contains(&dy),
        "should_query_cell: offset ({}, {}) out of range",
        dx,
        dy
    );
    let idx = ((dx + 1) * 3 + (dy + 1)) as usize;
    (pattern >> OFFSET_TO_BIT[idx]) & 1 == 1
}

// =============================================================================
// FOV-Tier Extended Cells
// =============================================================================
// Precomputed cell offsets for specialized perception beyond base 3×3 grid.
// Indexed by octant (0-7): E, NE, N, NW, W, SW, S, SE
//
// 5-TIER SYSTEM:
// - UltraNarrow (<75°): +4 front cells (apex predator tunnel vision)
// - Narrow (75-120°): +2 front cells (predator depth hunting)
// - Medium (120-200°): No extra cells (generalist)
// - Wide (200-280°): +2 side cells (alert prey peripheral)
// - UltraWide (>280°): +4 side cells (paranoid prey panoramic)
//
// BIOLOGICAL BASIS:
// - UltraNarrow (apex predators): Owls, sharks, eagles have extreme binocular
//   vision with near-zero peripheral awareness - they commit fully to forward
//   depth perception for precise strike distance estimation.
// - Narrow (predators): Wolves, cats sacrifice peripheral vision for depth.
// - Wide (alert prey): Deer, horses have wide peripheral awareness.
// - UltraWide (paranoid prey): Mice, rabbits have almost 360° vision,
//   sacrificing all depth perception for maximum threat detection.

/// UltraNarrow FOV extra cells: 4 cells extending deep forward (apex predator)
/// Used by apex predator archetypes (<75° FOV) for extreme tunnel vision hunting
///
/// Grid visualization (East-facing example):
/// ```text
///     ┌─┬─┬─┐
///     │ │ │ │
///     ├─┼─┼─┼─┬─┬─┬─┐
///     │ │●→│ │▓│▓│▓│▓│  ▓ = extra cells at (2,0), (3,0), (4,0), (5,0)
///     ├─┼─┼─┼─┴─┴─┴─┘
///     │ │ │ │
///     └─┴─┴─┘
/// ```
const ULTRA_NARROW_FRONT_CELLS: [[(i8, i8); 4]; 8] = [
    [(2, 0), (3, 0), (4, 0), (5, 0)],         // Octant 0 (E):  extends +x
    [(2, 2), (3, 3), (4, 4), (5, 5)],         // Octant 1 (NE): extends +x,+y diagonal
    [(0, 2), (0, 3), (0, 4), (0, 5)],         // Octant 2 (N):  extends +y
    [(-2, 2), (-3, 3), (-4, 4), (-5, 5)],     // Octant 3 (NW): extends -x,+y diagonal
    [(-2, 0), (-3, 0), (-4, 0), (-5, 0)],     // Octant 4 (W):  extends -x
    [(-2, -2), (-3, -3), (-4, -4), (-5, -5)], // Octant 5 (SW): extends -x,-y diagonal
    [(0, -2), (0, -3), (0, -4), (0, -5)],     // Octant 6 (S):  extends -y
    [(2, -2), (3, -3), (4, -4), (5, -5)],     // Octant 7 (SE): extends +x,-y diagonal
];

/// Narrow FOV extra cells: 2 cells extending forward in facing direction
/// Used by predator archetypes (75-120° FOV) for depth hunting
///
/// Grid visualization (East-facing example):
/// ```text
///     ┌─┬─┬─┐
///     │ │ │ │
///     ├─┼─┼─┼─┬─┐
///     │ │●→│ │▓│▓│  ▓ = extra cells at (2,0) and (3,0)
///     ├─┼─┼─┼─┴─┘
///     │ │ │ │
///     └─┴─┴─┘
/// ```
const NARROW_FRONT_CELLS: [[(i8, i8); 2]; 8] = [
    [(2, 0), (3, 0)],     // Octant 0 (E):  extends +x
    [(2, 2), (3, 3)],     // Octant 1 (NE): extends +x,+y diagonal
    [(0, 2), (0, 3)],     // Octant 2 (N):  extends +y
    [(-2, 2), (-3, 3)],   // Octant 3 (NW): extends -x,+y diagonal
    [(-2, 0), (-3, 0)],   // Octant 4 (W):  extends -x
    [(-2, -2), (-3, -3)], // Octant 5 (SW): extends -x,-y diagonal
    [(0, -2), (0, -3)],   // Octant 6 (S):  extends -y
    [(2, -2), (3, -3)],   // Octant 7 (SE): extends +x,-y diagonal
];

/// Wide FOV extra cells: 2 cells extending perpendicular to facing direction
/// Used by prey archetypes (200-280° FOV) for panoramic threat detection
///
/// Grid visualization (East-facing example):
/// ```text
///         ┌─┐
///         │▓│  ▓ = extra cell at (0, 2)
///     ┌─┬─┼─┼─┬─┐
///     │ │ │ │ │ │
///     ├─┼─┼─┼─┼─┤
///     │ │ │●│ │ │  ● = creature facing East
///     ├─┼─┼─┼─┼─┤
///     │ │ │ │ │ │
///     └─┴─┼─┼─┴─┘
///         │▓│  ▓ = extra cell at (0, -2)
///         └─┘
/// ```
const WIDE_SIDE_CELLS: [[(i8, i8); 2]; 8] = [
    [(0, 2), (0, -2)],    // Octant 0 (E):  sides are ±y
    [(-1, 2), (2, -1)],   // Octant 1 (NE): perpendicular is NW-SE axis
    [(2, 0), (-2, 0)],    // Octant 2 (N):  sides are ±x
    [(2, 1), (-1, -2)],   // Octant 3 (NW): perpendicular is NE-SW axis
    [(0, 2), (0, -2)],    // Octant 4 (W):  sides are ±y
    [(1, 2), (-2, -1)],   // Octant 5 (SW): perpendicular is NW-SE axis
    [(2, 0), (-2, 0)],    // Octant 6 (S):  sides are ±x
    [(-2, 1), (1, -2)],   // Octant 7 (SE): perpendicular is NE-SW axis
];

/// UltraWide FOV extra cells: 4 cells extending on sides (paranoid prey panoramic)
/// Used by paranoid prey archetypes (>280° FOV) for extreme peripheral awareness
///
/// Grid visualization (East-facing example):
/// ```text
///         ┌─┬─┐
///         │▓│▓│  ▓ = extra cells at (0,2) and (1,2)
///     ┌─┬─┼─┼─┼─┐
///     │ │ │ │ │ │
///     ├─┼─┼─┼─┼─┤
///     │ │ │●│ │ │  ● = creature facing East
///     ├─┼─┼─┼─┼─┤
///     │ │ │ │ │ │
///     └─┴─┼─┼─┼─┘
///         │▓│▓│  ▓ = extra cells at (0,-2) and (1,-2)
///         └─┴─┘
/// ```
const ULTRA_WIDE_SIDE_CELLS: [[(i8, i8); 4]; 8] = [
    [(0, 2), (0, -2), (1, 2), (1, -2)],       // Octant 0 (E):  sides ±y plus forward extension
    [(-1, 2), (2, -1), (-2, 1), (1, -2)],     // Octant 1 (NE): extended perpendicular
    [(2, 0), (-2, 0), (2, 1), (-2, 1)],       // Octant 2 (N):  sides ±x plus forward extension
    [(2, 1), (-1, -2), (1, 2), (-2, -1)],     // Octant 3 (NW): extended perpendicular
    [(0, 2), (0, -2), (-1, 2), (-1, -2)],     // Octant 4 (W):  sides ±y plus forward extension
    [(1, 2), (-2, -1), (2, 1), (-1, -2)],     // Octant 5 (SW): extended perpendicular
    [(2, 0), (-2, 0), (2, -1), (-2, -1)],     // Octant 6 (S):  sides ±x plus forward extension
    [(-2, 1), (1, -2), (-1, 2), (2, -1)],     // Octant 7 (SE): extended perpendicular
];

/// Get extra cell offsets for the given FOV tier and facing direction.
/// Returns None for Medium tier (generalists have no extended perception).
///
/// # Arguments
/// * `fov_tier` - The creature's FOV tier (determined at spawn from DNA)
/// * `fx`, `fy` - Facing direction vector (normalized or unnormalized)
///
/// # Returns
/// * `Some(&[(i8, i8)])` - Slice of cell offsets to query (2 or 4 depending on tier)
/// * `None` - No extra cells (Medium tier generalist)
///
/// # Cell counts by tier
/// * UltraNarrow: 4 cells (deep forward tunnel vision)
/// * Narrow: 2 cells (forward depth perception)
/// * Medium: 0 cells (no extra cells)
/// * Wide: 2 cells (perpendicular sides)
/// * UltraWide: 4 cells (extended sides panoramic)
#[inline]
pub fn get_extra_cells(fov_tier: FovTier, fx: f32, fy: f32) -> Option<&'static [(i8, i8)]> {
    get_extra_cells_by_octant(fov_tier, facing_to_octant(fx, fy))
}

/// Get extra cell offsets using pre-computed octant (avoids redundant atan2).
/// Use this when octant has already been calculated for FOV pattern lookup.
#[inline]
pub fn get_extra_cells_by_octant(fov_tier: FovTier, octant: usize) -> Option<&'static [(i8, i8)]> {
    match fov_tier {
        FovTier::UltraNarrow => Some(&ULTRA_NARROW_FRONT_CELLS[octant]),
        FovTier::Narrow => Some(&NARROW_FRONT_CELLS[octant]),
        FovTier::Medium => None,
        FovTier::Wide => Some(&WIDE_SIDE_CELLS[octant]),
        FovTier::UltraWide => Some(&ULTRA_WIDE_SIDE_CELLS[octant]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_facing_to_octant_cardinal_directions() {
        assert_eq!(facing_to_octant(1.0, 0.0), 0); // E
        assert_eq!(facing_to_octant(0.0, 1.0), 2); // N
        assert_eq!(facing_to_octant(-1.0, 0.0), 4); // W
        assert_eq!(facing_to_octant(0.0, -1.0), 6); // S
    }

    #[test]
    fn test_facing_to_octant_diagonal_directions() {
        assert_eq!(facing_to_octant(1.0, 1.0), 1); // NE
        assert_eq!(facing_to_octant(-1.0, 1.0), 3); // NW
        assert_eq!(facing_to_octant(-1.0, -1.0), 5); // SW
        assert_eq!(facing_to_octant(1.0, -1.0), 7); // SE
    }

    #[test]
    fn test_fov_to_bucket_narrow() {
        assert_eq!(fov_to_bucket(PI / 2.0), 0); // 90° → bucket 0
        assert_eq!(fov_to_bucket(120.0 * PI / 180.0), 0); // 120° → bucket 0
    }

    #[test]
    fn test_fov_to_bucket_medium() {
        assert_eq!(fov_to_bucket(136.0 * PI / 180.0), 1); // 136° → bucket 1 (the user's case!)
        assert_eq!(fov_to_bucket(PI), 1); // 180° → bucket 1
        assert_eq!(fov_to_bucket(200.0 * PI / 180.0), 1); // 200° → bucket 1
    }

    #[test]
    fn test_fov_to_bucket_wide() {
        assert_eq!(fov_to_bucket(220.0 * PI / 180.0), 2); // 220° → bucket 2
        assert_eq!(fov_to_bucket(2.0 * PI), 2); // 360° → bucket 2
    }

    #[test]
    fn test_narrow_fov_facing_east_culls_behind() {
        // 90° FOV (bucket 0), facing E
        let pattern = get_cell_pattern(PI / 2.0, 1.0, 0.0);
        assert!(should_query_cell(0, 0, pattern)); // own: visible
        assert!(should_query_cell(1, 0, pattern)); // E: visible
        assert!(should_query_cell(1, 1, pattern)); // NE: visible
        assert!(should_query_cell(0, 1, pattern)); // N: visible
        assert!(!should_query_cell(-1, 1, pattern)); // NW: culled
        assert!(!should_query_cell(-1, 0, pattern)); // W: culled
        assert!(!should_query_cell(-1, -1, pattern)); // SW: culled
        assert!(should_query_cell(0, -1, pattern)); // S: visible
        assert!(should_query_cell(1, -1, pattern)); // SE: visible
    }

    #[test]
    fn test_narrow_fov_facing_west_culls_behind() {
        // 90° FOV (bucket 0), facing W
        let pattern = get_cell_pattern(PI / 2.0, -1.0, 0.0);
        assert!(should_query_cell(0, 0, pattern)); // own: visible
        assert!(!should_query_cell(1, 0, pattern)); // E: culled
        assert!(!should_query_cell(1, 1, pattern)); // NE: culled
        assert!(should_query_cell(0, 1, pattern)); // N: visible
        assert!(should_query_cell(-1, 1, pattern)); // NW: visible
        assert!(should_query_cell(-1, 0, pattern)); // W: visible
        assert!(should_query_cell(-1, -1, pattern)); // SW: visible
        assert!(should_query_cell(0, -1, pattern)); // S: visible
        assert!(!should_query_cell(1, -1, pattern)); // SE: culled
    }

    #[test]
    fn test_medium_fov_136_culls_only_behind() {
        // 136° FOV (bucket 1 - the user's bug case), facing E
        // Should only cull W, keep NW and SW
        let pattern = get_cell_pattern(136.0 * PI / 180.0, 1.0, 0.0);
        assert_eq!(pattern.count_ones(), 8); // 8 cells (all but W)
        assert!(!should_query_cell(-1, 0, pattern)); // W: culled
        assert!(should_query_cell(-1, 1, pattern)); // NW: visible (was bug!)
        assert!(should_query_cell(-1, -1, pattern)); // SW: visible (was bug!)
    }

    #[test]
    fn test_wide_fov_culls_only_behind() {
        // 180° FOV (bucket 1), facing E
        let pattern = get_cell_pattern(PI, 1.0, 0.0);
        assert_eq!(pattern.count_ones(), 8); // 8 cells (all but W)
        assert!(!should_query_cell(-1, 0, pattern)); // W: culled
        assert!(should_query_cell(-1, 1, pattern)); // NW: visible
        assert!(should_query_cell(-1, -1, pattern)); // SW: visible
    }

    #[test]
    fn test_ultra_wide_fov_queries_all() {
        // 220° FOV (bucket 2), facing E
        let pattern = get_cell_pattern(220.0 * PI / 180.0, 1.0, 0.0);
        assert_eq!(pattern, 0x1FF); // all 9 bits set
        assert_eq!(pattern.count_ones(), 9);
    }

    #[test]
    fn test_bucket_0_pattern_values() {
        // Verify computed bitmask values
        assert_eq!(FOV_CELL_PATTERNS[0][0], 0x18F); // E facing
        assert_eq!(FOV_CELL_PATTERNS[0][4], 0x0F9); // W facing
    }

    #[test]
    fn test_bucket_1_pattern_values() {
        // Bucket 1: only cull 1 cell directly behind
        assert_eq!(FOV_CELL_PATTERNS[1][0], 0x1DF); // E facing, cull W
        assert_eq!(FOV_CELL_PATTERNS[1][4], 0x1FD); // W facing, cull E
    }

    #[test]
    fn test_bucket_2_all_visible() {
        // Bucket 2: all octants query all 9 cells
        for octant in 0..8 {
            assert_eq!(FOV_CELL_PATTERNS[2][octant], 0x1FF);
        }
    }

    // ==========================================================================
    // FOV Tier Extended Cells Tests
    // ==========================================================================

    #[test]
    fn test_fov_tier_thresholds() {
        // Updated for 5-tier system:
        // UltraNarrow: <75°, Narrow: 75-120°, Medium: 120-200°, Wide: 200-280°, UltraWide: >280°
        assert_eq!(FovTier::from_fov_degrees(45.0), FovTier::UltraNarrow);
        assert_eq!(FovTier::from_fov_degrees(74.9), FovTier::UltraNarrow);
        assert_eq!(FovTier::from_fov_degrees(75.0), FovTier::Narrow);
        assert_eq!(FovTier::from_fov_degrees(90.0), FovTier::Narrow);
        assert_eq!(FovTier::from_fov_degrees(119.9), FovTier::Narrow);
        assert_eq!(FovTier::from_fov_degrees(120.0), FovTier::Medium);
        assert_eq!(FovTier::from_fov_degrees(180.0), FovTier::Medium);
        assert_eq!(FovTier::from_fov_degrees(200.0), FovTier::Medium);
        assert_eq!(FovTier::from_fov_degrees(200.1), FovTier::Wide);
        assert_eq!(FovTier::from_fov_degrees(250.0), FovTier::Wide);
        assert_eq!(FovTier::from_fov_degrees(279.9), FovTier::Wide);
        assert_eq!(FovTier::from_fov_degrees(280.0), FovTier::UltraWide);
        assert_eq!(FovTier::from_fov_degrees(340.0), FovTier::UltraWide);
    }

    #[test]
    fn test_fov_tier_has_extra_cells() {
        // All tiers except Medium have extra cells
        assert!(FovTier::UltraNarrow.has_extra_cells());
        assert!(FovTier::Narrow.has_extra_cells());
        assert!(!FovTier::Medium.has_extra_cells());
        assert!(FovTier::Wide.has_extra_cells());
        assert!(FovTier::UltraWide.has_extra_cells());
    }

    #[test]
    fn test_get_extra_cells_medium_returns_none() {
        assert!(get_extra_cells(FovTier::Medium, 1.0, 0.0).is_none());
        assert!(get_extra_cells(FovTier::Medium, 0.0, 1.0).is_none());
        assert!(get_extra_cells(FovTier::Medium, -1.0, -1.0).is_none());
    }

    #[test]
    fn test_get_extra_cells_narrow_facing_east() {
        let cells = get_extra_cells(FovTier::Narrow, 1.0, 0.0).unwrap();
        assert_eq!(cells, [(2, 0), (3, 0)]);
    }

    #[test]
    fn test_get_extra_cells_narrow_facing_north() {
        let cells = get_extra_cells(FovTier::Narrow, 0.0, 1.0).unwrap();
        assert_eq!(cells, [(0, 2), (0, 3)]);
    }

    #[test]
    fn test_get_extra_cells_narrow_facing_west() {
        let cells = get_extra_cells(FovTier::Narrow, -1.0, 0.0).unwrap();
        assert_eq!(cells, [(-2, 0), (-3, 0)]);
    }

    #[test]
    fn test_get_extra_cells_narrow_facing_south() {
        let cells = get_extra_cells(FovTier::Narrow, 0.0, -1.0).unwrap();
        assert_eq!(cells, [(0, -2), (0, -3)]);
    }

    #[test]
    fn test_get_extra_cells_narrow_facing_northeast() {
        let cells = get_extra_cells(FovTier::Narrow, 1.0, 1.0).unwrap();
        assert_eq!(cells, [(2, 2), (3, 3)]);
    }

    #[test]
    fn test_get_extra_cells_wide_facing_east() {
        let cells = get_extra_cells(FovTier::Wide, 1.0, 0.0).unwrap();
        assert_eq!(cells, [(0, 2), (0, -2)]);
    }

    #[test]
    fn test_get_extra_cells_wide_facing_north() {
        let cells = get_extra_cells(FovTier::Wide, 0.0, 1.0).unwrap();
        assert_eq!(cells, [(2, 0), (-2, 0)]);
    }

    #[test]
    fn test_get_extra_cells_wide_perpendicular_pattern() {
        let east = get_extra_cells(FovTier::Wide, 1.0, 0.0).unwrap();
        let north = get_extra_cells(FovTier::Wide, 0.0, 1.0).unwrap();
        assert_ne!(east, north);
        assert_eq!(east[0].0, 0);
        assert_eq!(north[0].1, 0);
    }

    #[test]
    fn test_narrow_front_cells_symmetric() {
        let e = get_extra_cells(FovTier::Narrow, 1.0, 0.0).unwrap();
        let w = get_extra_cells(FovTier::Narrow, -1.0, 0.0).unwrap();
        assert_eq!(e[0].0, -w[0].0);
        assert_eq!(e[0].1, w[0].1);

        let n = get_extra_cells(FovTier::Narrow, 0.0, 1.0).unwrap();
        let s = get_extra_cells(FovTier::Narrow, 0.0, -1.0).unwrap();
        assert_eq!(n[0].0, s[0].0);
        assert_eq!(n[0].1, -s[0].1);
    }

    // ==========================================================================
    // 5-Tier FOV System Tests
    // ==========================================================================

    #[test]
    fn test_fov_tier_5_tier_thresholds() {
        // 5-tier system boundary tests:
        // UltraNarrow: <75°
        assert_eq!(FovTier::from_fov_degrees(45.0), FovTier::UltraNarrow);
        assert_eq!(FovTier::from_fov_degrees(74.9), FovTier::UltraNarrow);

        // Narrow: 75-120°
        assert_eq!(FovTier::from_fov_degrees(75.0), FovTier::Narrow);
        assert_eq!(FovTier::from_fov_degrees(90.0), FovTier::Narrow);
        assert_eq!(FovTier::from_fov_degrees(119.9), FovTier::Narrow);

        // Medium: 120-200°
        assert_eq!(FovTier::from_fov_degrees(120.0), FovTier::Medium);
        assert_eq!(FovTier::from_fov_degrees(180.0), FovTier::Medium);
        assert_eq!(FovTier::from_fov_degrees(200.0), FovTier::Medium);

        // Wide: 200-280°
        assert_eq!(FovTier::from_fov_degrees(200.1), FovTier::Wide);
        assert_eq!(FovTier::from_fov_degrees(250.0), FovTier::Wide);
        assert_eq!(FovTier::from_fov_degrees(279.9), FovTier::Wide);

        // UltraWide: >280°
        assert_eq!(FovTier::from_fov_degrees(280.0), FovTier::UltraWide);
        assert_eq!(FovTier::from_fov_degrees(320.0), FovTier::UltraWide);
        assert_eq!(FovTier::from_fov_degrees(340.0), FovTier::UltraWide);
    }

    #[test]
    fn test_ultra_narrow_extra_cells_facing_east() {
        let cells = get_extra_cells(FovTier::UltraNarrow, 1.0, 0.0).unwrap();
        assert_eq!(cells.len(), 4, "UltraNarrow should have 4 extra cells");
        assert_eq!(cells[0], (2, 0));
        assert_eq!(cells[1], (3, 0));
        assert_eq!(cells[2], (4, 0));
        assert_eq!(cells[3], (5, 0));
    }

    #[test]
    fn test_ultra_narrow_extra_cells_facing_north() {
        let cells = get_extra_cells(FovTier::UltraNarrow, 0.0, 1.0).unwrap();
        assert_eq!(cells.len(), 4, "UltraNarrow should have 4 extra cells");
        assert_eq!(cells[0], (0, 2));
        assert_eq!(cells[1], (0, 3));
        assert_eq!(cells[2], (0, 4));
        assert_eq!(cells[3], (0, 5));
    }

    #[test]
    fn test_ultra_narrow_extra_cells_facing_northeast() {
        let cells = get_extra_cells(FovTier::UltraNarrow, 1.0, 1.0).unwrap();
        assert_eq!(cells.len(), 4, "UltraNarrow should have 4 extra cells");
        assert_eq!(cells[0], (2, 2));
        assert_eq!(cells[1], (3, 3));
        assert_eq!(cells[2], (4, 4));
        assert_eq!(cells[3], (5, 5));
    }

    #[test]
    fn test_ultra_wide_extra_cells_facing_east() {
        let cells = get_extra_cells(FovTier::UltraWide, 1.0, 0.0).unwrap();
        assert_eq!(cells.len(), 4, "UltraWide should have 4 extra cells");
        // East facing: sides are ±y plus diagonal extensions
        assert_eq!(cells[0], (0, 2));
        assert_eq!(cells[1], (0, -2));
        assert_eq!(cells[2], (1, 2));
        assert_eq!(cells[3], (1, -2));
    }

    #[test]
    fn test_ultra_wide_extra_cells_facing_north() {
        let cells = get_extra_cells(FovTier::UltraWide, 0.0, 1.0).unwrap();
        assert_eq!(cells.len(), 4, "UltraWide should have 4 extra cells");
        // North facing: sides are ±x plus diagonal extensions
        assert_eq!(cells[0], (2, 0));
        assert_eq!(cells[1], (-2, 0));
        assert_eq!(cells[2], (2, 1));
        assert_eq!(cells[3], (-2, 1));
    }

    #[test]
    fn test_narrow_returns_2_cells_as_slice() {
        let cells = get_extra_cells(FovTier::Narrow, 1.0, 0.0).unwrap();
        assert_eq!(cells.len(), 2, "Narrow should have 2 extra cells");
        assert_eq!(cells[0], (2, 0));
        assert_eq!(cells[1], (3, 0));
    }

    #[test]
    fn test_wide_returns_2_cells_as_slice() {
        let cells = get_extra_cells(FovTier::Wide, 1.0, 0.0).unwrap();
        assert_eq!(cells.len(), 2, "Wide should have 2 extra cells");
        assert_eq!(cells[0], (0, 2));
        assert_eq!(cells[1], (0, -2));
    }

    #[test]
    fn test_all_tiers_have_extra_cells_except_medium() {
        assert!(FovTier::UltraNarrow.has_extra_cells());
        assert!(FovTier::Narrow.has_extra_cells());
        assert!(!FovTier::Medium.has_extra_cells());
        assert!(FovTier::Wide.has_extra_cells());
        assert!(FovTier::UltraWide.has_extra_cells());
    }

}
