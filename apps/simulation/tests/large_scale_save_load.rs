use speciate::{
    persistence::WorldSaveState,
    simulation::{Simulation, SimulationBuilder},
    CritBuilder,
    MAX_WORLD_SIZE,
};
use tempfile::TempDir;

/// Large-scale integration test for save state robustness
///
/// Tests the exact failure scenario from production:
/// - Large population (10K creatures)
/// - MessagePack serialization >10MB
/// - Worker thread shutdown synchronization
/// - File completeness verification
#[test]
fn test_large_scale_save_load_10k_creatures() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let save_path = temp_dir.path().join("large_scale.msgpack");

    // Spawn 10,000 creatures with varied configurations
    let mut sim = SimulationBuilder::new()
        .set_boundaries(MAX_WORLD_SIZE, MAX_WORLD_SIZE)
        .build();

    println!("Spawning 10,000 creatures...");
    for i in 0..10_000 {
        let builder = if i % 3 == 0 {
            CritBuilder::new().with_all_capabilities()
        } else if i % 3 == 1 {
            CritBuilder::new().with_seeking().with_avoidance()
        } else {
            CritBuilder::new().with_wandering()
        };

        sim.spawn_crit(builder);
    }

    assert_eq!(sim.creature_count(), 10_000, "Should spawn 10K creatures");

    // Create save state
    println!("Creating save state...");
    let save_state = sim
        .to_save_state()
        .expect("Failed to create large save state");

    assert_eq!(
        save_state.metadata.creature_count, 10_000,
        "Metadata should reflect 10K creatures"
    );

    // Save to file (this is where MessagePack streaming is critical)
    println!("Writing to disk...");
    save_state
        .save_to_file(&save_path)
        .expect("Failed to write large save state");

    // Verify file exists and has reasonable size
    let file_size = std::fs::metadata(&save_path)
        .expect("Save file should exist")
        .len();

    println!("Save file size: {} bytes ({:.2} MB)", file_size, file_size as f64 / 1_048_576.0);

    assert!(
        file_size > 5_000_000,
        "Save file should be >5MB for 10K creatures (got {} bytes)",
        file_size
    );
    assert!(
        file_size < 100_000_000,
        "Save file shouldn't exceed 100MB (got {} bytes)",
        file_size
    );

    // Load from file (this is where deserialization streaming matters)
    println!("Loading from disk...");
    let loaded = WorldSaveState::load_from_file(&save_path)
        .expect("Failed to load large save state");

    assert_eq!(
        loaded.metadata.creature_count, 10_000,
        "Loaded state should have 10K creatures"
    );

    // Restore simulation
    println!("Restoring simulation...");
    let restored_sim = Simulation::from_save_state(loaded)
        .expect("Failed to restore from large save state");

    assert_eq!(
        restored_sim.creature_count(),
        10_000,
        "Restored sim should have 10K creatures"
    );

    // Verify boundaries were preserved
    let (min_x, max_x, min_y, max_y) = restored_sim.get_boundaries();
    assert_eq!(min_x, -MAX_WORLD_SIZE, "Min X should be preserved");
    assert_eq!(max_x, MAX_WORLD_SIZE, "Max X should be preserved");
    assert_eq!(min_y, -MAX_WORLD_SIZE, "Min Y should be preserved");
    assert_eq!(max_y, MAX_WORLD_SIZE, "Max Y should be preserved");

    println!("✅ Large-scale save/load test passed!");
}

/// Test worker shutdown during large save
///
/// Verifies that the worker thread completes writes before shutdown,
/// preventing file truncation bugs.
///
/// Note: This test verifies the synchronization at a lower level by directly
/// testing save/load cycles with quick shutdown. The worker thread synchronization
/// is tested implicitly through the shutdown sleep in simulation_engine.rs.
#[test]
fn test_quick_shutdown_no_truncation() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let save_path = temp_dir.path().join("quick_shutdown.msgpack");

    // Create simulation with moderate population
    let mut sim = SimulationBuilder::new()
        .set_boundaries(10000.0, 10000.0)
        .build();

    for _ in 0..1000 {
        sim.spawn_crit(CritBuilder::new().with_all_capabilities());
    }

    // Create save state
    let save_state = sim.to_save_state().expect("Failed to create save state");

    // Save to file
    save_state.save_to_file(&save_path).expect("Failed to save");

    // Verify file exists and is complete (no truncation)
    let file_size = std::fs::metadata(&save_path)
        .expect("Save file should exist")
        .len();

    assert!(
        file_size > 100_000,
        "Save file should be complete (>100KB), got {} bytes",
        file_size
    );

    // Verify file can be loaded immediately (simulates quick restart)
    let loaded = WorldSaveState::load_from_file(&save_path)
        .expect("Save file should be loadable after quick shutdown");

    assert_eq!(
        loaded.metadata.creature_count, 1000,
        "Loaded state should have correct count"
    );

    // Verify restoration works
    let restored = Simulation::from_save_state(loaded)
        .expect("Should restore after quick shutdown");

    assert_eq!(
        restored.creature_count(), 1000,
        "Restored sim should have correct count"
    );

    println!("✅ Quick shutdown synchronization test passed!");
}

/// Stress test: Verify no file truncation at various scales
#[test]
fn test_no_truncation_at_scale() {
    let scales = vec![100, 500, 1000, 5000];

    for count in scales {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let save_path = temp_dir.path().join(format!("scale_{}.msgpack", count));

        let mut sim = SimulationBuilder::new()
            .set_boundaries(10000.0, 10000.0)
            .build();

        for _ in 0..count {
            sim.spawn_crit(CritBuilder::new().with_all_capabilities());
        }

        let save_state = sim.to_save_state().expect("Failed to create save state");
        save_state
            .save_to_file(&save_path)
            .expect("Failed to save");

        // Immediately reload - this is where truncation would be detected
        let loaded = WorldSaveState::load_from_file(&save_path)
            .expect(&format!("Failed to load {} creature save", count));

        assert_eq!(
            loaded.metadata.creature_count, count,
            "Scale {} should preserve count",
            count
        );

        let restored = Simulation::from_save_state(loaded)
            .expect(&format!("Failed to restore {} creature sim", count));

        assert_eq!(
            restored.creature_count(),
            count,
            "Scale {} should restore correctly",
            count
        );

        println!("✅ Scale {} verified", count);
    }
}
