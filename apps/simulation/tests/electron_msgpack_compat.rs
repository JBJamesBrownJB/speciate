/// Test MessagePack compatibility with Electron's msgpack-lite
///
/// This verifies that the exact bytes Electron sends can be decoded by Rust.
/// We manually encode the same structure Electron would send and verify deserialization.

use speciate::ipc::Command;

#[test]
#[cfg(feature = "dev-tools")]
fn test_msgpack_format_matches_electron() {
    // This is the exact MessagePack format that msgpack-lite in Electron produces
    // for: { type: 'dev_spawn_creature', x: 100.0, y: 200.0, dna: null }
    //
    // msgpack-lite uses map format (not array), field names included
    // Format: fixmap(3) + string fields + values

    let command = Command::DevSpawnCreature {
        x: 100.0,
        y: 200.0,
        dna: None,
    };

    // Encode using Command::to_msgpack (uses map format for Electron compatibility)
    let bytes = command.to_msgpack().unwrap();

    // Decode back to verify round-trip
    let decoded = Command::from_msgpack(&bytes).unwrap();

    assert_eq!(decoded, command);

    // Verify it's using map format (0x80-0x8F are fixmap)
    println!("First byte: 0x{:02x}", bytes[0]);
    assert!(
        bytes[0] >= 0x80 && bytes[0] <= 0x8F,
        "Should use fixmap format, got: 0x{:02x}",
        bytes[0]
    );
}

#[test]
#[cfg(feature = "dev-tools")]
fn test_dev_load_trial_msgpack() {
    let command = Command::DevLoadTrial {
        template: "default-spawn-baseline".to_string(),
    };

    let bytes = command.to_msgpack().unwrap();
    let decoded = Command::from_msgpack(&bytes).unwrap();

    assert_eq!(decoded, command);
}

#[test]
#[cfg(feature = "dev-tools")]
fn test_spawn_command_with_dna() {
    use serde_json::json;

    let command = Command::DevSpawnCreature {
        x: 50.0,
        y: 75.0,
        dna: Some(json!({"size": 10.0})),
    };

    let bytes = command.to_msgpack().unwrap();
    let decoded = Command::from_msgpack(&bytes).unwrap();

    assert_eq!(decoded, command);
}
