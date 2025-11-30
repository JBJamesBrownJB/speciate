// Universal physics constants

use std::f32::consts::PI;

pub const DEFAULT_BODY_LENGTH: f32 = 1.0; // Default creature body length in meters
pub const DEFAULT_MASS: f32 = 65.0; // Default mass for 1m creature (kg)
pub const MAX_SPEED: f32 = 50.0; // Maximum creature speed in m/s
pub const MAX_ACCELERATION: f32 = 5.0; // Maximum acceleration in m/s²
pub const MAX_TURN_RATE: f32 = 45.0; // Maximum turn rate in degrees/second
pub const MAX_TURN_RATE_RAD: f32 = MAX_TURN_RATE * PI / 180.0; // Turn rate in radians/second
pub const VELOCITY_DAMPING: f32 = 0.90; // Per-frame velocity damping (mimics air resistance, achieves ~90% reduction in 46 ticks/~2s at 22.2Hz)
pub const STOPPED_THRESHOLD: f32 = 0.01; // Speed below which creatures snap to zero (m/s)
pub const DT: f32 = 0.05; // Simulation time step in seconds (20 Hz)
pub const SLOW_ZONE_MULTIPLIER: f32 = 30.0; // Slow zone size as multiple of personal_space
