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
//! - [5]: neighbor_count
//! - [6..6+MAX]: neighbor_ids
//! - [6+MAX..6+2*MAX]: neighbor_xs
//! - [6+2*MAX..6+3*MAX]: neighbor_ys

#![cfg(feature = "dev-tools")]

pub const MAX_DEBUG_NEIGHBORS: usize = 64;
pub const HEADER_SIZE: usize = 6;
pub const BUFFER_SIZE: usize = HEADER_SIZE + MAX_DEBUG_NEIGHBORS * 3;

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

    pub fn write_debug_data(
        &mut self,
        target_id: u32,
        target_x: f32,
        target_y: f32,
        perception_range: f32,
        neighbors: &[(u32, f32, f32)], // (id, x, y)
    ) {
        let neighbor_count = neighbors.len().min(MAX_DEBUG_NEIGHBORS);

        self.write[0] = 1.0; // has_data = true
        self.write[1] = target_id as f32;
        self.write[2] = target_x;
        self.write[3] = target_y;
        self.write[4] = perception_range;
        self.write[5] = neighbor_count as f32;

        let id_offset = HEADER_SIZE;
        let x_offset = HEADER_SIZE + MAX_DEBUG_NEIGHBORS;
        let y_offset = HEADER_SIZE + MAX_DEBUG_NEIGHBORS * 2;

        for (i, (id, x, y)) in neighbors.iter().take(neighbor_count).enumerate() {
            self.write[id_offset + i] = *id as f32;
            self.write[x_offset + i] = *x;
            self.write[y_offset + i] = *y;
        }
    }

    pub fn has_data(&self) -> bool {
        self.read[0] > 0.5
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

        let neighbors = vec![(42, 10.0, 20.0), (43, 30.0, 40.0)];

        buffer.write_debug_data(1, 100.0, 200.0, 50.0, &neighbors);

        assert!(!buffer.has_data());

        buffer.swap();

        assert!(buffer.has_data());
        let slice = buffer.get_read_slice();
        assert_eq!(slice[1], 1.0); // target_id
        assert_eq!(slice[2], 100.0); // target_x
        assert_eq!(slice[3], 200.0); // target_y
        assert_eq!(slice[4], 50.0); // perception_range
        assert_eq!(slice[5], 2.0); // neighbor_count
        assert_eq!(slice[HEADER_SIZE], 42.0); // first neighbor id
        assert_eq!(slice[HEADER_SIZE + MAX_DEBUG_NEIGHBORS], 10.0); // first neighbor x
    }

    #[test]
    fn test_clear_write() {
        let mut buffer = PerceptionDebugBuffer::new();

        buffer.write_debug_data(1, 100.0, 200.0, 50.0, &[]);
        buffer.swap();
        assert!(buffer.has_data());

        buffer.clear_write();
        buffer.swap();
        assert!(!buffer.has_data());
    }

    #[test]
    fn test_max_neighbors_clamped() {
        let mut buffer = PerceptionDebugBuffer::new();

        let neighbors: Vec<_> = (0..100).map(|i| (i, i as f32, i as f32)).collect();

        buffer.write_debug_data(1, 0.0, 0.0, 50.0, &neighbors);
        buffer.swap();

        let slice = buffer.get_read_slice();
        assert_eq!(slice[5], MAX_DEBUG_NEIGHBORS as f32);
    }
}
