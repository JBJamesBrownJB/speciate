use super::*;
use crate::simulation::core::world_bounds::MAX_WORLD_SIZE;

// =============================================================================
// TerrainGrid Coordinate Conversion Tests
// =============================================================================

#[test]
fn test_world_to_cell_origin() {
    let grid = TerrainGrid::new();
    // Origin (0, 0) should map to center cell
    let (cx, cy) = grid.world_to_cell(0.0, 0.0);
    // With 250 cells and world [-2500, 2500], origin maps to cell 125
    assert_eq!(cx, 125);
    assert_eq!(cy, 125);
}

#[test]
fn test_world_to_cell_min_corner() {
    let grid = TerrainGrid::new();
    // Min corner (-2500, -2500) should map to cell (0, 0)
    let half = MAX_WORLD_SIZE / 2.0;
    let (cx, cy) = grid.world_to_cell(-half, -half);
    assert_eq!(cx, 0);
    assert_eq!(cy, 0);
}

#[test]
fn test_world_to_cell_max_corner() {
    let grid = TerrainGrid::new();
    // Max corner should map to last cell (249, 249)
    let half = MAX_WORLD_SIZE / 2.0;
    // Use value just inside bounds
    let (cx, cy) = grid.world_to_cell(half - 0.1, half - 0.1);
    assert_eq!(cx, 249);
    assert_eq!(cy, 249);
}

#[test]
fn test_world_to_cell_clamping() {
    let grid = TerrainGrid::new();
    // Values outside bounds should clamp to valid range
    let (cx, cy) = grid.world_to_cell(10000.0, -10000.0);
    assert_eq!(cx, 249); // Clamped to max
    assert_eq!(cy, 0); // Clamped to min
}

#[test]
fn test_cell_to_world_center_origin() {
    let grid = TerrainGrid::new();
    // Cell 125 should map to near origin (center of that cell)
    let (wx, wy) = grid.cell_to_world_center(125, 125);
    // Cell center = (125 * 20) + 10 - 2500 = 2510 - 2500 = 10
    assert!((wx - 10.0).abs() < 0.01);
    assert!((wy - 10.0).abs() < 0.01);
}

#[test]
fn test_cell_to_world_center_min_cell() {
    let grid = TerrainGrid::new();
    // Cell (0, 0) center should be at (-2490, -2490)
    // = (0 * 20) + 10 - 2500 = -2490
    let (wx, wy) = grid.cell_to_world_center(0, 0);
    assert!((wx - (-2490.0)).abs() < 0.01);
    assert!((wy - (-2490.0)).abs() < 0.01);
}

#[test]
fn test_cell_to_world_center_max_cell() {
    let grid = TerrainGrid::new();
    // Cell (249, 249) center should be at (2490, 2490)
    // = (249 * 20) + 10 - 2500 = 4980 + 10 - 2500 = 2490
    let (wx, wy) = grid.cell_to_world_center(249, 249);
    assert!((wx - 2490.0).abs() < 0.01);
    assert!((wy - 2490.0).abs() < 0.01);
}

#[test]
fn test_roundtrip_world_to_cell_to_world() {
    let grid = TerrainGrid::new();
    // Convert world -> cell -> world, should land in same cell
    let original_x = 100.0;
    let original_y = -200.0;

    let (cx, cy) = grid.world_to_cell(original_x, original_y);
    let (wx, wy) = grid.cell_to_world_center(cx, cy);

    // The returned world coord should be within the same cell
    let (cx2, cy2) = grid.world_to_cell(wx, wy);
    assert_eq!(cx, cx2);
    assert_eq!(cy, cy2);
}

// =============================================================================
// TerrainGrid Blocking Tests
// =============================================================================

#[test]
fn test_new_grid_all_open() {
    let grid = TerrainGrid::new();
    // Fresh grid should have no blocked cells
    assert!(!grid.is_blocked(0.0, 0.0));
    assert!(!grid.is_blocked(1000.0, -1000.0));
    assert!(!grid.is_blocked(-2000.0, 2000.0));
}

#[test]
fn test_set_blocked_and_query() {
    let mut grid = TerrainGrid::new();

    // Block cell at (100, 100)
    let (cx, cy) = grid.world_to_cell(100.0, 100.0);
    grid.set_blocked_cell(cx, cy, true);

    // Query using world coords in that cell
    assert!(grid.is_blocked(100.0, 100.0));
    assert!(grid.is_blocked(105.0, 105.0)); // Still same cell

    // Adjacent cells should still be open
    assert!(!grid.is_blocked(130.0, 100.0)); // Next cell over
}

#[test]
fn test_unblock_cell() {
    let mut grid = TerrainGrid::new();

    let (cx, cy) = grid.world_to_cell(0.0, 0.0);
    grid.set_blocked_cell(cx, cy, true);
    assert!(grid.is_blocked(0.0, 0.0));

    grid.set_blocked_cell(cx, cy, false);
    assert!(!grid.is_blocked(0.0, 0.0));
}

#[test]
fn test_is_blocked_cell_direct() {
    let mut grid = TerrainGrid::new();

    // Direct cell coordinate access
    assert!(!grid.is_blocked_cell(50, 50));

    grid.set_blocked_cell(50, 50, true);
    assert!(grid.is_blocked_cell(50, 50));

    // Adjacent cells unaffected
    assert!(!grid.is_blocked_cell(51, 50));
    assert!(!grid.is_blocked_cell(50, 51));
}

#[test]
fn test_is_blocked_cell_out_of_bounds() {
    let grid = TerrainGrid::new();

    // Out of bounds cells should be treated as blocked (safety)
    assert!(grid.is_blocked_cell(300, 0)); // x out of bounds
    assert!(grid.is_blocked_cell(0, 300)); // y out of bounds
    assert!(grid.is_blocked_cell(250, 250)); // Both at boundary
}

#[test]
fn test_is_blocked_cell_signed() {
    let grid = TerrainGrid::new();

    // Negative cell coords (from signed conversion) should be blocked
    assert!(grid.is_blocked_cell_signed(-1, 0));
    assert!(grid.is_blocked_cell_signed(0, -1));
    assert!(grid.is_blocked_cell_signed(-5, -5));

    // Valid signed coords should work
    assert!(!grid.is_blocked_cell_signed(100, 100));
}

// =============================================================================
// TerrainGrid Dimension Tests
// =============================================================================

#[test]
fn test_grid_dimensions() {
    let grid = TerrainGrid::new();
    assert_eq!(grid.cells_per_axis(), 250);
    assert_eq!(grid.cell_size(), TERRAIN_CELL_SIZE);
}

#[test]
fn test_grid_total_cells() {
    let grid = TerrainGrid::new();
    // 250 * 250 = 62,500 cells
    assert_eq!(grid.total_cells(), 62_500);
}

// =============================================================================
// Edge Cases
// =============================================================================

#[test]
fn test_cell_boundary_precision() {
    let grid = TerrainGrid::new();

    // Test points exactly on cell boundaries
    // Cell size is 20m, so boundary at x = -2500 + 20 = -2480
    let boundary_x = -2500.0 + 20.0;

    let (cx1, _) = grid.world_to_cell(boundary_x - 0.001, 0.0);
    let (cx2, _) = grid.world_to_cell(boundary_x + 0.001, 0.0);

    // Should be adjacent cells
    assert_eq!(cx2, cx1 + 1);
}

#[test]
fn test_multiple_blocked_cells() {
    let mut grid = TerrainGrid::new();

    // Block a row of cells
    for i in 0..10 {
        grid.set_blocked_cell(100 + i, 100, true);
    }

    // Verify the row is blocked
    for i in 0..10 {
        assert!(grid.is_blocked_cell(100 + i, 100));
    }

    // Cells above and below should be open
    for i in 0..10 {
        assert!(!grid.is_blocked_cell(100 + i, 99));
        assert!(!grid.is_blocked_cell(100 + i, 101));
    }
}
