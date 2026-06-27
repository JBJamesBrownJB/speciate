use crate::config::MovementConfig;
#[cfg(feature = "dev-tools")]
use crate::instrumentation::SystemTimings;
use crate::simulation::core::components::{
    Acceleration, BodySize, DeltaTime, PhysicsTick, Position, Rotation, Velocity,
};
use crate::simulation::core::WorldBounds;
use crate::simulation::creatures::components::CreatureState;
use crate::simulation::creatures::constants::{DRAG_COEFFICIENT, STOPPED_THRESHOLD};
use crate::simulation::movement::noise::NoiseTable;
use crate::simulation::movement::step::{step, IntegrateCtx};
use bevy_ecs::prelude::*;

pub fn integrate_motion_system(
    mut query: Query<(
        Entity,
        &BodySize,
        &mut Position,
        &mut Velocity,
        &mut Acceleration,
        &CreatureState,
        &mut Rotation,
    )>,
    delta_time: Res<DeltaTime>,
    physics_tick: Res<PhysicsTick>,
    world_bounds: Res<WorldBounds>,
    movement_config: Res<MovementConfig>,
    noise_table: Res<NoiseTable>,
    #[cfg(feature = "dev-tools")] timings: Res<SystemTimings>,
) {
    #[cfg(feature = "dev-tools")]
    crate::time_system!(timings, "movement");

    let dt = delta_time.0;
    let ctx = IntegrateCtx {
        dt,
        tick: physics_tick.get(),
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
        |(entity, size, mut position, mut velocity, mut acceleration, creature_state, mut rotation)| {
            step(
                entity.index(),
                size,
                &mut position,
                &mut velocity,
                &mut acceleration,
                creature_state,
                &mut rotation,
                &ctx,
            );
        },
    );
}

pub fn update_body_size_cache(mut query: Query<&mut BodySize, Changed<BodySize>>) {
    for mut size in query.iter_mut() {
        size.inv_sqrt_length = 1.0 / size.length.sqrt();
    }
}
