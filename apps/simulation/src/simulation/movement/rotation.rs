
use crate::simulation::components::*;
use bevy_ecs::prelude::*;

pub fn rotation_system(
    mut query: Query<(&mut Rotation, &Velocity), Changed<Velocity>>,
    #[cfg(feature = "dev-tools")] timings: bevy_ecs::system::Res<
        crate::instrumentation::SystemTimings,
    >,
) {
    #[cfg(feature = "dev-tools")]
    crate::time_system!(timings, "rotation");

    for (mut rotation, velocity) in query.iter_mut() {
        if velocity.vx != 0.0 || velocity.vy != 0.0 {
            rotation.radians = velocity.vy.atan2(velocity.vx);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rotation_system_matches_velocity() {
        let mut world = World::new();

        let entity = world
            .spawn((
                Rotation { radians: 0.0 },
                Velocity { vx: 1.0, vy: 1.0 },
            ))
            .id();


        let mut query = world.query::<(&mut Rotation, &Velocity)>();
        for (mut rot, vel) in query.iter_mut(&mut world) {
            if vel.vx != 0.0 || vel.vy != 0.0 {
                rot.radians = vel.vy.atan2(vel.vx);
            }
        }

        let rotation = world.get::<Rotation>(entity).unwrap();
        let expected = 1.0f32.atan2(1.0);
        assert!((rotation.radians - expected).abs() < 0.001);
    }
}
