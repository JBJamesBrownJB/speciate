use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use bevy_ecs::system::Resource;
use serde::{Deserialize, Serialize};

#[derive(Resource)]
pub struct SystemTimings {
    pub total_tick_us: AtomicU64,
    pub movement_us: AtomicU64,
    pub perception_us: AtomicU64,
    pub behavior_us: AtomicU64,
    pub behavior_transition_us: AtomicU64,
    pub wander_us: AtomicU64,
    pub flee_us: AtomicU64,
    pub avoidance_us: AtomicU64,
    pub rotation_us: AtomicU64,
}

impl SystemTimings {
    pub fn new() -> Self {
        Self {
            total_tick_us: AtomicU64::new(0),
            movement_us: AtomicU64::new(0),
            perception_us: AtomicU64::new(0),
            behavior_us: AtomicU64::new(0),
            behavior_transition_us: AtomicU64::new(0),
            wander_us: AtomicU64::new(0),
            flee_us: AtomicU64::new(0),
            avoidance_us: AtomicU64::new(0),
            rotation_us: AtomicU64::new(0),
        }
    }

    pub fn time(&self, name: &str) -> TimingGuard<'_> {
        let target = match name {
            "total_tick" => &self.total_tick_us,
            "movement" => &self.movement_us,
            "perception" => &self.perception_us,
            "behavior" => &self.behavior_us,
            "behavior_transition" => &self.behavior_transition_us,
            "wander" => &self.wander_us,
            "flee" => &self.flee_us,
            "avoidance" => &self.avoidance_us,
            "rotation" => &self.rotation_us,
            _ => panic!("Unknown system: {}", name),
        };
        TimingGuard::new(target)
    }

    pub fn snapshot(&self) -> SystemTimingsSnapshot {
        SystemTimingsSnapshot {
            total_tick_us: self.total_tick_us.load(Ordering::Relaxed),
            movement_us: self.movement_us.load(Ordering::Relaxed),
            perception_us: self.perception_us.load(Ordering::Relaxed),
            behavior_us: self.behavior_us.load(Ordering::Relaxed),
            behavior_transition_us: self.behavior_transition_us.load(Ordering::Relaxed),
            wander_us: self.wander_us.load(Ordering::Relaxed),
            flee_us: self.flee_us.load(Ordering::Relaxed),
            avoidance_us: self.avoidance_us.load(Ordering::Relaxed),
            rotation_us: self.rotation_us.load(Ordering::Relaxed),
        }
    }
}

impl Default for SystemTimings {
    fn default() -> Self {
        Self::new()
    }
}

pub struct TimingGuard<'a> {
    start: Instant,
    target: &'a AtomicU64,
}

impl<'a> TimingGuard<'a> {
    pub fn new(target: &'a AtomicU64) -> Self {
        Self {
            start: Instant::now(),
            target,
        }
    }
}

impl Drop for TimingGuard<'_> {
    fn drop(&mut self) {
        let elapsed = self.start.elapsed().as_micros() as u64;
        self.target.store(elapsed, Ordering::Relaxed);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SystemTimingsSnapshot {
    pub total_tick_us: u64,
    pub movement_us: u64,
    pub perception_us: u64,
    pub behavior_us: u64,
    pub behavior_transition_us: u64,
    pub wander_us: u64,
    pub flee_us: u64,
    pub avoidance_us: u64,
    pub rotation_us: u64,
}
