// Domain modules
pub mod core;
pub mod creatures;
pub mod math;
pub mod movement;
pub mod perception;
pub mod queries;
pub mod spatial;

// Legacy modules (to be refactored)
pub mod components; // Will be split across domains

// Re-export commonly used types from core
pub use core::components::{Acceleration, BodySize, BoundaryConfig, DeltaTime, PhysicsTick, Position, Velocity};
pub use core::{Simulation, SimulationBuilder};

#[cfg(test)]
mod tests;
