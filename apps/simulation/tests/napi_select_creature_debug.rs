//! Integration test for NAPI SelectCreatureDebug command flow
//!
//! Tests the complete pipeline from command receipt to perception buffer export.

use parking_lot::Mutex;
use speciate::ipc::bridge::{NapiApp, PerceptionDebugBuffer};
use speciate::ipc::SimCommand;
use std::sync::Arc;

#[test]
#[cfg(feature = "dev-tools")]
fn test_select_creature_debug_command_populates_buffer() {
    // 1. Create NapiApp with 5 creatures
    let (tx, rx) = crossbeam_channel::bounded(128);
    let mut app = NapiApp::new(rx, 5, ".".to_string(), None);

    // 2. Run one tick to ensure creatures are fully initialized
    app.update(0.045);

    // 3. Get the first creature's CritId from entity_id_map
    let crit_id = {
        let save_state = app.to_save_state().expect("Failed to get save state");
        assert!(
            !save_state.entity_id_map.is_empty(),
            "Should have creatures in entity_id_map"
        );
        // entity_id_map is Vec<(entity_index, crit_id)>
        save_state.entity_id_map[0].1
    };

    // 4. Send SelectCreatureDebug command
    tx.send(SimCommand::SelectCreatureDebug(Some(crit_id)))
        .expect("Failed to send command");

    // 5. Process the command
    app.process_commands();

    // 6. Run several updates to let perception system populate snapshot
    for _ in 0..3 {
        app.update(0.045);
    }

    // 7. Create perception debug buffer and export
    let perception_buffer = Arc::new(Mutex::new(PerceptionDebugBuffer::new()));
    app.export_perception_debug(&perception_buffer);

    // 8. Swap buffers to read the written data
    perception_buffer.lock().swap();
    let guard = perception_buffer.lock();
    let read_slice = guard.get_read_slice();

    // 9. Verify the buffer has valid data (has_data flag should be 1.0)
    assert!(
        read_slice[0] > 0.5,
        "has_data flag should be 1.0 (got {}). Selection flow is broken.",
        read_slice[0]
    );

    // 10. Verify entity_id matches
    let buffer_entity_id = read_slice[1] as u32;
    assert_eq!(
        buffer_entity_id, crit_id,
        "Buffer entity_id should match selected creature"
    );

    // 11. Verify position data exists
    let x = read_slice[2];
    let y = read_slice[3];
    assert!(
        x.is_finite() && y.is_finite(),
        "Position should be finite numbers (x={}, y={})",
        x,
        y
    );
}

#[test]
#[cfg(feature = "dev-tools")]
fn test_select_creature_debug_null_clears_selection() {
    // 1. Create NapiApp with creatures
    let (tx, rx) = crossbeam_channel::bounded(128);
    let mut app = NapiApp::new(rx, 5, ".".to_string(), None);
    app.update(0.045);

    // 2. Get a creature's CritId and select it
    let crit_id = {
        let save_state = app.to_save_state().expect("Failed to get save state");
        save_state.entity_id_map[0].1
    };

    tx.send(SimCommand::SelectCreatureDebug(Some(crit_id)))
        .expect("Failed to send select command");
    app.process_commands();

    for _ in 0..3 {
        app.update(0.045);
    }

    // 3. Verify selection is active
    let perception_buffer = Arc::new(Mutex::new(PerceptionDebugBuffer::new()));
    app.export_perception_debug(&perception_buffer);
    perception_buffer.lock().swap();

    {
        let guard = perception_buffer.lock();
        let read_slice = guard.get_read_slice();
        assert!(
            read_slice[0] > 0.5,
            "Selection should be active before clearing"
        );
    }

    // 4. Send null to clear selection
    tx.send(SimCommand::SelectCreatureDebug(None))
        .expect("Failed to send clear command");
    app.process_commands();
    app.update(0.045);

    // 5. Export again and verify selection is cleared
    app.export_perception_debug(&perception_buffer);
    perception_buffer.lock().swap();

    let guard = perception_buffer.lock();
    let read_slice = guard.get_read_slice();
    assert!(
        read_slice[0] < 0.5,
        "has_data should be 0 after clearing selection (got {})",
        read_slice[0]
    );
}

#[test]
#[cfg(feature = "dev-tools")]
fn test_select_nonexistent_creature_returns_empty() {
    // 1. Create NapiApp with creatures
    let (tx, rx) = crossbeam_channel::bounded(128);
    let mut app = NapiApp::new(rx, 5, ".".to_string(), None);
    app.update(0.045);

    // 2. Send SelectCreatureDebug with a nonexistent CritId
    tx.send(SimCommand::SelectCreatureDebug(Some(999999)))
        .expect("Failed to send command");
    app.process_commands();
    app.update(0.045);

    // 3. Export and verify no data
    let perception_buffer = Arc::new(Mutex::new(PerceptionDebugBuffer::new()));
    app.export_perception_debug(&perception_buffer);
    perception_buffer.lock().swap();

    let guard = perception_buffer.lock();
    let read_slice = guard.get_read_slice();
    assert!(
        read_slice[0] < 0.5,
        "has_data should be 0 for nonexistent creature (got {})",
        read_slice[0]
    );
}
