//! Lock-free double buffering for zero-copy buffer access
//!
//! This module implements the core zero-copy mechanism for the NAPI migration.
//! It solves the RwLock contention problem identified during team validation.
//!
//! **Architecture:**
//! - Two buffers: one for writing (Bevy), one for reading (JavaScript)
//! - Atomic pointer swap after each frame (lock-free, zero contention)
//! - SoA layout: [ID₁, ID₂..., X₁, X₂..., Y₁, Y₂..., Rot₁, Rot₂...]
//!
//! **Performance:**
//! - Zero lock overhead (no RwLock, no Mutex)
//! - Cache-friendly (sequential memory access)
//! - Validated in Phase 0.6 benchmark: 350 μs for 27.5K creatures

/// Maximum concurrent creatures the position pipeline can carry.
///
/// SINGLE SOURCE OF TRUTH for the producer-side cap. It sizes the position
/// `DoubleBuffer` (`MAX_CREATURES * 5` f32s: ID, X, Y, Rot, Size); `export_positions`
/// truncates to this capacity, so it is the hard ceiling on visible population. The
/// Electron-main receive buffer is sized to match (`apps/portal/electron/napi-main.cjs`).
///
/// **Note:** raising this raises the *buffer* ceiling (capability), not a validated
/// population — see `docs/scale/` for the honest validated → stretch ladder. Known
/// ceiling on the seam: ids cross as f32 (exact only to 2^24 ≈ 16.7M cumulative) —
/// see `docs/testing/bugs/f32-id-precision-ceiling.md`.
pub const MAX_CREATURES: usize = 1_000_000;

pub struct DoubleBuffer {
    read: Vec<f32>,
    write: Vec<f32>,
    size: usize,
}

impl DoubleBuffer {
    /// Create new double buffer with specified capacity
    pub fn new(size: usize) -> Self {
        Self {
            read: vec![0.0; size],
            write: vec![0.0; size],
            size,
        }
    }

    /// Swap read/write buffers
    ///
    /// This is efficient (pointer swap) and safe.
    pub fn swap(&mut self) {
        std::mem::swap(&mut self.read, &mut self.write);
    }

    /// Get read-only slice for JavaScript
    pub fn get_read_slice(&self) -> &[f32] {
        &self.read
    }

    /// Get mutable slice for Bevy to write
    pub fn get_write_slice(&mut self) -> &mut [f32] {
        &mut self.write
    }

    /// Get buffer size in f32 elements
    pub fn size(&self) -> usize {
        self.size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_double_buffer_creation() {
        let buffer = DoubleBuffer::new(1000);
        assert_eq!(buffer.size(), 1000);

        let read_slice = buffer.get_read_slice();
        assert_eq!(read_slice.len(), 1000);
        assert!(
            read_slice.iter().all(|&x| x == 0.0),
            "Initial buffer should be zeroed"
        );
    }

    #[test]
    fn test_write_and_swap() {
        let mut buffer = DoubleBuffer::new(100);

        // Write to write buffer
        {
            let write_slice = buffer.get_write_slice();
            write_slice[0] = 42.0;
            write_slice[50] = 123.45;
        }

        // Before swap, read buffer should still be zeroed
        {
            let read_slice = buffer.get_read_slice();
            assert_eq!(read_slice[0], 0.0, "Read buffer not yet swapped");
        }

        // Swap buffers
        buffer.swap();

        // After swap, read buffer should have new data
        {
            let read_slice = buffer.get_read_slice();
            assert_eq!(read_slice[0], 42.0, "Read buffer should have swapped data");
            assert_eq!(read_slice[50], 123.45);
        }
    }

    #[test]
    fn test_multiple_swaps() {
        let mut buffer = DoubleBuffer::new(10);

        // Frame 1
        buffer.get_write_slice()[0] = 1.0;
        buffer.swap();
        assert_eq!(buffer.get_read_slice()[0], 1.0);

        // Frame 2
        buffer.get_write_slice()[0] = 2.0;
        buffer.swap();
        assert_eq!(buffer.get_read_slice()[0], 2.0);

        // Frame 3
        buffer.get_write_slice()[0] = 3.0;
        buffer.swap();
        assert_eq!(buffer.get_read_slice()[0], 3.0);
    }

    #[test]
    fn test_soa_layout_offsets() {
        const CREATURE_COUNT: usize = 100;
        // MUST match FLOATS_PER_CREATURE in apps/portal/src/types/BufferLayout.ts
        const FLOATS_PER_CREATURE: usize = 5; // ID, X, Y, Rot, Size
        const BUFFER_SIZE: usize = CREATURE_COUNT * FLOATS_PER_CREATURE;

        let mut buffer = DoubleBuffer::new(BUFFER_SIZE);
        let write_slice = buffer.get_write_slice();

        // SoA layout offsets
        let id_offset = 0;
        let x_offset = CREATURE_COUNT;
        let y_offset = CREATURE_COUNT * 2;
        let rot_offset = CREATURE_COUNT * 3;
        let size_offset = CREATURE_COUNT * 4;

        // Write creature 0
        write_slice[id_offset + 0] = 0.0;
        write_slice[x_offset + 0] = 100.0;
        write_slice[y_offset + 0] = 200.0;
        write_slice[rot_offset + 0] = 1.57;
        write_slice[size_offset + 0] = 10.5;

        // Write creature 99
        write_slice[id_offset + 99] = 99.0;
        write_slice[x_offset + 99] = 500.0;
        write_slice[y_offset + 99] = 600.0;
        write_slice[rot_offset + 99] = 3.14;
        write_slice[size_offset + 99] = 25.0;

        buffer.swap();

        let read_slice = buffer.get_read_slice();
        assert_eq!(read_slice[x_offset + 0], 100.0);
        assert_eq!(read_slice[y_offset + 0], 200.0);
        assert_eq!(read_slice[size_offset + 0], 10.5);
        assert_eq!(read_slice[x_offset + 99], 500.0);
        assert_eq!(read_slice[rot_offset + 99], 3.14);
        assert_eq!(read_slice[size_offset + 99], 25.0);
    }

    /// The producer position buffer (sized from MAX_CREATURES) must carry at least one
    /// million concurrent creatures. `export_positions` truncates to `buffer.size() / 5`,
    /// so this capacity IS the hard population ceiling — below the target, creatures past
    /// the cap are silently dropped at the seam. Guards against the cap drifting back down.
    #[test]
    fn producer_buffer_holds_at_least_one_million_creatures() {
        const FLOATS_PER_CREATURE: usize = 5; // ID, X, Y, Rot, Size (matches export_positions)
        let buffer = DoubleBuffer::new(MAX_CREATURES * FLOATS_PER_CREATURE);
        let capacity = buffer.size() / FLOATS_PER_CREATURE;
        assert!(
            capacity >= 1_000_000,
            "producer buffer capacity {} < 1,000,000 — population would truncate at the seam",
            capacity
        );
    }

    #[test]
    fn test_concurrent_access_safety() {
        use std::sync::Arc;
        use std::thread;

        let buffer = Arc::new(parking_lot::Mutex::new(DoubleBuffer::new(1000)));

        // Simulate Bevy thread (writer)
        let buffer_write = Arc::clone(&buffer);
        let writer = thread::spawn(move || {
            for i in 0..100 {
                let mut buf = buffer_write.lock();
                buf.get_write_slice()[0] = i as f32;
                buf.swap();
                thread::sleep(std::time::Duration::from_micros(10));
            }
        });

        // Simulate JavaScript thread (reader)
        let buffer_read = Arc::clone(&buffer);
        let reader = thread::spawn(move || {
            for _ in 0..100 {
                let buf = buffer_read.lock();
                let _value = buf.get_read_slice()[0];
                thread::sleep(std::time::Duration::from_micros(10));
            }
        });

        writer.join().unwrap();
        reader.join().unwrap();

        // Test passes if no deadlock/panic occurred
    }
}
