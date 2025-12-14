//! State & Transitions constants
//!
//! Energy thresholds and timing parameters.
//!
//! BIOLOGICAL BASIS:
//! - 30% reserves: Hormonal hunger signals trigger foraging priority
//! - 10% reserves: Starvation mode - impaired decisions, reduced activity

/// [ACTIVE] Starting energy (arbitrary scale, ratios matter more than absolute value).
pub const DEFAULT_ENERGY: f32 = 100.0;

/// [ACTIVE] Hunger threshold - below this, prioritize foraging.
/// VALIDATED: 30% fat reserves trigger hormonal hunger signals in most animals.
pub const LOW_ENERGY_THRESHOLD: f32 = 30.0;

/// [ACTIVE] Exhaustion threshold - critical energy state.
/// VALIDATED: 10% reserves = starvation mode, impaired cognition.
pub const EXHAUSTED_THRESHOLD: f32 = 10.0;

/// [ACTIVE] Age increase per tick.
/// At 20 Hz: 0.001 × 20 = 0.02/sec, 1.2/min, 72/hour.
pub const AGE_INCREMENT_PER_TICK: f32 = 0.001;

/// [ACTIVE] Energy drain per tick while wandering.
/// At 20 Hz: 0.2 energy/sec, ~6 minutes from 100 to 30 (hunger threshold).
pub const ENERGY_COST_WANDERING: f32 = 0.01;

/// [ACTIVE] Tick interval for timing calculations.
pub const TICK_INTERVAL_SECONDS: f64 = 0.05;
