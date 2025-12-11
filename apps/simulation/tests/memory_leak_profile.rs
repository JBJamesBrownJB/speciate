//! Memory leak profiling test
//!
//! Run with: cargo test --features dhat-heap memory_leak_profile --release -- --nocapture
//! This will output dhat-heap.json which can be viewed at https://nnethercote.github.io/dh_view/dh_view.html

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

use speciate::SimulationBuilder;

#[test]
fn memory_leak_profile() {
    let _profiler = dhat::Profiler::new_heap();

    println!("\n=== Memory Leak Profiling ===\n");

    // Create simulation with 0 creatures
    let mut sim = SimulationBuilder::new().build();

    // Get initial memory
    let stats_start = dhat::HeapStats::get();
    println!("Initial: {} bytes in {} blocks", stats_start.curr_bytes, stats_start.curr_blocks);

    // Run 1000 ticks (simulates ~50 seconds at 20Hz)
    println!("\nRunning 1000 ticks with 0 creatures...");
    for tick in 0..1000 {
        sim.update(0.05); // 50ms delta

        if tick % 100 == 0 {
            let stats = dhat::HeapStats::get();
            println!("Tick {}: {} bytes ({} blocks), total allocs: {}",
                tick, stats.curr_bytes, stats.curr_blocks, stats.total_blocks);
        }
    }

    let stats_end = dhat::HeapStats::get();
    println!("\nFinal: {} bytes in {} blocks", stats_end.curr_bytes, stats_end.curr_blocks);
    println!("Growth: {} bytes", stats_end.curr_bytes as i64 - stats_start.curr_bytes as i64);
    println!("Total allocations: {}", stats_end.total_blocks);

    // If running with dhat, this will write dhat-heap.json
    println!("\n=== Profile written to dhat-heap.json ===");
    println!("View at: https://nnethercote.github.io/dh_view/dh_view.html");
}

// Note: memory_leak_profile_parallelization_only removed because dhat doesn't support
// multiple profilers in the same test run. Run manually with:
// cargo test --features dev-tools --test memory_leak_profile memory_leak_profile --release -- --nocapture
