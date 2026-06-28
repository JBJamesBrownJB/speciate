use crate::config::MovementConfig;
use crate::simulation::core::components::{
    Acceleration, BodySize, DeltaTime, FreqConfig, PhysicsTick, Position, Rotation, Velocity,
};
use crate::simulation::core::{FrequencyThrottle, WorldBounds};
use crate::simulation::creatures::behaviors::transitions::step::{step as behavior_step, BehaviorCtx};
use crate::simulation::creatures::components::{
    BehaviorMode, Brain, CanAvoidObstacles, CanSeek, CanWander, CreatureState, HomePosition,
    Target, WanderState,
};
use crate::simulation::creatures::constants::{
    DRAG_COEFFICIENT, SEEK_FORCE_MULT, STOPPED_THRESHOLD, TICK_INTERVAL_SECONDS, WANDER_FORCE_MULT,
};
use crate::simulation::movement::noise::NoiseTable;
use crate::simulation::movement::step::{step as integrate_step, IntegrateCtx};
use crate::simulation::creatures::steering::step::{step as steering_step, SteeringCtx};
use crate::simulation::perception::NeighborCache;
use bevy_ecs::prelude::*;

pub fn act_system(
    mut query: Query<(
        Entity,
        &mut CreatureState,
        &mut Brain,
        &BodySize,
        &mut Position,
        &mut Velocity,
        &mut Acceleration,
        &mut Rotation,
        &mut WanderState,
        &HomePosition,
        &Target,
        &NeighborCache,
        Has<CanWander>,
        Has<CanSeek>,
        Has<CanAvoidObstacles>,
    )>,
    physics_tick: Res<PhysicsTick>,
    freq: Res<FreqConfig>,
    delta_time: Res<DeltaTime>,
    world_bounds: Res<WorldBounds>,
    movement_config: Res<MovementConfig>,
    noise_table: Res<NoiseTable>,
    #[cfg(feature = "dev-tools")] timings: Res<crate::instrumentation::SystemTimings>,
) {
    #[cfg(feature = "dev-tools")]
    crate::time_system!(timings, "act");

    let tick = physics_tick.get();

    let behavior_ctx = BehaviorCtx {
        current_time: tick as f64 * TICK_INTERVAL_SECONDS,
    };
    let behavior_throttle = FrequencyThrottle::new(freq.behavior_divisor, tick);

    let steering_ctx = SteeringCtx {
        wander_force_mult: WANDER_FORCE_MULT.get(),
        seek_force_mult: SEEK_FORCE_MULT.get(),
    };

    let dt = delta_time.0;
    let integrate_ctx = IntegrateCtx {
        dt,
        tick,
        drag_factor: (-DRAG_COEFFICIENT * dt).exp(),
        noise_base: movement_config.locomotion_noise_base,
        noise_time_scale: movement_config.noise_time_scale,
        min_x: world_bounds.min_x,
        max_x: world_bounds.max_x,
        min_y: world_bounds.min_y,
        max_y: world_bounds.max_y,
        noise_table: &*noise_table,
        stopped_threshold_sq: STOPPED_THRESHOLD * STOPPED_THRESHOLD,
    };

    query.par_iter_mut().for_each(
        |(
            entity,
            mut creature_state,
            mut brain,
            size,
            mut position,
            mut velocity,
            mut acceleration,
            mut rotation,
            mut wander_state,
            _home,
            target,
            neighbor_cache,
            can_wander,
            can_seek,
            can_avoid,
        )| {
            if behavior_throttle.should_process(entity.index()) {
                behavior_step(&mut creature_state, &mut brain, &behavior_ctx);
            }

            if brain.mode.makes_decisions() {
                let wander_angle_change_radians =
                    if creature_state.behavior == BehaviorMode::Wandering && can_wander {
                        use rand::Rng;
                        let mut rng = rand::thread_rng();
                        rng.gen_range(-wander_state.angle_change..wander_state.angle_change)
                            .to_radians()
                    } else {
                        0.0
                    };

                debug_assert!(
                    acceleration.ax == 0.0 && acceleration.ay == 0.0,
                    "steering cap assumes zero Acceleration at entry"
                );

                let steering_output = steering_step(
                    &*position,
                    &*velocity,
                    size,
                    &*creature_state,
                    &mut wander_state,
                    target,
                    neighbor_cache,
                    can_wander,
                    can_seek,
                    can_avoid,
                    wander_angle_change_radians,
                    &steering_ctx,
                );

                acceleration.ax += steering_output.ax;
                acceleration.ay += steering_output.ay;
                if steering_output.arrived {
                    creature_state.behavior = BehaviorMode::Catatonic;
                }
            }

            integrate_step(
                entity.index(),
                size,
                &mut position,
                &mut velocity,
                &mut acceleration,
                &*creature_state,
                &mut rotation,
                &integrate_ctx,
            );
        },
    );
}

#[cfg(test)]
mod tests {
    use crate::simulation::core::SimulationBuilder;

    #[test]
    #[cfg(feature = "fuse-act")]
    fn fused_act_corridor_moves_seeker() {
        bevy_tasks::ComputeTaskPool::get_or_init(bevy_tasks::TaskPool::default);

        let mut sim = SimulationBuilder::new()
            .set_boundaries(500.0, 500.0)
            .with_deterministic_movement()
            .build();

        let _id = sim.spawn_seeker(0.0, 0.0, 200.0, 0.0);

        let initial_positions = {
            use crate::simulation::core::components::Position;
            sim.world_mut()
                .query::<&Position>()
                .iter(sim.world_mut())
                .map(|p| (p.x, p.y))
                .collect::<Vec<_>>()
        };

        for _ in 0..10 {
            sim.update(0.05);
        }

        let final_positions = {
            use crate::simulation::core::components::Position;
            sim.world_mut()
                .query::<&Position>()
                .iter(sim.world_mut())
                .map(|p| (p.x, p.y))
                .collect::<Vec<_>>()
        };

        assert_eq!(initial_positions.len(), 1, "one seeker spawned");
        assert_eq!(final_positions.len(), 1, "seeker still alive after 10 ticks");

        let (x0, y0) = initial_positions[0];
        let (x1, y1) = final_positions[0];
        let dist_sq = (x1 - x0) * (x1 - x0) + (y1 - y0) * (y1 - y0);
        assert!(
            dist_sq > 0.01,
            "fused act corridor must move seeker; start=({x0},{y0}) end=({x1},{y1})"
        );
    }
}
