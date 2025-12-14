use super::arrival::{calculate_arrival, ArrivalParams};
use crate::simulation::creatures::components::BehaviorMode;
use crate::simulation::creatures::constants::{MAX_SPEED, SEEK_FORCE_MULT};
use crate::simulation::queries::SeekQuery;
use rayon::prelude::*;
#[cfg(feature = "dev-tools")]
use crate::instrumentation::SystemTimings;
#[cfg(feature = "dev-tools")]
use bevy_ecs::system::Res;

pub fn seek_system(
    mut query: SeekQuery,
    #[cfg(feature = "dev-tools")] timings: Res<SystemTimings>,
) {
    #[cfg(feature = "dev-tools")]
    crate::time_system!(timings, "seek");

    let mut entities: Vec<_> = query.iter_mut().collect();

    entities.par_iter_mut().for_each(
        |(position, acceleration, velocity, size, target, creature_state)| {
            if creature_state.behavior != BehaviorMode::Seeking {
                return;
            }

            let to_target_x = target.x - position.x;
            let to_target_y = target.y - position.y;

            // Use the arrival behavior with correct F=ma physics
            let params = ArrivalParams {
                velocity: (velocity.vx, velocity.vy),
                to_target: (to_target_x, to_target_y),
                self_radius: size.radius(),
                target_radius: target.radius.get(),
                max_speed: MAX_SPEED,
                max_force: size.max_force() * SEEK_FORCE_MULT.get(),
                mass: size.mass(),
            };

            let result = calculate_arrival(&params);

            if result.arrived {
                creature_state.behavior = BehaviorMode::Catatonic;
                return;
            }

            acceleration.ax += result.acceleration.0;
            acceleration.ay += result.acceleration.1;
        },
    );
}
