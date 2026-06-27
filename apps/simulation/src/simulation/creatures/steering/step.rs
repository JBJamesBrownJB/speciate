use super::avoidance::{calculate_avoidance, AvoidanceConfig, AvoidanceInput, Neighbor};
use super::seek::{calculate_arrival, ArrivalParams};
use super::wander::{calculate_wander, WanderParams};
use crate::simulation::core::components::{BodySize, Position, Velocity};
use crate::simulation::creatures::components::{BehaviorMode, CreatureState, Target, WanderState};
use crate::simulation::math::SteeringContext;
use crate::simulation::perception::NeighborCache;

pub struct SteeringCtx {
    pub wander_force_mult: f32,
    pub seek_force_mult: f32,
}

pub struct SteeringOutput {
    pub ax: f32,
    pub ay: f32,
    pub arrived: bool,
}

#[inline(always)]
pub fn step(
    position: &Position,
    velocity: &Velocity,
    size: &BodySize,
    creature_state: &CreatureState,
    wander_state: &mut WanderState,
    target: &Target,
    neighbor_cache: &NeighborCache,
    can_wander: bool,
    can_seek: bool,
    can_avoid: bool,
    wander_angle_change_radians: f32,
    ctx: &SteeringCtx,
) -> SteeringOutput {
    let mut ax = 0.0_f32;
    let mut ay = 0.0_f32;
    let mut arrived = false;

    let steer_ctx = SteeringContext {
        velocity: (velocity.vx, velocity.vy),
        max_speed: size.max_speed(),
        max_force: size.max_force(),
        mass: size.mass(),
    };

    match creature_state.behavior {
        BehaviorMode::Wandering if can_wander => {
            let wander_params = WanderParams {
                wander_angle: wander_state.wander_angle,
                wander_radius: wander_state.wander_radius,
                wander_distance: wander_state.wander_distance,
                force_multiplier: ctx.wander_force_mult,
            };
            let result =
                calculate_wander(&steer_ctx, &wander_params, wander_angle_change_radians);
            wander_state.wander_angle = result.new_wander_angle;
            let (fx, fy) = result.acceleration;
            if fx.is_finite() && fy.is_finite() {
                ax += fx;
                ay += fy;
            }
        }
        BehaviorMode::Seeking if can_seek => {
            let params = ArrivalParams {
                velocity: (velocity.vx, velocity.vy),
                to_target: (target.x - position.x, target.y - position.y),
                self_radius: size.radius(),
                target_radius: target.radius.get(),
                max_speed: size.max_speed(),
                max_force: size.max_force() * ctx.seek_force_mult,
                mass: size.mass(),
            };
            let result = calculate_arrival(&params);
            if result.arrived {
                arrived = true;
            } else {
                ax += result.acceleration.0;
                ay += result.acceleration.1;
            }
        }
        _ => {}
    }

    if can_avoid && neighbor_cache.has_neighbors() {
        let max_accel = size.max_force() / size.mass();
        let input = AvoidanceInput {
            self_pos: (position.x, position.y),
            self_vel: (velocity.vx, velocity.vy),
            self_radius: size.radius(),
            max_accel,
        };
        let neighbors = neighbor_cache.iter_neighbors().map(|n| Neighbor {
            pos: (n.x, n.y),
            vel: (n.vx, n.vy),
            radius: n.radius,
        });
        let config = AvoidanceConfig::default();
        let avoidance_out = calculate_avoidance(&input, neighbors, &config);
        ax += avoidance_out.accel.0;
        ay += avoidance_out.accel.1;
    }

    let max_accel = size.max_force() / size.mass();
    let mag_sq = ax * ax + ay * ay;
    let max_sq = max_accel * max_accel;
    if mag_sq > max_sq && mag_sq > 0.0001 {
        let scale = max_accel / mag_sq.sqrt();
        ax *= scale;
        ay *= scale;
    }

    SteeringOutput { ax, ay, arrived }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::core::components::{BodySize, Position, Velocity};
    use crate::simulation::creatures::components::{
        BehaviorMode, CreatureState, Target, WanderState,
    };
    use crate::simulation::creatures::constants::{SEEK_FORCE_MULT, WANDER_FORCE_MULT};
    use crate::simulation::perception::components::NeighborData;
    use crate::simulation::perception::NeighborCache;
    use bevy_ecs::prelude::Entity;

    fn default_ctx() -> SteeringCtx {
        SteeringCtx {
            wander_force_mult: WANDER_FORCE_MULT.get(),
            seek_force_mult: SEEK_FORCE_MULT.get(),
        }
    }

    fn default_wander_state() -> WanderState {
        WanderState {
            wander_angle: 0.0,
            wander_radius: 10.0,
            wander_distance: 20.0,
            angle_change: 4.5,
        }
    }

    fn state_with_behavior(behavior: BehaviorMode) -> CreatureState {
        CreatureState { behavior, ..CreatureState::default() }
    }

    #[test]
    fn step_seek_far_east_yields_exact_7_0() {
        // pos=(0,0), vel=(0,0), target=(100,0), BodySize::default() (length=1.0)
        // mass=35, max_force=350, seek_force_mult=0.7 → seek_max_force=245
        // max_accel_seek=245/35=7.0, far from target → desired_speed=max_speed=12
        // steer=(12,0), steer_mag=12>7 → clamp → ax=7.0, ay=0.0
        // global cap: max_accel=350/35=10, mag_sq=49<100 → not capped
        let size = BodySize::default();
        let pos = Position { x: 0.0, y: 0.0 };
        let vel = Velocity { vx: 0.0, vy: 0.0 };
        let state = state_with_behavior(BehaviorMode::Seeking);
        let mut wander = default_wander_state();
        let target = Target::at_point(100.0, 0.0);
        let cache = NeighborCache::new();
        let ctx = default_ctx();

        let out = step(&pos, &vel, &size, &state, &mut wander, &target, &cache, false, true, false, 0.0, &ctx);

        assert!(!out.arrived, "must not have arrived at far target");
        assert!(
            (out.ax - 7.0).abs() < 1e-4,
            "ax={} expected 7.0 (seek_max_accel=245/35)",
            out.ax
        );
        assert!(
            out.ay.abs() < 1e-4,
            "ay={} expected 0.0 for east-only target",
            out.ay
        );
    }

    #[test]
    fn step_seek_far_north_yields_exact_0_7() {
        // Symmetric to east case; target=(0,100) → ax=0.0, ay=7.0
        let size = BodySize::default();
        let pos = Position { x: 0.0, y: 0.0 };
        let vel = Velocity { vx: 0.0, vy: 0.0 };
        let state = state_with_behavior(BehaviorMode::Seeking);
        let mut wander = default_wander_state();
        let target = Target::at_point(0.0, 100.0);
        let cache = NeighborCache::new();
        let ctx = default_ctx();

        let out = step(&pos, &vel, &size, &state, &mut wander, &target, &cache, false, true, false, 0.0, &ctx);

        assert!(!out.arrived);
        assert!(
            out.ax.abs() < 1e-4,
            "ax={} expected 0.0 for north-only target",
            out.ax
        );
        assert!(
            (out.ay - 7.0).abs() < 1e-4,
            "ay={} expected 7.0 (seek_max_accel=245/35)",
            out.ay
        );
    }

    #[test]
    fn step_seek_snap_sets_arrived_true_and_zero_accel() {
        // Creature very close to target (edge_dist < SNAP_EDGE_THRESHOLD=0.1) and slow
        // pos=(0.95,0), vel=(0.1,0), target=(1.0,0) with radius=1.0
        // center_dist=0.05, edge_dist=max(0, 0.05-0.5-1.0)=0 → snap!
        let size = BodySize::new(1.0); // radius=0.5
        let pos = Position { x: 0.95, y: 0.0 };
        let vel = Velocity { vx: 0.1, vy: 0.0 };
        let state = state_with_behavior(BehaviorMode::Seeking);
        let mut wander = default_wander_state();
        let target = Target::at_point(1.0, 0.0); // radius=1.0
        let cache = NeighborCache::new();
        let ctx = default_ctx();

        let out = step(&pos, &vel, &size, &state, &mut wander, &target, &cache, false, true, false, 0.0, &ctx);

        assert!(out.arrived, "must arrive when edge_dist < snap threshold and speed < snap max");
        assert_eq!(out.ax, 0.0, "arrived → no seek acceleration");
        assert_eq!(out.ay, 0.0, "arrived → no seek acceleration");
    }

    #[test]
    fn step_avoidance_right_threat_yields_exact_minus_10_0() {
        // Catatonic creature at (0,0) with neighbor approaching from right: (10,0) vel=(-5,0)
        // max_accel=350/35=10.0
        // dir_to_neighbor=(1,0), rel_vel=(-5,0), closing_speed=5.0
        // edge_dist=10-0.5-0.5=9.0, ttc=9/5=1.8, urgency=(2/1.8).clamp(0,1)=1.0
        // force_mag=10.0, avoidance=(- 10*1, -10*0)=(-10, 0)
        // combined ax=-10, ay=0; global cap: mag_sq=100=max_sq → not capped
        let size = BodySize::default();
        let pos = Position { x: 0.0, y: 0.0 };
        let vel = Velocity { vx: 0.0, vy: 0.0 };
        let state = state_with_behavior(BehaviorMode::Catatonic);
        let mut wander = default_wander_state();
        let target = Target::at_point(0.0, 0.0);
        let mut cache = NeighborCache::new();
        cache.add_neighbor(NeighborData {
            entity: Entity::PLACEHOLDER,
            x: 10.0,
            y: 0.0,
            vx: -5.0,
            vy: 0.0,
            radius: 0.5,
        });
        let ctx = default_ctx();

        let out = step(&pos, &vel, &size, &state, &mut wander, &target, &cache, false, false, true, 0.0, &ctx);

        assert!(!out.arrived);
        assert!(
            (out.ax - (-10.0)).abs() < 1e-4,
            "ax={} expected -10.0 (full urgency away from approaching neighbor)",
            out.ax
        );
        assert!(
            out.ay.abs() < 1e-4,
            "ay={} expected 0.0 for head-on X approach",
            out.ay
        );
    }

    #[test]
    fn step_wander_zero_angle_east_heading_yields_exact_2_5_0() {
        // Wandering creature at (0,0) vel=(1,0), wander_angle=0, wander_distance=20, radius=10
        // angle_change=0 → heading=(1,0), circle_center=(20,0), target_on_circle=(30,0)
        // desired_v=(12,0), steer=(11,0)
        // wander_max_force=350*0.25=87.5, wander_max_accel=87.5/35=2.5
        // steer_mag=11>2.5 → clamp → ax=2.5, ay=0.0
        // global cap: max_accel=10, mag_sq=6.25<100 → not capped
        let size = BodySize::default();
        let pos = Position { x: 0.0, y: 0.0 };
        let vel = Velocity { vx: 1.0, vy: 0.0 };
        let state = state_with_behavior(BehaviorMode::Wandering);
        let mut wander = WanderState {
            wander_angle: 0.0,
            wander_radius: 10.0,
            wander_distance: 20.0,
            angle_change: 4.5,
        };
        let target = Target::at_point(0.0, 0.0);
        let cache = NeighborCache::new();
        let ctx = default_ctx();

        let out = step(&pos, &vel, &size, &state, &mut wander, &target, &cache, true, false, false, 0.0, &ctx);

        assert!(!out.arrived);
        assert!(
            (out.ax - 2.5).abs() < 1e-4,
            "ax={} expected 2.5 (wander_max_accel=87.5/35)",
            out.ax
        );
        assert!(
            out.ay.abs() < 1e-4,
            "ay={} expected 0.0 for east heading with zero angle change",
            out.ay
        );
    }

    #[test]
    fn step_catatonic_no_neighbors_yields_zero_output() {
        // Catatonic, no avoidance markers, no neighbors → everything zero
        let size = BodySize::default();
        let pos = Position { x: 0.0, y: 0.0 };
        let vel = Velocity { vx: 0.0, vy: 0.0 };
        let state = state_with_behavior(BehaviorMode::Catatonic);
        let mut wander = default_wander_state();
        let target = Target::at_point(0.0, 0.0);
        let cache = NeighborCache::new();
        let ctx = default_ctx();

        let out = step(&pos, &vel, &size, &state, &mut wander, &target, &cache, false, false, false, 0.0, &ctx);

        assert_eq!(out.ax, 0.0, "catatonic+no_avoid → ax must be 0");
        assert_eq!(out.ay, 0.0, "catatonic+no_avoid → ay must be 0");
        assert!(!out.arrived, "catatonic is not an arrival transition");
    }

    #[test]
    fn step_cap_fires_on_combined_seek_and_avoidance() {
        // Seeking east (ax=7.0) + avoidance neighbor approaching from below (ay=+10.0)
        // combined=(7,10), mag=sqrt(149)>10=max_accel → capped
        // final_ax=7*10/sqrt(149), final_ay=10*10/sqrt(149)
        let size = BodySize::default();
        let pos = Position { x: 0.0, y: 0.0 };
        let vel = Velocity { vx: 0.0, vy: 0.0 };
        let state = state_with_behavior(BehaviorMode::Seeking);
        let mut wander = default_wander_state();
        let target = Target::at_point(100.0, 0.0);
        let mut cache = NeighborCache::new();
        cache.add_neighbor(NeighborData {
            entity: Entity::PLACEHOLDER,
            x: 0.0,
            y: -10.0,
            vx: 0.0,
            vy: 5.0, // approaching from below
            radius: 0.5,
        });
        let ctx = default_ctx();

        let out = step(&pos, &vel, &size, &state, &mut wander, &target, &cache, false, true, true, 0.0, &ctx);

        assert!(!out.arrived);
        let combined_mag = (149.0_f32).sqrt();
        let expected_ax = 7.0_f32 * 10.0 / combined_mag;
        let expected_ay = 10.0_f32 * 10.0 / combined_mag;
        assert!(
            (out.ax - expected_ax).abs() < 1e-4,
            "ax={} expected {expected_ax} (capped from seek+avoidance combined)",
            out.ax
        );
        assert!(
            (out.ay - expected_ay).abs() < 1e-4,
            "ay={} expected {expected_ay} (capped from seek+avoidance combined)",
            out.ay
        );
        let final_mag = (out.ax * out.ax + out.ay * out.ay).sqrt();
        assert!(
            (final_mag - 10.0).abs() < 1e-4,
            "capped magnitude must equal max_accel=10.0, got {final_mag}"
        );
    }
}
