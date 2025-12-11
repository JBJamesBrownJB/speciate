//! Spec: Crit can navigate through a crowd without collisions
//!
//! Trial: crowd-navigation.toml
//! - 10x10 grid of catatonic creatures at 2m spacing (staggered)
//! - 1 seeker starting at (-60, 0) targeting (30, 0)
//!
//! Expected behavior:
//! - Seeker navigates through the crowd using avoidance
//! - Minimal overlaps allowed (tunable thresholds below)

use super::{assert_no_overlaps, assert_reached_target, find_overlaps, run_trial, run_trial_with_callback};
use std::sync::atomic::{AtomicUsize, Ordering};

// ============================================================
// TEST CONFIGURATION - Adjust these to tune overlap tolerance
// ============================================================

/// Maximum overlapping pairs allowed at any single tick
const MAX_OVERLAPS_PER_TICK: usize = 3;

/// Maximum penetration depth allowed (in world units)
/// Overlaps deeper than this are considered failures
/// Note: Increased from 0.6 after Sprint 16 force refactor - forces are now
/// mass-relative (max_force = mass × 6.0 m/s²) rather than absolute values.
/// The old PANIC_FORCE of 90000N was unrealistically high; now it's capped at max_force.
const MAX_OVERLAP_DEPTH: f32 = 0.9;

/// Maximum number of ticks that can have ANY overlap
/// Set to usize::MAX to disable this check
const MAX_TICKS_WITH_OVERLAPS: usize = 200;

/// A seeker should navigate through a stationary crowd and reach its target
/// without overlapping any other creature at the end.
#[test]
fn spec_navigate_crowd_no_overlap_final() {
    // Load crowd-navigation trial: 10x10 grid of catatonics + 1 seeker
    // Run for 500 ticks at 50ms = 25 seconds simulated time
    let result = run_trial("crowd-navigation", 500, 0.05);

    // Assert: No creatures should overlap in final state
    assert_no_overlaps(&result);
}

/// Overlaps during navigation should stay within acceptable thresholds.
/// Tuned by constants at top of file: MAX_OVERLAPS_PER_TICK, MAX_OVERLAP_DEPTH, MAX_TICKS_WITH_OVERLAPS
#[test]
fn spec_navigate_crowd_acceptable_overlaps() {
    let max_overlaps_seen = AtomicUsize::new(0);
    let worst_tick = AtomicUsize::new(0);
    let ticks_with_overlaps = AtomicUsize::new(0);
    let deepest_overlap = std::sync::Mutex::new((0usize, 0u32, 0u32, 0.0f32)); // (tick, id_a, id_b, depth)
    let worst_violation = std::sync::Mutex::new(None::<String>);

    let _ = run_trial_with_callback("crowd-navigation", 500, 0.05, |tick, creatures| {
        let overlaps = find_overlaps(creatures);
        let count = overlaps.len();

        if count > 0 {
            ticks_with_overlaps.fetch_add(1, Ordering::Relaxed);
        }

        // Track worst overlap count
        if count > max_overlaps_seen.load(Ordering::Relaxed) {
            max_overlaps_seen.store(count, Ordering::Relaxed);
            worst_tick.store(tick, Ordering::Relaxed);
        }

        // Check each overlap against thresholds
        for (id_a, id_b, depth) in &overlaps {
            // Track deepest overlap
            {
                let mut deepest = deepest_overlap.lock().unwrap();
                if *depth > deepest.3 {
                    *deepest = (tick, *id_a, *id_b, *depth);
                }
            }

            // Check depth threshold
            if *depth > MAX_OVERLAP_DEPTH {
                let mut violation = worst_violation.lock().unwrap();
                if violation.is_none() {
                    *violation = Some(format!(
                        "Tick {}: Crit {} and Crit {} overlap by {:.4} units (max allowed: {:.4})",
                        tick, id_a, id_b, depth, MAX_OVERLAP_DEPTH
                    ));
                }
            }
        }

        // Check count threshold
        if count > MAX_OVERLAPS_PER_TICK {
            let mut violation = worst_violation.lock().unwrap();
            if violation.is_none() {
                *violation = Some(format!(
                    "Tick {}: {} overlapping pairs (max allowed: {})",
                    tick, count, MAX_OVERLAPS_PER_TICK
                ));
            }
        }
    });

    // Check ticks-with-overlaps threshold
    let total_overlap_ticks = ticks_with_overlaps.load(Ordering::Relaxed);
    if total_overlap_ticks > MAX_TICKS_WITH_OVERLAPS {
        let mut violation = worst_violation.lock().unwrap();
        if violation.is_none() {
            *violation = Some(format!(
                "{} ticks had overlaps (max allowed: {})",
                total_overlap_ticks, MAX_TICKS_WITH_OVERLAPS
            ));
        }
    }

    // Report results
    let max_seen = max_overlaps_seen.load(Ordering::Relaxed);
    let (deep_tick, deep_a, deep_b, deep_amount) = *deepest_overlap.lock().unwrap();

    if let Some(violation) = worst_violation.lock().unwrap().as_ref() {
        panic!(
            "Overlap threshold violated: {}\n\
             Summary: max {} pairs at tick {}, {} ticks with overlaps\n\
             Deepest: {:.4} units (Crit {} & {} at tick {})",
            violation,
            max_seen,
            worst_tick.load(Ordering::Relaxed),
            total_overlap_ticks,
            deep_amount,
            deep_a,
            deep_b,
            deep_tick
        );
    }

    // Print stats on success
    if max_seen > 0 {
        println!(
            "Overlap stats: max {} pairs, {} ticks affected, deepest {:.4} units",
            max_seen, total_overlap_ticks, deep_amount
        );
    }
}

/// The seeker should actually reach its target (not get stuck in the crowd)
#[test]
fn spec_seeker_reaches_target() {
    let result = run_trial("crowd-navigation", 500, 0.05);

    // Seeker has highest ID (spawned last in trial)
    let seeker = result
        .creatures
        .iter()
        .max_by_key(|c| c.id)
        .expect("Should have at least one creature");

    // Target is (30, 0) in crowd-navigation trial
    // Use 10 unit tolerance - we care about "reached general area", not exact position
    assert_reached_target(seeker, 30.0, 0.0, 10.0);
}

/// Verify the trial loaded correctly with expected creature count
#[test]
fn spec_trial_loads_correct_count() {
    let result = run_trial("crowd-navigation", 1, 0.05);

    // 10x10 grid = 100 catatonics + 1 seeker = 101 total
    assert_eq!(
        result.creatures.len(),
        101,
        "Expected 101 creatures (100 grid + 1 seeker)"
    );
}
