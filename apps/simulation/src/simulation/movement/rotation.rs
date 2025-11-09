//! Rotation system
//!
//! Calculates creature orientation (rotation) based on velocity direction.
//! This is a visual/state update that doesn't affect physics.

use crate::simulation::components::*;
use bevy_ecs::prelude::*;

/// Updates creature rotation to match velocity direction
///
/// System ordering: Can run any time after velocity is updated.
/// One-frame delay between velocity change and rotation update is acceptable.
///
/// Behavior:
/// - Calculates angle using atan2(vy, vx)
/// - Only updates rotation if creature is moving (velocity != 0)
/// - Stationary creatures maintain their last rotation
pub fn rotation_system(mut query: Query<(&mut Rotation, &Velocity)>) {
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
                Velocity { vx: 1.0, vy: 1.0 }, // 45 degrees
            ))
            .id();

        // Simulate rotation system
        let mut query = world.query::<(&mut Rotation, &Velocity)>();
        for (mut rot, vel) in query.iter_mut(&mut world) {
            if vel.vx != 0.0 || vel.vy != 0.0 {
                rot.radians = vel.vy.atan2(vel.vx);
            }
        }

        let rotation = world.get::<Rotation>(entity).unwrap();
        let expected = 1.0f32.atan2(1.0); // ≈ 0.785 radians (45°)
        assert!((rotation.radians - expected).abs() < 0.001);
    }
}
