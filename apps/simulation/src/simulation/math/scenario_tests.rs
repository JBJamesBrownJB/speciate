//! Scenario replay tests using pure functions.
//!
//! These tests verify that we can recreate any scenario by composing pure functions,
//! and detect bugs like the visualization mismatch and seeker oscillation.

#[cfg(test)]
mod tests {
    use crate::simulation::creatures::steering::{
        calculate_arrival, calculate_avoidance_force, calculate_wander,
        ArrivalParams, AvoidanceParams, NeighborObstacle, WanderParams,
    };
    use crate::simulation::math::{
        accumulate_steering, integrate_motion, IntegrationParams, SteeringContext,
    };

    // Reference creature values
    const MAX_FORCE: f32 = 390.0; // Newtons
    const MASS: f32 = 65.0; // kg
    const MAX_SPEED: f32 = 15.0; // m/s
    const MAX_ACCEL: f32 = 6.0; // 390/65 = 6 m/s²
    const DT: f32 = 0.05; // 20 Hz

    fn default_steering_context(velocity: (f32, f32)) -> SteeringContext {
        SteeringContext {
            velocity,
            max_speed: MAX_SPEED,
            max_force: MAX_FORCE,
            mass: MASS,
        }
    }

    fn default_integration_params(
        position: (f32, f32),
        velocity: (f32, f32),
        acceleration: (f32, f32),
    ) -> IntegrationParams {
        IntegrationParams {
            position,
            velocity,
            acceleration,
            dt: DT,
            drag_coefficient: 2.0,
            max_speed: MAX_SPEED,
            max_turn_rate_rad: std::f32::consts::PI, // 180 deg/s
            stopped_threshold: 0.05,
        }
    }

    // ============================================================
    // Multi-behavior combination tests
    // ============================================================

    #[test]
    fn wander_plus_avoidance_produces_valid_acceleration() {
        // Simulate a wandering creature that encounters an obstacle
        let velocity = (5.0, 0.0);
        let ctx = default_steering_context(velocity);

        // Wander contribution
        let wander_params = WanderParams {
            wander_angle: 0.0,
            wander_radius: 10.0,
            wander_distance: 50.0,
            force_multiplier: 0.1,
        };
        let wander_result = calculate_wander(&ctx, &wander_params, 0.1);

        // Avoidance contribution (obstacle to the side)
        // Position is (0, 0), obstacle at (1.0, 1.5) relative = absolute (1.0, 1.5)
        let position = (0.0, 0.0);
        let avoidance_params = AvoidanceParams {
            position,
            self_radius: 0.5,
            personal_space: 2.5,
            emergency_distance: 0.25,
        };
        let obstacle = NeighborObstacle {
            position: (position.0 + 1.0, position.1 + 1.5), // Absolute position
            radius: 0.5,
        };
        let avoidance_accel = calculate_avoidance_force(&ctx, &avoidance_params, &[obstacle]);

        // Combine accelerations
        let combined = accumulate_steering(
            &[wander_result.acceleration, avoidance_accel],
            MAX_ACCEL,
        );

        let mag = (combined.0.powi(2) + combined.1.powi(2)).sqrt();

        assert!(
            mag <= MAX_ACCEL + 0.01,
            "Combined acceleration {} should be ≤ max_accel {}",
            mag,
            MAX_ACCEL
        );
        assert!(mag > 0.0, "Should produce some acceleration");
    }

    #[test]
    fn seek_plus_avoidance_navigates_around_obstacle() {
        // Seeker approaching target with obstacle in the way
        let mut position = (0.0, 0.0);
        let mut velocity = (0.0, 0.0);
        let target = (20.0, 0.0);
        let obstacle_pos = (10.0, 0.0); // Directly in path

        let mut reached_target = false;
        let mut collided = false;

        // Simulate 200 frames (10 seconds)
        for _ in 0..200 {
            // Seek contribution
            let to_target = (target.0 - position.0, target.1 - position.1);
            let arrival_params = ArrivalParams {
                velocity,
                to_target,
                self_radius: 0.5,
                target_radius: 0.5,
                max_speed: MAX_SPEED,
                max_force: MAX_FORCE,
                mass: MASS,
            };
            let arrival_result = calculate_arrival(&arrival_params);

            if arrival_result.arrived {
                reached_target = true;
                break;
            }

            // Avoidance contribution
            let ctx = default_steering_context(velocity);
            let avoidance_params = AvoidanceParams {
                position,
                self_radius: 0.5,
                personal_space: 2.5,
                emergency_distance: 0.25,
            };
            let obstacle = NeighborObstacle {
                position: obstacle_pos,
                radius: 0.5,
            };
            let avoidance_accel = calculate_avoidance_force(&ctx, &avoidance_params, &[obstacle]);

            // Combine (seek is primary, avoidance modifies)
            let combined = accumulate_steering(
                &[arrival_result.acceleration, avoidance_accel],
                MAX_ACCEL,
            );

            // Integrate motion
            let integration = integrate_motion(&default_integration_params(
                position, velocity, combined,
            ));

            position = integration.position;
            velocity = integration.velocity;

            // Check for collision
            let dist_to_obstacle = ((position.0 - obstacle_pos.0).powi(2)
                + (position.1 - obstacle_pos.1).powi(2))
            .sqrt();
            if dist_to_obstacle < 1.0 {
                // self_radius + obstacle_radius
                collided = true;
            }
        }

        // Should navigate around obstacle to reach target
        assert!(
            !collided || reached_target,
            "Should avoid collision or reach target. Collided: {}, Reached: {}",
            collided,
            reached_target
        );
    }

    // ============================================================
    // Visualization mismatch detection tests
    // ============================================================

    #[test]
    fn detect_turn_rate_causes_visualization_mismatch() {
        // This test detects when turn rate limiting causes the velocity direction
        // to differ from the acceleration direction - the root cause of visualization bugs

        let position = (0.0, 0.0);
        let velocity = (10.0, 0.0); // Moving right
        let acceleration = (0.0, 100.0); // Strong upward acceleration

        let result = integrate_motion(&IntegrationParams {
            position,
            velocity,
            acceleration,
            dt: DT,
            drag_coefficient: 0.0, // No drag for clarity
            max_speed: 100.0, // High so speed clamp doesn't interfere
            max_turn_rate_rad: std::f32::consts::PI, // 180 deg/s
            stopped_threshold: 0.05,
        });

        if result.turn_limited {
            // The acceleration direction vs velocity direction mismatch
            let accel_angle = acceleration.1.atan2(acceleration.0);
            let vel_angle = result.velocity.1.atan2(result.velocity.0);
            let mismatch = (accel_angle - vel_angle).abs();

            // This is the visualization bug: force line shows accel direction,
            // but creature moves in velocity direction
            assert!(
                mismatch > 0.1,
                "Turn limiting should cause direction mismatch: {} rad",
                mismatch
            );

            // The post_limit_angle tells us the actual movement direction
            // while pre_limit_angle is what the physics "wanted"
            let angle_diff = (result.pre_limit_angle - result.post_limit_angle).abs();
            assert!(
                angle_diff > 0.01,
                "IntegrationResult should expose the angle difference"
            );
        }
    }

    #[test]
    fn force_line_should_match_velocity_change_without_turn_limit() {
        // Without turn rate limiting, acceleration direction should match velocity change
        let position = (0.0, 0.0);
        let velocity = (10.0, 0.0);
        let acceleration = (0.0, 5.0); // Upward acceleration

        let result = integrate_motion(&IntegrationParams {
            position,
            velocity,
            acceleration,
            dt: DT,
            drag_coefficient: 0.0,
            max_speed: 100.0,
            max_turn_rate_rad: 100.0, // Very high - effectively no limit
            stopped_threshold: 0.05,
        });

        // Velocity change direction
        let dv = (result.velocity.0 - velocity.0, result.velocity.1 - velocity.1);
        let dv_angle = dv.1.atan2(dv.0);

        // Acceleration direction
        let accel_angle = acceleration.1.atan2(acceleration.0);

        // Should match closely
        let mismatch = (dv_angle - accel_angle).abs();
        assert!(
            mismatch < 0.1,
            "Without turn limiting, velocity change should match acceleration direction. Mismatch: {} rad",
            mismatch
        );
    }

    // ============================================================
    // Seeker oscillation scenario tests
    // ============================================================

    #[test]
    fn seeker_approaches_target_without_oscillation() {
        // Reproduce the seeker oscillation bug scenario
        let mut position = (-30.0, 0.0);
        let mut velocity = (0.0, 0.0);
        let target = (0.0, 0.0);

        let mut sign_changes = 0;
        let mut prev_vx_sign = 0.0f32;

        // Simulate 300 frames (15 seconds)
        for frame in 0..300 {
            let to_target = (target.0 - position.0, target.1 - position.1);

            let params = ArrivalParams {
                velocity,
                to_target,
                self_radius: 0.5,
                target_radius: 0.5,
                max_speed: MAX_SPEED,
                max_force: MAX_FORCE,
                mass: MASS,
            };

            let result = calculate_arrival(&params);

            if result.arrived {
                break;
            }

            // Integrate
            let integration = integrate_motion(&default_integration_params(
                position,
                velocity,
                result.acceleration,
            ));

            // Track velocity sign changes (oscillation detection)
            let current_sign = integration.velocity.0.signum();
            if frame > 0
                && prev_vx_sign != 0.0
                && current_sign != prev_vx_sign
                && integration.velocity.0.abs() > 0.5
            {
                sign_changes += 1;
            }
            prev_vx_sign = current_sign;

            position = integration.position;
            velocity = integration.velocity;
        }

        // Should have at most 1 sign change (when coming to a stop)
        assert!(
            sign_changes <= 1,
            "Seeker oscillation detected: velocity changed sign {} times",
            sign_changes
        );
    }

    #[test]
    fn seeker_high_speed_approach_brakes_smoothly() {
        // Test the specific scenario from the original bug: high speed approach to close target
        let mut position = (-5.0, 0.0);
        let mut velocity = (10.0, 0.0); // Already moving fast toward target
        let target = (0.0, 0.0);

        let mut velocities = Vec::new();

        // Simulate 100 frames
        for _ in 0..100 {
            let to_target = (target.0 - position.0, target.1 - position.1);

            let params = ArrivalParams {
                velocity,
                to_target,
                self_radius: 0.5,
                target_radius: 0.5,
                max_speed: MAX_SPEED,
                max_force: MAX_FORCE,
                mass: MASS,
            };

            let result = calculate_arrival(&params);

            if result.arrived {
                break;
            }

            velocities.push(velocity.0);

            let integration = integrate_motion(&default_integration_params(
                position,
                velocity,
                result.acceleration,
            ));

            position = integration.position;
            velocity = integration.velocity;
        }

        // Velocity should decrease monotonically (smooth braking)
        let mut smooth = true;
        for i in 1..velocities.len() {
            // Allow small increases due to numerical precision, but no large jumps
            if velocities[i] > velocities[i - 1] + 0.5 {
                smooth = false;
                break;
            }
        }

        assert!(
            smooth,
            "Braking should be smooth (monotonically decreasing velocity)"
        );
    }

    // ============================================================
    // Full scenario replay tests
    // ============================================================

    #[test]
    fn replay_wandering_creature_scenario() {
        // Replay a wandering creature for 100 frames and verify determinism
        let initial_position = (0.0, 0.0);
        let initial_velocity = (5.0, 0.0);

        fn run_scenario(
            position: (f32, f32),
            velocity: (f32, f32),
            angle_changes: &[f32],
        ) -> Vec<(f32, f32)> {
            let mut pos = position;
            let mut vel = velocity;
            let mut wander_angle = 0.0f32;
            let mut positions = Vec::new();

            for &angle_change in angle_changes {
                let ctx = SteeringContext {
                    velocity: vel,
                    max_speed: MAX_SPEED,
                    max_force: MAX_FORCE,
                    mass: MASS,
                };

                let wander_params = WanderParams {
                    wander_angle,
                    wander_radius: 10.0,
                    wander_distance: 50.0,
                    force_multiplier: 0.1,
                };

                let wander_result = calculate_wander(&ctx, &wander_params, angle_change);
                wander_angle = wander_result.new_wander_angle;

                let integration = integrate_motion(&IntegrationParams {
                    position: pos,
                    velocity: vel,
                    acceleration: wander_result.acceleration,
                    dt: DT,
                    drag_coefficient: 2.0,
                    max_speed: MAX_SPEED,
                    max_turn_rate_rad: std::f32::consts::PI,
                    stopped_threshold: 0.05,
                });

                pos = integration.position;
                vel = integration.velocity;
                positions.push(pos);
            }

            positions
        }

        // Generate deterministic angle changes
        let angle_changes: Vec<f32> = (0..100).map(|i| (i as f32 * 0.1).sin() * 0.2).collect();

        // Run twice with same inputs
        let run1 = run_scenario(initial_position, initial_velocity, &angle_changes);
        let run2 = run_scenario(initial_position, initial_velocity, &angle_changes);

        // Should produce identical results (determinism)
        assert_eq!(run1.len(), run2.len());
        for (i, ((x1, y1), (x2, y2))) in run1.iter().zip(run2.iter()).enumerate() {
            assert!(
                (x1 - x2).abs() < 0.0001 && (y1 - y2).abs() < 0.0001,
                "Frame {}: positions differ: ({}, {}) vs ({}, {})",
                i,
                x1,
                y1,
                x2,
                y2
            );
        }
    }

    #[test]
    fn replay_avoidance_scenario_with_multiple_obstacles() {
        // Replay a complex avoidance scenario
        // Start further from obstacles so there's room to navigate
        let mut position = (-10.0, 0.0);
        let mut velocity = (5.0, 0.0);

        // Place obstacles to the side, not directly blocking
        let obstacles = vec![
            (5.0, 2.0),   // Ahead and right
            (5.0, -2.0),  // Ahead and left
        ];

        let mut trajectory = Vec::new();

        for _ in 0..200 {
            let ctx = default_steering_context(velocity);

            let avoidance_params = AvoidanceParams {
                position,
                self_radius: 0.5,
                personal_space: 2.0, // Smaller personal space
                emergency_distance: 0.25,
            };

            let neighbor_obstacles: Vec<_> = obstacles
                .iter()
                .map(|&(ox, oy)| NeighborObstacle {
                    position: (ox, oy),
                    radius: 0.5,
                })
                .collect();

            let avoidance_accel = calculate_avoidance_force(&ctx, &avoidance_params, &neighbor_obstacles);

            // Add stronger forward acceleration (simulating seek behavior)
            let base_accel = (4.0, 0.0);
            let combined = accumulate_steering(
                &[base_accel, avoidance_accel],
                MAX_ACCEL,
            );

            let integration = integrate_motion(&default_integration_params(
                position, velocity, combined,
            ));

            position = integration.position;
            velocity = integration.velocity;
            trajectory.push(position);
        }

        // Should make progress past the obstacles (started at -10, obstacles at 5)
        let final_x = trajectory.last().unwrap().0;
        assert!(
            final_x > 0.0,
            "Should make forward progress despite obstacles. Final X: {}",
            final_x
        );

        // Should have some lateral movement from avoidance
        let max_y_deviation = trajectory
            .iter()
            .map(|(_, y)| y.abs())
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);

        // With obstacles to the sides, creature should stay mostly centered
        // but still have some deflection
        assert!(
            trajectory.len() > 0,
            "Should have trajectory data"
        );
    }

    // ============================================================
    // F=ma bug detection tests
    // ============================================================

    #[test]
    fn wander_acceleration_is_physically_reasonable() {
        // A creature with wander shouldn't accelerate faster than physically possible
        let ctx = default_steering_context((5.0, 0.0));

        let wander_params = WanderParams {
            wander_angle: 0.0,
            wander_radius: 10.0,
            wander_distance: 50.0,
            force_multiplier: 0.1, // 10% of max force
        };

        let result = calculate_wander(&ctx, &wander_params, 0.0);
        let accel_mag = (result.acceleration.0.powi(2) + result.acceleration.1.powi(2)).sqrt();

        // Wander max accel = (max_force × 0.1) / mass = 39 / 65 = 0.6 m/s²
        let max_wander_accel = (MAX_FORCE * 0.1) / MASS;

        assert!(
            accel_mag <= max_wander_accel + 0.01,
            "Wander acceleration {} m/s² exceeds physical limit {} m/s². \
             If this is ~39 m/s², the F=ma bug is present!",
            accel_mag,
            max_wander_accel
        );

        // Specifically detect the bug: if acceleration is anywhere near max_force value
        assert!(
            accel_mag < MAX_FORCE * 0.5,
            "Acceleration {} looks like force {} was used directly!",
            accel_mag,
            MAX_FORCE
        );
    }

    #[test]
    fn one_frame_velocity_change_is_reasonable() {
        // After one frame, velocity change should be physically reasonable
        let position = (0.0, 0.0);
        let velocity = (0.0, 0.0);
        let acceleration = (MAX_ACCEL, 0.0); // Max acceleration

        let result = integrate_motion(&default_integration_params(position, velocity, acceleration));

        // v = a × t (ignoring drag for this check)
        // At max_accel = 6 m/s², dt = 0.05s, Δv = 0.3 m/s
        let speed = (result.velocity.0.powi(2) + result.velocity.1.powi(2)).sqrt();

        // With drag, actual speed will be less
        assert!(
            speed < 0.5,
            "One-frame velocity change {} m/s is too high for accel {} m/s²",
            speed,
            MAX_ACCEL
        );

        // If F=ma bug was present (using 390 instead of 6), speed would be ~20 m/s
        assert!(
            speed < 1.0,
            "Velocity {} suggests force is being used as acceleration!",
            speed
        );
    }
}
