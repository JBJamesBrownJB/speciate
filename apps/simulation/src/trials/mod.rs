use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod director;
pub mod loader;

pub use director::{OverlapData, TrialDirector, TrialResult, TrialSnapshot, TrialState, TaggedEntities};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trial_config_deserialize() {
        let toml = r#"
            name = "Test Trial"
            description = "Test description"

            [[spawns]]
            type = "single"
            x = 10.0
            y = 20.0
            creature_type = "catatonic"
        "#;

        let config: TrialConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.name, "Test Trial");
        assert_eq!(config.description, "Test description");
        assert_eq!(config.spawns.len(), 1);

        match &config.spawns[0] {
            SpawnPattern::Single {
                x,
                y,
                creature_type,
                target_x,
                target_y,
                ..
            } => {
                assert_eq!(*x, 10.0);
                assert_eq!(*y, 20.0);
                assert_eq!(*creature_type, CreatureType::Catatonic);
                assert_eq!(*target_x, None);
                assert_eq!(*target_y, None);
            }
            _ => panic!("Expected Single spawn pattern"),
        }
    }

    #[test]
    fn test_grid_pattern_deserialize() {
        let toml = r#"
            name = "Grid Test"

            [[spawns]]
            type = "grid"
            start_x = 0.0
            start_y = 0.0
            spacing = 5.0
            rows = 10
            cols = 10
            creature_type = "wanderer"
        "#;

        let config: TrialConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.spawns.len(), 1);

        match &config.spawns[0] {
            SpawnPattern::Grid {
                start_x,
                start_y,
                spacing,
                rows,
                cols,
                creature_type,
                grid_offset_y,
                ..
            } => {
                assert_eq!(*start_x, 0.0);
                assert_eq!(*start_y, 0.0);
                assert_eq!(*spacing, 5.0);
                assert_eq!(*rows, 10);
                assert_eq!(*cols, 10);
                assert_eq!(*creature_type, CreatureType::Wanderer);
                assert_eq!(*grid_offset_y, None);
            }
            _ => panic!("Expected Grid spawn pattern"),
        }
    }

    #[test]
    fn test_circle_pattern_deserialize() {
        let toml = r#"
            name = "Circle Test"

            [[spawns]]
            type = "circle"
            center_x = 100.0
            center_y = 200.0
            radius = 50.0
            count = 20
            creature_type = "seeker"
        "#;

        let config: TrialConfig = toml::from_str(toml).unwrap();

        match &config.spawns[0] {
            SpawnPattern::Circle {
                center_x,
                center_y,
                radius,
                count,
                creature_type,
                target_x,
                target_y,
                ..
            } => {
                assert_eq!(*center_x, 100.0);
                assert_eq!(*center_y, 200.0);
                assert_eq!(*radius, 50.0);
                assert_eq!(*count, 20);
                assert_eq!(*creature_type, CreatureType::Seeker);
                assert_eq!(*target_x, None);
                assert_eq!(*target_y, None);
            }
            _ => panic!("Expected Circle spawn pattern"),
        }
    }

    #[test]
    fn test_multiple_spawn_patterns() {
        let toml = r#"
            name = "Multi Pattern"

            [[spawns]]
            type = "single"
            x = 0.0
            y = 0.0

            [[spawns]]
            type = "grid"
            start_x = 10.0
            start_y = 10.0
            spacing = 2.0
            rows = 5
            cols = 5
        "#;

        let config: TrialConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.spawns.len(), 2);
    }

    #[test]
    fn test_world_config_override() {
        let toml = r#"
            name = "World Override Test"

            [[spawns]]
            type = "single"
            x = 0.0
            y = 0.0

            [world]
            delta_time = 0.016

            [world.boundary]
            min_x = -1000.0
            max_x = 1000.0
            min_y = -1000.0
            max_y = 1000.0
        "#;

        let config: TrialConfig = toml::from_str(toml).unwrap();
        assert!(config.world.is_some());

        let world = config.world.as_ref().unwrap();
        assert_eq!(world.delta_time, Some(0.016));
        assert!(world.boundary.is_some());

        let boundary = world.boundary.as_ref().unwrap();
        assert_eq!(boundary.min_x, -1000.0);
        assert_eq!(boundary.max_x, 1000.0);
    }

    #[test]
    fn test_creature_type_default() {
        let toml = r#"
            name = "Default Type Test"

            [[spawns]]
            type = "single"
            x = 0.0
            y = 0.0
        "#;

        let config: TrialConfig = toml::from_str(toml).unwrap();

        match &config.spawns[0] {
            SpawnPattern::Single { creature_type, .. } => {
                assert_eq!(*creature_type, CreatureType::Wanderer);
            }
            _ => panic!("Expected Single spawn pattern"),
        }
    }

    #[test]
    fn test_description_default() {
        let toml = r#"
            name = "No Description"

            [[spawns]]
            type = "single"
            x = 0.0
            y = 0.0
        "#;

        let config: TrialConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.description, "");
    }

    #[test]
    fn test_world_config_optional() {
        let toml = r#"
            name = "No World Config"

            [[spawns]]
            type = "single"
            x = 0.0
            y = 0.0
        "#;

        let config: TrialConfig = toml::from_str(toml).unwrap();
        assert!(config.world.is_none());
    }

    #[test]
    fn test_partial_world_config() {
        let toml = r#"
            name = "Partial World Config"

            [[spawns]]
            type = "single"
            x = 0.0
            y = 0.0

            [world]
            delta_time = 0.033
        "#;

        let config: TrialConfig = toml::from_str(toml).unwrap();
        let world = config.world.as_ref().unwrap();
        assert_eq!(world.delta_time, Some(0.033));
        assert!(world.boundary.is_none());
    }

    #[test]
    fn test_single_seeker_with_target() {
        let toml = r#"
            name = "Seeker with Target"

            [[spawns]]
            type = "single"
            x = 20.0
            y = 0.0
            creature_type = "seeker"
            target_x = -10.0
            target_y = 0.0
        "#;

        let config: TrialConfig = toml::from_str(toml).unwrap();

        match &config.spawns[0] {
            SpawnPattern::Single {
                x,
                y,
                creature_type,
                target_x,
                target_y,
                ..
            } => {
                assert_eq!(*x, 20.0);
                assert_eq!(*y, 0.0);
                assert_eq!(*creature_type, CreatureType::Seeker);
                assert_eq!(*target_x, Some(-10.0));
                assert_eq!(*target_y, Some(0.0));
            }
            _ => panic!("Expected Single spawn pattern"),
        }
    }

    #[test]
    fn test_circle_seekers_with_shared_target() {
        let toml = r#"
            name = "Circle Seekers Converging"

            [[spawns]]
            type = "circle"
            center_x = 0.0
            center_y = 0.0
            radius = 50.0
            count = 4
            creature_type = "seeker"
            target_x = 0.0
            target_y = 0.0
        "#;

        let config: TrialConfig = toml::from_str(toml).unwrap();

        match &config.spawns[0] {
            SpawnPattern::Circle {
                center_x,
                center_y,
                radius,
                count,
                creature_type,
                target_x,
                target_y,
                ..
            } => {
                assert_eq!(*center_x, 0.0);
                assert_eq!(*center_y, 0.0);
                assert_eq!(*radius, 50.0);
                assert_eq!(*count, 4);
                assert_eq!(*creature_type, CreatureType::Seeker);
                assert_eq!(*target_x, Some(0.0));
                assert_eq!(*target_y, Some(0.0));
            }
            _ => panic!("Expected Circle spawn pattern"),
        }
    }

    #[test]
    fn test_target_fields_default_to_none() {
        let toml = r#"
            name = "Backward Compatibility"

            [[spawns]]
            type = "single"
            x = 10.0
            y = 20.0
            creature_type = "seeker"
        "#;

        let config: TrialConfig = toml::from_str(toml).unwrap();

        match &config.spawns[0] {
            SpawnPattern::Single { target_x, target_y, .. } => {
                assert_eq!(*target_x, None);
                assert_eq!(*target_y, None);
            }
            _ => panic!("Expected Single spawn pattern"),
        }
    }

    #[test]
    fn test_grid_offset_y_deserialize() {
        let toml = r#"
            name = "Grid with Offset"

            [[spawns]]
            type = "grid"
            start_x = 0.0
            start_y = 0.0
            spacing = 1.0
            rows = 5
            cols = 5
            creature_type = "catatonic"
            grid_offset_y = 0.5
        "#;

        let config: TrialConfig = toml::from_str(toml).unwrap();

        match &config.spawns[0] {
            SpawnPattern::Grid { grid_offset_y, .. } => {
                assert_eq!(*grid_offset_y, Some(0.5));
            }
            _ => panic!("Expected Grid spawn pattern"),
        }
    }

    // ========================================================================
    // Spec Schema Tests (Sprint 19)
    // ========================================================================

    #[test]
    fn test_spec_config_minimal() {
        let toml = r#"
            [meta]
            name = "Minimal Spec"
        "#;

        let spec: SpecConfig = toml::from_str(toml).unwrap();
        assert_eq!(spec.meta.name, "Minimal Spec");
        assert_eq!(spec.meta.description, "");
        assert_eq!(spec.meta.timeout_seconds, 30.0); // default (wall-clock seconds)
        assert_eq!(spec.meta.seed, None);
        assert!(spec.variants.is_empty());
        assert!(spec.assertions.is_empty());
        assert!(spec.spawns.is_empty());
    }

    #[test]
    fn test_spec_config_with_meta() {
        let toml = r#"
            [meta]
            name = "Full Meta Spec"
            description = "A spec with all meta fields"
            timeout_seconds = 25
            seed = 12345
        "#;

        let spec: SpecConfig = toml::from_str(toml).unwrap();
        assert_eq!(spec.meta.name, "Full Meta Spec");
        assert_eq!(spec.meta.description, "A spec with all meta fields");
        assert_eq!(spec.meta.timeout_seconds, 25.0); // wall-clock seconds
        assert_eq!(spec.meta.seed, Some(12345));
    }

    #[test]
    fn test_spec_config_with_variants() {
        let toml = r#"
            [meta]
            name = "Parameterized Spec"

            [variants]
            crit_size = { min = 0.5, max = 2.0, steps = 10 }
            speed = { min = 1.0, max = 5.0, steps = 5 }
        "#;

        let spec: SpecConfig = toml::from_str(toml).unwrap();
        assert_eq!(spec.variants.len(), 2);

        let crit_size = spec.variants.get("crit_size").unwrap();
        assert_eq!(crit_size.min, 0.5);
        assert_eq!(crit_size.max, 2.0);
        assert_eq!(crit_size.steps, 10);

        let speed = spec.variants.get("speed").unwrap();
        assert_eq!(speed.min, 1.0);
        assert_eq!(speed.max, 5.0);
        assert_eq!(speed.steps, 5);
    }

    #[test]
    fn test_spec_assertion_no_overlaps() {
        let toml = r#"
            [meta]
            name = "Overlap Test"

            [[assertions]]
            type = "no_overlaps"
        "#;

        let spec: SpecConfig = toml::from_str(toml).unwrap();
        assert_eq!(spec.assertions.len(), 1);
        assert!(matches!(spec.assertions[0], Assertion::NoOverlaps));
    }

    #[test]
    fn test_spec_assertion_creature_reached_target() {
        let toml = r#"
            [meta]
            name = "Target Test"

            [[assertions]]
            type = "creature_reached_target"
            tag = "seeker"
        "#;

        let spec: SpecConfig = toml::from_str(toml).unwrap();
        assert_eq!(spec.assertions.len(), 1);
        match &spec.assertions[0] {
            Assertion::CreatureReachedTarget { tag } => {
                assert_eq!(tag, "seeker");
            }
            _ => panic!("Expected CreatureReachedTarget"),
        }
    }

    #[test]
    fn test_spec_assertion_creature_count() {
        let toml = r#"
            [meta]
            name = "Count Test"

            [[assertions]]
            type = "creature_count"
            min = 10
            max = 100
        "#;

        let spec: SpecConfig = toml::from_str(toml).unwrap();
        match &spec.assertions[0] {
            Assertion::CreatureCount { min, max } => {
                assert_eq!(*min, 10);
                assert_eq!(*max, 100);
            }
            _ => panic!("Expected CreatureCount"),
        }
    }

    #[test]
    fn test_spec_assertion_ticks_completed() {
        let toml = r#"
            [meta]
            name = "Duration Test"

            [[assertions]]
            type = "ticks_completed"
            count = 500
        "#;

        let spec: SpecConfig = toml::from_str(toml).unwrap();
        match &spec.assertions[0] {
            Assertion::TicksCompleted { count } => {
                assert_eq!(*count, 500);
            }
            _ => panic!("Expected TicksCompleted"),
        }
    }

    #[test]
    fn test_spec_spawn_with_tag() {
        let toml = r#"
            [meta]
            name = "Tagged Spawn"

            [[spawns]]
            type = "single"
            tag = "hero"
            x = 10.0
            y = 20.0
            creature_type = "seeker"
        "#;

        let spec: SpecConfig = toml::from_str(toml).unwrap();
        match &spec.spawns[0] {
            SpawnPattern::Single { tag, x, y, creature_type, .. } => {
                assert_eq!(*tag, Some("hero".to_string()));
                assert_eq!(*x, 10.0);
                assert_eq!(*y, 20.0);
                assert_eq!(*creature_type, CreatureType::Seeker);
            }
            _ => panic!("Expected Single spawn"),
        }
    }

    #[test]
    fn test_spec_complete_crowd_navigation() {
        let toml = r#"
            [meta]
            name = "Crowd Navigation"
            description = "Test navigation through dense crowd"
            timeout_seconds = 50
            seed = 12345

            [variants]
            crit_size = { min = 0.5, max = 2.0, steps = 10 }

            [[assertions]]
            type = "ticks_completed"
            count = 500

            [[assertions]]
            type = "creature_reached_target"
            tag = "seeker"

            [[spawns]]
            type = "grid"
            start_x = -10.0
            start_y = -10.0
            spacing = 2.0
            rows = 10
            cols = 10
            creature_type = "catatonic"

            [[spawns]]
            type = "single"
            tag = "seeker"
            x = -12.0
            y = 0.0
            creature_type = "seeker"
            target_x = 30.0
            target_y = 0.0
        "#;

        let spec: SpecConfig = toml::from_str(toml).unwrap();

        // Meta
        assert_eq!(spec.meta.name, "Crowd Navigation");
        assert_eq!(spec.meta.timeout_seconds, 50.0); // wall-clock seconds
        assert_eq!(spec.meta.seed, Some(12345));

        // Variants
        assert_eq!(spec.variants.len(), 1);
        let variant = spec.variants.get("crit_size").unwrap();
        assert_eq!(variant.steps, 10);

        // Assertions
        assert_eq!(spec.assertions.len(), 2);
        assert!(matches!(spec.assertions[0], Assertion::TicksCompleted { count: 500 }));
        assert!(matches!(&spec.assertions[1], Assertion::CreatureReachedTarget { tag } if tag == "seeker"));

        // Spawns
        assert_eq!(spec.spawns.len(), 2);
        assert!(matches!(&spec.spawns[0], SpawnPattern::Grid { .. }));
        match &spec.spawns[1] {
            SpawnPattern::Single { tag, creature_type, target_x, target_y, .. } => {
                assert_eq!(*tag, Some("seeker".to_string()));
                assert_eq!(*creature_type, CreatureType::Seeker);
                assert_eq!(*target_x, Some(30.0));
                assert_eq!(*target_y, Some(0.0));
            }
            _ => panic!("Expected Single spawn"),
        }
    }

    #[test]
    fn test_spec_assertion_max_overlaps() {
        let toml = r#"
            [meta]
            name = "Max Overlaps Test"

            [[assertions]]
            type = "max_overlaps"
            count = 5
        "#;

        let spec: SpecConfig = toml::from_str(toml).unwrap();
        match &spec.assertions[0] {
            Assertion::MaxOverlaps { count } => {
                assert_eq!(*count, 5);
            }
            _ => panic!("Expected MaxOverlaps"),
        }
    }

    #[test]
    fn test_spec_assertion_max_overlap_depth() {
        let toml = r#"
            [meta]
            name = "Max Overlap Depth Test"

            [[assertions]]
            type = "max_overlap_depth"
            depth = 0.9
        "#;

        let spec: SpecConfig = toml::from_str(toml).unwrap();
        match &spec.assertions[0] {
            Assertion::MaxOverlapDepth { depth } => {
                assert!((*depth - 0.9).abs() < 0.001);
            }
            _ => panic!("Expected MaxOverlapDepth"),
        }
    }

    #[test]
    fn test_spec_assertion_max_ticks_with_overlaps() {
        let toml = r#"
            [meta]
            name = "Max Ticks With Overlaps Test"

            [[assertions]]
            type = "max_ticks_with_overlaps"
            count = 200
        "#;

        let spec: SpecConfig = toml::from_str(toml).unwrap();
        match &spec.assertions[0] {
            Assertion::MaxTicksWithOverlaps { count } => {
                assert_eq!(*count, 200);
            }
            _ => panic!("Expected MaxTicksWithOverlaps"),
        }
    }

    #[test]
    fn test_spec_assertion_max_avg_tick_latency() {
        let toml = r#"
            [meta]
            name = "Performance Test"

            [[assertions]]
            type = "max_avg_tick_latency"
            microseconds = 5000
        "#;

        let spec: SpecConfig = toml::from_str(toml).unwrap();
        match &spec.assertions[0] {
            Assertion::MaxAvgTickLatency { microseconds } => {
                assert_eq!(*microseconds, 5000);
            }
            _ => panic!("Expected MaxAvgTickLatency"),
        }
    }

    #[test]
    fn test_body_size_field_parsing() {
        let toml = r#"
            [meta]
            name = "Body Size Test"

            [[spawns]]
            type = "single"
            tag = "giant"
            x = 0.0
            y = 0.0
            creature_type = "wanderer"
            body_size = 5.0

            [[spawns]]
            type = "single"
            tag = "mouse"
            x = 10.0
            y = 0.0
            creature_type = "catatonic"
            body_size = 1.0
        "#;

        let spec: SpecConfig = toml::from_str(toml).unwrap();
        assert_eq!(spec.spawns.len(), 2);

        match &spec.spawns[0] {
            SpawnPattern::Single { tag, body_size, .. } => {
                assert_eq!(*tag, Some("giant".to_string()));
                assert_eq!(*body_size, Some(5.0));
            }
            _ => panic!("Expected Single spawn"),
        }

        match &spec.spawns[1] {
            SpawnPattern::Single { tag, body_size, .. } => {
                assert_eq!(*tag, Some("mouse".to_string()));
                assert_eq!(*body_size, Some(1.0));
            }
            _ => panic!("Expected Single spawn"),
        }
    }

    #[test]
    fn test_body_size_defaults_to_none() {
        let toml = r#"
            [meta]
            name = "No Body Size"

            [[spawns]]
            type = "single"
            x = 0.0
            y = 0.0
        "#;

        let spec: SpecConfig = toml::from_str(toml).unwrap();
        match &spec.spawns[0] {
            SpawnPattern::Single { body_size, .. } => {
                assert_eq!(*body_size, None);
            }
            _ => panic!("Expected Single spawn"),
        }
    }
}
