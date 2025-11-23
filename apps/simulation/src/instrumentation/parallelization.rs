use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use bevy_ecs::system::Resource;

#[cfg(feature = "dev-tools")]
use sysinfo::{CpuRefreshKind, RefreshKind, System};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParallelizationSnapshot {
    pub cpu_cores_total: usize,
    pub cpu_cores_active: usize,
    pub cpu_utilization_pct: f32,
    pub estimated_parallelism_factor: f32,
    pub concurrent_systems_estimate: usize,
}

impl Default for ParallelizationSnapshot {
    fn default() -> Self {
        Self {
            cpu_cores_total: std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1),
            cpu_cores_active: 0,
            cpu_utilization_pct: 0.0,
            estimated_parallelism_factor: 0.0,
            concurrent_systems_estimate: 1,
        }
    }
}

#[cfg(feature = "dev-tools")]
#[derive(Resource)]
pub struct ParallelizationMetrics {
    system: Mutex<System>,
    cpu_cores_total: usize,
}

#[cfg(feature = "dev-tools")]
impl ParallelizationMetrics {
    pub fn new() -> Self {
        let cpu_cores_total = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);

        Self {
            system: Mutex::new(System::new_with_specifics(
                RefreshKind::new().with_cpu(CpuRefreshKind::everything()),
            )),
            cpu_cores_total,
        }
    }

    pub fn read(&mut self) -> ParallelizationSnapshot {
        let mut system = self.system.lock()
            .unwrap_or_else(|poisoned| {
                // Mutex poisoned due to panic in another thread
                // Recover by accessing the data anyway (safe for our use case)
                poisoned.into_inner()
            });

        system.refresh_cpu_all();

        let cpus = system.cpus();
        let total_usage: f32 = cpus.iter().map(|cpu| cpu.cpu_usage()).sum();
        let avg_usage = if !cpus.is_empty() {
            total_usage / cpus.len() as f32
        } else {
            0.0
        };

        let active_cores = cpus
            .iter()
            .filter(|cpu| cpu.cpu_usage() > 10.0)
            .count();

        let parallelism_factor = if self.cpu_cores_total > 0 {
            active_cores as f32 / self.cpu_cores_total as f32
        } else {
            0.0
        };

        let concurrent_systems_estimate = if active_cores > 1 {
            (active_cores as f32 * 0.7) as usize
        } else {
            1
        };

        ParallelizationSnapshot {
            cpu_cores_total: self.cpu_cores_total,
            cpu_cores_active: active_cores,
            cpu_utilization_pct: avg_usage,
            estimated_parallelism_factor: parallelism_factor,
            concurrent_systems_estimate,
        }
    }
}

#[cfg(feature = "dev-tools")]
impl Default for ParallelizationMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(feature = "dev-tools"))]
#[derive(Resource)]
pub struct ParallelizationMetrics;

#[cfg(not(feature = "dev-tools"))]
impl ParallelizationMetrics {
    pub fn new() -> Self {
        Self
    }

    pub fn read(&mut self) -> ParallelizationSnapshot {
        ParallelizationSnapshot::default()
    }
}

#[cfg(not(feature = "dev-tools"))]
impl Default for ParallelizationMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parallelization_snapshot_default() {
        let snapshot = ParallelizationSnapshot::default();
        assert!(snapshot.cpu_cores_total >= 1);
        assert!(snapshot.cpu_utilization_pct >= 0.0);
        assert!(snapshot.cpu_utilization_pct <= 100.0);
        assert!(snapshot.estimated_parallelism_factor >= 0.0);
        assert!(snapshot.estimated_parallelism_factor <= 1.0);
    }

    #[cfg(feature = "dev-tools")]
    #[test]
    fn test_parallelization_metrics_read() {
        let mut metrics = ParallelizationMetrics::new();
        let snapshot = metrics.read();

        assert!(snapshot.cpu_cores_total >= 1);
        assert!(snapshot.cpu_cores_active <= snapshot.cpu_cores_total);
        assert!(snapshot.cpu_utilization_pct >= 0.0);
        assert!(snapshot.estimated_parallelism_factor >= 0.0);
        assert!(snapshot.estimated_parallelism_factor <= 1.0);
    }
}
