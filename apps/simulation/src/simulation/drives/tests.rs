use super::components::*;

// ============================================================
// DriveContributions Tests
// ============================================================

#[test]
fn test_drive_contributions_starts_empty() {
    let dc = DriveContributions::new();
    assert!(dc.is_empty());
    assert!(!dc.has_flee());
    assert!(!dc.has_approach());
    assert!(!dc.has_disperse());
}

#[test]
fn test_push_flee_adds_contribution() {
    let mut dc = DriveContributions::new();
    dc.push_flee((1.0, 0.0), 0.5);

    assert!(dc.has_flee());
    assert_eq!(dc.flee_count, 1);
    assert!(!dc.is_empty());

    let contributions: Vec<_> = dc.iter_flee().collect();
    assert_eq!(contributions.len(), 1);
    assert_eq!(contributions[0].direction, (1.0, 0.0));
    assert_eq!(contributions[0].magnitude, 0.5);
}

#[test]
fn test_push_approach_adds_contribution() {
    let mut dc = DriveContributions::new();
    dc.push_approach((0.0, 1.0), 0.8);

    assert!(dc.has_approach());
    assert_eq!(dc.approach_count, 1);

    let contributions: Vec<_> = dc.iter_approach().collect();
    assert_eq!(contributions.len(), 1);
    assert_eq!(contributions[0].direction, (0.0, 1.0));
    assert_eq!(contributions[0].magnitude, 0.8);
}

#[test]
fn test_push_disperse_adds_contribution() {
    let mut dc = DriveContributions::new();
    dc.push_disperse((-1.0, -1.0), 0.3);

    assert!(dc.has_disperse());
    assert_eq!(dc.disperse_count, 1);

    let contributions: Vec<_> = dc.iter_disperse().collect();
    assert_eq!(contributions.len(), 1);
    assert_eq!(contributions[0].direction, (-1.0, -1.0));
    assert_eq!(contributions[0].magnitude, 0.3);
}

#[test]
fn test_push_multiple_contributions() {
    let mut dc = DriveContributions::new();
    dc.push_flee((1.0, 0.0), 0.5);
    dc.push_flee((0.0, 1.0), 0.7);
    dc.push_flee((-1.0, 0.0), 0.3);

    assert_eq!(dc.flee_count, 3);
    let contributions: Vec<_> = dc.iter_flee().collect();
    assert_eq!(contributions.len(), 3);
}

#[test]
fn test_push_respects_max_contributions() {
    let mut dc = DriveContributions::new();

    // Push more than MAX_DRIVE_CONTRIBUTIONS (4)
    for i in 0..10 {
        dc.push_flee((i as f32, 0.0), 1.0);
    }

    // Should cap at MAX_DRIVE_CONTRIBUTIONS
    assert_eq!(dc.flee_count, MAX_DRIVE_CONTRIBUTIONS as u8);
    let contributions: Vec<_> = dc.iter_flee().collect();
    assert_eq!(contributions.len(), MAX_DRIVE_CONTRIBUTIONS);

    // First 4 should be preserved
    assert_eq!(contributions[0].direction.0, 0.0);
    assert_eq!(contributions[3].direction.0, 3.0);
}

#[test]
fn test_clear_resets_all_counts() {
    let mut dc = DriveContributions::new();
    dc.push_flee((1.0, 0.0), 0.5);
    dc.push_approach((0.0, 1.0), 0.8);
    dc.push_disperse((-1.0, 0.0), 0.3);

    assert!(!dc.is_empty());

    dc.clear();

    assert!(dc.is_empty());
    assert_eq!(dc.flee_count, 0);
    assert_eq!(dc.approach_count, 0);
    assert_eq!(dc.disperse_count, 0);
}

#[test]
fn test_iterators_respect_count() {
    let mut dc = DriveContributions::new();
    dc.push_flee((1.0, 0.0), 1.0);
    dc.push_flee((0.0, 1.0), 1.0);

    // Clear and add one
    dc.clear();
    dc.push_flee((0.5, 0.5), 0.5);

    // Should only iterate the one new contribution
    let contributions: Vec<_> = dc.iter_flee().collect();
    assert_eq!(contributions.len(), 1);
    assert_eq!(contributions[0].direction, (0.5, 0.5));
}

// ============================================================
// DriveOutput Tests
// ============================================================

#[test]
fn test_drive_output_default_is_zero() {
    let output = DriveOutput::new();
    assert!(output.is_zero());
    assert_eq!(output.magnitude(), 0.0);
}

#[test]
fn test_drive_output_magnitude() {
    let output = DriveOutput {
        combined: (3.0, 4.0),
    };
    assert_eq!(output.magnitude(), 5.0); // 3-4-5 triangle
    assert!(!output.is_zero());
}

#[test]
fn test_drive_output_is_zero_threshold() {
    // Very small values should be considered zero
    let output = DriveOutput {
        combined: (0.0001, 0.0001),
    };
    assert!(output.is_zero());

    // Values above threshold should not be zero
    let output = DriveOutput {
        combined: (0.01, 0.0),
    };
    assert!(!output.is_zero());
}

// ============================================================
// FreezeState Tests
// ============================================================

#[test]
fn test_freeze_state_starts_not_desperate() {
    let fs = FreezeState::new();
    assert!(!fs.is_desperate());
    assert_eq!(fs.ticks_frozen, 0);
}

#[test]
fn test_freeze_state_tick_increments() {
    let mut fs = FreezeState::new();
    fs.tick();
    assert_eq!(fs.ticks_frozen, 1);
    fs.tick();
    assert_eq!(fs.ticks_frozen, 2);
}

#[test]
fn test_freeze_state_becomes_desperate_at_threshold() {
    let mut fs = FreezeState::new();

    // Tick to just before threshold
    for _ in 0..(FreezeState::DESPERATE_THRESHOLD - 1) {
        fs.tick();
        assert!(!fs.is_desperate());
    }

    // One more tick should trigger desperate
    fs.tick();
    assert!(fs.is_desperate());

    // Escape direction should be set (random, but non-zero magnitude)
    let (ex, ey) = fs.escape_direction;
    let escape_mag = (ex * ex + ey * ey).sqrt();
    assert!(
        (escape_mag - 1.0).abs() < 0.01,
        "Escape direction should be normalized"
    );
}

#[test]
fn test_freeze_state_reset() {
    let mut fs = FreezeState::new();

    // Get to desperate state
    for _ in 0..FreezeState::DESPERATE_THRESHOLD {
        fs.tick();
    }
    assert!(fs.is_desperate());

    // Reset should clear everything
    fs.reset();
    assert!(!fs.is_desperate());
    assert_eq!(fs.ticks_frozen, 0);
    assert_eq!(fs.escape_direction, (0.0, 0.0));
}

#[test]
fn test_freeze_state_saturating_add() {
    let mut fs = FreezeState::new();
    fs.ticks_frozen = u16::MAX - 1;
    fs.tick();
    assert_eq!(fs.ticks_frozen, u16::MAX);
    fs.tick();
    assert_eq!(fs.ticks_frozen, u16::MAX); // Saturates, doesn't wrap
}
