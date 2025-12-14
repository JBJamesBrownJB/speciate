#![cfg(feature = "dev-tools")]

use speciate::instrumentation::SystemTimings;
use speciate::time_system;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;

#[test]
fn test_system_timings_resource_creation() {
    let timings = SystemTimings::new();
    assert_eq!(timings.movement_us.load(Ordering::Relaxed), 0);
    assert_eq!(timings.perception_us.load(Ordering::Relaxed), 0);
    assert_eq!(timings.behavior_us.load(Ordering::Relaxed), 0);
}

#[test]
fn test_timing_guard_records_elapsed_time() {
    let timings = SystemTimings::new();

    {
        let _guard = timings.time("movement");
        thread::sleep(Duration::from_micros(100));
    }

    let elapsed = timings.movement_us.load(Ordering::Relaxed);
    assert!(
        elapsed >= 100,
        "Timer should record at least 100us, got {}us",
        elapsed
    );
    assert!(
        elapsed < 10000,
        "Timer overhead too high: {}us",
        elapsed
    );
}

#[test]
fn test_multiple_systems_can_be_timed_independently() {
    let timings = SystemTimings::new();

    {
        let _guard = timings.time("movement");
        thread::sleep(Duration::from_micros(50));
    }

    {
        let _guard = timings.time("perception");
        thread::sleep(Duration::from_micros(100));
    }

    let movement = timings.movement_us.load(Ordering::Relaxed);
    let perception = timings.perception_us.load(Ordering::Relaxed);

    assert!(movement >= 50, "Movement timing: {}us", movement);
    assert!(perception >= 100, "Perception timing: {}us", perception);
    assert!(
        perception > movement,
        "Perception should take longer: {}us vs {}us",
        perception,
        movement
    );
}

#[test]
fn test_timing_overwrites_previous_value() {
    let timings = SystemTimings::new();

    {
        let _guard = timings.time("movement");
        thread::sleep(Duration::from_micros(200));
    }

    let first = timings.movement_us.load(Ordering::Relaxed);

    {
        let _guard = timings.time("movement");
        thread::sleep(Duration::from_micros(50));
    }

    let second = timings.movement_us.load(Ordering::Relaxed);

    assert!(second < first, "Second timing should overwrite: {}us vs {}us", second, first);
    assert!(second >= 50, "Second timing should be at least 50us: {}us", second);
}

#[test]
fn test_snapshot_returns_all_timings() {
    let timings = SystemTimings::new();

    {
        let _guard = timings.time("movement");
        thread::sleep(Duration::from_micros(10));
    }
    {
        let _guard = timings.time("perception");
        thread::sleep(Duration::from_micros(20));
    }

    let snapshot = timings.snapshot();

    assert!(snapshot.movement_us >= 10);
    assert!(snapshot.perception_us >= 20);
}

#[test]
fn test_time_system_macro_records_timing() {
    let timings = SystemTimings::new();

    {
        time_system!(timings, "movement");
        thread::sleep(Duration::from_micros(50));
    }

    let elapsed = timings.movement_us.load(Ordering::Relaxed);
    assert!(elapsed >= 50, "Macro should record timing: {}us", elapsed);
}

#[test]
fn test_time_system_macro_with_code_block() {
    let timings = SystemTimings::new();
    let mut result = 0;

    {
        time_system!(timings, "behavior");
        for i in 0..100 {
            result += i;
        }
    }

    let elapsed = timings.behavior_us.load(Ordering::Relaxed);
    assert!(elapsed > 0, "Macro should record non-zero timing");
    assert_eq!(result, 4950, "Code should execute normally");
}

#[test]
fn test_gamestate_includes_timing_fields() {
    use speciate::ipc::GameState;
    use speciate::instrumentation::SystemTimingsSnapshot;

    let state = GameState {
        protocol_version: 1,
        tick: 100,
        tick_rate_hz: 60.0,
        creatures: vec![],
        entity_count: 0,
        system_timings_us: SystemTimingsSnapshot {
            total_tick_us: 600,
            movement_us: 150,
            perception_us: 250,
            spatial_grid_rebuild_us: 50,
            behavior_us: 100,
            behavior_transition_us: 50,
            wander_us: 30,
            seek_us: 25,
            flee_us: 10,
            avoidance_us: 20,
            steering_cap_us: 3,
            rotation_us: 5,
            capture_debug_accel_us: 2,
            archetype_count: 5,
            entity_count: 100,
        },
        hardware_metrics: None,
        parallelization_metrics: None,
    };

    let serialized = rmp_serde::to_vec(&state).unwrap();
    let deserialized: GameState = rmp_serde::from_slice(&serialized).unwrap();

    assert_eq!(deserialized.tick, 100);
    assert_eq!(deserialized.entity_count, 0);
    assert_eq!(deserialized.system_timings_us.movement_us, 150);
    assert_eq!(deserialized.system_timings_us.perception_us, 250);
    assert_eq!(deserialized.system_timings_us.behavior_us, 100);
}
