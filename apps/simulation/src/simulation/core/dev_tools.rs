//! Dev-tools resource registration helper.
//!
//! This module consolidates all dev-tools resource registration into a single function,
//! avoiding scattered #[cfg(feature = "dev-tools")] blocks throughout the codebase.

use bevy_ecs::prelude::World;

use crate::instrumentation::{
    HardwareMetrics, HardwareSnapshotResource, ParallelizationMetrics, SystemTimings,
};
use crate::simulation::perception::{PerceptionDebugSnapshot, PerceptionDebugTarget};

/// Register all dev-tools resources with the world.
///
/// This should be called once during simulation initialization when the dev-tools feature is enabled.
pub fn register_dev_resources(world: &mut World) {
    world.insert_resource(SystemTimings::new());
    world.insert_resource(HardwareMetrics::new());
    world.insert_resource(HardwareSnapshotResource::default());
    world.insert_resource(ParallelizationMetrics::new());
    world.insert_resource(PerceptionDebugTarget::default());
    world.insert_resource(PerceptionDebugSnapshot::default());
}
