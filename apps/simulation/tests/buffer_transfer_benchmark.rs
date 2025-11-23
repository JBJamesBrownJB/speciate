#![cfg(feature = "dev-tools")]
//! Dry-run benchmark for buffer transfer overhead
//!
//! This benchmark establishes the theoretical minimum overhead for zero-copy
//! buffer access in the NAPI-RS architecture. It simulates reading creature
//! positions from a shared memory buffer WITHOUT any sprite updates or rendering.
//!
//! **Purpose:** Baseline for Phase 4 post-migration validation.
//!
//! Run with:
//! ```bash
//! cargo test --features dev-tools --test buffer_transfer_benchmark -- --nocapture
//! ```

use std::time::Instant;

const FIELDS_PER_CREATURE: usize = 4; // ID, X, Y, Rotation

/// Simulates zero-copy buffer read (what NAPI will provide)
fn simulate_buffer_read(buffer: &[f32], creature_count: usize) -> u64 {
    let start = Instant::now();

    // SoA layout offsets
    let id_offset = 0;
    let x_offset = creature_count;
    let y_offset = creature_count * 2;
    let rot_offset = creature_count * 3;

    let mut checksum = 0u64;

    // Simulate reading all creatures (sequential access, cache-friendly)
    for i in 0..creature_count {
        let id = buffer[id_offset + i];
        let x = buffer[x_offset + i];
        let y = buffer[y_offset + i];
        let rot = buffer[rot_offset + i];

        // Prevent compiler optimization (force read)
        checksum = checksum.wrapping_add(id as u64);
        checksum = checksum.wrapping_add(x as u64);
        checksum = checksum.wrapping_add(y as u64);
        checksum = checksum.wrapping_add(rot as u64);
    }

    // Force checksum to be used
    std::hint::black_box(checksum);

    start.elapsed().as_micros() as u64
}

/// Simulates sequential memory access (baseline comparison)
fn simulate_sequential_read(buffer: &[f32]) -> u64 {
    let start = Instant::now();

    let mut checksum = 0u64;
    for &value in buffer.iter() {
        checksum = checksum.wrapping_add(value as u64);
    }

    std::hint::black_box(checksum);

    start.elapsed().as_micros() as u64
}

fn create_buffer(creature_count: usize) -> Vec<f32> {
    let buffer_size = creature_count * FIELDS_PER_CREATURE;
    let mut buffer = vec![0.0f32; buffer_size];

    // Populate with realistic data (SoA layout)
    let id_offset = 0;
    let x_offset = creature_count;
    let y_offset = creature_count * 2;
    let rot_offset = creature_count * 3;

    for i in 0..creature_count {
        buffer[id_offset + i] = i as f32;
        buffer[x_offset + i] = (i % 1000) as f32;
        buffer[y_offset + i] = (i / 1000) as f32;
        buffer[rot_offset + i] = (i as f32 * 0.01) % 6.28;
    }

    buffer
}

#[test]
fn benchmark_buffer_read_27_5k_creatures() {
    const CREATURE_COUNT: usize = 27_500;
    const WARMUP_ITERATIONS: usize = 10;
    const BENCHMARK_ITERATIONS: usize = 100;

    let buffer = create_buffer(CREATURE_COUNT);

    println!("\n=== BUFFER TRANSFER BENCHMARK: 27.5K CREATURES ===");
    println!("Buffer size: {} f32s ({} KB)", buffer.len(), (buffer.len() * 4) / 1024);
    println!("Warmup iterations: {}", WARMUP_ITERATIONS);
    println!("Benchmark iterations: {}", BENCHMARK_ITERATIONS);

    // Warmup
    for _ in 0..WARMUP_ITERATIONS {
        simulate_buffer_read(&buffer, CREATURE_COUNT);
    }

    // Benchmark
    let mut timings = Vec::with_capacity(BENCHMARK_ITERATIONS);
    for _ in 0..BENCHMARK_ITERATIONS {
        let elapsed = simulate_buffer_read(&buffer, CREATURE_COUNT);
        timings.push(elapsed);
    }

    let avg = timings.iter().sum::<u64>() / timings.len() as u64;
    let min = *timings.iter().min().unwrap();
    let max = *timings.iter().max().unwrap();

    println!("\nZero-copy buffer read (SoA layout):");
    println!("  Average: {} μs", avg);
    println!("  Min: {} μs", min);
    println!("  Max: {} μs", max);

    // Sequential access comparison
    let mut seq_timings = Vec::with_capacity(BENCHMARK_ITERATIONS);
    for _ in 0..BENCHMARK_ITERATIONS {
        let elapsed = simulate_sequential_read(&buffer);
        seq_timings.push(elapsed);
    }

    let seq_avg = seq_timings.iter().sum::<u64>() / seq_timings.len() as u64;

    println!("\nSequential memory scan (baseline):");
    println!("  Average: {} μs", seq_avg);

    println!("\nOverhead (SoA vs sequential): {} μs", avg.saturating_sub(seq_avg));
    println!("=== END BENCHMARK ===\n");

    // Sanity check: buffer read should be < 1ms for 27.5K creatures
    assert!(
        avg < 1000,
        "Buffer read overhead should be < 1ms, got {} μs",
        avg
    );
}

#[test]
fn benchmark_buffer_read_100k_creatures() {
    const CREATURE_COUNT: usize = 100_000;
    const BENCHMARK_ITERATIONS: usize = 50;

    let buffer = create_buffer(CREATURE_COUNT);

    println!("\n=== BUFFER TRANSFER BENCHMARK: 100K CREATURES ===");
    println!("Buffer size: {} f32s ({} MB)", buffer.len(), (buffer.len() * 4) / (1024 * 1024));

    let mut timings = Vec::with_capacity(BENCHMARK_ITERATIONS);
    for _ in 0..BENCHMARK_ITERATIONS {
        let elapsed = simulate_buffer_read(&buffer, CREATURE_COUNT);
        timings.push(elapsed);
    }

    let avg = timings.iter().sum::<u64>() / timings.len() as u64;
    let min = *timings.iter().min().unwrap();
    let max = *timings.iter().max().unwrap();

    println!("  Average: {} μs", avg);
    println!("  Min: {} μs", min);
    println!("  Max: {} μs", max);
    println!("=== END BENCHMARK ===\n");

    assert!(
        avg < 3000,
        "Buffer read overhead should be < 3ms for 100K, got {} μs",
        avg
    );
}

#[test]
fn benchmark_buffer_read_150k_creatures() {
    const CREATURE_COUNT: usize = 150_000;
    const BENCHMARK_ITERATIONS: usize = 50;

    let buffer = create_buffer(CREATURE_COUNT);

    println!("\n=== BUFFER TRANSFER BENCHMARK: 150K CREATURES (TARGET) ===");
    println!("Buffer size: {} f32s ({} MB)", buffer.len(), (buffer.len() * 4) / (1024 * 1024));

    let mut timings = Vec::with_capacity(BENCHMARK_ITERATIONS);
    for _ in 0..BENCHMARK_ITERATIONS {
        let elapsed = simulate_buffer_read(&buffer, CREATURE_COUNT);
        timings.push(elapsed);
    }

    let avg = timings.iter().sum::<u64>() / timings.len() as u64;
    let min = *timings.iter().min().unwrap();
    let max = *timings.iter().max().unwrap();

    println!("  Average: {} μs", avg);
    println!("  Min: {} μs", min);
    println!("  Max: {} μs", max);
    println!("=== END BENCHMARK ===\n");

    assert!(
        avg < 5000,
        "Buffer read overhead should be < 5ms for 150K, got {} μs",
        avg
    );
}

#[test]
fn benchmark_buffer_read_200k_creatures() {
    const CREATURE_COUNT: usize = 200_000;
    const BENCHMARK_ITERATIONS: usize = 50;

    let buffer = create_buffer(CREATURE_COUNT);

    println!("\n=== BUFFER TRANSFER BENCHMARK: 200K CREATURES (STRETCH) ===");
    println!("Buffer size: {} f32s ({} MB)", buffer.len(), (buffer.len() * 4) / (1024 * 1024));

    let mut timings = Vec::with_capacity(BENCHMARK_ITERATIONS);
    for _ in 0..BENCHMARK_ITERATIONS {
        let elapsed = simulate_buffer_read(&buffer, CREATURE_COUNT);
        timings.push(elapsed);
    }

    let avg = timings.iter().sum::<u64>() / timings.len() as u64;
    let min = *timings.iter().min().unwrap();
    let max = *timings.iter().max().unwrap();

    println!("  Average: {} μs", avg);
    println!("  Min: {} μs", min);
    println!("  Max: {} μs", max);
    println!("=== END BENCHMARK ===\n");

    assert!(
        avg < 7000,
        "Buffer read overhead should be < 7ms for 200K, got {} μs",
        avg
    );
}

#[test]
fn benchmark_cache_locality_soa_vs_aos() {
    const CREATURE_COUNT: usize = 50_000;
    const ITERATIONS: usize = 50;

    println!("\n=== CACHE LOCALITY: SoA vs AoS ===");

    // SoA layout: [ID1, ID2, ..., X1, X2, ..., Y1, Y2, ..., Rot1, Rot2, ...]
    let soa_buffer = create_buffer(CREATURE_COUNT);

    // AoS layout: [ID1, X1, Y1, Rot1, ID2, X2, Y2, Rot2, ...]
    let mut aos_buffer = vec![0.0f32; CREATURE_COUNT * FIELDS_PER_CREATURE];
    for i in 0..CREATURE_COUNT {
        aos_buffer[i * 4 + 0] = i as f32;
        aos_buffer[i * 4 + 1] = (i % 1000) as f32;
        aos_buffer[i * 4 + 2] = (i / 1000) as f32;
        aos_buffer[i * 4 + 3] = (i as f32 * 0.01) % 6.28;
    }

    // Benchmark SoA access
    let mut soa_timings = Vec::with_capacity(ITERATIONS);
    for _ in 0..ITERATIONS {
        let elapsed = simulate_buffer_read(&soa_buffer, CREATURE_COUNT);
        soa_timings.push(elapsed);
    }
    let soa_avg = soa_timings.iter().sum::<u64>() / soa_timings.len() as u64;

    // Benchmark AoS access (interleaved)
    let mut aos_timings = Vec::with_capacity(ITERATIONS);
    for _ in 0..ITERATIONS {
        let start = Instant::now();
        let mut checksum = 0u64;
        for i in 0..CREATURE_COUNT {
            let id = aos_buffer[i * 4 + 0];
            let x = aos_buffer[i * 4 + 1];
            let y = aos_buffer[i * 4 + 2];
            let rot = aos_buffer[i * 4 + 3];
            checksum = checksum.wrapping_add(id as u64 + x as u64 + y as u64 + rot as u64);
        }
        std::hint::black_box(checksum);
        aos_timings.push(start.elapsed().as_micros() as u64);
    }
    let aos_avg = aos_timings.iter().sum::<u64>() / aos_timings.len() as u64;

    println!("SoA layout average: {} μs", soa_avg);
    println!("AoS layout average: {} μs", aos_avg);

    let improvement = if soa_avg < aos_avg {
        ((aos_avg - soa_avg) as f64 / aos_avg as f64) * 100.0
    } else {
        0.0
    };

    println!("SoA improvement: {:.1}%", improvement);
    println!("=== END CACHE LOCALITY TEST ===\n");

    // SoA should be at least as fast as AoS (may be faster due to cache locality)
    // Allow some variance due to CPU scheduling
    assert!(
        soa_avg <= aos_avg * 2,
        "SoA should not be significantly slower than AoS"
    );
}

#[test]
fn test_buffer_size_scaling() {
    println!("\n=== BUFFER SIZE SCALING ANALYSIS ===");

    let sizes = vec![
        (10_000, "10K"),
        (27_500, "27.5K (current ceiling)"),
        (50_000, "50K"),
        (100_000, "100K"),
        (150_000, "150K (target)"),
        (200_000, "200K (stretch)"),
    ];

    for (creature_count, label) in sizes {
        let buffer = create_buffer(creature_count);
        let buffer_size_kb = (buffer.len() * 4) / 1024;

        let mut timings = Vec::new();
        for _ in 0..20 {
            let elapsed = simulate_buffer_read(&buffer, creature_count);
            timings.push(elapsed);
        }

        let avg = timings.iter().sum::<u64>() / timings.len() as u64;
        let per_creature_ns = (avg * 1000) / creature_count as u64;

        println!(
            "{:25} | Buffer: {:4} KB | Avg: {:5} μs | Per creature: {:3} ns",
            label, buffer_size_kb, avg, per_creature_ns
        );
    }

    println!("=== END SCALING ANALYSIS ===\n");
}
