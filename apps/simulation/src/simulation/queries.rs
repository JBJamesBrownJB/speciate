use bevy_ecs::prelude::*;
use crate::simulation::components::*;
use crate::simulation::core::components::*;
use crate::simulation::perception::{AvoidanceBehavior, Perception};

pub type WanderQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut Acceleration,
        &'static mut WanderState,
        &'static Velocity,
        &'static Position,
        &'static HomePosition,
        &'static CreatureState,
    ),
    With<CanWander>,
>;

pub type SeekQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static Position,  // Read-only: seek only reads position
        &'static mut Acceleration,
        &'static Velocity,
        &'static BodySize,
        &'static Target,
        &'static mut CreatureState,
    ),
    With<CanSeek>,
>;

pub type AvoidanceQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Position,
        &'static BodySize,
        &'static mut Acceleration,
        &'static Perception,
        &'static AvoidanceBehavior,
        &'static CreatureState,
    ),
    With<CanAvoidObstacles>,
>;
