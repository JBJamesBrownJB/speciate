mod simulation_engine;

pub use simulation_engine::{init_logger, SimulationEngine};

// Re-export bridge types for convenience
pub use crate::ipc::bridge::{DoubleBuffer, NapiApp, TelemetrySnapshot};
