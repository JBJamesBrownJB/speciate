use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use bevy_ecs::system::Resource;
use serde::{Deserialize, Serialize};

#[cfg(feature = "dev-tools")]
mod hardware_metrics;

#[cfg(feature = "dev-tools")]
mod snapshot;

#[cfg(feature = "dev-tools")]
mod parallelization;

#[cfg(feature = "dev-tools")]
pub use hardware_metrics::{HardwareMetrics, HardwareSnapshot, HardwareSnapshotResource};

#[cfg(feature = "dev-tools")]
pub use snapshot::{EcsMetrics, PerformanceSnapshot};

#[cfg(feature = "dev-tools")]
pub use parallelization::{ParallelizationMetrics, ParallelizationSnapshot};

#[cfg(not(feature = "dev-tools"))]
pub use hardware_metrics_stub::{HardwareMetrics, HardwareSnapshot};

#[cfg(not(feature = "dev-tools"))]
pub use parallelization_stub::{ParallelizationMetrics, ParallelizationSnapshot};

#[cfg(not(feature = "dev-tools"))]
mod hardware_metrics_stub {
    use serde::{Deserialize, Serialize};

    /// Stub HardwareSnapshot that matches the real structure exactly
    /// (used when dev-tools feature is disabled)
    #[derive(Debug, Clone, Serialize, Deserialize, Default)]
    #[serde(rename_all = "camelCase")]
    pub struct HardwareSnapshot {
        pub cycles_delta: u64,
        pub instructions_delta: u64,
        pub cache_refs_delta: u64,
        pub cache_misses_delta: u64,
        pub l1d_misses_delta: u64,
        pub l1i_misses_delta: u64,
        pub branch_instructions_delta: u64,
        pub branch_misses_delta: u64,
        pub stalled_frontend_delta: u64,
        pub stalled_backend_delta: u64,

        pub ipc: f64,
        pub l1d_miss_rate: f64,
        pub l1i_miss_rate: f64,
        pub llc_miss_rate: f64,
        pub branch_miss_rate: f64,
        pub frontend_stall_ratio: f64,
        pub backend_stall_ratio: f64,
    }

    pub struct HardwareMetrics;

    impl HardwareMetrics {
        pub fn new() -> Self {
            Self
        }

        pub fn read(&mut self) -> HardwareSnapshot {
            HardwareSnapshot::default()
        }
    }

    impl Default for HardwareMetrics {
        fn default() -> Self {
            Self::new()
        }
    }
}

#[cfg(not(feature = "dev-tools"))]
mod parallelization_stub {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize, Default)]
    #[serde(rename_all = "camelCase")]
    pub struct ParallelizationSnapshot {
        pub cpu_cores_total: usize,
        pub cpu_cores_active: usize,
        pub cpu_utilization_pct: f32,
        pub estimated_parallelism_factor: f32,
        pub concurrent_systems_estimate: usize,
        pub process_memory_bytes: u64,
    }

    pub struct ParallelizationMetrics;

    impl ParallelizationMetrics {
        pub fn new() -> Self {
            Self
        }

        pub fn read(&mut self) -> ParallelizationSnapshot {
            ParallelizationSnapshot::default()
        }
    }

    impl Default for ParallelizationMetrics {
        fn default() -> Self {
            Self::new()
        }
    }
}

#[derive(Resource)]
pub struct SystemTimings {
    pub total_tick_us: AtomicU64,
    pub movement_us: AtomicU64, // Now includes rotation (fused in Sprint 20)
    pub perception_us: AtomicU64,
    pub spatial_grid_rebuild_us: AtomicU64,
    pub l1_aggregation_us: AtomicU64, // L1 coarse grid aggregation (Phase A)
    pub behavior_transition_us: AtomicU64,
    pub steering_us: AtomicU64, // Fused steering system (Sprint 20)
    pub capture_debug_accel_us: AtomicU64,
    pub export_positions_us: AtomicU64, // IPC buffer export with parallel sort (Sprint 16)
    // Count metrics (not timings)
    pub cells_queried_total: AtomicU64, // Sum of L0 cells queried across all creatures this tick
}

impl SystemTimings {
    pub fn new() -> Self {
        Self {
            total_tick_us: AtomicU64::new(0),
            movement_us: AtomicU64::new(0),
            perception_us: AtomicU64::new(0),
            spatial_grid_rebuild_us: AtomicU64::new(0),
            l1_aggregation_us: AtomicU64::new(0),
            behavior_transition_us: AtomicU64::new(0),
            steering_us: AtomicU64::new(0),
            capture_debug_accel_us: AtomicU64::new(0),
            export_positions_us: AtomicU64::new(0),
            cells_queried_total: AtomicU64::new(0),
        }
    }

    pub fn time(&self, name: &str) -> TimingGuard<'_> {
        let target = match name {
            "total_tick" => &self.total_tick_us,
            "movement" => &self.movement_us,
            "perception" => &self.perception_us,
            "spatial_grid_rebuild" => &self.spatial_grid_rebuild_us,
            "l1_aggregation" => &self.l1_aggregation_us,
            "behavior_transition" => &self.behavior_transition_us,
            "steering" => &self.steering_us,
            "capture_debug_accel" => &self.capture_debug_accel_us,
            "export_positions" => &self.export_positions_us,
            _ => panic!("Unknown system: {}", name),
        };
        TimingGuard::new(target)
    }

    pub fn snapshot(&self) -> SystemTimingsSnapshot {
        SystemTimingsSnapshot {
            total_tick_us: self.total_tick_us.load(Ordering::Relaxed),
            movement_us: self.movement_us.load(Ordering::Relaxed),
            perception_us: self.perception_us.load(Ordering::Relaxed),
            spatial_grid_rebuild_us: self.spatial_grid_rebuild_us.load(Ordering::Relaxed),
            l1_aggregation_us: self.l1_aggregation_us.load(Ordering::Relaxed),
            behavior_transition_us: self.behavior_transition_us.load(Ordering::Relaxed),
            steering_us: self.steering_us.load(Ordering::Relaxed),
            capture_debug_accel_us: self.capture_debug_accel_us.load(Ordering::Relaxed),
            export_positions_us: self.export_positions_us.load(Ordering::Relaxed),
            cells_queried_total: self.cells_queried_total.swap(0, Ordering::Relaxed),
            archetype_count: 0,
            entity_count: 0,
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
    pub spatial_grid_rebuild_us: u64,
    pub l1_aggregation_us: u64,
    pub behavior_transition_us: u64,
    pub steering_us: u64,
    pub capture_debug_accel_us: u64,
    pub export_positions_us: u64,
    // Count metrics (reset-on-read via swap)
    pub cells_queried_total: u64,

    pub archetype_count: u64,
    pub entity_count: u64,
}

use bevy_ecs::world::World;

pub fn extract_ecs_metrics(world: &World) -> (u64, u64) {
    let archetype_count = world.archetypes().len() as u64;
    let entity_count = world.entities().len() as u64;
    (archetype_count, entity_count)
}
