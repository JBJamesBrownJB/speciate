/// End-to-end integration test for NATS publishing
///
/// This test verifies that creatures spawned in the simulation are correctly
/// published to NATS with all required data (AgentId, position, velocity, rotation).
///
/// Prerequisites:
/// - NATS server must be running on nats://localhost:4222
///
/// Run with: cargo test --test nats_e2e_test -- --nocapture

use speciate::simulation::SimulationBuilder;
use speciate::nats::frame::SimulationFrame;
use async_nats;
use futures::StreamExt;

#[tokio::test]
async fn test_nats_publishes_agent_data() {
    // Enable logging to see what's happening
    let _ = env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .try_init();

    println!("\n=== NATS E2E Test: Agent Data Publishing ===\n");

    // Step 1: Connect to NATS
    println!("1. Connecting to NATS...");
    let nats_url = std::env::var("NATS_URL").unwrap_or_else(|_| "nats://nats:4222".to_string());
    let nats_client = async_nats::connect(&nats_url)
        .await
        .expect("Failed to connect to NATS - is the server running?");
    println!("   ✓ Connected to NATS at {}", nats_url);

    // Step 2: Subscribe to simulation topic
    println!("2. Subscribing to speciate.agents.transform...");
    let mut subscriber = nats_client
        .subscribe("speciate.agents.transform")
        .await
        .expect("Failed to subscribe to NATS topic");
    println!("   ✓ Subscribed");

    // Step 3: Create simulation
    println!("3. Creating simulation...");
    let mut sim = SimulationBuilder::new()
        .set_boundaries(180.0, 130.0)
        .build();
    println!("   ✓ Simulation created (180x130)");

    // Step 4: Spawn a creature
    println!("4. Spawning test creature...");
    let agent_id = sim.spawn_creature(90.0, 65.0, 180.0, 130.0);
    println!("   ✓ Spawned creature with AgentId: {}", agent_id);

    // Step 5: Wait for NATS publisher to connect
    // The simulation builder already set up NATS, but the background thread needs time
    println!("5. Waiting for NATS publisher to connect...");
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    println!("   ✓ Publisher should be ready");

    // Step 6: Run a simulation update (which includes NATS publishing)
    println!("6. Running simulation update...");
    sim.update(0.016); // One frame at ~60 FPS
    println!("   ✓ Update executed (NATS publish should have happened)");

    // Step 7: Give the publisher thread a moment to publish
    println!("7. Waiting for message to be published...");
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Step 8: Read the message from NATS
    println!("8. Reading message from NATS...");
    let message = tokio::time::timeout(
        tokio::time::Duration::from_secs(2),
        subscriber.next()
    )
    .await
    .expect("Timeout waiting for NATS message")
    .expect("No message received from NATS");

    let payload = message.payload.to_vec();
    println!("   ✓ Received message ({} bytes)", payload.len());

    // Step 9: Parse and validate the message (MessagePack format)
    println!("9. Validating message structure...");
    let frame = SimulationFrame::from_msgpack_bytes(&payload)
        .expect("Failed to parse MessagePack");

    // Validate tick exists
    let tick = frame.tick;
    println!("   ✓ Tick: {}", tick);

    // Validate timestamp is ISO 8601 string
    let timestamp = frame.timestamp.to_rfc3339();
    assert!(timestamp.contains("T"), "Timestamp should be ISO 8601 format");
    // Accept both 'Z' and '+00:00' as UTC indicators
    assert!(timestamp.contains("Z") || timestamp.contains("+00:00"),
        "Timestamp should have UTC indicator (got: {})", timestamp);
    println!("   ✓ Timestamp: {}", timestamp);

    // Validate agents array
    let agents = &frame.agents;
    println!("   ✓ Agents array length: {}", agents.len());

    // THIS IS THE KEY ASSERTION - if this fails, the bug is confirmed
    assert_eq!(
        agents.len(),
        1,
        "❌ BUG CONFIRMED: Expected 1 agent in array, found {}. \
         This means spawned creatures are not being included in NATS messages!",
        agents.len()
    );

    // Step 10: Validate agent data structure
    println!("10. Validating agent data...");
    let agent = &agents[0];

    let id = agent.id;
    let x = agent.x as f64;
    let y = agent.y as f64;
    let vx = agent.vx as f64;
    let vy = agent.vy as f64;
    let rotation = agent.rotation as f64;

    println!("   ✓ Agent ID: {}", id);
    println!("   ✓ Position: ({}, {})", x, y);
    println!("   ✓ Velocity: ({}, {})", vx, vy);
    println!("   ✓ Rotation: {}", rotation);

    // Validate the agent ID matches what we spawned
    assert_eq!(id, agent_id, "Agent ID mismatch");

    // Validate position is roughly where we spawned (allowing for initial velocity)
    assert!((x - 90.0).abs() < 10.0, "X position too far from spawn point");
    assert!((y - 65.0).abs() < 10.0, "Y position too far from spawn point");

    println!("\n✅ TEST PASSED: Agent data is correctly published to NATS!\n");
}
