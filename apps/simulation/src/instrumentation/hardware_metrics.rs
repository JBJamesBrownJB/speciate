use serde::{Deserialize, Serialize};

#[cfg(feature = "dev-tools")]
use bevy_ecs::system::Resource;

#[cfg(feature = "dev-tools")]
use perf_event::{Builder, Counter, Group};

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

#[cfg(feature = "dev-tools")]
#[derive(Resource)]
pub struct HardwareMetrics {
    ipc_group: Option<Group>,
    cycles: Option<Counter>,
    instructions: Option<Counter>,
    cache_references: Option<Counter>,
    cache_misses: Option<Counter>,
    l1d_misses: Option<Counter>,
    l1d_accesses: Option<Counter>,
    l1i_misses: Option<Counter>,
    l1i_accesses: Option<Counter>,
    llc_misses: Option<Counter>,
    branch_instructions: Option<Counter>,
    branch_misses: Option<Counter>,
    stalled_frontend: Option<Counter>,
    stalled_backend: Option<Counter>,
    enabled: bool,

    prev_cycles: u64,
    prev_instructions: u64,
    prev_cache_refs: u64,
    prev_cache_misses: u64,
    prev_l1d_misses: u64,
    prev_l1d_accesses: u64,
    prev_l1i_misses: u64,
    prev_l1i_accesses: u64,
    prev_branch_instructions: u64,
    prev_branch_misses: u64,
    prev_stalled_frontend: u64,
    prev_stalled_backend: u64,
}

#[cfg(feature = "dev-tools")]
impl HardwareMetrics {
    pub fn new() -> Self {
        match Self::try_init() {
            Ok(metrics) => metrics,
            Err(e) => {
                eprintln!("⚠️  Failed to initialize hardware counters: {}", e);
                eprintln!("   Falling back to disabled state");
                Self {
                    ipc_group: None,
                    cycles: None,
                    instructions: None,
                    cache_references: None,
                    cache_misses: None,
                    l1d_misses: None,
                    l1d_accesses: None,
                    l1i_misses: None,
                    l1i_accesses: None,
                    llc_misses: None,
                    branch_instructions: None,
                    branch_misses: None,
                    stalled_frontend: None,
                    stalled_backend: None,
                    enabled: false,
                    prev_cycles: 0,
                    prev_instructions: 0,
                    prev_cache_refs: 0,
                    prev_cache_misses: 0,
                    prev_l1d_misses: 0,
                    prev_l1d_accesses: 0,
                    prev_l1i_misses: 0,
                    prev_l1i_accesses: 0,
                    prev_branch_instructions: 0,
                    prev_branch_misses: 0,
                    prev_stalled_frontend: 0,
                    prev_stalled_backend: 0,
                }
            }
        }
    }

    fn try_init() -> Result<Self, std::io::Error> {
        use perf_event::events::{Hardware, Cache, CacheOp, CacheResult, WhichCache};

        // Create group for atomic IPC measurement
        let mut ipc_group = Group::new()
            .map_err(|e| {
                eprintln!("   Failed to create IPC counter group: {}", e);
                e
            })?;

        // Build cycles counter attached to group
        let cycles = Builder::new()
            .group(&mut ipc_group)
            .kind(Hardware::CPU_CYCLES)
            .build()
            .map_err(|e| {
                eprintln!("   Failed to build CPU_CYCLES counter: {}", e);
                e
            })?;

        // Build instructions counter attached to same group
        let instructions = Builder::new()
            .group(&mut ipc_group)
            .kind(Hardware::INSTRUCTIONS)
            .build()
            .map_err(|e| {
                eprintln!("   Failed to build INSTRUCTIONS counter: {}", e);
                e
            })?;

        // Enable group atomically (replaces individual enable calls)
        ipc_group.enable()?;

        let mut cache_references = Builder::new()
            .kind(Hardware::CACHE_REFERENCES)
            .build()
            .map_err(|e| {
                eprintln!("   Failed to build CACHE_REFERENCES counter: {}", e);
                e
            })?;
        cache_references.enable()?;

        let mut cache_misses = Builder::new()
            .kind(Hardware::CACHE_MISSES)
            .build()
            .map_err(|e| {
                eprintln!("   Failed to build CACHE_MISSES counter: {}", e);
                e
            })?;
        cache_misses.enable()?;

        let l1d_misses = Builder::new()
            .kind(Cache {
                which: WhichCache::L1D,
                operation: CacheOp::READ,
                result: CacheResult::MISS,
            })
            .build()
            .and_then(|mut c| {
                c.enable()?;
                Ok(c)
            })
            .ok();

        let l1d_accesses = Builder::new()
            .kind(Cache {
                which: WhichCache::L1D,
                operation: CacheOp::READ,
                result: CacheResult::ACCESS,
            })
            .build()
            .and_then(|mut c| {
                c.enable()?;
                Ok(c)
            })
            .ok();

        let l1i_misses = Builder::new()
            .kind(Cache {
                which: WhichCache::L1I,
                operation: CacheOp::READ,
                result: CacheResult::MISS,
            })
            .build()
            .and_then(|mut c| {
                c.enable()?;
                Ok(c)
            })
            .ok();

        let l1i_accesses = Builder::new()
            .kind(Cache {
                which: WhichCache::L1I,
                operation: CacheOp::READ,
                result: CacheResult::ACCESS,
            })
            .build()
            .and_then(|mut c| {
                c.enable()?;
                Ok(c)
            })
            .ok();

        let llc_misses = Builder::new()
            .kind(Cache {
                which: WhichCache::LL,
                operation: CacheOp::READ,
                result: CacheResult::MISS,
            })
            .build()
            .and_then(|mut c| {
                c.enable()?;
                Ok(c)
            })
            .ok();

        if l1d_misses.is_none() {
            eprintln!("   L1D cache counter not available (optional)");
        }
        if l1i_misses.is_none() {
            eprintln!("   L1I cache counter not available (optional)");
        }
        if llc_misses.is_none() {
            eprintln!("   LLC counter not available (optional)");
        }

        let mut branch_instructions = Builder::new()
            .kind(Hardware::BRANCH_INSTRUCTIONS)
            .build()
            .map_err(|e| {
                eprintln!("   Failed to build BRANCH_INSTRUCTIONS counter: {}", e);
                e
            })?;
        branch_instructions.enable()?;

        let mut branch_misses = Builder::new()
            .kind(Hardware::BRANCH_MISSES)
            .build()
            .map_err(|e| {
                eprintln!("   Failed to build BRANCH_MISSES counter: {}", e);
                e
            })?;
        branch_misses.enable()?;

        let stalled_frontend = Builder::new()
            .kind(Hardware::STALLED_CYCLES_FRONTEND)
            .build()
            .and_then(|mut c| {
                c.enable()?;
                Ok(c)
            })
            .ok();

        let stalled_backend = Builder::new()
            .kind(Hardware::STALLED_CYCLES_BACKEND)
            .build()
            .and_then(|mut c| {
                c.enable()?;
                Ok(c)
            })
            .ok();

        if stalled_frontend.is_none() {
            eprintln!("   Frontend stall counter not available (optional)");
        }
        if stalled_backend.is_none() {
            eprintln!("   Backend stall counter not available (optional)");
        }

        Ok(Self {
            ipc_group: Some(ipc_group),
            cycles: Some(cycles),
            instructions: Some(instructions),
            cache_references: Some(cache_references),
            cache_misses: Some(cache_misses),
            l1d_misses,
            l1d_accesses,
            l1i_misses,
            l1i_accesses,
            llc_misses,
            branch_instructions: Some(branch_instructions),
            branch_misses: Some(branch_misses),
            stalled_frontend,
            stalled_backend,
            enabled: true,
            prev_cycles: 0,
            prev_instructions: 0,
            prev_cache_refs: 0,
            prev_cache_misses: 0,
            prev_l1d_misses: 0,
            prev_l1d_accesses: 0,
            prev_l1i_misses: 0,
            prev_l1i_accesses: 0,
            prev_branch_instructions: 0,
            prev_branch_misses: 0,
            prev_stalled_frontend: 0,
            prev_stalled_backend: 0,
        })
    }

    pub fn read(&mut self) -> Option<HardwareSnapshot> {
        if !self.enabled || self.cycles.is_none() {
            return None;
        }

        // Read from group to ensure same time slice
        let (cycles, instructions) = if let Some(ref mut group) = self.ipc_group {
            if let Ok(counts) = group.read() {
                let c = self.cycles.as_ref()
                    .and_then(|ctr| counts.get(ctr).copied())
                    .unwrap_or(0);
                let i = self.instructions.as_ref()
                    .and_then(|ctr| counts.get(ctr).copied())
                    .unwrap_or(0);
                (c, i)
            } else {
                (0, 0)
            }
        } else {
            (0, 0)
        };
        let cache_refs = self.cache_references.as_mut().and_then(|c| c.read().ok()).unwrap_or(0);
        let cache_misses = self.cache_misses.as_mut().and_then(|c| c.read().ok()).unwrap_or(0);
        let l1d_misses = self.l1d_misses.as_mut().and_then(|c| c.read().ok()).unwrap_or(0);
        let l1d_accesses = self.l1d_accesses.as_mut().and_then(|c| c.read().ok()).unwrap_or(0);
        let l1i_misses = self.l1i_misses.as_mut().and_then(|c| c.read().ok()).unwrap_or(0);
        let l1i_accesses = self.l1i_accesses.as_mut().and_then(|c| c.read().ok()).unwrap_or(0);
        let branch_instructions = self.branch_instructions.as_mut().and_then(|c| c.read().ok()).unwrap_or(0);
        let branch_misses = self.branch_misses.as_mut().and_then(|c| c.read().ok()).unwrap_or(0);
        let stalled_frontend = self.stalled_frontend.as_mut().and_then(|c| c.read().ok()).unwrap_or(0);
        let stalled_backend = self.stalled_backend.as_mut().and_then(|c| c.read().ok()).unwrap_or(0);

        let cycles_delta = cycles.saturating_sub(self.prev_cycles);
        let instructions_delta = instructions.saturating_sub(self.prev_instructions);
        let cache_refs_delta = cache_refs.saturating_sub(self.prev_cache_refs);
        let cache_misses_delta = cache_misses.saturating_sub(self.prev_cache_misses);
        let l1d_misses_delta = l1d_misses.saturating_sub(self.prev_l1d_misses);
        let l1d_accesses_delta = l1d_accesses.saturating_sub(self.prev_l1d_accesses);
        let l1i_misses_delta = l1i_misses.saturating_sub(self.prev_l1i_misses);
        let l1i_accesses_delta = l1i_accesses.saturating_sub(self.prev_l1i_accesses);
        let branch_instructions_delta = branch_instructions.saturating_sub(self.prev_branch_instructions);
        let branch_misses_delta = branch_misses.saturating_sub(self.prev_branch_misses);
        let stalled_frontend_delta = stalled_frontend.saturating_sub(self.prev_stalled_frontend);
        let stalled_backend_delta = stalled_backend.saturating_sub(self.prev_stalled_backend);

        self.prev_cycles = cycles;
        self.prev_instructions = instructions;
        self.prev_cache_refs = cache_refs;
        self.prev_cache_misses = cache_misses;
        self.prev_l1d_misses = l1d_misses;
        self.prev_l1d_accesses = l1d_accesses;
        self.prev_l1i_misses = l1i_misses;
        self.prev_l1i_accesses = l1i_accesses;
        self.prev_branch_instructions = branch_instructions;
        self.prev_branch_misses = branch_misses;
        self.prev_stalled_frontend = stalled_frontend;
        self.prev_stalled_backend = stalled_backend;

        let ipc = if cycles_delta > 0 {
            instructions_delta as f64 / cycles_delta as f64
        } else {
            0.0
        };

        let l1d_miss_rate = if l1d_accesses_delta > 0 {
            (l1d_misses_delta as f64 / l1d_accesses_delta as f64) * 100.0
        } else {
            0.0
        };

        let l1i_miss_rate = if l1i_accesses_delta > 0 {
            (l1i_misses_delta as f64 / l1i_accesses_delta as f64) * 100.0
        } else {
            0.0
        };

        let llc_miss_rate = if cache_refs_delta > 0 && cache_misses_delta > 0 {
            (cache_misses_delta as f64 / cache_refs_delta as f64) * 100.0
        } else {
            0.0
        };

        let branch_miss_rate = if branch_instructions_delta > 0 {
            (branch_misses_delta as f64 / branch_instructions_delta as f64) * 100.0
        } else {
            0.0
        };

        let frontend_stall_ratio = if cycles_delta > 0 {
            (stalled_frontend_delta as f64 / cycles_delta as f64) * 100.0
        } else {
            0.0
        };

        let backend_stall_ratio = if cycles_delta > 0 {
            (stalled_backend_delta as f64 / cycles_delta as f64) * 100.0
        } else {
            0.0
        };

        Some(HardwareSnapshot {
            cycles_delta,
            instructions_delta,
            cache_refs_delta,
            cache_misses_delta,
            l1d_misses_delta,
            l1i_misses_delta,
            branch_instructions_delta,
            branch_misses_delta,
            stalled_frontend_delta,
            stalled_backend_delta,
            ipc,
            l1d_miss_rate,
            l1i_miss_rate,
            llc_miss_rate,
            branch_miss_rate,
            frontend_stall_ratio,
            backend_stall_ratio,
        })
    }

    #[allow(dead_code)]
    pub fn enable(&mut self) {
        if let Some(ref mut group) = self.ipc_group { group.enable().ok(); }
        if let Some(ref mut c) = self.cache_references { c.enable().ok(); }
        if let Some(ref mut c) = self.cache_misses { c.enable().ok(); }
        if let Some(ref mut c) = self.l1d_misses { c.enable().ok(); }
        if let Some(ref mut c) = self.l1i_misses { c.enable().ok(); }
        if let Some(ref mut c) = self.llc_misses { c.enable().ok(); }
        if let Some(ref mut c) = self.branch_instructions { c.enable().ok(); }
        if let Some(ref mut c) = self.branch_misses { c.enable().ok(); }
        if let Some(ref mut c) = self.stalled_frontend { c.enable().ok(); }
        if let Some(ref mut c) = self.stalled_backend { c.enable().ok(); }
        self.enabled = true;
    }

    #[allow(dead_code)]
    pub fn disable(&mut self) {
        if let Some(ref mut group) = self.ipc_group { group.disable().ok(); }
        if let Some(ref mut c) = self.cache_references { c.disable().ok(); }
        if let Some(ref mut c) = self.cache_misses { c.disable().ok(); }
        if let Some(ref mut c) = self.l1d_misses { c.disable().ok(); }
        if let Some(ref mut c) = self.l1i_misses { c.disable().ok(); }
        if let Some(ref mut c) = self.llc_misses { c.disable().ok(); }
        if let Some(ref mut c) = self.branch_instructions { c.disable().ok(); }
        if let Some(ref mut c) = self.branch_misses { c.disable().ok(); }
        if let Some(ref mut c) = self.stalled_frontend { c.disable().ok(); }
        if let Some(ref mut c) = self.stalled_backend { c.disable().ok(); }
        self.enabled = false;
    }
}

#[cfg(feature = "dev-tools")]
impl Default for HardwareMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(feature = "dev-tools"))]
pub struct HardwareMetrics;

#[cfg(not(feature = "dev-tools"))]
impl HardwareMetrics {
    pub fn new() -> Self {
        Self
    }

    pub fn read(&mut self) -> Option<HardwareSnapshot> {
        None
    }
}

#[cfg(not(feature = "dev-tools"))]
impl Default for HardwareMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hardware_snapshot_defaults() {
        let snapshot = HardwareSnapshot::default();
        assert_eq!(snapshot.cycles_delta, 0);
        assert_eq!(snapshot.instructions_delta, 0);
        assert_eq!(snapshot.ipc, 0.0);
    }

    #[test]
    #[cfg(feature = "dev-tools")]
    fn test_hardware_metrics_initialization() {
        let metrics = HardwareMetrics::new();
        assert!(metrics.cycles.is_some() || !metrics.enabled);
    }

    #[test]
    #[cfg(feature = "dev-tools")]
    fn test_hardware_metrics_actually_count() {
        let mut metrics = HardwareMetrics::new();

        if !metrics.enabled {
            eprintln!("Hardware counters not available on this system, skipping test");
            return;
        }

        metrics.read();

        let mut sum = 0u64;
        for i in 0..10000 {
            sum = sum.wrapping_add(i * 3);
        }
        std::hint::black_box(sum);

        let snapshot = metrics.read();
        assert!(snapshot.is_some());
        let delta = snapshot.unwrap();

        assert!(
            delta.cycles_delta > 0,
            "Cycles delta should be > 0 after work, got {}",
            delta.cycles_delta
        );
        assert!(
            delta.instructions_delta > 0,
            "Instructions delta should be > 0 after work, got {}",
            delta.instructions_delta
        );
        assert!(
            delta.ipc > 0.0,
            "IPC should be non-zero after work, got {}",
            delta.ipc
        );
        assert!(
            delta.branch_instructions_delta > 0,
            "Branch instructions delta should be > 0 after work, got {}",
            delta.branch_instructions_delta
        );
    }

    #[test]
    #[cfg(feature = "dev-tools")]
    fn test_disabled_metrics_returns_none() {
        let mut metrics = HardwareMetrics {
            cycles: None,
            instructions: None,
            ipc_group: None,
            cache_references: None,
            cache_misses: None,
            l1d_misses: None,
            l1d_accesses: None,
            l1i_misses: None,
            l1i_accesses: None,
            llc_misses: None,
            branch_instructions: None,
            branch_misses: None,
            stalled_frontend: None,
            stalled_backend: None,
            enabled: false,
            prev_cycles: 0,
            prev_instructions: 0,
            prev_cache_refs: 0,
            prev_cache_misses: 0,
            prev_l1d_misses: 0,
            prev_l1d_accesses: 0,
            prev_l1i_misses: 0,
            prev_l1i_accesses: 0,
            prev_branch_instructions: 0,
            prev_branch_misses: 0,
            prev_stalled_frontend: 0,
            prev_stalled_backend: 0,
        };

        let snapshot = metrics.read();
        assert!(snapshot.is_none(), "Disabled metrics should return None");
    }

    #[test]
    #[cfg(not(feature = "dev-tools"))]
    fn test_production_build_compiles_without_dev_tools() {
        let mut metrics = HardwareMetrics::new();
        let snapshot = metrics.read();
        assert!(snapshot.is_none(), "Production builds should return None");
    }
}
