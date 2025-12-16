#![cfg(feature = "dev-tools")]

use speciate::instrumentation::{
    HardwareSnapshot, PerformanceSnapshot, SystemTimingsSnapshot,
};
use serde_json;

#[test]
fn test_performance_snapshot_schema_completeness() {
    let hw_snapshot = HardwareSnapshot {
        cycles_delta: 3_000_000_000,
        instructions_delta: 3_600_000_000,
        cache_refs_delta: 100_000,
        cache_misses_delta: 2_000,
        l1d_misses_delta: 1_000,
        l1i_misses_delta: 500,
        branch_instructions_delta: 500_000,
        branch_misses_delta: 7_500,
        stalled_frontend_delta: 300_000_000,
        stalled_backend_delta: 600_000_000,
        ipc: 1.2,
        l1d_miss_rate: 2.0,
        l1i_miss_rate: 1.0,
        llc_miss_rate: 2.0,
        branch_miss_rate: 1.5,
        frontend_stall_ratio: 10.0,
        backend_stall_ratio: 20.0,
    };

    let sys_timings = SystemTimingsSnapshot {
        total_tick_us: 1500,
        movement_us: 200,
        perception_us: 300,
        spatial_grid_rebuild_us: 100,
        behavior_transition_us: 50,
        steering_us: 115, // Fused steering (Sprint 20)
        capture_debug_accel_us: 2,
        archetype_count: 5,
        entity_count: 27500,
    };

    let snapshot = PerformanceSnapshot::new(
        "post-napi-migration".to_string(),
        "Baseline snapshot before NAPI-RS migration".to_string(),
        27500,
        hw_snapshot.clone(),
        &sys_timings,
    );

    assert_eq!(snapshot.label, "post-napi-migration");
    assert_eq!(snapshot.creature_count, 27500);
    assert!(!snapshot.timestamp.is_empty(), "Timestamp should not be empty");
    assert!(
        !snapshot.git_commit.is_empty(),
        "Git commit should not be empty"
    );
    assert!(
        !snapshot.git_branch.is_empty(),
        "Git branch should not be empty"
    );
    assert!(
        snapshot.build_type == "debug" || snapshot.build_type == "release",
        "Build type should be 'debug' or 'release'"
    );

    assert_eq!(snapshot.hardware_metrics.ipc, 1.2);
    assert_eq!(snapshot.hardware_metrics.l1d_miss_rate, 2.0);
    assert_eq!(snapshot.hardware_metrics.llc_miss_rate, 2.0);
    assert_eq!(snapshot.hardware_metrics.branch_miss_rate, 1.5);

    assert_eq!(snapshot.ecs_metrics.entity_count, 27500);
    assert_eq!(snapshot.ecs_metrics.archetype_count, 5);
    assert_eq!(snapshot.ecs_metrics.system_tick_ms, 1.5);
}

#[test]
fn test_performance_snapshot_json_serialization_round_trip() {
    let hw_snapshot = HardwareSnapshot::default();
    let sys_timings = SystemTimingsSnapshot::default();

    let original = PerformanceSnapshot::new(
        "test-snapshot".to_string(),
        "Test description".to_string(),
        1000,
        hw_snapshot,
        &sys_timings,
    );

    let json = serde_json::to_string_pretty(&original)
        .expect("Should serialize to JSON");

    assert!(json.contains("\"timestamp\""), "JSON should contain timestamp");
    assert!(json.contains("\"label\""), "JSON should contain label");
    assert!(
        json.contains("\"description\""),
        "JSON should contain description"
    );
    assert!(json.contains("\"gitCommit\""), "JSON should contain gitCommit (camelCase)");
    assert!(json.contains("\"gitBranch\""), "JSON should contain gitBranch (camelCase)");
    assert!(json.contains("\"gitDirty\""), "JSON should contain gitDirty (camelCase)");
    assert!(
        json.contains("\"buildType\""),
        "JSON should contain buildType (camelCase)"
    );
    assert!(
        json.contains("\"rustVersion\""),
        "JSON should contain rustVersion (camelCase)"
    );
    assert!(
        json.contains("\"creatureCount\""),
        "JSON should contain creatureCount (camelCase)"
    );
    assert!(
        json.contains("\"hardwareMetrics\""),
        "JSON should contain hardwareMetrics (camelCase)"
    );
    assert!(
        json.contains("\"ecsMetrics\""),
        "JSON should contain ecsMetrics (camelCase)"
    );

    let deserialized: PerformanceSnapshot =
        serde_json::from_str(&json).expect("Should deserialize from JSON");

    assert_eq!(deserialized.label, original.label);
    assert_eq!(deserialized.creature_count, original.creature_count);
    assert_eq!(deserialized.git_commit, original.git_commit);
    assert_eq!(deserialized.git_branch, original.git_branch);
    assert_eq!(deserialized.build_type, original.build_type);
}

#[test]
fn test_hardware_metrics_all_fields_present_in_json() {
    let hw_snapshot = HardwareSnapshot {
        cycles_delta: 1000,
        instructions_delta: 1200,
        cache_refs_delta: 100,
        cache_misses_delta: 5,
        l1d_misses_delta: 10,
        l1i_misses_delta: 2,
        branch_instructions_delta: 200,
        branch_misses_delta: 3,
        stalled_frontend_delta: 50,
        stalled_backend_delta: 75,
        ipc: 1.2,
        l1d_miss_rate: 1.0,
        l1i_miss_rate: 0.5,
        llc_miss_rate: 5.0,
        branch_miss_rate: 1.5,
        frontend_stall_ratio: 5.0,
        backend_stall_ratio: 7.5,
    };

    let json =
        serde_json::to_string_pretty(&hw_snapshot).expect("Should serialize");

    let required_fields = vec![
        "cyclesDelta",
        "instructionsDelta",
        "cacheRefsDelta",
        "cacheMissesDelta",
        "l1dMissesDelta",
        "l1iMissesDelta",
        "branchInstructionsDelta",
        "branchMissesDelta",
        "stalledFrontendDelta",
        "stalledBackendDelta",
        "ipc",
        "l1dMissRate",
        "l1iMissRate",
        "llcMissRate",
        "branchMissRate",
        "frontendStallRatio",
        "backendStallRatio",
    ];

    for field in required_fields {
        assert!(
            json.contains(&format!("\"{}\"", field)),
            "JSON should contain field: {}",
            field
        );
    }

    let deserialized: HardwareSnapshot =
        serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(deserialized.cycles_delta, hw_snapshot.cycles_delta);
    assert_eq!(
        deserialized.instructions_delta,
        hw_snapshot.instructions_delta
    );
    assert_eq!(deserialized.ipc, hw_snapshot.ipc);
    assert_eq!(deserialized.l1d_miss_rate, hw_snapshot.l1d_miss_rate);
    assert_eq!(deserialized.llc_miss_rate, hw_snapshot.llc_miss_rate);
    assert_eq!(
        deserialized.branch_miss_rate,
        hw_snapshot.branch_miss_rate
    );
}

#[test]
fn test_system_timings_snapshot_all_fields_present() {
    let sys_timings = SystemTimingsSnapshot {
        total_tick_us: 1500,
        movement_us: 200,
        perception_us: 300,
        spatial_grid_rebuild_us: 100,
        behavior_transition_us: 50,
        steering_us: 115, // Fused steering (Sprint 20)
        capture_debug_accel_us: 2,
        archetype_count: 5,
        entity_count: 27500,
    };

    let json =
        serde_json::to_string_pretty(&sys_timings).expect("Should serialize");

    let required_fields = vec![
        "totalTickUs",
        "movementUs", // Now includes rotation (fused)
        "perceptionUs",
        "spatialGridRebuildUs",
        "behaviorTransitionUs",
        "steeringUs", // Fused steering (Sprint 20)
        "captureDebugAccelUs",
        "archetypeCount",
        "entityCount",
    ];

    for field in required_fields {
        assert!(
            json.contains(&format!("\"{}\"", field)),
            "JSON should contain field: {}",
            field
        );
    }

    let deserialized: SystemTimingsSnapshot =
        serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(deserialized.total_tick_us, sys_timings.total_tick_us);
    assert_eq!(deserialized.movement_us, sys_timings.movement_us);
    assert_eq!(deserialized.entity_count, sys_timings.entity_count);
}

#[test]
fn test_baseline_snapshot_27_5k_creatures_schema() {
    let hw_snapshot = HardwareSnapshot {
        cycles_delta: 3_000_000_000,
        instructions_delta: 3_600_000_000,
        cache_refs_delta: 100_000,
        cache_misses_delta: 2_000,
        l1d_misses_delta: 1_000,
        l1i_misses_delta: 500,
        branch_instructions_delta: 500_000,
        branch_misses_delta: 7_500,
        stalled_frontend_delta: 300_000_000,
        stalled_backend_delta: 600_000_000,
        ipc: 1.2,
        l1d_miss_rate: 2.0,
        l1i_miss_rate: 1.0,
        llc_miss_rate: 2.0,
        branch_miss_rate: 1.5,
        frontend_stall_ratio: 10.0,
        backend_stall_ratio: 20.0,
    };

    let sys_timings = SystemTimingsSnapshot {
        total_tick_us: 1500,
        movement_us: 200,
        perception_us: 300,
        spatial_grid_rebuild_us: 100,
        behavior_transition_us: 50,
        steering_us: 115, // Fused steering (Sprint 20)
        capture_debug_accel_us: 2,
        archetype_count: 5,
        entity_count: 27500,
    };

    let snapshot = PerformanceSnapshot::new(
        "post-napi-migration".to_string(),
        "Baseline at 27.5K entities after NAPI-RS migration. Zero-copy IPC.".to_string(),
        27500,
        hw_snapshot,
        &sys_timings,
    );

    let json = serde_json::to_string_pretty(&snapshot)
        .expect("Should serialize baseline snapshot");

    println!("=== BASELINE SNAPSHOT SCHEMA ===");
    println!("{}", json);
    println!("=== END BASELINE SNAPSHOT ===");

    assert!(json.contains("\"post-napi-migration\""));
    assert!(json.contains("27500"));
    assert!(
        json.contains("\"ipc\": 1.2"),
        "Hardware metrics IPC should be in snapshot"
    );
    assert!(
        json.contains("\"l1dMissRate\": 2.0"),
        "L1D miss rate should be in snapshot"
    );

    let deserialized: PerformanceSnapshot = serde_json::from_str(&json)
        .expect("Baseline snapshot should deserialize");

    assert_eq!(deserialized.label, "post-napi-migration");
    assert_eq!(deserialized.creature_count, 27500);
    assert_eq!(deserialized.hardware_metrics.ipc, 1.2);
}

#[test]
fn test_snapshot_preserves_precision() {
    let hw_snapshot = HardwareSnapshot {
        cycles_delta: u64::MAX - 1000,
        instructions_delta: u64::MAX - 500,
        cache_refs_delta: 123_456_789,
        cache_misses_delta: 987_654,
        l1d_misses_delta: 54321,
        l1i_misses_delta: 12345,
        branch_instructions_delta: 999_999_999,
        branch_misses_delta: 123_456,
        stalled_frontend_delta: 888_888_888,
        stalled_backend_delta: 777_777_777,
        ipc: 1.234567890123456,
        l1d_miss_rate: 2.345678901234567,
        l1i_miss_rate: 0.123456789012345,
        llc_miss_rate: 3.456789012345678,
        branch_miss_rate: 1.567890123456789,
        frontend_stall_ratio: 10.123456789012345,
        backend_stall_ratio: 20.234567890123456,
    };

    let json = serde_json::to_string_pretty(&hw_snapshot)
        .expect("Should serialize with full precision");

    let deserialized: HardwareSnapshot = serde_json::from_str(&json)
        .expect("Should deserialize with full precision");

    assert_eq!(deserialized.cycles_delta, hw_snapshot.cycles_delta);
    assert_eq!(
        deserialized.instructions_delta,
        hw_snapshot.instructions_delta
    );

    let epsilon = 1e-10;
    assert!(
        (deserialized.ipc - hw_snapshot.ipc).abs() < epsilon,
        "IPC precision should be preserved"
    );
    assert!(
        (deserialized.l1d_miss_rate - hw_snapshot.l1d_miss_rate).abs()
            < epsilon,
        "L1D miss rate precision should be preserved"
    );
}

#[test]
fn test_git_info_extracted_correctly() {
    let hw_snapshot = HardwareSnapshot::default();
    let sys_timings = SystemTimingsSnapshot::default();

    let snapshot = PerformanceSnapshot::new(
        "test".to_string(),
        "test".to_string(),
        100,
        hw_snapshot,
        &sys_timings,
    );

    assert!(
        !snapshot.git_commit.is_empty(),
        "Git commit should not be empty (got: {})",
        snapshot.git_commit
    );

    assert!(
        !snapshot.git_branch.is_empty(),
        "Git branch should not be empty (got: {})",
        snapshot.git_branch
    );

    if snapshot.git_branch.contains("sprint-13") {
        println!(
            "✓ Correctly on Sprint 13 branch: {}",
            snapshot.git_branch
        );
    }

    println!("Git commit: {}", snapshot.git_commit);
    println!("Git branch: {}", snapshot.git_branch);
    println!("Git dirty: {}", snapshot.git_dirty);
}
