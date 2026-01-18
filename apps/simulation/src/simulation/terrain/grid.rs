use bevy_ecs::prelude::*;

use crate::simulation::core::world_bounds::MAX_WORLD_SIZE;
use crate::simulation::spatial::CELL_SIZE;

const CELLS_PER_AXIS: u32 = (MAX_WORLD_SIZE / CELL_SIZE) as u32; // 250
const TOTAL_CELLS: usize = (CELLS_PER_AXIS * CELLS_PER_AXIS) as usize; // 62,500
const BITMAP_SIZE: usize = (TOTAL_CELLS + 63) / 64; // 977 u64s

#[derive(Resource)]
pub struct TerrainGrid {
    blocked: [u64; BITMAP_SIZE],
}

impl TerrainGrid {
    pub fn new() -> Self {
        Self {
            blocked: [0; BITMAP_SIZE],
        }
    }

    pub fn cells_per_axis(&self) -> u32 {
        CELLS_PER_AXIS
    }

    pub fn cell_size(&self) -> f32 {
        CELL_SIZE
    }

    pub fn total_cells(&self) -> usize {
        TOTAL_CELLS
    }

    pub fn world_to_cell(&self, world_x: f32, world_y: f32) -> (u32, u32) {
        let half_world = MAX_WORLD_SIZE / 2.0;

        // Shift from [-half, +half] to [0, MAX_WORLD_SIZE]
        let shifted_x = world_x + half_world;
        let shifted_y = world_y + half_world;

        // Convert to cell coordinates
        let cell_x = (shifted_x / CELL_SIZE) as i32;
        let cell_y = (shifted_y / CELL_SIZE) as i32;

        // Clamp to valid range
        let cell_x = cell_x.clamp(0, CELLS_PER_AXIS as i32 - 1) as u32;
        let cell_y = cell_y.clamp(0, CELLS_PER_AXIS as i32 - 1) as u32;

        (cell_x, cell_y)
    }

    pub fn cell_to_world_center(&self, cell_x: u32, cell_y: u32) -> (f32, f32) {
        let half_world = MAX_WORLD_SIZE / 2.0;
        let half_cell = CELL_SIZE / 2.0;

        let world_x = (cell_x as f32 * CELL_SIZE) + half_cell - half_world;
        let world_y = (cell_y as f32 * CELL_SIZE) + half_cell - half_world;

        (world_x, world_y)
    }

    fn cell_index(&self, cell_x: u32, cell_y: u32) -> usize {
        (cell_y * CELLS_PER_AXIS + cell_x) as usize
    }

    pub fn is_blocked(&self, world_x: f32, world_y: f32) -> bool {
        let (cell_x, cell_y) = self.world_to_cell(world_x, world_y);
        self.is_blocked_cell(cell_x, cell_y)
    }

    pub fn is_blocked_cell(&self, cell_x: u32, cell_y: u32) -> bool {
        // Out of bounds = blocked (safety)
        if cell_x >= CELLS_PER_AXIS || cell_y >= CELLS_PER_AXIS {
            return true;
        }

        let idx = self.cell_index(cell_x, cell_y);
        let word_idx = idx / 64;
        let bit_idx = idx % 64;

        (self.blocked[word_idx] & (1u64 << bit_idx)) != 0
    }

    pub fn is_blocked_cell_signed(&self, cell_x: i32, cell_y: i32) -> bool {
        // Negative coords = blocked
        if cell_x < 0 || cell_y < 0 {
            return true;
        }
        self.is_blocked_cell(cell_x as u32, cell_y as u32)
    }

    pub fn set_blocked_cell(&mut self, cell_x: u32, cell_y: u32, blocked: bool) {
        if cell_x >= CELLS_PER_AXIS || cell_y >= CELLS_PER_AXIS {
            return; // Ignore out of bounds
        }

        let idx = self.cell_index(cell_x, cell_y);
        let word_idx = idx / 64;
        let bit_idx = idx % 64;

        if blocked {
            self.blocked[word_idx] |= 1u64 << bit_idx;
        } else {
            self.blocked[word_idx] &= !(1u64 << bit_idx);
        }
    }

    /// Get all blocked cell coordinates.
    /// Used for sending initial terrain state to frontend.
    pub fn get_all_blocked_cells(&self) -> Vec<(u32, u32)> {
        let mut cells = Vec::new();

        for word_idx in 0..BITMAP_SIZE {
            let word = self.blocked[word_idx];
            if word == 0 {
                continue; // Skip empty words
            }

            // Check each bit in this word
            for bit_idx in 0..64 {
                if (word & (1u64 << bit_idx)) != 0 {
                    let idx = word_idx * 64 + bit_idx;
                    if idx < TOTAL_CELLS {
                        let cell_x = (idx % CELLS_PER_AXIS as usize) as u32;
                        let cell_y = (idx / CELLS_PER_AXIS as usize) as u32;
                        cells.push((cell_x, cell_y));
                    }
                }
            }
        }

        cells
    }

    /// Count the number of blocked cells (useful for debugging/telemetry).
    pub fn blocked_count(&self) -> u32 {
        let mut count = 0u32;
        for word in &self.blocked {
            count += word.count_ones();
        }
        count
    }
}

impl Default for TerrainGrid {
    fn default() -> Self {
        Self::new()
    }
}
