use bevy_ecs::prelude::*;
use crate::simulation::core::components::{Acceleration, BodySize, Position, Velocity};
use crate::simulation::creatures::components::{
    CanAvoidObstacles, CanSeek, CanWander, CreatureState, HomePosition, Target, WanderState,
};
use crate::simulation::perception::{AvoidanceBehavior, Perception};

// Wander behavior query. Used by: territory_wandering_system
// MUTATES: Acceleration (force), WanderState (angle)
// READS: Entity, Velocity, Position, HomePosition, CreatureState, BodySize
pub type WanderQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static mut Acceleration,
        &'static mut WanderState,
        &'static Velocity,
        &'static Position,
        &'static HomePosition,
        &'static CreatureState,
        &'static BodySize,
    ),
    With<CanWander>,
>;

// Seek behavior query. Used by: seek_system
// MUTATES: Acceleration (force), CreatureState (behavior transition)
// READS: Position, Velocity, BodySize, Target
pub type SeekQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static Position,
        &'static mut Acceleration,
        &'static Velocity,
        &'static BodySize,
        &'static Target,
        &'static mut CreatureState,
    ),
    With<CanSeek>,
>;

// Avoidance behavior query. Used by: avoidance_system
// MUTATES: Acceleration (force)
// READS: Entity, Position, Velocity, BodySize, Perception, AvoidanceBehavior, CreatureState
pub type AvoidanceQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Position,
        &'static Velocity,
        &'static BodySize,
        &'static mut Acceleration,
        &'static Perception,
        &'static AvoidanceBehavior,
        &'static CreatureState,
    ),
    With<CanAvoidObstacles>,
>;
