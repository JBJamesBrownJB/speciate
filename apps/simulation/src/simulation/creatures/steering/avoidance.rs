//! Avoidance steering behavior pure functions.
//!
//! This module provides testable pure functions for obstacle avoidance,
//! separate from the ECS system. All functions use correct F=ma physics.

use crate::simulation::math::{magnitude_sq, SteeringContext};

/// Minimum speed² below which we allow full avoidance (can't define "forward" when stationary)
const MIN_SPEED_SQ_FOR_STEERING: f32 = 0.01;

/// Parameters for avoidance force calculation.
#[derive(Debug, Clone, Copy)]
pub struct AvoidanceParams {
    /// Position of the creature
    pub position: (f32, f32),
    /// Creature radius (meters)
    pub self_radius: f32,
    /// Effective personal space (meters) - already adjusted for energy/seeking
    pub personal_space: f32,
    /// Emergency brake distance threshold (meters)
    pub emergency_distance: f32,
}

/// Data for a single neighbor obstacle.
#[derive(Debug, Clone, Copy)]
pub struct NeighborObstacle {
    /// Neighbor position
    pub position: (f32, f32),
    /// Neighbor radius (meters)
    pub radius: f32,
}

/// Calculate avoidance acceleration from multiple neighbors.
///
/// This is the main entry point for avoidance behavior. It:
/// 1. Calculates repulsion from each neighbor within personal space
/// 2. Sums the repulsions
/// 3. Projects to remove forward thrust component (avoidance shouldn't speed us up)
/// 4. Clamps to max acceleration
pub fn calculate_avoidance_force(
    ctx: &SteeringContext,
    params: &AvoidanceParams,
    neighbors: &[NeighborObstacle],
) -> (f32, f32) {
    if neighbors.is_empty() {
        return (0.0, 0.0);
    }

    let max_accel = ctx.max_accel();
    if max_accel < 0.001 {
        return (0.0, 0.0);
    }

    let (self_x, self_y) = params.position;
    let base_interaction = params.personal_space + params.self_radius;

    let mut total_repulsion_x = 0.0;
    let mut total_repulsion_y = 0.0;

    // Accumulate repulsion from all neighbors
    for neighbor in neighbors {
        let away_x = self_x - neighbor.position.0;
        let away_y = self_y - neighbor.position.1;
        let center_distance_sq = magnitude_sq(away_x, away_y);

        // Degenerate case: overlapping
        if center_distance_sq < 0.000001 {
            continue;
        }

        let max_interaction_distance = base_interaction + neighbor.radius;
        let max_interaction_distance_sq = max_interaction_distance * max_interaction_distance;

        // Outside interaction range
        if center_distance_sq > max_interaction_distance_sq {
            continue;
        }

        // Compute distance and inverse for direction normalization
        let center_distance = center_distance_sq.sqrt();
        let inv_distance = 1.0 / center_distance;

        let edge_distance = center_distance - params.self_radius - neighbor.radius;
        let safe_distance = edge_distance.max(0.01);

        // Urgency scales with inverse square of distance
        let ratio = params.personal_space / safe_distance;
        let urgency = ratio * ratio;

        // Emergency brake: max ACCELERATION when very close, otherwise scale by urgency
        let accel_magnitude = if safe_distance < params.emergency_distance {
            max_accel
        } else {
            (max_accel * urgency).min(max_accel)
        };

        // Direction away from neighbor
        let accel_x = away_x * inv_distance * accel_magnitude;
        let accel_y = away_y * inv_distance * accel_magnitude;

        total_repulsion_x += accel_x;
        total_repulsion_y += accel_y;
    }

    // Project to remove forward thrust (avoidance = braking + steering, never forward)
    let (steer_x, steer_y) = project_avoidance_steering(
        (total_repulsion_x, total_repulsion_y),
        ctx.velocity,
    );

    // Clamp to max acceleration
    let mag_sq = steer_x * steer_x + steer_y * steer_y;
    let max_sq = max_accel * max_accel;

    if mag_sq > max_sq && mag_sq > 0.0001 {
        let mag = mag_sq.sqrt();
        let scale = max_accel / mag;
        (steer_x * scale, steer_y * scale)
    } else {
        (steer_x, steer_y)
    }
}

/// Project avoidance force to remove forward thrust component.
///
/// Avoidance should be BRAKING + STEERING, never forward acceleration.
/// - If obstacle is ahead (dot < 0): keep full force (braking + lateral)
/// - If obstacle is behind (dot > 0): remove forward component, keep only lateral
fn project_avoidance_steering(
    repulsion: (f32, f32),
    velocity: (f32, f32),
) -> (f32, f32) {
    let (rx, ry) = repulsion;
    let (vx, vy) = velocity;

    let speed_sq = magnitude_sq(vx, vy);

    if speed_sq <= MIN_SPEED_SQ_FOR_STEERING {
        // Stationary: allow full avoidance
        return (rx, ry);
    }

    // dot > 0: avoidance pushes same direction as velocity (FORWARD - bad!)
    // dot < 0: avoidance pushes opposite to velocity (BRAKING - good!)
    let dot = rx * vx + ry * vy;

    if dot > 0.0 {
        // Obstacle behind us - remove forward component, keep only lateral steering
        let parallel_factor = dot / speed_sq;
        (rx - parallel_factor * vx, ry - parallel_factor * vy)
    } else {
        // Obstacle ahead/side - keep full force (braking + steering)
        (rx, ry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Reference values for a typical creature
    const DEFAULT_MAX_FORCE: f32 = 390.0; // Newtons
    const DEFAULT_MASS: f32 = 65.0; // kg
    const DEFAULT_MAX_SPEED: f32 = 15.0; // m/s
    const EXPECTED_MAX_ACCEL: f32 = 6.0; // 390/65 = 6 m/s²
    const DEFAULT_PERSONAL_SPACE: f32 = 2.5; // meters
    const DEFAULT_EMERGENCY_DISTANCE: f32 = 0.25; // meters

    fn default_context() -> SteeringContext {
        SteeringContext {
            velocity: (10.0, 0.0), // Moving right
            max_speed: DEFAULT_MAX_SPEED,
            max_force: DEFAULT_MAX_FORCE,
            mass: DEFAULT_MASS,
        }
    }

    fn stationary_context() -> SteeringContext {
        SteeringContext {
            velocity: (0.0, 0.0),
            max_speed: DEFAULT_MAX_SPEED,
            max_force: DEFAULT_MAX_FORCE,
            mass: DEFAULT_MASS,
        }
    }

    fn default_params() -> AvoidanceParams {
        AvoidanceParams {
            position: (0.0, 0.0),
            self_radius: 0.5,
            personal_space: DEFAULT_PERSONAL_SPACE,
            emergency_distance: DEFAULT_EMERGENCY_DISTANCE,
        }
    }

    // ============================================================
    // F=ma verification tests
    // ============================================================

    #[test]
    fn avoidance_returns_acceleration_not_force() {
        let ctx = default_context();
        let params = default_params();
        let neighbors = vec![NeighborObstacle {
            position: (1.0, 0.0), // Obstacle 1m ahead
            radius: 0.5,
        }];

        let (ax, ay) = calculate_avoidance_force(&ctx, &params, &neighbors);
        let accel_mag = (ax * ax + ay * ay).sqrt();

        assert!(
            accel_mag <= EXPECTED_MAX_ACCEL + 0.01,
            "Avoidance acceleration {} m/s² should be ≤ {} m/s² (max_force/mass). \
             If this is ~390 m/s², the F=ma bug is present!",
            accel_mag,
            EXPECTED_MAX_ACCEL
        );
    }

    #[test]
    fn emergency_zone_uses_max_accel() {
        let ctx = stationary_context();
        let params = default_params();
        let neighbors = vec![NeighborObstacle {
            position: (0.6, 0.0), // Very close (edge distance < emergency)
            radius: 0.5,
        }];

        let (ax, ay) = calculate_avoidance_force(&ctx, &params, &neighbors);
        let accel_mag = (ax * ax + ay * ay).sqrt();

        assert!(
            (accel_mag - EXPECTED_MAX_ACCEL).abs() < 0.1,
            "Emergency zone should use max_accel {} m/s², got {}",
            EXPECTED_MAX_ACCEL,
            accel_mag
        );
    }

    // ============================================================
    // Basic avoidance tests
    // ============================================================

    #[test]
    fn no_avoidance_when_no_neighbors() {
        let ctx = default_context();
        let params = default_params();
        let neighbors: Vec<NeighborObstacle> = vec![];

        let (ax, ay) = calculate_avoidance_force(&ctx, &params, &neighbors);

        assert_eq!(ax, 0.0);
        assert_eq!(ay, 0.0);
    }

    #[test]
    fn single_neighbor_repulsion_direction() {
        let ctx = stationary_context();
        let params = default_params();
        let neighbors = vec![NeighborObstacle {
            position: (1.5, 0.0), // Neighbor to the right
            radius: 0.5,
        }];

        let (ax, ay) = calculate_avoidance_force(&ctx, &params, &neighbors);

        assert!(ax < 0.0, "Should repel in -X direction, got {}", ax);
        assert!(ay.abs() < 0.001, "Should have no Y component");
    }

    #[test]
    fn no_avoidance_outside_personal_space() {
        let ctx = default_context();
        let params = default_params();
        let neighbors = vec![NeighborObstacle {
            position: (10.0, 0.0), // Far away
            radius: 0.5,
        }];

        let (ax, ay) = calculate_avoidance_force(&ctx, &params, &neighbors);

        assert_eq!(ax, 0.0, "No force outside personal space");
        assert_eq!(ay, 0.0);
    }

    // ============================================================
    // Forward thrust removal tests
    // ============================================================

    #[test]
    fn obstacle_behind_removes_forward_component() {
        // Creature moving right, obstacle behind (to the left)
        let ctx = SteeringContext {
            velocity: (10.0, 0.0),
            ..default_context()
        };
        let params = default_params();
        let neighbors = vec![NeighborObstacle {
            position: (-1.5, 0.0), // Behind
            radius: 0.5,
        }];

        let (ax, _ay) = calculate_avoidance_force(&ctx, &params, &neighbors);

        // Forward component should be removed (obstacle behind shouldn't push forward)
        assert!(
            ax.abs() < 0.001,
            "Forward thrust should be removed, got {}",
            ax
        );
    }

    #[test]
    fn obstacle_ahead_keeps_braking() {
        // Creature moving right, obstacle ahead
        let ctx = SteeringContext {
            velocity: (10.0, 0.0),
            ..default_context()
        };
        let params = default_params();
        let neighbors = vec![NeighborObstacle {
            position: (1.5, 0.0), // Ahead
            radius: 0.5,
        }];

        let (ax, _ay) = calculate_avoidance_force(&ctx, &params, &neighbors);

        // Braking should be preserved
        assert!(
            ax < -1.0,
            "Braking force should be preserved, got {}",
            ax
        );
    }

    #[test]
    fn obstacle_side_keeps_lateral() {
        // Creature moving right, obstacle to the side (above)
        let ctx = SteeringContext {
            velocity: (10.0, 0.0),
            ..default_context()
        };
        let params = default_params();
        let neighbors = vec![NeighborObstacle {
            position: (0.0, 1.5), // Above
            radius: 0.5,
        }];

        let (ax, ay) = calculate_avoidance_force(&ctx, &params, &neighbors);

        // Lateral should be preserved
        assert!(
            ay < -1.0,
            "Lateral force should be preserved, got {}",
            ay
        );
        assert!(ax.abs() < 0.1, "Should have minimal X component");
    }

    // ============================================================
    // Multi-obstacle tests
    // ============================================================

    #[test]
    fn multiple_obstacles_accumulate() {
        let ctx = stationary_context();
        let params = default_params();
        let neighbors = vec![
            NeighborObstacle {
                position: (1.5, 0.0), // Right
                radius: 0.5,
            },
            NeighborObstacle {
                position: (0.0, 1.5), // Above
                radius: 0.5,
            },
        ];

        let (ax, ay) = calculate_avoidance_force(&ctx, &params, &neighbors);

        assert!(ax < 0.0, "Should repel in -X from right obstacle");
        assert!(ay < 0.0, "Should repel in -Y from top obstacle");
    }

    #[test]
    fn opposing_obstacles_cancel() {
        let ctx = stationary_context();
        let params = default_params();
        let neighbors = vec![
            NeighborObstacle {
                position: (1.5, 0.0), // Right
                radius: 0.5,
            },
            NeighborObstacle {
                position: (-1.5, 0.0), // Left (same distance)
                radius: 0.5,
            },
        ];

        let (ax, _ay) = calculate_avoidance_force(&ctx, &params, &neighbors);

        // Opposing forces should roughly cancel
        assert!(
            ax.abs() < 0.1,
            "Opposing forces should cancel, got {}",
            ax
        );
    }

    // ============================================================
    // Acceleration clamping tests
    // ============================================================

    #[test]
    fn multiple_obstacles_clamped_to_max_accel() {
        let ctx = stationary_context();
        let params = default_params();
        // Many close obstacles to accumulate high force
        let neighbors: Vec<_> = (0..10)
            .map(|i| {
                let angle = i as f32 * 0.5;
                NeighborObstacle {
                    position: (1.0 * angle.cos(), 1.0 * angle.sin()),
                    radius: 0.5,
                }
            })
            .collect();

        let (ax, ay) = calculate_avoidance_force(&ctx, &params, &neighbors);
        let accel_mag = (ax * ax + ay * ay).sqrt();

        assert!(
            accel_mag <= EXPECTED_MAX_ACCEL + 0.01,
            "Accumulated acceleration {} should be clamped to {}",
            accel_mag,
            EXPECTED_MAX_ACCEL
        );
    }

    // ============================================================
    // Edge case tests
    // ============================================================

    #[test]
    fn handles_overlapping_obstacle() {
        let ctx = default_context();
        let params = default_params();
        let neighbors = vec![NeighborObstacle {
            position: (0.0, 0.0), // Exactly at same position
            radius: 0.5,
        }];

        let (ax, ay) = calculate_avoidance_force(&ctx, &params, &neighbors);

        // Should not panic, should return zero (degenerate case)
        assert!(ax.is_finite() && ay.is_finite());
    }

    #[test]
    fn handles_zero_mass() {
        let ctx = SteeringContext {
            mass: 0.0,
            ..default_context()
        };
        let params = default_params();
        let neighbors = vec![NeighborObstacle {
            position: (1.5, 0.0),
            radius: 0.5,
        }];

        let (ax, ay) = calculate_avoidance_force(&ctx, &params, &neighbors);

        // Zero mass means zero max_accel, so acceleration should be zero
        assert!(
            ax.abs() < 0.001 && ay.abs() < 0.001,
            "Zero mass should result in zero acceleration"
        );
    }
}
