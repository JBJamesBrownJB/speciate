/// End-to-end integration test for snapshot + NATS publishing
///
/// This test verifies that agents loaded from snapshots are correctly published to NATS.
///
/// Bug being tested: When loading from snapshots, the AgentId component was not being
/// inserted on restored entities, causing the NATS publishing query to return zero agents
/// even though the entities existed in the ECS.
///
/// Test flow:
/// 1. Create simulation and spawn agents
/// 2. Verify NATS publishes agents from original simulation
/// 3. Save snapshot to file
/// 4. Load snapshot and restore simulation
/// 5. Verify NATS publishes agents from restored simulation (this would fail before fix)
/// 6. Verify agent IDs are preserved after restoration
///
/// Prerequisites:
/// - NATS server must be running on nats://nats:4222
///
/// Run with: cargo test --test snapshot_nats_integration -- --nocapture

use speciate::simulation::{Simulation, SimulationBuilder};
use speciate::nats::frame::SimulationFrame;
use speciate::snapshot::WorldSnapshot;
use async_nats;
use futures::StreamExt;
use std::path::PathBuf;
use std::fs;

#[tokio::test]
async fn test_snapshot_restore_with_nats_publishing() {
    // Enable logging for debugging
    let _ = env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .try_init();

    println!("\n=== Snapshot + NATS Integration Test ===\n");

    // Step 1: Connect to NATS
    println!("1. Connecting to NATS...");
    let nats_url = std::env::var("NATS_URL")
        .unwrap_or_else(|_| "nats://nats:4222".to_string());
    let nats_client = async_nats::connect(&nats_url)
        .await
        .expect("Failed to connect to NATS - is the server running?");
    let mut subscriber = nats_client
        .subscribe("speciate.agents.transform")
        .await
        .expect("Failed to subscribe to NATS topic");
    println!("   ✓ Connected to NATS at {}", nats_url);

    // Step 2: Create original simulation and spawn agents
    println!("2. Creating simulation and spawning agents...");
    let mut sim = SimulationBuilder::new()
        .set_boundaries(180.0, 130.0)
        .build();

    let agent_ids = vec![
        sim.spawn_creature(50.0, 50.0, 180.0, 130.0),
        sim.spawn_creature(90.0, 65.0, 180.0, 130.0),
        sim.spawn_creature(130.0, 90.0, 180.0, 130.0),
    ];

    println!("   ✓ Spawned {} agents: {:?}", agent_ids.len(), agent_ids);

    // Step 3: Wait for NATS publisher to connect
    println!("3. Waiting for NATS publisher to connect...");
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    println!("   ✓ Publisher should be ready");

    // Step 4: Run original simulation and verify NATS publishes correctly
    println!("4. Running original simulation...");
    sim.update(0.016); // One frame at ~60 FPS
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    println!("5. Reading message from original simulation...");
    let msg1 = tokio::time::timeout(
        tokio::time::Duration::from_secs(2),
        subscriber.next()
    )
    .await
    .expect("Timeout waiting for NATS message")
    .expect("No message received from NATS");

    let frame1 = SimulationFrame::from_msgpack_bytes(&msg1.payload)
        .expect("Failed to parse MessagePack");

    assert_eq!(
        frame1.agents.len(),
        3,
        "Original simulation should publish 3 agents"
    );
    println!("   ✓ Original simulation published {} agents", frame1.agents.len());

    // Step 6: Save snapshot to file
    println!("6. Saving snapshot...");
    let snapshot = sim.to_snapshot();
    let snapshot_path = PathBuf::from("/tmp/test_snapshot_nats.msgpack");
    snapshot
        .save_to_file(&snapshot_path)
        .expect("Failed to save snapshot");
    println!("   ✓ Snapshot saved to {:?}", snapshot_path);

    // Step 7: Drop original simulation
    println!("7. Dropping original simulation...");
    drop(sim);
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    println!("   ✓ Original simulation dropped");

    // Step 8: Load snapshot and restore simulation
    println!("8. Loading snapshot and restoring simulation...");
    let loaded_snapshot = WorldSnapshot::load_from_file(&snapshot_path)
        .expect("Failed to load snapshot");
    let mut restored_sim = Simulation::from_snapshot(loaded_snapshot);

    assert_eq!(
        restored_sim.creature_count(),
        3,
        "Restored simulation should have 3 creatures"
    );
    println!("   ✓ Restored simulation has {} agents", restored_sim.creature_count());

    // Step 9: Wait for restored NATS publisher to connect
    println!("9. Waiting for restored NATS publisher to connect...");
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    println!("   ✓ Restored publisher should be ready");

    // Step 10: Run restored simulation
    println!("10. Running restored simulation...");
    restored_sim.update(0.016);
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // Step 11: Verify restored simulation publishes correctly to NATS
    println!("11. Reading message from restored simulation...");
    let msg2 = tokio::time::timeout(
        tokio::time::Duration::from_secs(2),
        subscriber.next()
    )
    .await
    .expect("Timeout waiting for NATS message")
    .expect("No message received from NATS");

    let frame2 = SimulationFrame::from_msgpack_bytes(&msg2.payload)
        .expect("Failed to parse MessagePack");

    // THIS IS THE KEY ASSERTION - verifies the bug is fixed
    assert_eq!(
        frame2.agents.len(),
        3,
        "❌ BUG: Restored simulation should publish 3 agents to NATS! \
         This fails if AgentId component is not inserted during snapshot restoration."
    );
    println!("   ✓ Restored simulation published {} agents", frame2.agents.len());

    // Step 12: Verify agent IDs are preserved after restoration
    println!("12. Verifying agent IDs are preserved...");
    let restored_ids: Vec<u32> = frame2.agents.iter().map(|a| a.id).collect();
    for id in &agent_ids {
        assert!(
            restored_ids.contains(id),
            "Agent ID {} should be preserved after restore",
            id
        );
    }
    println!("   ✓ All agent IDs preserved: {:?}", restored_ids);

    // Cleanup
    println!("13. Cleaning up...");
    fs::remove_file(&snapshot_path).ok();
    println!("   ✓ Snapshot file removed");

    println!("\n✅ TEST PASSED: Snapshot restore + NATS integration working!\n");
}
