use bevy_ecs::prelude::*;
use rayon::prelude::*;

use super::{DriveContribution, DriveContributions, DriveOutput};

#[cfg(feature = "dev-tools")]
use super::DriveSimplex;

#[cfg(feature = "dev-tools")]
use crate::instrumentation::SystemTimings;

/// Priority weights for drive categories (zoologist-tom consultation).
/// FLEE dominates (survival), APPROACH secondary (hunting), DISPERSE tertiary (comfort).
const FLEE_WEIGHT: f32 = 1.0;
const APPROACH_WEIGHT: f32 = 0.7;
const DISPERSE_WEIGHT: f32 = 0.3;

/// Drive Combine System: Processes contributions into final drive output.
///
/// For each creature:
/// 1. Weighted sum per category (flee/approach/disperse)
/// 2. Clamp each category to magnitude 1.0 (prevents "summation overpower")
/// 3. Apply priority weights
/// 4. Write to DriveOutput.combined (hot path for steering)
/// 5. Clear contribution arrays for next tick
#[cfg(feature = "dev-tools")]
pub fn drive_combine_system(
    mut query: Query<(&mut DriveContributions, &mut DriveOutput, Option<&mut DriveSimplex>)>,
    timings: Res<SystemTimings>,
) {
    crate::time_system!(timings, "drive_combine");

    let mut entities: Vec<_> = query.iter_mut().collect();

    entities
        .par_iter_mut()
        .with_min_len(256)
        .for_each(|(contributions, output, simplex)| {
            // Convert Option<Mut<DriveSimplex>> to Option<&mut DriveSimplex>
            let simplex_ref = simplex.as_mut().map(|m| m.as_mut());
            process_drive_combine(contributions, output, simplex_ref);
        });
}

#[cfg(not(feature = "dev-tools"))]
pub fn drive_combine_system(mut query: Query<(&mut DriveContributions, &mut DriveOutput)>) {
    let mut entities: Vec<_> = query.iter_mut().collect();

    entities
        .par_iter_mut()
        .with_min_len(256)
        .for_each(|(contributions, output)| {
            process_drive_combine(contributions, output);
        });
}

/// Process drive combination for a single creature.
#[cfg(feature = "dev-tools")]
fn process_drive_combine(
    contributions: &mut DriveContributions,
    output: &mut DriveOutput,
    simplex: Option<&mut DriveSimplex>,
) {
    // 1. Weighted sum per category
    let flee_vec = weighted_sum(contributions.iter_flee());
    let approach_vec = weighted_sum(contributions.iter_approach());
    let disperse_vec = weighted_sum(contributions.iter_disperse());

    // 2. Clamp magnitude to 1.0 (prevents "summation overpower")
    let flee_clamped = clamp_magnitude(flee_vec, 1.0);
    let approach_clamped = clamp_magnitude(approach_vec, 1.0);
    let disperse_clamped = clamp_magnitude(disperse_vec, 1.0);

    // 3. Apply priority weights and sum
    let combined = (
        flee_clamped.0 * FLEE_WEIGHT
            + approach_clamped.0 * APPROACH_WEIGHT
            + disperse_clamped.0 * DISPERSE_WEIGHT,
        flee_clamped.1 * FLEE_WEIGHT
            + approach_clamped.1 * APPROACH_WEIGHT
            + disperse_clamped.1 * DISPERSE_WEIGHT,
    );

    // 4. Write to hot-path component
    output.combined = combined;

    // 5. Dev-tools: capture simplex for visualization
    if let Some(simplex) = simplex {
        simplex.flee = flee_clamped;
        simplex.approach = approach_clamped;
        simplex.disperse = disperse_clamped;
    }

    // 6. Clear contributions for next tick
    contributions.clear();
}

/// Process drive combination for a single creature (non-dev-tools version).
#[cfg(not(feature = "dev-tools"))]
fn process_drive_combine(contributions: &mut DriveContributions, output: &mut DriveOutput) {
    // 1. Weighted sum per category
    let flee_vec = weighted_sum(contributions.iter_flee());
    let approach_vec = weighted_sum(contributions.iter_approach());
    let disperse_vec = weighted_sum(contributions.iter_disperse());

    // 2. Clamp magnitude to 1.0
    let flee_clamped = clamp_magnitude(flee_vec, 1.0);
    let approach_clamped = clamp_magnitude(approach_vec, 1.0);
    let disperse_clamped = clamp_magnitude(disperse_vec, 1.0);

    // 3. Apply priority weights and sum
    let combined = (
        flee_clamped.0 * FLEE_WEIGHT
            + approach_clamped.0 * APPROACH_WEIGHT
            + disperse_clamped.0 * DISPERSE_WEIGHT,
        flee_clamped.1 * FLEE_WEIGHT
            + approach_clamped.1 * APPROACH_WEIGHT
            + disperse_clamped.1 * DISPERSE_WEIGHT,
    );

    // 4. Write to hot-path component
    output.combined = combined;

    // 5. Clear contributions for next tick
    contributions.clear();
}

/// Compute weighted sum of drive contributions.
/// Each contribution's direction is scaled by its magnitude.
fn weighted_sum<'a>(contributions: impl Iterator<Item = &'a DriveContribution>) -> (f32, f32) {
    let mut sum = (0.0f32, 0.0f32);
    for c in contributions {
        sum.0 += c.direction.0 * c.magnitude;
        sum.1 += c.direction.1 * c.magnitude;
    }
    sum
}

/// Clamp vector magnitude to max_mag, preserving direction.
fn clamp_magnitude(vec: (f32, f32), max_mag: f32) -> (f32, f32) {
    let mag_sq = vec.0 * vec.0 + vec.1 * vec.1;
    if mag_sq <= max_mag * max_mag {
        vec
    } else {
        let mag = mag_sq.sqrt();
        let scale = max_mag / mag;
        (vec.0 * scale, vec.1 * scale)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weighted_sum_empty() {
        let contributions: Vec<DriveContribution> = vec![];
        let result = weighted_sum(contributions.iter());
        assert_eq!(result, (0.0, 0.0));
    }

    #[test]
    fn test_weighted_sum_single() {
        let contributions = vec![DriveContribution {
            direction: (1.0, 0.0),
            magnitude: 0.5,
        }];
        let result = weighted_sum(contributions.iter());
        assert_eq!(result, (0.5, 0.0));
    }

    #[test]
    fn test_weighted_sum_multiple() {
        let contributions = vec![
            DriveContribution {
                direction: (1.0, 0.0),
                magnitude: 1.0,
            },
            DriveContribution {
                direction: (0.0, 1.0),
                magnitude: 1.0,
            },
        ];
        let result = weighted_sum(contributions.iter());
        assert_eq!(result, (1.0, 1.0));
    }

    #[test]
    fn test_weighted_sum_cancellation() {
        let contributions = vec![
            DriveContribution {
                direction: (1.0, 0.0),
                magnitude: 1.0,
            },
            DriveContribution {
                direction: (-1.0, 0.0),
                magnitude: 1.0,
            },
        ];
        let result = weighted_sum(contributions.iter());
        assert!((result.0).abs() < 0.001);
        assert!((result.1).abs() < 0.001);
    }

    #[test]
    fn test_clamp_magnitude_under_limit() {
        let vec = (0.5, 0.0);
        let result = clamp_magnitude(vec, 1.0);
        assert_eq!(result, (0.5, 0.0));
    }

    #[test]
    fn test_clamp_magnitude_at_limit() {
        let vec = (1.0, 0.0);
        let result = clamp_magnitude(vec, 1.0);
        assert_eq!(result, (1.0, 0.0));
    }

    #[test]
    fn test_clamp_magnitude_over_limit() {
        let vec = (3.0, 4.0); // magnitude = 5
        let result = clamp_magnitude(vec, 1.0);
        let mag = (result.0 * result.0 + result.1 * result.1).sqrt();
        assert!((mag - 1.0).abs() < 0.001);
        // Direction preserved
        assert!((result.0 / result.1 - 0.75).abs() < 0.001); // 3/4 ratio
    }

    #[test]
    fn test_clamp_magnitude_zero() {
        let vec = (0.0, 0.0);
        let result = clamp_magnitude(vec, 1.0);
        assert_eq!(result, (0.0, 0.0));
    }

    #[test]
    fn test_flee_priority_over_approach() {
        // When flee and approach are equal magnitude but opposite,
        // flee should dominate due to higher weight
        let mut contributions = DriveContributions::default();
        contributions.push_flee((1.0, 0.0), 1.0);
        contributions.push_approach((-1.0, 0.0), 1.0);

        let flee_vec = weighted_sum(contributions.iter_flee());
        let approach_vec = weighted_sum(contributions.iter_approach());

        let flee_clamped = clamp_magnitude(flee_vec, 1.0);
        let approach_clamped = clamp_magnitude(approach_vec, 1.0);

        let combined_x =
            flee_clamped.0 * FLEE_WEIGHT + approach_clamped.0 * APPROACH_WEIGHT;

        // Flee (+1.0 * 1.0) + Approach (-1.0 * 0.7) = 0.3
        assert!(combined_x > 0.0, "Flee should dominate: combined_x = {}", combined_x);
        assert!((combined_x - 0.3).abs() < 0.01);
    }
}
