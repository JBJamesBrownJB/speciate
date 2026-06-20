//! Windows-only process telemetry.
//!
//! perf_event (the Linux PMU source) has no Windows user-space equivalent, so on
//! Windows the dev-ui hardware-counter panel is replaced by a "Linux only" badge.
//! This module fills part of that gap with cheap, documented Win32 calls:
//! process cycle time (QueryProcessCycleTime) and page-fault / working-set counts
//! (GetProcessMemoryInfo). Values are reported as per-second rates plus cumulative
//! totals; see docs/scale/windows-parity-strategy.md §4 (Tier 1/2).

use std::sync::{Mutex, OnceLock};
use std::time::Instant;

use windows_sys::Win32::System::ProcessStatus::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};
use windows_sys::Win32::System::Threading::GetCurrentProcess;
use windows_sys::Win32::System::WindowsProgramming::QueryProcessCycleTime;

use crate::ipc::bridge::telemetry::WindowsMetricsSnapshot;

/// Cumulative process cycle count (reference cycles, RDTSC-based — NOT true
/// core-clock cycles). Returns None if the syscall fails.
fn query_process_cycles() -> Option<u64> {
    let mut cycles: u64 = 0;
    let ok = unsafe { QueryProcessCycleTime(GetCurrentProcess(), &mut cycles) };
    (ok != 0).then_some(cycles)
}

/// (cumulative page-fault count, working-set bytes). None if the syscall fails.
fn query_memory_counters() -> Option<(u64, u64)> {
    let mut counters: PROCESS_MEMORY_COUNTERS = unsafe { std::mem::zeroed() };
    counters.cb = std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32;
    let ok = unsafe { GetProcessMemoryInfo(GetCurrentProcess(), &mut counters, counters.cb) };
    (ok != 0).then(|| (counters.PageFaultCount as u64, counters.WorkingSetSize as u64))
}

/// Holds the previous sample so we can turn cumulative counters into rates.
struct WindowsMetrics {
    prev_cycles: u64,
    prev_page_faults: u64,
    last_sample: Option<Instant>,
}

impl WindowsMetrics {
    fn new() -> Self {
        Self {
            prev_cycles: 0,
            prev_page_faults: 0,
            last_sample: None,
        }
    }

    fn read(&mut self) -> WindowsMetricsSnapshot {
        let cycles = query_process_cycles();
        let mem = query_memory_counters();

        // If either probe failed entirely, report unavailable rather than zeros.
        let (Some(cycles), Some((page_faults, working_set))) = (cycles, mem) else {
            return WindowsMetricsSnapshot::default();
        };

        let now = Instant::now();
        let (cycles_per_sec, page_faults_per_sec) = match self.last_sample {
            Some(prev) => {
                let dt = now.duration_since(prev).as_secs_f64().max(1e-6);
                let dc = cycles.saturating_sub(self.prev_cycles) as f64 / dt;
                let df = page_faults.saturating_sub(self.prev_page_faults) as f64 / dt;
                (dc, df)
            }
            // First sample has no baseline to diff against.
            None => (0.0, 0.0),
        };

        self.prev_cycles = cycles;
        self.prev_page_faults = page_faults;
        self.last_sample = Some(now);

        WindowsMetricsSnapshot {
            available: true,
            process_cycles_per_sec: cycles_per_sec,
            page_faults_per_sec,
            page_fault_count: page_faults,
            working_set_bytes: working_set,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_snapshot_is_available_and_plausible() {
        let snap = read_snapshot();
        assert!(snap.available, "Windows process telemetry should be available");
        // A running process always has a non-zero working set and has taken page faults.
        assert!(snap.working_set_bytes > 0, "working set should be non-zero");
        assert!(snap.page_fault_count > 0, "page fault count should be non-zero");
        // Rates are 0 on the very first sample (no baseline), and non-negative after.
        assert!(snap.process_cycles_per_sec >= 0.0);
        assert!(snap.page_faults_per_sec >= 0.0);
    }
}

static STATE: OnceLock<Mutex<WindowsMetrics>> = OnceLock::new();

/// Sample current Windows process telemetry. Persists the previous sample across
/// calls (process-global) so cumulative counters become per-second rates.
pub fn read_snapshot() -> WindowsMetricsSnapshot {
    STATE
        .get_or_init(|| Mutex::new(WindowsMetrics::new()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .read()
}
