use crate::simulation::core::components::{
    Acceleration, BodySize, Position, Rotation, Velocity,
};
use crate::simulation::creatures::components::{BehaviorMode, CreatureState};
use crate::simulation::creatures::constants::{
    MAX_TURN_RATE, MAX_TURN_RATE_DEG, MIN_TURN_RATE_DEG, NOISE_SPEED_THRESHOLD_SQ,
    TURN_RATE_SIZE_EXPONENT, TURN_RATE_SPEED_PENALTY,
};
use crate::simulation::math::{fast_atan2, normalize_angle};
use crate::simulation::movement::noise::NoiseTable;

pub struct IntegrateCtx<'a> {
    pub dt: f32,
    pub tick: u64,
    pub drag_factor: f32,
    pub noise_base: f32,
    pub noise_time_scale: f32,
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
    pub noise_table: &'a NoiseTable,
    pub stopped_threshold_sq: f32,
}

#[inline]
pub fn step(
    entity_index: u32,
    size: &BodySize,
    position: &mut Position,
    velocity: &mut Velocity,
    acceleration: &mut Acceleration,
    creature_state: &CreatureState,
    rotation: &mut Rotation,
    ctx: &IntegrateCtx<'_>,
) {
    let dt = ctx.dt;
    let drag_factor = ctx.drag_factor;
    let stopped_threshold_sq = ctx.stopped_threshold_sq;

    if creature_state.behavior == BehaviorMode::Catatonic {
        acceleration.ax = 0.0;
        acceleration.ay = 0.0;

        let speed_sq = velocity.vx * velocity.vx + velocity.vy * velocity.vy;
        if speed_sq < stopped_threshold_sq {
            if velocity.vx != 0.0 || velocity.vy != 0.0 {
                velocity.vx = 0.0;
                velocity.vy = 0.0;
            }
            return;
        }

        velocity.vx *= drag_factor;
        velocity.vy *= drag_factor;

        position.x += velocity.vx * dt;
        position.y += velocity.vy * dt;

        if position.x < ctx.min_x {
            position.x = ctx.min_x;
            velocity.vx = velocity.vx.max(0.0);
        } else if position.x > ctx.max_x {
            position.x = ctx.max_x;
            velocity.vx = velocity.vx.min(0.0);
        }
        if position.y < ctx.min_y {
            position.y = ctx.min_y;
            velocity.vy = velocity.vy.max(0.0);
        } else if position.y > ctx.max_y {
            position.y = ctx.max_y;
            velocity.vy = velocity.vy.min(0.0);
        }

        return;
    }

    let old_speed_sq = velocity.vx * velocity.vx + velocity.vy * velocity.vy;
    let old_angle = if old_speed_sq > stopped_threshold_sq {
        fast_atan2(velocity.vy, velocity.vx)
    } else {
        rotation.radians
    };

    velocity.vx += acceleration.ax * dt;
    velocity.vy += acceleration.ay * dt;
    velocity.vx *= drag_factor;
    velocity.vy *= drag_factor;

    let max_speed = size.max_speed();
    let max_speed_sq = max_speed * max_speed;

    let mut speed_sq = velocity.vx * velocity.vx + velocity.vy * velocity.vy;
    let mut current_speed = 0.0_f32;
    let mut speed_computed = false;

    if speed_sq > NOISE_SPEED_THRESHOLD_SQ {
        current_speed = speed_sq.sqrt();
        let speed_ratio = current_speed / max_speed;
        let size_factor = size.inv_sqrt_length;
        let noise_magnitude = ctx.noise_base * speed_ratio * speed_ratio * size_factor;

        let noise_x = ctx
            .noise_table
            .get(entity_index, ctx.tick, 0, ctx.noise_time_scale);
        let noise_y = ctx
            .noise_table
            .get(entity_index, ctx.tick, 1, ctx.noise_time_scale);

        let inv_speed = 1.0 / current_speed;
        let perpendicular_x = -velocity.vy * inv_speed;
        let perpendicular_y = velocity.vx * inv_speed;

        velocity.vx += perpendicular_x * noise_x * noise_magnitude * dt;
        velocity.vy += perpendicular_y * noise_y * noise_magnitude * dt;

        speed_sq = velocity.vx * velocity.vx + velocity.vy * velocity.vy;
        speed_computed = false;
    }

    let was_clamped = speed_sq > max_speed_sq;
    if was_clamped {
        if !speed_computed {
            current_speed = speed_sq.sqrt();
        }
        let scale = max_speed / current_speed;
        velocity.vx *= scale;
        velocity.vy *= scale;
        current_speed = max_speed;
        speed_sq = max_speed_sq;
    }

    if speed_sq > stopped_threshold_sq {
        let base_turn_rate_deg =
            (MAX_TURN_RATE / size.length.powf(TURN_RATE_SIZE_EXPONENT))
                .clamp(MIN_TURN_RATE_DEG, MAX_TURN_RATE_DEG);

        let current_speed_for_penalty = if speed_computed || was_clamped {
            current_speed
        } else {
            speed_sq.sqrt()
        };
        let normalized_speed = (current_speed_for_penalty / max_speed).min(1.0);
        let speed_factor =
            1.0 - TURN_RATE_SPEED_PENALTY * normalized_speed * normalized_speed;
        let effective_turn_rate_deg = base_turn_rate_deg * speed_factor;

        let max_delta = effective_turn_rate_deg.to_radians() * dt;

        let new_angle = fast_atan2(velocity.vy, velocity.vx);
        let delta = normalize_angle(new_angle - old_angle);

        if delta.abs() > max_delta {
            let clamped_delta = delta.clamp(-max_delta, max_delta);
            let final_angle = old_angle + clamped_delta;
            let new_speed = current_speed_for_penalty;
            velocity.vx = new_speed * final_angle.cos();
            velocity.vy = new_speed * final_angle.sin();
        }
    }

    acceleration.ax = 0.0;
    acceleration.ay = 0.0;

    position.x += velocity.vx * dt;
    position.y += velocity.vy * dt;

    if position.x < ctx.min_x {
        position.x = ctx.min_x;
        velocity.vx = velocity.vx.max(0.0);
    } else if position.x > ctx.max_x {
        position.x = ctx.max_x;
        velocity.vx = velocity.vx.min(0.0);
    }
    if position.y < ctx.min_y {
        position.y = ctx.min_y;
        velocity.vy = velocity.vy.max(0.0);
    } else if position.y > ctx.max_y {
        position.y = ctx.max_y;
        velocity.vy = velocity.vy.min(0.0);
    }

    let final_speed_sq = velocity.vx * velocity.vx + velocity.vy * velocity.vy;
    if final_speed_sq > stopped_threshold_sq {
        rotation.set_from_velocity(velocity.vx, velocity.vy);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::core::components::{Acceleration, BodySize, Position, Rotation, Velocity};
    use crate::simulation::creatures::components::{BehaviorMode, CreatureState};
    use crate::simulation::creatures::constants::DRAG_COEFFICIENT;
    use crate::simulation::movement::noise::NoiseTable;

    fn make_ctx<'a>(
        dt: f32,
        noise_base: f32,
        noise_table: &'a NoiseTable,
        min_x: f32,
        max_x: f32,
        min_y: f32,
        max_y: f32,
    ) -> IntegrateCtx<'a> {
        use crate::simulation::creatures::constants::STOPPED_THRESHOLD;
        IntegrateCtx {
            dt,
            tick: 0,
            drag_factor: (-DRAG_COEFFICIENT * dt).exp(),
            noise_base,
            noise_time_scale: 0.05,
            min_x,
            max_x,
            min_y,
            max_y,
            noise_table,
            stopped_threshold_sq: STOPPED_THRESHOLD * STOPPED_THRESHOLD,
        }
    }

    fn default_ctx(noise_table: &NoiseTable) -> IntegrateCtx<'_> {
        make_ctx(0.05, 0.0, noise_table, -100.0, 100.0, -100.0, 100.0)
    }

    fn wide_ctx(noise_table: &NoiseTable) -> IntegrateCtx<'_> {
        make_ctx(0.05, 0.0, noise_table, -10000.0, 10000.0, -10000.0, 10000.0)
    }

    fn wandering() -> CreatureState {
        let mut s = CreatureState::default();
        s.behavior = BehaviorMode::Wandering;
        s
    }

    fn catatonic() -> CreatureState {
        CreatureState::default()
    }

    #[test]
    fn step_catatonic_clears_accel_and_applies_drag_to_coasting_velocity() {
        let nt = NoiseTable::default();
        let ctx = default_ctx(&nt);
        let mut pos = Position { x: 5.0, y: 3.0 };
        let mut vel = Velocity { vx: 2.0, vy: 1.0 };
        let mut acc = Acceleration { ax: 9.0, ay: 4.0 };
        let state = catatonic();
        let mut rot = Rotation::default();

        step(0, &BodySize::new(1.0), &mut pos, &mut vel, &mut acc, &state, &mut rot, &ctx);

        assert_eq!(acc.ax, 0.0, "acceleration x must be cleared");
        assert_eq!(acc.ay, 0.0, "acceleration y must be cleared");

        let expected_vx = 2.0_f32 * ctx.drag_factor;
        let expected_vy = 1.0_f32 * ctx.drag_factor;
        assert!(
            (vel.vx - expected_vx).abs() < 1e-6,
            "vx={} expected={}", vel.vx, expected_vx
        );
        assert!(
            (vel.vy - expected_vy).abs() < 1e-6,
            "vy={} expected={}", vel.vy, expected_vy
        );
        let expected_x = 5.0 + expected_vx * ctx.dt;
        let expected_y = 3.0 + expected_vy * ctx.dt;
        assert!(
            (pos.x - expected_x).abs() < 1e-6,
            "pos.x={} expected={}", pos.x, expected_x
        );
        assert!(
            (pos.y - expected_y).abs() < 1e-6,
            "pos.y={} expected={}", pos.y, expected_y
        );
    }

    #[test]
    fn step_catatonic_below_stopped_threshold_zeroes_velocity_exactly() {
        let nt = NoiseTable::default();
        let ctx = default_ctx(&nt);
        // speed_sq = 0.01^2 = 0.0001 < stopped_threshold_sq = 0.0025
        let mut pos = Position { x: 1.0, y: 2.0 };
        let mut vel = Velocity { vx: 0.01, vy: 0.0 };
        let mut acc = Acceleration { ax: 0.0, ay: 0.0 };
        let state = catatonic();
        let mut rot = Rotation::default();
        let pos_before = (pos.x, pos.y);

        step(0, &BodySize::new(1.0), &mut pos, &mut vel, &mut acc, &state, &mut rot, &ctx);

        assert_eq!(vel.vx, 0.0, "velocity x must be zeroed exactly below threshold");
        assert_eq!(vel.vy, 0.0, "velocity y must be zeroed exactly below threshold");
        assert_eq!(pos.x, pos_before.0, "position must not change when stopped");
        assert_eq!(pos.y, pos_before.1, "position must not change when stopped");
    }

    #[test]
    fn step_catatonic_at_zero_velocity_leaves_position_and_clears_accel() {
        let nt = NoiseTable::default();
        let ctx = default_ctx(&nt);
        let mut pos = Position { x: 7.0, y: -3.0 };
        let mut vel = Velocity { vx: 0.0, vy: 0.0 };
        let mut acc = Acceleration { ax: 5.0, ay: -2.0 };
        let state = catatonic();
        let mut rot = Rotation::default();

        step(0, &BodySize::new(1.0), &mut pos, &mut vel, &mut acc, &state, &mut rot, &ctx);

        assert_eq!(pos.x, 7.0);
        assert_eq!(pos.y, -3.0);
        assert_eq!(vel.vx, 0.0);
        assert_eq!(vel.vy, 0.0);
        assert_eq!(acc.ax, 0.0, "acceleration must be cleared even for stopped catatonic");
        assert_eq!(acc.ay, 0.0);
    }

    #[test]
    fn step_catatonic_beyond_max_x_clamps_position_and_velocity() {
        let nt = NoiseTable::default();
        let ctx = default_ctx(&nt);
        // starts at x=110, beyond max_x=100, moving away
        let mut pos = Position { x: 110.0, y: 0.0 };
        let mut vel = Velocity { vx: 5.0, vy: 0.0 };
        let mut acc = Acceleration { ax: 2.0, ay: 0.0 };
        let state = catatonic();
        let mut rot = Rotation::default();

        step(0, &BodySize::new(1.0), &mut pos, &mut vel, &mut acc, &state, &mut rot, &ctx);

        assert_eq!(pos.x, 100.0, "catatonic position must be clamped to max_x");
        assert!(
            vel.vx <= 0.0,
            "catatonic velocity must be non-positive at max_x boundary; vx={}", vel.vx
        );
    }

    #[test]
    fn step_normal_integrates_eastward_motion_with_exact_drag() {
        let nt = NoiseTable::default();
        let ctx = wide_ctx(&nt);
        let drag = ctx.drag_factor;
        let dt = ctx.dt;

        let mut pos = Position { x: 0.0, y: 0.0 };
        let mut vel = Velocity { vx: 10.0, vy: 0.0 };
        let mut acc = Acceleration { ax: 0.0, ay: 0.0 };
        let state = wandering();
        let mut rot = Rotation::new(0.0);

        step(0, &BodySize::new(1.0), &mut pos, &mut vel, &mut acc, &state, &mut rot, &ctx);

        let expected_vx = 10.0_f32 * drag;
        assert!(
            (vel.vx - expected_vx).abs() < 1e-6,
            "vx={} expected={}", vel.vx, expected_vx
        );
        assert_eq!(vel.vy, 0.0);
        assert!(
            (pos.x - expected_vx * dt).abs() < 1e-6,
            "pos.x={} expected={}", pos.x, expected_vx * dt
        );
        assert_eq!(pos.y, 0.0);
    }

    #[test]
    fn step_normal_acceleration_applied_before_drag_in_exact_order() {
        let nt = NoiseTable::default();
        let ctx = wide_ctx(&nt);
        let drag = ctx.drag_factor;
        let dt = ctx.dt;

        let mut pos = Position { x: 0.0, y: 0.0 };
        let mut vel = Velocity { vx: 0.0, vy: 0.0 };
        let mut acc = Acceleration { ax: 10.0, ay: 0.0 };
        let state = wandering();
        let mut rot = Rotation::new(0.0);

        step(0, &BodySize::new(1.0), &mut pos, &mut vel, &mut acc, &state, &mut rot, &ctx);

        let expected_vx = (0.0 + 10.0 * dt) * drag;
        assert!(
            (vel.vx - expected_vx).abs() < 1e-6,
            "order must be v += a*dt then *= drag; vx={} expected={}", vel.vx, expected_vx
        );
        assert_eq!(vel.vy, 0.0);
    }

    #[test]
    fn step_normal_clears_acceleration_after_integration() {
        let nt = NoiseTable::default();
        let ctx = wide_ctx(&nt);

        let mut pos = Position { x: 0.0, y: 0.0 };
        let mut vel = Velocity { vx: 5.0, vy: 0.0 };
        let mut acc = Acceleration { ax: 3.0, ay: -2.0 };
        let state = wandering();
        let mut rot = Rotation::new(0.0);

        step(0, &BodySize::new(1.0), &mut pos, &mut vel, &mut acc, &state, &mut rot, &ctx);

        assert_eq!(acc.ax, 0.0, "acceleration x must be cleared after step");
        assert_eq!(acc.ay, 0.0, "acceleration y must be cleared after step");
    }

    #[test]
    fn step_normal_clamps_speed_to_max_for_body_size() {
        let nt = NoiseTable::default();
        let ctx = wide_ctx(&nt);
        let size = BodySize::new(1.0);
        let max_speed = size.max_speed();

        let mut pos = Position { x: 0.0, y: 0.0 };
        let mut vel = Velocity {
            vx: max_speed * 10.0,
            vy: 0.0,
        };
        let mut acc = Acceleration { ax: 0.0, ay: 0.0 };
        let state = wandering();
        let mut rot = Rotation::new(0.0);

        step(0, &size, &mut pos, &mut vel, &mut acc, &state, &mut rot, &ctx);

        let actual_speed = (vel.vx * vel.vx + vel.vy * vel.vy).sqrt();
        assert!(
            actual_speed <= max_speed + 1e-5,
            "speed={} must be <= max_speed={}", actual_speed, max_speed
        );
    }

    #[test]
    fn step_normal_beyond_min_x_clamps_position_and_velocity() {
        let nt = NoiseTable::default();
        let ctx = default_ctx(&nt);

        let mut pos = Position { x: -200.0, y: 0.0 };
        let mut vel = Velocity { vx: -5.0, vy: 0.0 };
        let mut acc = Acceleration { ax: 0.0, ay: 0.0 };
        let state = wandering();
        let mut rot = Rotation::new(std::f32::consts::PI);

        step(0, &BodySize::new(1.0), &mut pos, &mut vel, &mut acc, &state, &mut rot, &ctx);

        assert_eq!(pos.x, -100.0, "position must be clamped to min_x");
        assert!(
            vel.vx >= 0.0,
            "velocity x must be non-negative at min_x boundary; got {}", vel.vx
        );
    }

    #[test]
    fn step_normal_beyond_max_x_clamps_position_and_velocity() {
        let nt = NoiseTable::default();
        let ctx = default_ctx(&nt);

        let mut pos = Position { x: 200.0, y: 0.0 };
        let mut vel = Velocity { vx: 5.0, vy: 0.0 };
        let mut acc = Acceleration { ax: 0.0, ay: 0.0 };
        let state = wandering();
        let mut rot = Rotation::new(0.0);

        step(0, &BodySize::new(1.0), &mut pos, &mut vel, &mut acc, &state, &mut rot, &ctx);

        assert_eq!(pos.x, 100.0, "position must be clamped to max_x");
        assert!(
            vel.vx <= 0.0,
            "velocity x must be non-positive at max_x boundary; got {}", vel.vx
        );
    }

    #[test]
    fn step_normal_beyond_min_y_clamps_position_and_velocity() {
        let nt = NoiseTable::default();
        let ctx = default_ctx(&nt);

        let mut pos = Position { x: 0.0, y: -200.0 };
        let mut vel = Velocity { vx: 0.0, vy: -5.0 };
        let mut acc = Acceleration { ax: 0.0, ay: 0.0 };
        let state = wandering();
        let mut rot = Rotation::new(-std::f32::consts::FRAC_PI_2);

        step(0, &BodySize::new(1.0), &mut pos, &mut vel, &mut acc, &state, &mut rot, &ctx);

        assert_eq!(pos.y, -100.0, "position must be clamped to min_y");
        assert!(
            vel.vy >= 0.0,
            "velocity y must be non-negative at min_y boundary; got {}", vel.vy
        );
    }

    #[test]
    fn step_normal_beyond_max_y_clamps_position_and_velocity() {
        let nt = NoiseTable::default();
        let ctx = default_ctx(&nt);

        let mut pos = Position { x: 0.0, y: 200.0 };
        let mut vel = Velocity { vx: 0.0, vy: 5.0 };
        let mut acc = Acceleration { ax: 0.0, ay: 0.0 };
        let state = wandering();
        let mut rot = Rotation::new(std::f32::consts::FRAC_PI_2);

        step(0, &BodySize::new(1.0), &mut pos, &mut vel, &mut acc, &state, &mut rot, &ctx);

        assert_eq!(pos.y, 100.0, "position must be clamped to max_y");
        assert!(
            vel.vy <= 0.0,
            "velocity y must be non-positive at max_y boundary; got {}", vel.vy
        );
    }

    #[test]
    fn step_normal_updates_rotation_when_final_speed_above_threshold() {
        let nt = NoiseTable::default();
        let ctx = wide_ctx(&nt);

        let mut pos = Position { x: 0.0, y: 0.0 };
        let mut vel = Velocity { vx: 0.0, vy: 5.0 };
        let mut acc = Acceleration { ax: 0.0, ay: 0.0 };
        let state = wandering();
        let mut rot = Rotation::new(0.0);

        step(0, &BodySize::new(1.0), &mut pos, &mut vel, &mut acc, &state, &mut rot, &ctx);

        let final_speed = (vel.vx * vel.vx + vel.vy * vel.vy).sqrt();
        assert!(final_speed > ctx.stopped_threshold_sq.sqrt(), "precondition: final speed above threshold");

        let expected_angle = std::f32::consts::FRAC_PI_2;
        let angle_diff = (rot.radians - expected_angle).abs();
        assert!(
            angle_diff < 0.20,
            "rotation must update to ~North (π/2); got {:.3} rad ({:.1}°)",
            rot.radians, rot.radians.to_degrees()
        );
    }

    #[test]
    fn step_normal_preserves_rotation_when_speed_zero_crosses_below_threshold() {
        let nt = NoiseTable::default();
        let ctx = wide_ctx(&nt);

        let size = BodySize::new(10.0);
        let mut pos = Position { x: 0.0, y: 0.0 };
        let mut vel = Velocity { vx: 0.12, vy: 0.0 };
        let mut acc = Acceleration { ax: -3.162, ay: 0.0 };
        let state = wandering();
        let mut rot = Rotation::new(0.0);

        step(0, &size, &mut pos, &mut vel, &mut acc, &state, &mut rot, &ctx);

        let new_speed_sq = vel.vx * vel.vx + vel.vy * vel.vy;
        assert!(
            new_speed_sq < ctx.stopped_threshold_sq,
            "precondition: zero-crossing must have occurred; speed_sq={}", new_speed_sq
        );
        assert!(
            rot.radians.abs() < 0.35,
            "rotation must be preserved at East when speed zero-crosses below threshold; \
             got {:.1}° — ±180° indicates set_from_velocity is called after zero-cross",
            rot.radians.to_degrees()
        );
    }

    #[test]
    fn step_turn_rate_constrains_heading_from_stored_east_when_stopped() {
        let nt = NoiseTable::default();
        let ctx = wide_ctx(&nt);

        let size = BodySize::new(1.0);
        let mut pos = Position { x: 0.0, y: 0.0 };
        let mut vel = Velocity { vx: 0.0, vy: 0.0 };
        let mut acc = Acceleration { ax: 0.0, ay: 10.0 };
        let state = wandering();
        let mut rot = Rotation::new(0.0);

        step(0, &size, &mut pos, &mut vel, &mut acc, &state, &mut rot, &ctx);

        let speed = (vel.vx * vel.vx + vel.vy * vel.vy).sqrt();
        assert!(speed > 0.0, "creature must have started moving");

        assert!(
            rot.radians.abs() < 0.20,
            "heading must be turn-rate-limited to ~9° from stored East; got {:.1}°",
            rot.radians.to_degrees()
        );
        assert!(
            vel.vx > vel.vy.abs(),
            "East component must dominate after one tick of rate-limiting from stored East; \
             vx={:.4}, vy={:.4}", vel.vx, vel.vy
        );
    }

    #[test]
    fn step_normal_zero_velocity_zero_accel_position_unchanged() {
        let nt = NoiseTable::default();
        let ctx = wide_ctx(&nt);

        let mut pos = Position { x: 3.0, y: -7.0 };
        let mut vel = Velocity { vx: 0.0, vy: 0.0 };
        let mut acc = Acceleration { ax: 0.0, ay: 0.0 };
        let state = wandering();
        let mut rot = Rotation::default();

        step(0, &BodySize::new(1.0), &mut pos, &mut vel, &mut acc, &state, &mut rot, &ctx);

        assert_eq!(pos.x, 3.0, "position x unchanged with zero velocity and accel");
        assert_eq!(pos.y, -7.0, "position y unchanged with zero velocity and accel");
    }
}
