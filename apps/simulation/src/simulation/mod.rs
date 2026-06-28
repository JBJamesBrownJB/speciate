// Domain modules
#[cfg(feature = "fuse-act")]
pub mod act;
pub mod core;
pub mod creatures;
pub mod math;
pub mod movement;
pub mod perception;
pub mod plants;
pub mod spatial;
pub mod tick_controller;

// Re-export commonly used types from core
pub use core::components::{
    Acceleration, BodySize, BoundaryConfig, DeltaTime, PhysicsTick, Position, Rotation, Velocity,
};
pub use core::{Simulation, SimulationBuilder};
pub use tick_controller::{TickController, TickMetrics};

#[cfg(test)]
mod tests;
