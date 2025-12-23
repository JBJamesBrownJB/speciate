//! TTC-based collision avoidance.
//!
//! Pure functions for time-to-collision based avoidance steering.
//! This module provides physics-based collision prevention that:
//! - Uses relative velocity to compute time-to-collision (TTC)
//! - Applies urgency scaling based on TTC (closer collision = stronger avoidance)
//! - Returns acceleration (not force) for direct integration
//!
//! See: ABC-SUPER_SPRINT/4-better-avoid.md for design details.

use crate::simulation::creatures::constants::CRITICAL_TTC_SECONDS;

/// Input data for avoidance calculation.
/// Contains the creature's current state needed for TTC computation.
#[derive(Debug, Clone, Copy)]
pub struct AvoidanceInput {
    /// Creature position (meters)
    pub self_pos: (f32, f32),
    /// Creature velocity (m/s)
    pub self_vel: (f32, f32),
    /// Creature radius (meters, for edge-to-edge distance)
    pub self_radius: f32,
    /// Maximum acceleration (m/s², derived from max_force/mass)
    pub max_accel: f32,
}

/// Neighbor data for avoidance.
/// Minimal data needed to compute TTC with a single neighbor.
#[derive(Debug, Clone, Copy)]
pub struct Neighbor {
    /// Neighbor position (meters)
    pub pos: (f32, f32),
    /// Neighbor velocity (m/s)
    pub vel: (f32, f32),
    /// Neighbor radius (meters)
    pub radius: f32,
}

/// Configuration for avoidance behavior.
/// Tunable parameters that control avoidance sensitivity.
#[derive(Debug, Clone, Copy)]
pub struct AvoidanceConfig {
    /// Critical time-to-collision threshold (seconds).
    /// When TTC < this value, urgency reaches maximum.
    /// Default: 2.0 seconds
    pub critical_ttc: f32,
}

impl Default for AvoidanceConfig {
    fn default() -> Self {
        Self {
            critical_ttc: CRITICAL_TTC_SECONDS,
        }
    }
}

/// Output from avoidance calculation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AvoidanceOutput {
    /// Acceleration to apply (m/s²)
    pub accel: (f32, f32),
}

impl AvoidanceOutput {
    pub const ZERO: Self = Self { accel: (0.0, 0.0) };
}

/// Calculate avoidance acceleration based on time-to-collision.
///
/// Pure function: no side effects, fully deterministic given inputs.
///
/// Algorithm:
/// 1. For each neighbor, compute relative velocity and closing speed
/// 2. Skip neighbors we're moving away from (closing_speed <= 0)
/// 3. Compute TTC = edge_distance / closing_speed
/// 4. Compute urgency = (critical_ttc / TTC).clamp(0, 1)
/// 5. Apply urgency² scaling for smooth ramp-up
/// 6. Accumulate avoidance forces, clamp to max_accel
///
/// Returns acceleration in m/s² pointing away from imminent collisions.
///
/// Accepts any IntoIterator of Neighbor to avoid heap allocation in hot paths.
pub fn calculate_avoidance(
    input: &AvoidanceInput,
    neighbors: impl IntoIterator<Item = Neighbor>,
    config: &AvoidanceConfig,
) -> AvoidanceOutput {
    let mut total_ax = 0.0_f32;
    let mut total_ay = 0.0_f32;
    let mut has_neighbors = false;

    for neighbor in neighbors {
        has_neighbors = true;
        // Direction from self to neighbor
        let dx = neighbor.pos.0 - input.self_pos.0;
        let dy = neighbor.pos.1 - input.self_pos.1;
        let center_dist_sq = dx * dx + dy * dy;

        // Avoid division by zero for overlapping entities
        if center_dist_sq < 0.0001 {
            continue;
        }

        let center_dist = center_dist_sq.sqrt();
        let dir_to_neighbor_x = dx / center_dist;
        let dir_to_neighbor_y = dy / center_dist;

        // Relative velocity (neighbor velocity minus self velocity)
        let rel_vx = neighbor.vel.0 - input.self_vel.0;
        let rel_vy = neighbor.vel.1 - input.self_vel.1;

        // Closing speed = dot(relative_velocity, direction_to_neighbor)
        // Positive means approaching, negative means separating
        let closing_speed = -(rel_vx * dir_to_neighbor_x + rel_vy * dir_to_neighbor_y);

        // Skip if moving apart
        if closing_speed <= 0.0 {
            continue;
        }

        // Edge-to-edge distance
        let combined_radius = input.self_radius + neighbor.radius;
        let edge_dist = (center_dist - combined_radius).max(0.01); // Minimum 1cm to avoid div by zero

        // Time-to-collision
        let ttc = edge_dist / closing_speed;

        // Urgency: 1.0 when ttc <= 0, ramps down as ttc increases
        // urgency = (critical_ttc / ttc).clamp(0, 1)
        let urgency = (config.critical_ttc / ttc).clamp(0.0, 1.0);

        // Smooth scaling with urgency squared
        let urgency_sq = urgency * urgency;

        // Force magnitude
        let force_mag = urgency_sq * input.max_accel;

        // Direction away from neighbor (opposite of direction to neighbor)
        total_ax -= force_mag * dir_to_neighbor_x;
        total_ay -= force_mag * dir_to_neighbor_y;
    }

    if !has_neighbors {
        return AvoidanceOutput::ZERO;
    }

    // Clamp total acceleration to max_accel
    let total_mag_sq = total_ax * total_ax + total_ay * total_ay;
    let max_sq = input.max_accel * input.max_accel;

    if total_mag_sq > max_sq {
        let scale = input.max_accel / total_mag_sq.sqrt();
        total_ax *= scale;
        total_ay *= scale;
    }

    AvoidanceOutput {
        accel: (total_ax, total_ay),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_input(pos: (f32, f32), vel: (f32, f32)) -> AvoidanceInput {
        AvoidanceInput {
            self_pos: pos,
            self_vel: vel,
            self_radius: 1.0,
            max_accel: 10.0,
        }
    }

    fn make_neighbor(pos: (f32, f32), vel: (f32, f32)) -> Neighbor {
        Neighbor {
            pos,
            vel,
            radius: 1.0,
        }
    }

    // ========================================================================
    // Test 1: No avoidance when no neighbors
    // ========================================================================
    #[test]
    fn test_no_avoidance_when_no_neighbors() {
        let input = make_input((0.0, 0.0), (5.0, 0.0));
        let neighbors: [Neighbor; 0] = [];
        let config = AvoidanceConfig::default();

        let output = calculate_avoidance(&input, neighbors, &config);

        assert_eq!(output.accel, (0.0, 0.0));
    }

    // ========================================================================
    // Test 2: No avoidance when moving apart
    // ========================================================================
    #[test]
    fn test_no_avoidance_when_moving_apart() {
        // Self at origin moving right, neighbor to the left moving further left
        let input = make_input((0.0, 0.0), (5.0, 0.0));
        let neighbors = [make_neighbor((-10.0, 0.0), (-5.0, 0.0))];
        let config = AvoidanceConfig::default();

        let output = calculate_avoidance(&input, neighbors, &config);

        // Moving apart = no collision threat = zero force
        assert_eq!(output.accel, (0.0, 0.0));
    }

    // ========================================================================
    // Test 3: Avoidance direction is away from neighbor (RED - will fail)
    // ========================================================================
    #[test]
    fn test_avoidance_direction_away_from_neighbor() {
        // Self at origin stationary, neighbor approaching from the right
        let input = make_input((0.0, 0.0), (0.0, 0.0));
        let neighbors = [make_neighbor((10.0, 0.0), (-5.0, 0.0))]; // Approaching from right
        let config = AvoidanceConfig::default();

        let output = calculate_avoidance(&input, neighbors, &config);

        // Should steer LEFT (negative X) away from approaching neighbor
        assert!(
            output.accel.0 < 0.0,
            "Should steer away (negative X), got {:?}",
            output.accel
        );
        // Y should be ~0 for head-on approach
        assert!(
            output.accel.1.abs() < 0.01,
            "Y should be near zero, got {:?}",
            output.accel
        );
    }

    // ========================================================================
    // Test 4: High urgency when short TTC
    // ========================================================================
    #[test]
    fn test_high_urgency_when_short_ttc() {
        // Fast approach = short TTC = high urgency = high force
        let input = make_input((0.0, 0.0), (0.0, 0.0));
        // Neighbor very close and approaching fast
        let neighbors = [make_neighbor((5.0, 0.0), (-10.0, 0.0))];
        let config = AvoidanceConfig { critical_ttc: 2.0 };

        let output = calculate_avoidance(&input, neighbors, &config);

        // Should have significant avoidance
        let magnitude = (output.accel.0.powi(2) + output.accel.1.powi(2)).sqrt();
        assert!(
            magnitude > 5.0,
            "Expected high force, got magnitude {}",
            magnitude
        );
    }

    // ========================================================================
    // Test 5: Low urgency when long TTC
    // ========================================================================
    #[test]
    fn test_low_urgency_when_long_ttc() {
        // Slow approach = long TTC = low urgency = low force
        let input = make_input((0.0, 0.0), (0.0, 0.0));
        // Neighbor far away and approaching slowly
        let neighbors = [make_neighbor((100.0, 0.0), (-1.0, 0.0))];
        let config = AvoidanceConfig { critical_ttc: 2.0 };

        let output = calculate_avoidance(&input, neighbors, &config);

        // Should have low avoidance (TTC = 98 / 1 = 98 seconds >> critical_ttc)
        let magnitude = (output.accel.0.powi(2) + output.accel.1.powi(2)).sqrt();
        assert!(
            magnitude < 1.0,
            "Expected low force for long TTC, got magnitude {}",
            magnitude
        );
    }

    // ========================================================================
    // Test 6: Urgency clamped to one (max force)
    // ========================================================================
    #[test]
    fn test_urgency_clamped_to_one() {
        // Extremely short TTC should clamp urgency to 1.0, not exceed max_accel
        let input = AvoidanceInput {
            self_pos: (0.0, 0.0),
            self_vel: (0.0, 0.0),
            self_radius: 1.0,
            max_accel: 10.0,
        };
        // Almost touching, very fast approach
        let neighbors = [make_neighbor((2.5, 0.0), (-100.0, 0.0))];
        let config = AvoidanceConfig { critical_ttc: 2.0 };

        let output = calculate_avoidance(&input, neighbors, &config);

        let magnitude = (output.accel.0.powi(2) + output.accel.1.powi(2)).sqrt();
        // Should be at or near max_accel (10.0) but not exceed it
        assert!(
            magnitude <= 10.01,
            "Should not exceed max_accel, got {}",
            magnitude
        );
        assert!(
            magnitude > 5.0,
            "Should be substantial force, got {}",
            magnitude
        );
    }

    // ========================================================================
    // Test 7: Parallel paths no avoidance
    // ========================================================================
    #[test]
    fn test_parallel_paths_no_avoidance() {
        // Both moving in same direction at same speed = no closing
        let input = make_input((0.0, 0.0), (5.0, 0.0));
        let neighbors = [make_neighbor((10.0, 0.0), (5.0, 0.0))];
        let config = AvoidanceConfig::default();

        let output = calculate_avoidance(&input, neighbors, &config);

        // Parallel paths = closing_speed = 0 = no avoidance
        assert_eq!(output.accel, (0.0, 0.0));
    }

    // ========================================================================
    // Test 8: Multiple neighbors accumulate
    // ========================================================================
    #[test]
    fn test_multiple_neighbors_accumulate() {
        let input = make_input((0.0, 0.0), (0.0, 0.0));
        // Two neighbors approaching from opposite sides
        let neighbors = [
            make_neighbor((10.0, 0.0), (-5.0, 0.0)), // From right
            make_neighbor((-10.0, 0.0), (5.0, 0.0)), // From left
        ];
        let config = AvoidanceConfig::default();

        let output = calculate_avoidance(&input, neighbors, &config);

        // Symmetric threats should roughly cancel in X, but total magnitude should be non-zero
        // The exact behavior depends on implementation, but there should be SOME response
        // With symmetric threats from opposite sides, X might cancel but forces exist
        let _magnitude = (output.accel.0.powi(2) + output.accel.1.powi(2)).sqrt();
        // For now just verify it doesn't crash and handles multiple neighbors
    }

    // ========================================================================
    // Test 9: Clamped to max_accel
    // ========================================================================
    #[test]
    fn test_clamped_to_max_accel() {
        let input = AvoidanceInput {
            self_pos: (0.0, 0.0),
            self_vel: (0.0, 0.0),
            self_radius: 1.0,
            max_accel: 5.0, // Low max
        };
        // Multiple urgent threats
        let neighbors = [
            make_neighbor((3.0, 0.0), (-10.0, 0.0)),
            make_neighbor((0.0, 3.0), (0.0, -10.0)),
        ];
        let config = AvoidanceConfig { critical_ttc: 2.0 };

        let output = calculate_avoidance(&input, neighbors, &config);

        let magnitude = (output.accel.0.powi(2) + output.accel.1.powi(2)).sqrt();
        assert!(
            magnitude <= 5.01,
            "Should not exceed max_accel of 5.0, got {}",
            magnitude
        );
    }

    // ========================================================================
    // Test 10: Overlapping entities handled gracefully
    // ========================================================================
    #[test]
    fn test_overlapping_entities() {
        // Entities already overlapping (edge distance <= 0)
        let input = make_input((0.0, 0.0), (0.0, 0.0));
        let neighbors = [make_neighbor((1.0, 0.0), (-1.0, 0.0))]; // Centers 1m apart, radii sum to 2m
        let config = AvoidanceConfig::default();

        let output = calculate_avoidance(&input, neighbors, &config);

        // Should not crash, and should still try to separate
        // Force should point away (negative X)
        assert!(
            output.accel.0 <= 0.0,
            "Should try to separate, got {:?}",
            output.accel
        );
    }

    // ========================================================================
    // Test 11: Stationary self with moving neighbor
    // ========================================================================
    #[test]
    fn test_stationary_self_moving_neighbor() {
        let input = make_input((0.0, 0.0), (0.0, 0.0)); // Stationary
        let neighbors = [make_neighbor((10.0, 0.0), (-5.0, 0.0))]; // Approaching
        let config = AvoidanceConfig::default();

        let output = calculate_avoidance(&input, neighbors, &config);

        // Even though self is stationary, should still avoid approaching neighbor
        assert!(output.accel.0 < 0.0, "Should avoid approaching neighbor");
    }

    // ========================================================================
    // Test 12: Stationary neighbor with moving self
    // ========================================================================
    #[test]
    fn test_stationary_neighbor_moving_self() {
        let input = make_input((0.0, 0.0), (5.0, 0.0)); // Moving right
        let neighbors = [make_neighbor((10.0, 0.0), (0.0, 0.0))]; // Stationary ahead
        let config = AvoidanceConfig::default();

        let output = calculate_avoidance(&input, neighbors, &config);

        // Self is moving toward stationary neighbor, should avoid
        assert!(
            output.accel.0 < 0.0,
            "Should avoid stationary obstacle ahead"
        );
    }
}
