//! Trial system for regression testing simulation scenarios
//!
//! Trials are predefined starting conditions (spawn patterns, world config)
//! stored in TOML files. They enable:
//! - Reproducible regression tests across refactors
//! - Debugging specific scenarios (crowd navigation, boundary behavior)
//! - Benchmarking performance with controlled conditions

use serde::{Deserialize, Serialize};

pub mod loader;

/// Trial configuration loaded from TOML template
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrialConfig {
    /// Display name (e.g., "Crowd Navigation Test")
    pub name: String,

    /// Description of test scenario
    #[serde(default)]
    pub description: String,

    /// Spawn patterns to apply when loading trial
    pub spawns: Vec<SpawnPattern>,

    /// Optional world configuration overrides
    #[serde(default)]
    pub world: Option<WorldConfig>,
}

/// Spawn pattern defining how creatures are placed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SpawnPattern {
    /// Single creature at specific position
    Single {
        x: f32,
        y: f32,
        #[serde(default)]
        creature_type: CreatureType,
        /// Optional target position for seeker creatures (defaults to origin)
        #[serde(default)]
        target_x: Option<f32>,
        #[serde(default)]
        target_y: Option<f32>,
    },

    /// Grid of creatures with spacing
    Grid {
        /// Top-left corner X coordinate
        start_x: f32,
        /// Top-left corner Y coordinate
        start_y: f32,
        /// Spacing between creatures (meters)
        spacing: f32,
        /// Number of rows
        rows: u32,
        /// Number of columns
        cols: u32,
        #[serde(default)]
        creature_type: CreatureType,
    },

    /// Circle formation
    Circle {
        /// Center X coordinate
        center_x: f32,
        /// Center Y coordinate
        center_y: f32,
        /// Radius (meters)
        radius: f32,
        /// Number of creatures
        count: u32,
        #[serde(default)]
        creature_type: CreatureType,
        /// Optional shared target position for all seeker creatures (defaults to origin)
        #[serde(default)]
        target_x: Option<f32>,
        #[serde(default)]
        target_y: Option<f32>,
    },
}

/// Type of creature to spawn (affects components added)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CreatureType {
    /// Stationary creature (Catatonic marker)
    Catatonic,

    /// Mobile creature with seeking behavior
    Seeker,

    /// Default wanderer (all capabilities enabled)
    Wanderer,
}

impl Default for CreatureType {
    fn default() -> Self {
        Self::Wanderer
    }
}

/// Optional world configuration overrides
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorldConfig {
    /// Override delta time (default: 0.05 for 20 Hz)
    #[serde(default)]
    pub delta_time: Option<f32>,

    /// Override boundary limits (default: ±1,000,000m)
    #[serde(default)]
    pub boundary: Option<BoundaryOverride>,
}

/// Boundary configuration override
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
            } => {
                assert_eq!(*start_x, 0.0);
                assert_eq!(*start_y, 0.0);
                assert_eq!(*spacing, 5.0);
                assert_eq!(*rows, 10);
                assert_eq!(*cols, 10);
                assert_eq!(*creature_type, CreatureType::Wanderer);
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
}
