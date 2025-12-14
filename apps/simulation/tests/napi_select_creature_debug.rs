//! Integration test for NAPI SelectCreatureDebug command flow
//!
//! Tests the complete pipeline from command receipt to perception buffer export.

#[cfg(feature = "dev-tools")]
use parking_lot::Mutex;
#[cfg(feature = "dev-tools")]
use speciate::ipc::bridge::{NapiApp, PerceptionDebugBuffer};
#[cfg(feature = "dev-tools")]
use speciate::ipc::SimCommand;
#[cfg(feature = "dev-tools")]
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
fn test_acceleration_is_captured_for_active_creature() {
    // This test verifies that the capture_debug_acceleration_system
    // correctly captures non-zero acceleration for wandering creatures.

    // 1. Create NapiApp with creatures in Wandering mode
    let (tx, rx) = crossbeam_channel::bounded(128);
    let mut app = NapiApp::new(rx, 5, ".".to_string(), None);

    // Run initial tick
    app.update(0.045);

    // 2. Get first creature's CritId
    let crit_id = {
        let save_state = app.to_save_state().expect("Failed to get save state");
        eprintln!("Entity map has {} entries", save_state.entity_id_map.len());
        save_state.entity_id_map[0].1
    };

    // 3. Select the creature
    tx.send(SimCommand::SelectCreatureDebug(Some(crit_id)))
        .expect("Failed to send command");
    app.process_commands();

    // 4. Run multiple ticks and collect acceleration values
    let perception_buffer = Arc::new(Mutex::new(PerceptionDebugBuffer::new()));
    let mut found_nonzero_accel = false;
    let mut debug_info = String::new();

    for tick in 0..20 {
        app.update(0.045);

        // Check system timings to see if capture_debug_accel ran
        let telemetry = app.get_telemetry(tick as u64, 22.2);
        if tick < 3 {
            eprintln!(
                "Tick {} timings: wander={}us, capture_debug_accel={}us",
                tick,
                telemetry.system_timings.wander_us,
                telemetry.system_timings.capture_debug_accel_us
            );
        }

        app.export_perception_debug(&perception_buffer);
        perception_buffer.lock().swap();

        let guard = perception_buffer.lock();
        let read_slice = guard.get_read_slice();

        // Check if we have valid data
        let has_data = read_slice[0] > 0.5;
        let entity_id = read_slice[1] as u32;
        let x = read_slice[2];
        let y = read_slice[3];
        let ax = read_slice[7];
        let ay = read_slice[8];
        let accel_magnitude = (ax * ax + ay * ay).sqrt();

        if tick < 5 {
            debug_info.push_str(&format!(
                "Tick {}: has_data={}, entity_id={}, pos=({:.1},{:.1}), accel=({:.4},{:.4}), mag={:.4}\n",
                tick, has_data, entity_id, x, y, ax, ay, accel_magnitude
            ));
        }

        if has_data && accel_magnitude > 0.001 {
            found_nonzero_accel = true;
            eprintln!("Tick {} found non-zero acceleration: ax={:.4}, ay={:.4}, mag={:.4}", tick, ax, ay, accel_magnitude);
            break;
        }
    }

    if !found_nonzero_accel {
        eprintln!("Debug info for first 5 ticks:\n{}", debug_info);
    }

    assert!(
        found_nonzero_accel,
        "Expected non-zero acceleration for wandering creature within 20 ticks. \
         Check that capture_debug_acceleration_system runs after behavior systems."
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
