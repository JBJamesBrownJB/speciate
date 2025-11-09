//! Dev command definitions
//!
//! Defines the JSON schema for development commands sent via NATS.

use serde::{Deserialize, Serialize};

/// Development commands for controlling the simulation
///
/// These commands are sent as JSON messages from the admin UI to the
/// simulation via NATS on the `dev.sim.*` subject hierarchy.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum DevCommand {
    /// Spawn a single creature at the specified position
    ///
    /// Example JSON:
    /// ```json
    /// {
    ///   "type": "Spawn",
    ///   "x": 100.0,
    ///   "y": 50.0,
    ///   "behavior": "seeking",
    ///   "target_x": 200.0,
    ///   "target_y": 100.0,
    ///   "energy": 100.0,
    ///   "max_speed": 20.0
    /// }
    /// ```
    Spawn {
        /// X position (meters)
        x: f32,
        /// Y position (meters)
        y: f32,
        /// Behavior type: "seeking", "wandering", "catatonic"
        behavior: String,
        /// Target X position (optional, for seeking behavior)
        #[serde(skip_serializing_if = "Option::is_none")]
        target_x: Option<f32>,
        /// Target Y position (optional, for seeking behavior)
        #[serde(skip_serializing_if = "Option::is_none")]
        target_y: Option<f32>,
        /// Initial energy (optional, default: 100.0)
        #[serde(skip_serializing_if = "Option::is_none")]
        energy: Option<f32>,
        /// Maximum speed (optional, default: 20.0)
        #[serde(skip_serializing_if = "Option::is_none")]
        max_speed: Option<f32>,
    },

    /// Remove all creatures from the simulation
    ///
    /// Example JSON:
    /// ```json
    /// {
    ///   "type": "Clear"
    /// }
    /// ```
    Clear,

    /// Adjust simulation speed (tick rate multiplier)
    ///
    /// Example JSON:
    /// ```json
    /// {
    ///   "type": "Speed",
    ///   "multiplier": 2.0
    /// }
    /// ```
    Speed {
        /// Speed multiplier (0.25 = slow-mo, 1.0 = normal, 5.0 = fast)
        /// Clamped to [0.1, 10.0]
        multiplier: f32,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_command_deserialize() {
        let json = r#"{
            "type": "Spawn",
            "x": 100.0,
            "y": 50.0,
            "behavior": "seeking",
            "target_x": 200.0,
            "target_y": 100.0
        }"#;

        let cmd: DevCommand = serde_json::from_str(json).unwrap();
        match cmd {
            DevCommand::Spawn {
                x,
                y,
                behavior,
                target_x,
                target_y,
                ..
            } => {
                assert_eq!(x, 100.0);
                assert_eq!(y, 50.0);
                assert_eq!(behavior, "seeking");
                assert_eq!(target_x, Some(200.0));
                assert_eq!(target_y, Some(100.0));
            }
            _ => panic!("Expected Spawn command"),
        }
    }

    #[test]
    fn test_clear_command_deserialize() {
        let json = r#"{"type": "Clear"}"#;
        let cmd: DevCommand = serde_json::from_str(json).unwrap();
        matches!(cmd, DevCommand::Clear);
    }

    #[test]
    fn test_speed_command_deserialize() {
        let json = r#"{"type": "Speed", "multiplier": 2.0}"#;
        let cmd: DevCommand = serde_json::from_str(json).unwrap();
        match cmd {
            DevCommand::Speed { multiplier } => {
                assert_eq!(multiplier, 2.0);
            }
            _ => panic!("Expected Speed command"),
        }
    }
}
