
use crossbeam::queue::ArrayQueue;
use serde::{Deserialize, Serialize};
use bevy_ecs::system::Resource;
use std::sync::Arc;
#[cfg(feature = "dev-tools")]
use crate::instrumentation::SystemTimingsSnapshot;

#[derive(Clone, Resource)]
pub struct SharedSnapshotQueue(pub Arc<SnapshotQueue>);

impl SharedSnapshotQueue {
    pub fn new(capacity: usize) -> Self {
        Self(Arc::new(SnapshotQueue::new(capacity)))
    }

    pub fn inner(&self) -> Arc<SnapshotQueue> {
        self.0.clone()
    }
}

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

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GameState {
    pub tick: u64,
    pub tick_rate_hz: f32,
    pub creatures: Vec<CreatureSnapshot>,
    #[cfg(feature = "dev-tools")]
    pub entity_count: usize,
    #[cfg(feature = "dev-tools")]
    pub system_timings_us: SystemTimingsSnapshot,
}

#[derive(Clone)]
pub struct SnapshotQueue {
    queue: Arc<ArrayQueue<GameState>>,
}

impl SnapshotQueue {
    pub fn new(capacity: usize) -> Self {
        Self {
            queue: Arc::new(ArrayQueue::new(capacity)),
        }
    }

    pub fn push(&self, state: GameState) {

        if self.queue.is_full() {
            let _ = self.queue.pop();
        }


        let _ = self.queue.push(state);
    }

    pub fn pop(&self) -> Option<GameState> {
        self.queue.pop()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

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
            entity_count: 0,
            system_timings_us: Default::default(),
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

        queue.push(GameState { tick: 1, tick_rate_hz: 90.0, creatures: vec![], entity_count: 0, system_timings_us: Default::default() });
        queue.push(GameState { tick: 2, tick_rate_hz: 90.0, creatures: vec![], entity_count: 0, system_timings_us: Default::default() });
        queue.push(GameState { tick: 3, tick_rate_hz: 90.0, creatures: vec![], entity_count: 0, system_timings_us: Default::default() });

        assert_eq!(queue.len(), 2);
    }
}
