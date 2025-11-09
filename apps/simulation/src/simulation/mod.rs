// Domain modules
pub mod core;
pub mod creatures;
pub mod dna;
pub mod movement;
pub mod perception;

// Legacy modules (to be refactored)
pub mod components; // Will be split across domains
pub mod resources; // Internal use

// Re-export commonly used types from core
pub use core::components::{Acceleration, BodySize, BoundaryConfig, DeltaTime, PhysicsTick, Position, Velocity};
pub use core::{Simulation, SimulationBuilder};

#[cfg(test)]
mod tests;
