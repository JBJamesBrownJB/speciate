use speciate::nats::frame::{AgentTransform, SimulationFrame};

fn main() {
    let frame = SimulationFrame::new(
        12450,
        vec![
            AgentTransform {
                id: 1,
                x: 45.23,
                y: 78.91,
                vx: 2.15,
                vy: -0.87,
                rotation: 1.57,
            },
            AgentTransform {
                id: 2,
                x: 120.50,
                y: 34.12,
                vx: 0.0,
                vy: 1.42,
                rotation: 0.0,
            },
        ],
    );

    let json = serde_json::to_string_pretty(&frame).unwrap();
    println!("Sample NATS message:");
    println!("{}", json);

    // Verify timestamp format
    if json.contains("\"timestamp\":\"") && json.contains("T") && json.contains("Z") {
        println!("\n✅ Timestamp is in ISO 8601 format (string)");
    } else {
        println!("\n❌ Timestamp is NOT in correct format");
    }

    // Verify agent IDs are present
    if json.contains("\"id\":1") && json.contains("\"id\":2") {
        println!("✅ Agent IDs are present");
    } else {
        println!("❌ Agent IDs are missing");
    }
}
