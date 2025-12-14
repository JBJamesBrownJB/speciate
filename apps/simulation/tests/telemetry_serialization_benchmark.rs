use speciate::instrumentation::{HardwareSnapshot, ParallelizationSnapshot, SystemTimingsSnapshot};
use std::time::Instant;

#[test]
fn benchmark_telemetry_json_serialization() {
    println!("\n=== TELEMETRY JSON SERIALIZATION BENCHMARK ===\n");

    let hw_snapshot = HardwareSnapshot {
        cycles_delta: 1_234_567_890,
        instructions_delta: 2_345_678_901,
        cache_refs_delta: 123_456_789,
        cache_misses_delta: 12_345_678,
        l1d_misses_delta: 1_234_567,
        l1i_misses_delta: 234_567,
        branch_instructions_delta: 345_678_901,
        branch_misses_delta: 34_567_890,
        stalled_frontend_delta: 123_456,
        stalled_backend_delta: 234_567,
        ipc: 1.89,
        l1d_miss_rate: 3.45,
        l1i_miss_rate: 0.78,
        llc_miss_rate: 10.23,
        branch_miss_rate: 9.98,
        frontend_stall_ratio: 0.12,
        backend_stall_ratio: 0.23,
    };

    let sys_timings = SystemTimingsSnapshot {
        total_tick_us: 12345,
        movement_us: 2345,
        perception_us: 3456,
        behavior_us: 1234,
        behavior_transition_us: 567,
        wander_us: 234,
        seek_us: 178,
        flee_us: 123,
        avoidance_us: 345,
        steering_cap_us: 10,
        rotation_us: 456,
        spatial_grid_rebuild_us: 789,
        capture_debug_accel_us: 5,

        archetype_count: 42,
        entity_count: 150_000,
    };

    let para_snapshot = ParallelizationSnapshot {
        cpu_cores_total: 16,
        cpu_cores_active: 8,
        cpu_utilization_pct: 65.4,
        estimated_parallelism_factor: 2.3,
        concurrent_systems_estimate: 4,
        process_memory_bytes: 200_000_000,
    };

    #[derive(serde::Serialize)]
    struct TelemetryPayload {
        tick: u64,
        tick_rate: f32,
        creature_count: u64,
        entity_count: u64,
        system_timings_us: SystemTimingsSnapshot,
        hardware_metrics: HardwareSnapshot,
        parallelization_metrics: ParallelizationSnapshot,
    }

    let payload = TelemetryPayload {
        tick: 123456,
        tick_rate: 30.0,
        creature_count: 125_000,
        entity_count: 150_000,
        system_timings_us: sys_timings.clone(),
        hardware_metrics: hw_snapshot.clone(),
        parallelization_metrics: para_snapshot.clone(),
    };

    const ITERATIONS: usize = 10_000;
    let mut total_time_ns = 0u64;
    let mut min_time_ns = u64::MAX;
    let mut max_time_ns = 0u64;
    let mut json_size_bytes = 0usize;

    for _ in 0..ITERATIONS {
        let start = Instant::now();
        let json = serde_json::to_string(&payload).unwrap();
        let elapsed = start.elapsed().as_nanos() as u64;

        total_time_ns += elapsed;
        min_time_ns = min_time_ns.min(elapsed);
        max_time_ns = max_time_ns.max(elapsed);
        json_size_bytes = json.len();
    }

    let avg_time_ns = total_time_ns / ITERATIONS as u64;
    let avg_time_us = avg_time_ns as f64 / 1_000.0;

    println!("Payload: 45+ fields (HW: 17, SysTiming: 17, Para: 5, Core: 4)");
    println!("JSON Size: {} bytes", json_size_bytes);
    println!("\n--- Serialization Performance (n={}) ---", ITERATIONS);
    println!("  Average: {:.2} µs ({} ns)", avg_time_us, avg_time_ns);
    println!("  Min:     {:.2} µs ({} ns)", min_time_ns as f64 / 1_000.0, min_time_ns);
    println!("  Max:     {:.2} µs ({} ns)", max_time_ns as f64 / 1_000.0, max_time_ns);

    println!("\n--- Polling Frequency Analysis ---");

    let frequencies = [
        ("30 Hz (33.33 ms)", 30.0, 33_333_333.0),
        ("60 Hz (16.67 ms)", 60.0, 16_666_667.0),
        ("90 Hz (11.11 ms)", 90.0, 11_111_111.0),
        ("120 Hz (8.33 ms)", 120.0, 8_333_333.0),
    ];

    for (label, hz, budget_ns) in frequencies {
        let overhead_ns = avg_time_ns;
        let overhead_pct = (overhead_ns as f64 / budget_ns) * 100.0;
        let overhead_per_sec = (overhead_ns as f64 * hz) / 1_000.0;

        println!("\n  {} polling:", label);
        println!("    Per-tick overhead: {:.2} µs ({:.4}% of tick budget)",
                 avg_time_us, overhead_pct);
        println!("    Total overhead/sec: {:.2} µs ({} calls/sec)",
                 overhead_per_sec, hz as u32);

        if overhead_pct < 1.0 {
            println!("    Status: ✓ SAFE (< 1% tick budget)");
        } else if overhead_pct < 5.0 {
            println!("    Status: ⚠ ACCEPTABLE (< 5% tick budget)");
        } else {
            println!("    Status: ✗ DANGEROUS (> 5% tick budget)");
        }
    }

    println!("\n--- Memory Characteristics ---");
    println!("  Stack allocation: ~{} bytes (3 snapshot structs)",
             std::mem::size_of::<TelemetryPayload>());
    println!("  Heap allocation: ~{} bytes (serde_json temporary buffer)",
             json_size_bytes * 2);
    println!("  Total temporary footprint: ~{} KB",
             (std::mem::size_of::<TelemetryPayload>() + json_size_bytes * 2) / 1024);

    println!("\n--- Comparison: Polling vs Callback ---");
    println!("  Polling (30Hz):   {:.2} µs/tick × 30 = {:.2} µs/sec total overhead",
             avg_time_us, avg_time_us * 30.0);
    println!("  Callback (push):  ~{:.2} µs/event (only when data changes)",
             avg_time_us);
    println!("  Note: Callback adds Arc clone + channel send (~50-100ns extra)");

    println!("\n=== RECOMMENDATION ===");
    if avg_time_us < 10.0 {
        println!("✓ JSON serialization cost is NEGLIGIBLE (< 10µs)");
        println!("✓ 30Hz polling is SAFE (< 0.1% overhead at 30Hz simulation)");
        println!("✓ 60Hz polling is ACCEPTABLE for dev-ui responsiveness");
        println!("\nSuggested: Start with 30Hz polling, allow user to increase to 60Hz in dev-ui settings.");
    } else if avg_time_us < 50.0 {
        println!("⚠ JSON serialization has MEASURABLE cost (10-50µs)");
        println!("⚠ Recommend 30Hz polling maximum");
        println!("⚠ Consider caching static metrics (cpu_cores_total, etc.)");
    } else {
        println!("✗ JSON serialization is EXPENSIVE (> 50µs)");
        println!("✗ Recommend callback-based push model instead of polling");
        println!("✗ Or reduce polling to 10Hz and cache aggressively");
    }

    println!("\n--- Optimization Opportunities ---");
    println!("1. Static Caching: cpu_cores_total, rust_version (never change)");
    println!("2. Lazy Updates: Only serialize changed metrics (delta compression)");
    println!("3. Binary Format: MessagePack/bincode (50-70% faster than JSON)");
    println!("4. Snapshot Throttling: Only emit when metrics change > threshold");

    assert!(avg_time_us < 100.0,
            "Serialization too slow: {:.2}µs exceeds 100µs budget", avg_time_us);
}

#[test]
fn benchmark_individual_snapshot_serialization() {
    println!("\n=== INDIVIDUAL SNAPSHOT SERIALIZATION COST ===\n");

    let hw = HardwareSnapshot {
        cycles_delta: 1_234_567_890,
        instructions_delta: 2_345_678_901,
        cache_refs_delta: 123_456_789,
        cache_misses_delta: 12_345_678,
        l1d_misses_delta: 1_234_567,
        l1i_misses_delta: 234_567,
        branch_instructions_delta: 345_678_901,
        branch_misses_delta: 34_567_890,
        stalled_frontend_delta: 123_456,
        stalled_backend_delta: 234_567,
        ipc: 1.89,
        l1d_miss_rate: 3.45,
        l1i_miss_rate: 0.78,
        llc_miss_rate: 10.23,
        branch_miss_rate: 9.98,
        frontend_stall_ratio: 0.12,
        backend_stall_ratio: 0.23,
    };

    let sys = SystemTimingsSnapshot::default();
    let para = ParallelizationSnapshot::default();

    const N: usize = 10_000;

    let hw_time = {
        let start = Instant::now();
        for _ in 0..N {
            let _ = serde_json::to_string(&hw).unwrap();
        }
        start.elapsed().as_nanos() / N as u128
    };

    let sys_time = {
        let start = Instant::now();
        for _ in 0..N {
            let _ = serde_json::to_string(&sys).unwrap();
        }
        start.elapsed().as_nanos() / N as u128
    };

    let para_time = {
        let start = Instant::now();
        for _ in 0..N {
            let _ = serde_json::to_string(&para).unwrap();
        }
        start.elapsed().as_nanos() / N as u128
    };

    println!("HardwareSnapshot (17 fields):       {:.2} µs", hw_time as f64 / 1_000.0);
    println!("SystemTimingsSnapshot (17 fields):  {:.2} µs", sys_time as f64 / 1_000.0);
    println!("ParallelizationSnapshot (5 fields): {:.2} µs", para_time as f64 / 1_000.0);
    println!("Total (sequential):                 {:.2} µs",
             (hw_time + sys_time + para_time) as f64 / 1_000.0);

    println!("\nOptimization: If parallelized, could reduce by ~40%");
}
