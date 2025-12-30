//! Zero-copy buffer for perception debug visualization (dev-tools only)
//!
//! Provides per-tick perception data for the selected creature without
//! impacting telemetry performance.
//!
//! **Layout (f32 array):**
//! - [0]: has_data (1.0 = valid, 0.0 = no selection)
//! - [1]: target_id
//! - [2]: target_x
//! - [3]: target_y
//! - [4]: perception_range
//! - [5]: query_radius (actual radius used for cell queries)
//! - [6]: fov_angle (radians)
//! - [7]: rotation (radians)
//! - [8]: ax (acceleration x component)
//! - [9]: ay (acceleration y component)
//! - [10]: neighbor_count
//! - [11..11+MAX_NEIGHBORS]: neighbor_ids
//! - [11+MAX_NEIGHBORS..11+2*MAX_NEIGHBORS]: neighbor_xs
//! - [11+2*MAX_NEIGHBORS..11+3*MAX_NEIGHBORS]: neighbor_ys
//!
//! **Grid cell section (starting at CELL_SECTION_OFFSET):**
//! - [203]: cell_size (world units per cell)
//! - [204]: num_queried_cells
//! - [205]: creature_cell_x
//! - [206]: creature_cell_y
//! - [207..207+MAX_CELLS*2]: queried cells (x, y pairs interleaved)
//!
//! **Checked cells section (starting at CHECKED_CELL_SECTION_OFFSET):**
//! - [407]: num_checked_cells
//! - [408..408+MAX_CELLS*2]: checked cells (x, y pairs interleaved)

#![cfg(feature = "dev-tools")]

pub const MAX_DEBUG_NEIGHBORS: usize = 64;
pub const HEADER_SIZE: usize = 11;
pub const NEIGHBOR_SECTION_SIZE: usize = MAX_DEBUG_NEIGHBORS * 3;

pub const CELL_SECTION_OFFSET: usize = HEADER_SIZE + NEIGHBOR_SECTION_SIZE; // 203
pub const CELL_HEADER_SIZE: usize = 4; // cell_size, num_cells, creature_x, creature_y
pub const MAX_QUERIED_CELLS: usize = 100; // Up to 10x10 grid query

pub const CHECKED_CELL_SECTION_OFFSET: usize =
    CELL_SECTION_OFFSET + CELL_HEADER_SIZE + MAX_QUERIED_CELLS * 2; // 407
pub const CHECKED_CELL_HEADER_SIZE: usize = 1; // num_checked_cells
pub const MAX_CHECKED_CELLS: usize = 100;

// L1 Vision section (after checked cells)
// Layout: [count, entries...] where each entry is 6 floats:
//   cell_idx, classification, center_x, center_y, direction_x, direction_y
pub const L1_VISION_SECTION_OFFSET: usize =
    CHECKED_CELL_SECTION_OFFSET + CHECKED_CELL_HEADER_SIZE + MAX_CHECKED_CELLS * 2; // 608
pub const L1_VISION_HEADER_SIZE: usize = 1; // num_l1_vision_entries
pub const MAX_L1_VISION_ENTRIES: usize = 48;
pub const L1_VISION_ENTRY_SIZE: usize = 6; // cell_idx, classification, center_x, center_y, dir_x, dir_y

pub const BUFFER_SIZE: usize =
    L1_VISION_SECTION_OFFSET + L1_VISION_HEADER_SIZE + MAX_L1_VISION_ENTRIES * L1_VISION_ENTRY_SIZE;

pub trait NeighborFields {
    fn id(&self) -> u32;
    fn x(&self) -> f32;
    fn y(&self) -> f32;
}

impl NeighborFields for (u32, f32, f32) {
    fn id(&self) -> u32 {
        self.0
    }
    fn x(&self) -> f32 {
        self.1
    }
    fn y(&self) -> f32 {
        self.2
    }
}

pub trait CellFields {
    fn x(&self) -> i32;
    fn y(&self) -> i32;
}

impl CellFields for (i32, i32) {
    fn x(&self) -> i32 {
        self.0
    }
    fn y(&self) -> i32 {
        self.1
    }
}

use crate::simulation::perception::{NeighborDebugInfo, QueriedCell};

impl NeighborFields for &NeighborDebugInfo {
    fn id(&self) -> u32 {
        self.id
    }
    fn x(&self) -> f32 {
        self.x
    }
    fn y(&self) -> f32 {
        self.y
    }
}

impl CellFields for &QueriedCell {
    fn x(&self) -> i32 {
        self.x
    }
    fn y(&self) -> i32 {
        self.y
    }
}

pub struct PerceptionDebugBuffer {
    read: [f32; BUFFER_SIZE],
    write: [f32; BUFFER_SIZE],
}

impl Default for PerceptionDebugBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl PerceptionDebugBuffer {
    pub fn new() -> Self {
        Self {
            read: [0.0; BUFFER_SIZE],
            write: [0.0; BUFFER_SIZE],
        }
    }

    pub fn swap(&mut self) {
        std::mem::swap(&mut self.read, &mut self.write);
    }

    pub fn get_read_slice(&self) -> &[f32] {
        &self.read
    }

    pub fn clear_write(&mut self) {
        self.write[0] = 0.0; // has_data = false
    }

    pub fn write_debug_data<N, F>(
        &mut self,
        target_id: u32,
        target_x: f32,
        target_y: f32,
        perception_range: f32,
        query_radius: f32,
        fov_angle: f32,
        rotation: f32,
        ax: f32,
        ay: f32,
        neighbors: N,
    ) where
        N: ExactSizeIterator<Item = F>,
        F: NeighborFields,
    {
        let neighbor_count = neighbors.len().min(MAX_DEBUG_NEIGHBORS);

        self.write[0] = 1.0; // has_data = true
        self.write[1] = target_id as f32;
        self.write[2] = target_x;
        self.write[3] = target_y;
        self.write[4] = perception_range;
        self.write[5] = query_radius;
        self.write[6] = fov_angle;
        self.write[7] = rotation;
        self.write[8] = ax;
        self.write[9] = ay;
        self.write[10] = neighbor_count as f32;

        let id_offset = HEADER_SIZE;
        let x_offset = HEADER_SIZE + MAX_DEBUG_NEIGHBORS;
        let y_offset = HEADER_SIZE + MAX_DEBUG_NEIGHBORS * 2;

        for (i, neighbor) in neighbors.take(neighbor_count).enumerate() {
            self.write[id_offset + i] = neighbor.id() as f32;
            self.write[x_offset + i] = neighbor.x();
            self.write[y_offset + i] = neighbor.y();
        }
    }

    pub fn write_cell_data<Q, C, QF, CF>(
        &mut self,
        cell_size: f32,
        creature_cell: (i32, i32),
        queried_cells: Q,
        checked_cells: C,
    ) where
        Q: ExactSizeIterator<Item = QF>,
        C: ExactSizeIterator<Item = CF>,
        QF: CellFields,
        CF: CellFields,
    {
        // Write queried cells section
        let queried_count = queried_cells.len().min(MAX_QUERIED_CELLS);

        self.write[CELL_SECTION_OFFSET] = cell_size;
        self.write[CELL_SECTION_OFFSET + 1] = queried_count as f32;
        self.write[CELL_SECTION_OFFSET + 2] = creature_cell.0 as f32;
        self.write[CELL_SECTION_OFFSET + 3] = creature_cell.1 as f32;

        let queried_offset = CELL_SECTION_OFFSET + CELL_HEADER_SIZE;
        for (i, cell) in queried_cells.take(queried_count).enumerate() {
            self.write[queried_offset + i * 2] = cell.x() as f32;
            self.write[queried_offset + i * 2 + 1] = cell.y() as f32;
        }

        // Write checked cells section
        let checked_count = checked_cells.len().min(MAX_CHECKED_CELLS);
        self.write[CHECKED_CELL_SECTION_OFFSET] = checked_count as f32;

        let checked_offset = CHECKED_CELL_SECTION_OFFSET + CHECKED_CELL_HEADER_SIZE;
        for (i, cell) in checked_cells.take(checked_count).enumerate() {
            self.write[checked_offset + i * 2] = cell.x() as f32;
            self.write[checked_offset + i * 2 + 1] = cell.y() as f32;
        }
    }

    pub fn has_data(&self) -> bool {
        self.read[0] > 0.5
    }

    pub fn write_l1_vision_data<I>(&mut self, entries: I)
    where
        I: ExactSizeIterator<Item = (u32, u8, f32, f32, f32, f32)>,
    {
        let count = entries.len().min(MAX_L1_VISION_ENTRIES);
        self.write[L1_VISION_SECTION_OFFSET] = count as f32;

        let data_offset = L1_VISION_SECTION_OFFSET + L1_VISION_HEADER_SIZE;
        for (i, (cell_idx, classification, center_x, center_y, dir_x, dir_y)) in
            entries.take(count).enumerate()
        {
            let entry_offset = data_offset + i * L1_VISION_ENTRY_SIZE;
            self.write[entry_offset] = cell_idx as f32;
            self.write[entry_offset + 1] = classification as f32;
            self.write[entry_offset + 2] = center_x;
            self.write[entry_offset + 3] = center_y;
            self.write[entry_offset + 4] = dir_x;
            self.write[entry_offset + 5] = dir_y;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_creation() {
        let buffer = PerceptionDebugBuffer::new();
        assert!(!buffer.has_data());
        assert_eq!(buffer.get_read_slice().len(), BUFFER_SIZE);
    }

    #[test]
    fn test_write_and_swap() {
        let mut buffer = PerceptionDebugBuffer::new();
        let pi = std::f32::consts::PI;

        let neighbors: Vec<(u32, f32, f32)> = vec![(42, 10.0, 20.0), (43, 30.0, 40.0)];

        buffer.write_debug_data(
            1,
            100.0,
            200.0,
            50.0,
            55.0,
            pi,
            0.5,
            1.5,
            -2.5,
            neighbors.iter().copied(),
        );

        assert!(!buffer.has_data());

        buffer.swap();

        assert!(buffer.has_data());
        let slice = buffer.get_read_slice();
        assert_eq!(slice[1], 1.0); // target_id
        assert_eq!(slice[2], 100.0); // target_x
        assert_eq!(slice[3], 200.0); // target_y
        assert_eq!(slice[4], 50.0); // perception_range
        assert_eq!(slice[5], 55.0); // query_radius
        assert_eq!(slice[6], pi); // fov_angle
        assert_eq!(slice[7], 0.5); // rotation
        assert_eq!(slice[8], 1.5); // ax
        assert_eq!(slice[9], -2.5); // ay
        assert_eq!(slice[10], 2.0); // neighbor_count
        assert_eq!(slice[HEADER_SIZE], 42.0); // first neighbor id
        assert_eq!(slice[HEADER_SIZE + MAX_DEBUG_NEIGHBORS], 10.0); // first neighbor x
    }

    #[test]
    fn test_clear_write() {
        let mut buffer = PerceptionDebugBuffer::new();
        let pi = std::f32::consts::PI;

        buffer.write_debug_data(
            1,
            100.0,
            200.0,
            50.0,
            55.0,
            pi,
            0.0,
            0.0,
            0.0,
            std::iter::empty::<(u32, f32, f32)>(),
        );
        buffer.swap();
        assert!(buffer.has_data());

        buffer.clear_write();
        buffer.swap();
        assert!(!buffer.has_data());
    }

    #[test]
    fn test_max_neighbors_clamped() {
        let mut buffer = PerceptionDebugBuffer::new();
        let pi = std::f32::consts::PI;

        let neighbors: Vec<(u32, f32, f32)> = (0..100).map(|i| (i, i as f32, i as f32)).collect();

        buffer.write_debug_data(
            1,
            0.0,
            0.0,
            50.0,
            55.0,
            pi,
            0.0,
            0.0,
            0.0,
            neighbors.iter().copied(),
        );
        buffer.swap();

        let slice = buffer.get_read_slice();
        assert_eq!(slice[10], MAX_DEBUG_NEIGHBORS as f32); // neighbor_count at index 10 now
    }
}
