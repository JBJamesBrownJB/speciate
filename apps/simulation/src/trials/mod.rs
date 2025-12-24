use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod director;
pub mod loader;

#[cfg(test)]
mod tests;

pub use director::{
    OverlapData, TaggedEntities, TrialDirector, TrialResult, TrialSnapshot, TrialState,
};

/// Production simulation tick rate (Hz)
/// Must match TARGET_SIMULATION_HZ in napi_addon/simulation_engine.rs
///
/// Production uses FIXED delta_time (1/20 = 0.05s), not measured wall-clock.
/// Production SLEEPS after each tick to maintain real-time 20Hz.
/// Tests use the same delta_time but NO sleep - runs as fast as CPU allows.
pub const PRODUCTION_TICK_RATE_HZ: f32 = 20.0;

/// Fixed delta_time used in production (seconds per tick)
/// Both production and tests use this same value (0.05s)
pub const PRODUCTION_DELTA_TIME: f32 = 1.0 / PRODUCTION_TICK_RATE_HZ;

// ============================================================================
// Spec Schema (Sprint 19) - Extends trials with assertions and variants
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpecConfig {
    pub meta: MetaConfig,

    #[serde(default)]
    pub variants: HashMap<String, VariantConfig>,

    #[serde(default)]
    pub assertions: Vec<Assertion>,

    #[serde(default)]
    pub spawns: Vec<SpawnPattern>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MetaConfig {
    pub name: String,

    #[serde(default)]
    pub description: String,

    /// Timeout in WALL-CLOCK seconds. Default: 30 seconds.
    /// The trial runs for this many real seconds, completing as many ticks as possible.
    #[serde(default = "default_timeout_seconds")]
    pub timeout_seconds: f32,

    #[serde(default)]
    pub seed: Option<u64>,
}

fn default_timeout_seconds() -> f32 {
    30.0
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VariantConfig {
    pub min: f32,
    pub max: f32,
    pub steps: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Assertion {
    NoOverlaps,
    MaxOverlaps { count: usize },
    MaxOverlapDepth { depth: f32 },
    MaxTicksWithOverlaps { count: u32 },
    CreatureReachedTarget { tag: String },
    CreatureCount { min: usize, max: usize },
    TicksCompleted { count: u32 },
    MaxAvgTickLatency { microseconds: u64 },
}

// ============================================================================
// Legacy Trial Schema (backward compatible)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrialConfig {
    pub name: String,

    #[serde(default)]
    pub description: String,

    pub spawns: Vec<SpawnPattern>,

    #[serde(default)]
    pub world: Option<WorldConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SpawnPattern {
    Single {
        #[serde(default)]
        tag: Option<String>,
        x: f32,
        y: f32,
        #[serde(default)]
        creature_type: CreatureType,
        #[serde(default)]
        target_x: Option<f32>,
        #[serde(default)]
        target_y: Option<f32>,
        #[serde(default)]
        body_size: Option<f32>,
    },

    Grid {
        #[serde(default)]
        tag: Option<String>,
        start_x: f32,
        start_y: f32,
        spacing: f32,
        rows: u32,
        cols: u32,
        #[serde(default)]
        creature_type: CreatureType,
        #[serde(default)]
        grid_offset_y: Option<f32>,
        #[serde(default)]
        target_x: Option<f32>,
        #[serde(default)]
        target_y: Option<f32>,
        #[serde(default)]
        body_size: Option<f32>,
    },

    Circle {
        #[serde(default)]
        tag: Option<String>,
        center_x: f32,
        center_y: f32,
        radius: f32,
        count: u32,
        #[serde(default)]
        creature_type: CreatureType,
        #[serde(default)]
        target_x: Option<f32>,
        #[serde(default)]
        target_y: Option<f32>,
        #[serde(default)]
        body_size: Option<f32>,
    },
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CreatureType {
    Catatonic,

    Seeker,

    #[default]
    Wanderer,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorldConfig {
    #[serde(default)]
    pub delta_time: Option<f32>,

    #[serde(default)]
    pub boundary: Option<BoundaryOverride>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BoundaryOverride {
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
}
