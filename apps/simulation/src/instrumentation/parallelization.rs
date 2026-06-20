use bevy_ecs::system::Resource;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;

#[cfg(feature = "dev-tools")]
use sysinfo::{get_current_pid, CpuRefreshKind, Pid, ProcessesToUpdate, RefreshKind, System};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParallelizationSnapshot {
    pub cpu_cores_total: usize,
    pub cpu_cores_active: usize,
    pub cpu_utilization_pct: f32,
    pub estimated_parallelism_factor: f32,
    pub concurrent_systems_estimate: usize,
    pub process_memory_bytes: u64,
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
            process_memory_bytes: 0,
        }
    }
}

#[cfg(feature = "dev-tools")]
const CPU_REFRESH_INTERVAL: u32 = 10; // Only refresh CPU every N reads (reduces allocations)

#[cfg(feature = "dev-tools")]
#[derive(Resource)]
pub struct ParallelizationMetrics {
    system: Mutex<System>,
    cpu_cores_total: usize,
    read_count: AtomicU32,
    cached_cpu_usage: f32,
    cached_active_cores: usize,
    pid: Option<Pid>,
    cached_process_memory: u64,
}

#[cfg(feature = "dev-tools")]
impl ParallelizationMetrics {
    fn create_cpu_system() -> System {
        // Only request CPU info - no process info (much lighter)
        System::new_with_specifics(RefreshKind::new().with_cpu(CpuRefreshKind::everything()))
    }

    pub fn new() -> Self {
        let cpu_cores_total = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);

        Self {
            system: Mutex::new(Self::create_cpu_system()),
            cpu_cores_total,
            read_count: AtomicU32::new(0),
            cached_cpu_usage: 0.0,
            cached_active_cores: 0,
            pid: get_current_pid().ok(),
            cached_process_memory: 0,
        }
    }

    pub fn read(&mut self) -> ParallelizationSnapshot {
        let count = self.read_count.fetch_add(1, Ordering::Relaxed);

        // Only refresh CPU metrics every N reads to reduce sysinfo allocations
        if count % CPU_REFRESH_INTERVAL == 0 {
            let mut system = self
                .system
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            // Recreate system periodically to prevent any internal accumulation
            if count % (CPU_REFRESH_INTERVAL * 60) == 0 && count > 0 {
                *system = Self::create_cpu_system();
            }

            system.refresh_cpu_all();

            let cpus = system.cpus();
            let total_usage: f32 = cpus.iter().map(|cpu| cpu.cpu_usage()).sum();
            self.cached_cpu_usage = if !cpus.is_empty() {
                total_usage / cpus.len() as f32
            } else {
                0.0
            };

            self.cached_active_cores = cpus.iter().filter(|cpu| cpu.cpu_usage() > 10.0).count();

            // Cross-platform process RSS via sysinfo (replaces the Linux-only
            // /proc/self/statm read, which returned 0 on Windows). Refreshed only
            // the current PID, on the same cadence, to keep this light.
            if let Some(pid) = self.pid {
                system.refresh_processes(ProcessesToUpdate::Some(&[pid]));
                self.cached_process_memory =
                    system.process(pid).map(|p| p.memory()).unwrap_or(0);
            }
        }

        let parallelism_factor = if self.cpu_cores_total > 0 {
            self.cached_active_cores as f32 / self.cpu_cores_total as f32
        } else {
            0.0
        };

        let concurrent_systems_estimate = if self.cached_active_cores > 1 {
            (self.cached_active_cores as f32 * 0.7) as usize
        } else {
            1
        };

        ParallelizationSnapshot {
            cpu_cores_total: self.cpu_cores_total,
            cpu_cores_active: self.cached_active_cores,
            cpu_utilization_pct: self.cached_cpu_usage,
            estimated_parallelism_factor: parallelism_factor,
            concurrent_systems_estimate,
            process_memory_bytes: self.cached_process_memory,
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

    #[cfg(feature = "dev-tools")]
    #[test]
    fn test_process_memory_metrics() {
        let mut metrics = ParallelizationMetrics::new();
        let snapshot = metrics.read();

        assert!(
            snapshot.process_memory_bytes > 0,
            "Process memory should be non-zero"
        );
        assert!(
            snapshot.process_memory_bytes < 10 * 1024 * 1024 * 1024,
            "Process memory should be < 10GB"
        );
    }
}
