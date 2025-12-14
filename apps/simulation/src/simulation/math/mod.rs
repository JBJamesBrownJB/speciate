pub mod integration;
pub mod physics;
#[cfg(test)]
mod scenario_tests;
pub mod steering;
pub mod unit_interval;
pub mod vector_ops;

pub use integration::{
    integrate_motion, integrate_motion_no_turn_limit, IntegrationParams, IntegrationResult,
};
pub use physics::{
    calculate_avoidance, clamp_steering_to_max_accel, force_to_acceleration, AvoidanceParams,
    AvoidanceResult,
};
pub use steering::{
    accumulate_steering, steering_to_acceleration, SteeringContext, SteeringResult,
};
pub use unit_interval::UnitInterval;
pub use vector_ops::{
    clamp_force, fast_atan2, fast_inv_sqrt, magnitude, magnitude_sq, normalize, normalize_angle,
    normalize_fast,
};
