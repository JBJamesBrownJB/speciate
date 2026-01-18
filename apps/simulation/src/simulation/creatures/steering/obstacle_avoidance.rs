//! Distance-based obstacle avoidance.
//!
//! Pure functions for static obstacle avoidance steering.
//! Unlike creature avoidance (TTC-based), obstacles don't move so we use
//! simple distance-based repulsion with quadratic falloff.
//!
//! Biological parallel: Spatial memory (hippocampus) vs social tracking (different regions).
//! Obstacles are perceived passively and always available, unlike active creature tracking.

use crate::simulation::terrain::PerceivedObstacle;

/// Distance at which creature begins to react to obstacles (meters).
/// Beyond this distance, obstacles are ignored.
pub const OBSTACLE_AWARENESS_DIST: f32 = 15.0;

/// Base strength of obstacle repulsion force (m/s²).
/// Multiplied by urgency factor based on proximity.
pub const OBSTACLE_REPULSION_STRENGTH: f32 = 20.0;

/// Emergency repulsion when creature is inside obstacle bounds (m/s²).
/// Very high to push creature out quickly.
pub const EMERGENCY_REPULSION: f32 = 50.0;

/// Calculate obstacle avoidance acceleration based on distance.
///
/// Pure function: no side effects, fully deterministic given inputs.
///
/// Algorithm:
/// 1. For each obstacle, compute distance to obstacle edge
/// 2. If inside obstacle, apply emergency push
/// 3. If within awareness distance, apply quadratic inverse distance repulsion
/// 4. Accumulate forces, clamp to max_accel
///
/// Returns acceleration in m/s² pointing away from nearby obstacles.
pub fn calculate_obstacle_avoidance<'a>(
    self_pos: (f32, f32),
    obstacles: impl IntoIterator<Item = &'a PerceivedObstacle>,
    max_accel: f32,
) -> (f32, f32) {
    let mut total_ax = 0.0_f32;
    let mut total_ay = 0.0_f32;

    for obstacle in obstacles {
        let dx = self_pos.0 - obstacle.center_x;
        let dy = self_pos.1 - obstacle.center_y;
        let center_dist_sq = dx * dx + dy * dy;

        // Avoid division by zero
        if center_dist_sq < 0.0001 {
            // Exactly at obstacle center - push in arbitrary direction
            total_ax += EMERGENCY_REPULSION;
            continue;
        }

        let center_dist = center_dist_sq.sqrt();
        let edge_dist = center_dist - obstacle.radius;

        // Normalize direction (from obstacle center toward creature)
        let nx = dx / center_dist;
        let ny = dy / center_dist;

        if edge_dist <= 0.0 {
            // Inside obstacle - emergency push
            total_ax += nx * EMERGENCY_REPULSION;
            total_ay += ny * EMERGENCY_REPULSION;
        } else if edge_dist < OBSTACLE_AWARENESS_DIST {
            // Within awareness range - apply quadratic falloff repulsion
            let urgency = 1.0 - (edge_dist / OBSTACLE_AWARENESS_DIST);
            let force_mag = OBSTACLE_REPULSION_STRENGTH * urgency * urgency;

            total_ax += nx * force_mag;
            total_ay += ny * force_mag;
        }
        // Beyond awareness distance - no force
    }

    // Clamp total acceleration to max_accel
    let total_mag_sq = total_ax * total_ax + total_ay * total_ay;
    let max_sq = max_accel * max_accel;

    if total_mag_sq > max_sq && total_mag_sq > 0.0001 {
        let scale = max_accel / total_mag_sq.sqrt();
        total_ax *= scale;
        total_ay *= scale;
    }

    (total_ax, total_ay)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_obstacle(x: f32, y: f32, radius: f32) -> PerceivedObstacle {
        PerceivedObstacle::with_radius(x, y, radius)
    }

    // ========================================================================
    // Test 1: No avoidance when no obstacles
    // ========================================================================
    #[test]
    fn test_no_avoidance_when_no_obstacles() {
        let obstacles: [PerceivedObstacle; 0] = [];
        let (ax, ay) = calculate_obstacle_avoidance((0.0, 0.0), &obstacles, 10.0);
        assert_eq!((ax, ay), (0.0, 0.0));
    }

    // ========================================================================
    // Test 2: No avoidance when obstacle is far away
    // ========================================================================
    #[test]
    fn test_no_avoidance_when_far() {
        let obstacles = [make_obstacle(100.0, 0.0, 10.0)];
        let (ax, ay) = calculate_obstacle_avoidance((0.0, 0.0), &obstacles, 10.0);

        // Obstacle is 90m from edge (100 - 10 radius), well beyond awareness distance
        assert_eq!((ax, ay), (0.0, 0.0));
    }

    // ========================================================================
    // Test 3: Avoidance direction is away from obstacle
    // ========================================================================
    #[test]
    fn test_avoidance_direction_away_from_obstacle() {
        // Obstacle to the right, creature should be pushed left
        let obstacles = [make_obstacle(20.0, 0.0, 10.0)];
        let (ax, ay) = calculate_obstacle_avoidance((0.0, 0.0), &obstacles, 10.0);

        // Edge is at x=10, creature at x=0, so edge distance = 10m
        // Should push left (negative X)
        assert!(ax < 0.0, "Should push away (negative X), got ax={}", ax);
        assert!(ay.abs() < 0.01, "Y should be near zero, got ay={}", ay);
    }

    // ========================================================================
    // Test 4: Stronger force when closer
    // ========================================================================
    #[test]
    fn test_stronger_force_when_closer() {
        // Close obstacle (edge at 5m)
        let close_obstacles = [make_obstacle(10.0, 0.0, 5.0)];
        let (close_ax, _) = calculate_obstacle_avoidance((0.0, 0.0), &close_obstacles, 100.0);

        // Far obstacle (edge at 10m)
        let far_obstacles = [make_obstacle(15.0, 0.0, 5.0)];
        let (far_ax, _) = calculate_obstacle_avoidance((0.0, 0.0), &far_obstacles, 100.0);

        // Close obstacle should produce stronger force
        assert!(
            close_ax.abs() > far_ax.abs(),
            "Closer obstacle should produce stronger force: close={}, far={}",
            close_ax,
            far_ax
        );
    }

    // ========================================================================
    // Test 5: Emergency push when inside obstacle
    // ========================================================================
    #[test]
    fn test_emergency_push_when_inside() {
        // Creature at (5, 0), obstacle center at (0, 0) with radius 10
        // Creature is inside obstacle (5 < 10)
        let obstacles = [make_obstacle(0.0, 0.0, 10.0)];
        let (ax, ay) = calculate_obstacle_avoidance((5.0, 0.0), &obstacles, 100.0);

        // Should apply emergency repulsion in +X direction
        assert!(
            ax > 40.0,
            "Should apply emergency repulsion, got ax={}",
            ax
        );
        assert!(ay.abs() < 0.01, "Y should be near zero");
    }

    // ========================================================================
    // Test 6: Multiple obstacles accumulate
    // ========================================================================
    #[test]
    fn test_multiple_obstacles_accumulate() {
        // Obstacle from left and right - forces should partially cancel
        let obstacles = [
            make_obstacle(-15.0, 0.0, 5.0), // Left: edge at -10
            make_obstacle(15.0, 0.0, 5.0),  // Right: edge at 10
        ];
        let (ax, ay) = calculate_obstacle_avoidance((0.0, 0.0), &obstacles, 100.0);

        // Symmetric obstacles should roughly cancel in X
        assert!(ax.abs() < 0.01, "Symmetric obstacles should cancel, got ax={}", ax);
        assert!(ay.abs() < 0.01, "Y should be zero");
    }

    // ========================================================================
    // Test 7: Clamped to max_accel
    // ========================================================================
    #[test]
    fn test_clamped_to_max_accel() {
        // Creature inside obstacle - emergency force
        let obstacles = [make_obstacle(0.0, 0.0, 20.0)];
        let (ax, ay) = calculate_obstacle_avoidance((5.0, 0.0), &obstacles, 5.0);

        let magnitude = (ax * ax + ay * ay).sqrt();
        assert!(
            magnitude <= 5.01,
            "Should be clamped to max_accel 5.0, got {}",
            magnitude
        );
    }

    // ========================================================================
    // Test 8: Diagonal approach
    // ========================================================================
    #[test]
    fn test_diagonal_approach() {
        // Obstacle at (10, 10) with radius 4
        // Distance from origin to center = sqrt(200) ≈ 14.14
        // Edge distance = 14.14 - 4 = 10.14m (within 15m awareness)
        let obstacles = [make_obstacle(10.0, 10.0, 4.0)];
        let (ax, ay) = calculate_obstacle_avoidance((0.0, 0.0), &obstacles, 10.0);

        // Should push in -X and -Y direction (away from obstacle)
        assert!(ax < 0.0, "Should push in -X direction, got ax={}", ax);
        assert!(ay < 0.0, "Should push in -Y direction, got ay={}", ay);
        // Diagonal push should be equal in both axes
        assert!(
            (ax - ay).abs() < 0.01,
            "Diagonal push should be equal, got ax={}, ay={}",
            ax,
            ay
        );
    }

    // ========================================================================
    // Test 9: Edge case - exactly at awareness boundary
    // ========================================================================
    #[test]
    fn test_at_awareness_boundary() {
        // Obstacle edge exactly at awareness distance
        let edge_dist = OBSTACLE_AWARENESS_DIST;
        let obstacles = [make_obstacle(edge_dist + 5.0, 0.0, 5.0)];
        let (ax, ay) = calculate_obstacle_avoidance((0.0, 0.0), &obstacles, 10.0);

        // At boundary, urgency = 0, so force should be essentially zero
        assert!(ax.abs() < 0.01, "At boundary should have near-zero force");
        assert!(ay.abs() < 0.01);
    }

    // ========================================================================
    // Test 10: Quadratic falloff behavior
    // ========================================================================
    #[test]
    fn test_quadratic_falloff() {
        // At half awareness distance, urgency = 0.5, force = 0.25 * strength
        let half_dist = OBSTACLE_AWARENESS_DIST / 2.0;
        let obstacles = [make_obstacle(half_dist + 5.0, 0.0, 5.0)];
        let (ax, _) = calculate_obstacle_avoidance((0.0, 0.0), &obstacles, 100.0);

        // Expected force: urgency = 0.5, force = 0.25 * OBSTACLE_REPULSION_STRENGTH
        let expected = -0.25 * OBSTACLE_REPULSION_STRENGTH; // Negative because pushing left
        assert!(
            (ax - expected).abs() < 0.1,
            "Quadratic falloff: expected {}, got {}",
            expected,
            ax
        );
    }
}
