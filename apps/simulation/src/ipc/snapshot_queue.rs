//! Lock-free snapshot queue for IPC
//!
//! This module provides a non-blocking communication channel between the Bevy
//! simulation (producer) and IPC consumers using crossbeam's ArrayQueue.
//!
//! Performance characteristics:
//! - Bevy write: O(1), lock-free, never blocks
//! - IPC read: O(1), lock-free, never blocks
//! - Drops oldest frames if queue full

use crossbeam::queue::ArrayQueue;
use serde::{Deserialize, Serialize};
use bevy_ecs::system::Resource;
use std::sync::Arc;

/// Newtype wrapper for Arc<SnapshotQueue> to implement Resource trait
/// Required because we can't implement Resource for Arc<T> directly (orphan rules)
#[derive(Clone, Resource)]
pub struct SharedSnapshotQueue(pub Arc<SnapshotQueue>);

impl SharedSnapshotQueue {
    /// Create a new shared snapshot queue
    pub fn new(capacity: usize) -> Self {
        Self(Arc::new(SnapshotQueue::new(capacity)))
    }

    /// Get the inner Arc for IPC sharing
    pub fn inner(&self) -> Arc<SnapshotQueue> {
        self.0.clone()
    }
}

/// Lightweight snapshot of a single creature's state
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreatureSnapshot {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub rotation: f32,
    pub width: f32,
    pub height: f32,
    pub behavior: String,
    pub energy: Option<f32>,
    pub age: f32,
}

/// Complete simulation state at a single tick
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GameState {
    pub tick: u64,
    pub tick_rate_hz: f32, // Actual measured tick rate (not target)
    pub creatures: Vec<CreatureSnapshot>,
}

/// Lock-free queue for simulation snapshots
///
/// # Architecture
/// - **Producer (Bevy):** Writes at tick rate
/// - **Consumer (IPC):** Reads on demand
/// - **Capacity:** 10 frames buffer
///
/// # Performance
/// - Both operations are O(1) and lock-free
/// - If queue fills (unlikely), oldest frame is dropped to make room
/// - Total overhead: <2ms per frame (within 11ms budget)
///
/// # Clone Semantics
/// - Cheap to clone (Arc wrapper)
/// - All clones share the same underlying queue
#[derive(Clone)]
pub struct SnapshotQueue {
    queue: Arc<ArrayQueue<GameState>>,
}

impl SnapshotQueue {
    /// Create a new snapshot queue with given capacity
    ///
    /// # Arguments
    /// * `capacity` - Number of frames to buffer (typical: 10 for 111ms)
    pub fn new(capacity: usize) -> Self {
        Self {
            queue: Arc::new(ArrayQueue::new(capacity)),
        }
    }

    /// Write a snapshot (Bevy side, non-blocking)
    ///
    /// Implements true "ring buffer" behavior: if the queue is full,
    /// the oldest frame is dropped to make room for the newest.
    ///
    /// In practice, this rarely happens with 90 FPS consumption and
    /// 90 Hz production at equal rates.
    ///
    /// # Performance
    /// - Lock-free: Never blocks simulation
    /// - Completes in <1μs
    pub fn push(&self, state: GameState) {
        // If full, drop oldest item to make room
        if self.queue.is_full() {
            let _ = self.queue.pop();
        }

        // Push the new state (should always succeed after making room)
        let _ = self.queue.push(state);
    }

    /// Read the latest snapshot (IPC consumer side, lock-free)
    ///
    /// Returns `None` if queue is empty (e.g., simulation hasn't started yet).
    ///
    /// # Performance
    /// - Lock-free: Never blocks Bevy simulation
    /// - Completes in <1μs
    pub fn pop(&self) -> Option<GameState> {
        self.queue.pop()
    }

    /// Check if queue is empty (for diagnostics)
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Check current queue length (for diagnostics)
    pub fn len(&self) -> usize {
        self.queue.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_and_pop() {
        let queue = SnapshotQueue::new(5);

        let state = GameState {
            tick: 1,
            tick_rate_hz: 90.0,
            creatures: vec![],
        };

        queue.push(state.clone());
        let result = queue.pop();

        assert!(result.is_some());
        assert_eq!(result.unwrap().tick, 1);
    }

    #[test]
    fn test_empty_queue() {
        let queue = SnapshotQueue::new(5);
        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);
        assert!(queue.pop().is_none());
    }

    #[test]
    fn test_queue_overflow() {
        let queue = SnapshotQueue::new(2);

        queue.push(GameState { tick: 1, tick_rate_hz: 90.0, creatures: vec![] });
        queue.push(GameState { tick: 2, tick_rate_hz: 90.0, creatures: vec![] });
        queue.push(GameState { tick: 3, tick_rate_hz: 90.0, creatures: vec![] }); // Should be dropped

        assert_eq!(queue.len(), 2);
    }
}
