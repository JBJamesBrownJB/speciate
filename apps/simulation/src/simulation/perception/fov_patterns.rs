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

use std::f32::consts::{FRAC_PI_4, FRAC_PI_8, TAU};

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

/// Quantize FOV (radians) to bucket 0-2
#[inline]
pub fn fov_to_bucket(fov_rad: f32) -> usize {
    if fov_rad < FOV_THRESHOLD_125 {
        0
    } else if fov_rad < FOV_THRESHOLD_215 {
        1
    } else {
        2
    }
}

/// Quantize facing direction to octant 0-7 (E=0, NE=1, N=2, ...)
#[inline]
pub fn facing_to_octant(fx: f32, fy: f32) -> usize {
    let angle = fy.atan2(fx);
    let norm = if angle < 0.0 { angle + TAU } else { angle };
    ((norm + FRAC_PI_8) / FRAC_PI_4) as usize % 8
}

/// Get bitmask of cells to query for given FOV and facing direction
#[inline]
pub fn get_cell_pattern(fov_rad: f32, fx: f32, fy: f32) -> u16 {
    FOV_CELL_PATTERNS[fov_to_bucket(fov_rad)][facing_to_octant(fx, fy)]
}

/// Check if cell at offset (dx, dy) should be queried given the pattern
#[inline]
pub fn should_query_cell(dx: i32, dy: i32, pattern: u16) -> bool {
    let bit = match (dx, dy) {
        (0, 0) => 0,
        (1, 0) => 1,
        (1, 1) => 2,
        (0, 1) => 3,
        (-1, 1) => 4,
        (-1, 0) => 5,
        (-1, -1) => 6,
        (0, -1) => 7,
        (1, -1) => 8,
        _ => return true, // Unknown offset: query to be safe
    };
    (pattern >> bit) & 1 == 1
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

    #[test]
    fn test_unknown_offset_queries_safely() {
        // Out-of-range offsets should query to be safe
        assert!(should_query_cell(2, 0, 0x000));
        assert!(should_query_cell(0, 2, 0x000));
        assert!(should_query_cell(-2, -2, 0x000));
    }
}
