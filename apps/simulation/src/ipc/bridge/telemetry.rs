use serde::{Deserialize, Serialize};

#[cfg(feature = "dev-tools")]
use crate::instrumentation::{HardwareSnapshot, ParallelizationSnapshot, SystemTimingsSnapshot};

#[cfg(not(feature = "dev-tools"))]
use crate::instrumentation::SystemTimingsSnapshot;

/// Windows-only process telemetry, shown in the dev-ui where the Linux PMU
/// hardware counters are unavailable. Always present in the JSON (camelCase);
/// `available` is false on non-Windows hosts and when the Win32 probes fail.
/// See `crate::instrumentation::windows_metrics` and
/// docs/scale/windows-parity-strategy.md §4.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WindowsMetricsSnapshot {
    pub available: bool,
    /// Process reference cycles per second (RDTSC-based; not true core-clock cycles).
    pub process_cycles_per_sec: f64,
    /// Page faults per second (rate of the cumulative count).
    pub page_faults_per_sec: f64,
    /// Cumulative page-fault count since process start.
    pub page_fault_count: u64,
    /// Working-set (resident) memory in bytes.
    pub working_set_bytes: u64,
}

/// L1 cell data for heatmap visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct L1CellData {
    pub x: i32,
    pub y: i32,
    pub total_mass: f32,
    pub creature_count: u16,
}

/// Telemetry snapshot for NAPI polling
///
/// This struct mirrors the old stdio IPC telemetry format to maintain
/// compatibility with dev-ui and portal. All metrics are preserved.
///
/// **Performance:**
/// - JSON serialization: 3-8µs per call
/// - 30Hz polling overhead: 0.015% of tick budget (negligible)
///
/// **Feature Gating:**
/// - Hardware counters: Behind `#[cfg(feature = "dev-tools")]`
/// - System timings: Always included (cheap, ~40 bytes)
/// - Creature count: Always included
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TelemetrySnapshot {
    pub tick: u64,
    pub creature_count: usize,
    pub tick_rate_hz: f32,
    pub spatial_grid_cell_size: f32,
    pub l1_cell_size: f32,
    pub spatial_grid_min_x: f32,
    pub spatial_grid_max_x: f32,
    pub spatial_grid_min_y: f32,
    pub spatial_grid_max_y: f32,

    #[cfg(feature = "dev-tools")]
    pub hardware_metrics: HardwareSnapshot,

    #[cfg(feature = "dev-tools")]
    pub parallelization_metrics: ParallelizationSnapshot,

    pub system_timings: SystemTimingsSnapshot,

    /// Windows-only process telemetry (cycle time, page faults, working set).
    /// `available` is false off Windows. Populated in `get_telemetry`.
    pub windows_metrics: WindowsMetricsSnapshot,

    #[cfg(not(feature = "dev-tools"))]
    pub hardware_metrics: HardwareSnapshotStub,

    #[cfg(not(feature = "dev-tools"))]
    pub parallelization_metrics: ParallelizationSnapshotStub,
}

#[cfg(not(feature = "dev-tools"))]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct HardwareSnapshotStub {
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

#[cfg(not(feature = "dev-tools"))]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ParallelizationSnapshotStub {
    pub cpu_cores_total: usize,
    pub cpu_cores_active: usize,
    pub cpu_utilization_pct: f32,
    pub estimated_parallelism_factor: f32,
    pub concurrent_systems_estimate: usize,
    pub process_memory_bytes: u64,
}

impl TelemetrySnapshot {
    pub fn new(
        tick: u64,
        creature_count: usize,
        tick_rate_hz: f32,
        spatial_grid_cell_size: f32,
        l1_cell_size: f32,
        spatial_grid_bounds: (f32, f32, f32, f32), // (min_x, max_x, min_y, max_y)
        system_timings: SystemTimingsSnapshot,
        #[cfg(feature = "dev-tools")] hardware_metrics: HardwareSnapshot,
        #[cfg(feature = "dev-tools")] parallelization_metrics: ParallelizationSnapshot,
    ) -> Self {
        Self {
            tick,
            creature_count,
            tick_rate_hz,
            spatial_grid_cell_size,
            l1_cell_size,
            spatial_grid_min_x: spatial_grid_bounds.0,
            spatial_grid_max_x: spatial_grid_bounds.1,
            spatial_grid_min_y: spatial_grid_bounds.2,
            spatial_grid_max_y: spatial_grid_bounds.3,
            system_timings,
            windows_metrics: WindowsMetricsSnapshot::default(),
            #[cfg(feature = "dev-tools")]
            hardware_metrics,
            #[cfg(feature = "dev-tools")]
            parallelization_metrics,
            #[cfg(not(feature = "dev-tools"))]
            hardware_metrics: HardwareSnapshotStub::default(),
            #[cfg(not(feature = "dev-tools"))]
            parallelization_metrics: ParallelizationSnapshotStub::default(),
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

impl Default for TelemetrySnapshot {
    fn default() -> Self {
        use crate::simulation::core::MAX_WORLD_SIZE;
        use crate::simulation::spatial::constants::L1_CELL_SIZE;
        Self {
            tick: 0,
            creature_count: 0,
            tick_rate_hz: 0.0,
            spatial_grid_cell_size: crate::simulation::spatial::CELL_SIZE,
            l1_cell_size: L1_CELL_SIZE,
            spatial_grid_min_x: -MAX_WORLD_SIZE,
            spatial_grid_max_x: MAX_WORLD_SIZE,
            spatial_grid_min_y: -MAX_WORLD_SIZE,
            spatial_grid_max_y: MAX_WORLD_SIZE,
            system_timings: SystemTimingsSnapshot::default(),
            windows_metrics: WindowsMetricsSnapshot::default(),
            #[cfg(feature = "dev-tools")]
            hardware_metrics: HardwareSnapshot::default(),
            #[cfg(feature = "dev-tools")]
            parallelization_metrics: ParallelizationSnapshot::default(),
            #[cfg(not(feature = "dev-tools"))]
            hardware_metrics: HardwareSnapshotStub::default(),
            #[cfg(not(feature = "dev-tools"))]
            parallelization_metrics: ParallelizationSnapshotStub::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_snapshot_creation() {
        let snapshot = TelemetrySnapshot::default();
        assert_eq!(snapshot.tick, 0);
        assert_eq!(snapshot.creature_count, 0);
    }

    #[test]
    fn test_telemetry_json_serialization() {
        let snapshot = TelemetrySnapshot::default();
        let json = snapshot.to_json().expect("Failed to serialize to JSON");

        assert!(json.contains("tick"));
        assert!(json.contains("creatureCount"));
        assert!(json.contains("systemTimings"));
        assert!(json.contains("hardwareMetrics"));
        assert!(json.contains("parallelizationMetrics"));
    }

    #[test]
    fn test_telemetry_json_schema() {
        let snapshot = TelemetrySnapshot::default();
        let json = snapshot.to_json().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert!(parsed.get("tick").is_some());
        assert!(parsed.get("creatureCount").is_some());
        assert!(parsed.get("systemTimings").is_some());
        assert!(parsed.get("hardwareMetrics").is_some());

        let system_timings = parsed.get("systemTimings").unwrap();
        assert!(system_timings.get("totalTickUs").is_some());
        assert!(system_timings.get("movementUs").is_some());
        assert!(system_timings.get("perceptionUs").is_some());
        assert!(system_timings.get("behaviorTransitionUs").is_some());
        assert!(system_timings.get("steeringUs").is_some());
    }

    #[test]
    fn test_telemetry_with_values() {
        let system_timings = SystemTimingsSnapshot {
            total_tick_us: 5000,
            movement_us: 1000,
            perception_us: 500,
            spatial_grid_rebuild_us: 100,
            l1_aggregation_us: 50,
            l2_aggregation_us: 25,
            behavior_transition_us: 100,
            steering_us: 230,
            capture_debug_accel_us: 5,
            export_positions_us: 135,
            cells_queried_total: 4500,
            archetype_count: 10,
            entity_count: 1000,
        };

        let snapshot = TelemetrySnapshot::new(
            42,
            1000,
            29.5,
            50.0,                           // spatial_grid_cell_size (L0)
            30.0,                           // l1_cell_size
            (-500.0, 500.0, -500.0, 500.0), // spatial_grid_bounds
            system_timings,
            #[cfg(feature = "dev-tools")]
            HardwareSnapshot::default(),
            #[cfg(feature = "dev-tools")]
            ParallelizationSnapshot::default(),
        );

        assert_eq!(snapshot.tick, 42);
        assert_eq!(snapshot.creature_count, 1000);
        assert_eq!(snapshot.system_timings.total_tick_us, 5000);
        assert_eq!(snapshot.system_timings.movement_us, 1000);
    }

    #[test]
    #[cfg(feature = "dev-tools")]
    fn test_hardware_metrics_included_with_feature() {
        let snapshot = TelemetrySnapshot::default();
        let json = snapshot.to_json().unwrap();

        assert!(json.contains("cyclesDelta"));
        assert!(json.contains("instructionsDelta"));
        assert!(json.contains("ipc"));
        assert!(json.contains("llcMissRate"));
    }

    #[test]
    #[cfg(not(feature = "dev-tools"))]
    fn test_hardware_metrics_stub_without_feature() {
        let snapshot = TelemetrySnapshot::default();

        assert_eq!(snapshot.hardware_metrics.cycles_delta, 0);
        assert_eq!(snapshot.hardware_metrics.instructions_delta, 0);
        assert_eq!(snapshot.hardware_metrics.ipc, 0.0);
    }
}
