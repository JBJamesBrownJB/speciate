
use serde::{Deserialize, Serialize};

pub mod loader;

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
        x: f32,
        y: f32,
        #[serde(default)]
        creature_type: CreatureType,
        #[serde(default)]
        target_x: Option<f32>,
        #[serde(default)]
        target_y: Option<f32>,
    },

    Grid {
        start_x: f32,
        start_y: f32,
        spacing: f32,
        rows: u32,
        cols: u32,
        #[serde(default)]
        creature_type: CreatureType,
        #[serde(default)]
        grid_offset_y: Option<f32>,
    },

    Circle {
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
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CreatureType {
    Catatonic,

    Seeker,

    Wanderer,

    Cycling,
}

impl Default for CreatureType {
    fn default() -> Self {
        Self::Wanderer
    }
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
}
