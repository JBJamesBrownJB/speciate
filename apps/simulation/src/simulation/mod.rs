// Domain modules
pub mod core;
pub mod creatures;
pub mod math;
pub mod movement;
pub mod perception;
pub mod queries;
pub mod spatial;

// Re-export commonly used types from core
pub use core::components::{Acceleration, BodySize, BoundaryConfig, DeltaTime, PhysicsTick, Position, Rotation, Velocity};
pub use core::{Simulation, SimulationBuilder};

#[cfg(test)]
mod tests;
