//! Standalone profiling binary for branch misprediction analysis.
//!
//! Build: cargo build --release --features test-helpers --test branch_profile
//! Run:   ./target/release/deps/branch_profile-* --nocapture
//!
//! Profile with perf:
//!   perf stat -e branches,branch-misses ./target/release/deps/branch_profile-*
//!   perf record -e branch-misses --call-graph dwarf ./target/release/deps/branch_profile-*
//!   perf report --no-children --stdio

use rand::Rng;
use speciate::{BehaviorMode, CritBuilder, Simulation, SimulationBuilder};

const CREATURE_COUNT: usize = 200_000;
const TICK_COUNT: usize = 100;
const TICK_DELTA: f32 = 0.045;

#[test]
fn profile_branch_misprediction() {
    eprintln!("=== Branch Misprediction Profiling ===");
    eprintln!("Spawning {} creatures...", CREATURE_COUNT);

    let mut sim = SimulationBuilder::new()
        .set_boundaries(5000.0, 5000.0) // 10k x 10k world
        .build();
    let mut rng = rand::thread_rng();

    for i in 0..CREATURE_COUNT {
        let x = (rng.gen::<f32>() - 0.5) * 10000.0;
        let y = (rng.gen::<f32>() - 0.5) * 10000.0;
        let builder = CritBuilder::new()
            .at(x, y)
            .with_all_capabilities()
            .in_behavior(BehaviorMode::Wandering);
        sim.spawn_crit(builder);

        if i % 50000 == 0 && i > 0 {
            eprintln!("  Spawned {}/{}", i, CREATURE_COUNT);
        }
    }

    eprintln!("Running {} ticks...", TICK_COUNT);
    for i in 0..TICK_COUNT {
        sim.update(TICK_DELTA);
        if i % 10 == 0 {
            eprintln!("  Tick {}/{}", i, TICK_COUNT);
        }
    }
    eprintln!("=== Done ===");
}
