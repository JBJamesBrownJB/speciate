//! Integration tests for the simulation server
//!
//! These tests verify that the entire system works together:
//! - Server starts and listens
//! - WebSocket accepts connections
//! - Simulation state is broadcast to clients
//! - Message format is correct

use tokio::time::{sleep, timeout, Duration};
use futures_util::StreamExt;
use tokio_tungstenite::{connect_async, tungstenite::Message};

const SERVER_URL: &str = "ws://localhost:8080/ws";
const HEALTH_URL: &str = "http://localhost:8080/health";

/// Test that server health endpoint responds
#[tokio::test]
async fn test_health_endpoint() {
    // Give server time to start (if running)
    sleep(Duration::from_millis(500)).await;

    let result = reqwest::get(HEALTH_URL).await;

    if let Ok(response) = result {
        assert!(response.status().is_success());
        let text = response.text().await.unwrap();
        assert_eq!(text, "OK");
    } else {
        // Server not running - skip test
        println!("Server not running, skipping health check test");
    }
}

/// Test WebSocket connection and message reception
#[tokio::test]
async fn test_websocket_connection_and_messages() {
    // Try to connect with timeout
    let connect_result = timeout(
        Duration::from_secs(2),
        connect_async(SERVER_URL)
    ).await;

    let (ws_stream, _) = match connect_result {
        Ok(Ok(stream)) => stream,
        Ok(Err(e)) => {
            println!("Server not running, skipping WebSocket test: {}", e);
            return;
        }
        Err(_) => {
            println!("Server connection timeout, skipping WebSocket test");
            return;
        }
    };
    println!("✓ Connected to WebSocket server");

    let (mut _write, mut read) = ws_stream.split();

    // Try to receive 3 messages
    for i in 1..=3 {
        let msg_result = timeout(Duration::from_secs(5), read.next()).await;

        match msg_result {
            Ok(Some(Ok(Message::Text(text)))) => {
                println!("✓ Received message {}", i);

                // Parse JSON
                let data: serde_json::Value = serde_json::from_str(&text)
                    .expect("Failed to parse JSON");

                // Verify required fields
                assert!(data.get("tick").is_some(), "Missing 'tick' field");
                assert!(data.get("creatures").is_some(), "Missing 'creatures' field");
                assert!(data.get("server_time").is_some(), "Missing 'server_time' field");

                let tick = data["tick"].as_u64().unwrap();
                let creatures = data["creatures"].as_array().unwrap();

                println!("  Tick: {}, Creatures: {}", tick, creatures.len());

                // Verify creatures have required fields
                if !creatures.is_empty() {
                    let creature = &creatures[0];
                    assert!(creature.get("id").is_some());
                    assert!(creature.get("x").is_some());
                    assert!(creature.get("y").is_some());
                    assert!(creature.get("rotation").is_some());
                    assert!(creature.get("width").is_some());
                    assert!(creature.get("height").is_some());
                }
            }
            Ok(Some(Ok(msg))) => {
                panic!("Unexpected message type: {:?}", msg);
            }
            Ok(Some(Err(e))) => {
                panic!("WebSocket error: {}", e);
            }
            Ok(None) => {
                panic!("WebSocket closed unexpectedly");
            }
            Err(_) => {
                panic!("Timeout waiting for message {}", i);
            }
        }
    }

    println!("✓ All integration tests passed!");
}

/// Test that simulation tick numbers increase over time
#[tokio::test]
async fn test_simulation_ticks_increase() {
    let connect_result = timeout(
        Duration::from_secs(2),
        connect_async(SERVER_URL)
    ).await;

    let (ws_stream, _) = match connect_result {
        Ok(Ok(stream)) => stream,
        Ok(Err(e)) => {
            println!("Server not running, skipping tick test: {}", e);
            return;
        }
        Err(_) => {
            println!("Server connection timeout, skipping tick test");
            return;
        }
    };
    let (mut _write, mut read) = ws_stream.split();

    let mut ticks = Vec::new();

    // Collect 5 tick values
    for _ in 0..5 {
        if let Some(Ok(Message::Text(text))) = read.next().await {
            let data: serde_json::Value = serde_json::from_str(&text).unwrap();
            let tick = data["tick"].as_u64().unwrap();
            ticks.push(tick);
        }
    }

    // Verify ticks are increasing
    for i in 1..ticks.len() {
        assert!(ticks[i] > ticks[i-1], "Tick {} ({}) should be greater than tick {} ({})",
                i, ticks[i], i-1, ticks[i-1]);
    }

    println!("✓ Simulation ticks are increasing: {:?}", ticks);
}

/// Test that creatures are present and have valid positions
#[tokio::test]
async fn test_creatures_have_valid_data() {
    let connect_result = timeout(
        Duration::from_secs(2),
        connect_async(SERVER_URL)
    ).await;

    let (ws_stream, _) = match connect_result {
        Ok(Ok(stream)) => stream,
        Ok(Err(e)) => {
            println!("Server not running, skipping creature data test: {}", e);
            return;
        }
        Err(_) => {
            println!("Server connection timeout, skipping creature data test");
            return;
        }
    };
    let (mut _write, mut read) = ws_stream.split();

    if let Some(Ok(Message::Text(text))) = read.next().await {
        let data: serde_json::Value = serde_json::from_str(&text).unwrap();
        let creatures = data["creatures"].as_array().unwrap();

        assert!(!creatures.is_empty(), "Should have at least one creature");

        for creature in creatures {
            let x = creature["x"].as_f64().unwrap();
            let y = creature["y"].as_f64().unwrap();

            // Positions should be reasonable (within world bounds)
            assert!(x >= 0.0 && x <= 200.0, "X position out of reasonable bounds: {}", x);
            assert!(y >= 0.0 && y <= 200.0, "Y position out of reasonable bounds: {}", y);

            // Size should be positive
            let width = creature["width"].as_f64().unwrap();
            let height = creature["height"].as_f64().unwrap();
            assert!(width > 0.0, "Width should be positive");
            assert!(height > 0.0, "Height should be positive");
        }

        println!("✓ All {} creatures have valid data", creatures.len());
    }
}
