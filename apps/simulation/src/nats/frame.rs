//! Simulation frame data structures
//!
//! Defines the data contract for simulation frames published to NATS.
//! Conforms to: SPRINT_DOCS/NATS_CONTRACT.md

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A complete simulation frame ready for NATS publishing
/// Conforms to the NATS contract: SPRINT_DOCS/NATS_CONTRACT.md
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationFrame {
    /// Monotonically increasing tick counter
    pub tick: u64,

    /// ISO 8601 timestamp (UTC)
    pub timestamp: DateTime<Utc>,

    /// All active crits in the simulation
    pub crits: Vec<CritTransform>,
}

/// Crit position, velocity, and rotation snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CritTransform {
    /// Unique, stable crit identifier
    pub id: u32,

    /// Position X in world coordinates
    pub x: f32,

    /// Position Y in world coordinates
    pub y: f32,

    /// Velocity X in units per second
    pub vx: f32,

    /// Velocity Y in units per second
    pub vy: f32,

    /// Rotation in radians (0 to 2π)
    /// 0 = facing right (+X), π/2 = facing up (+Y)
    pub rotation: f32,
}

impl SimulationFrame {
    /// Create a new simulation frame
    pub fn new(tick: u64, crits: Vec<CritTransform>) -> Self {
        Self {
            tick,
            timestamp: Utc::now(),
            crits,
        }
    }

    /// Serialize to JSON bytes
    pub fn to_json_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    /// Deserialize from MessagePack bytes
    pub fn from_msgpack_bytes(bytes: &[u8]) -> Result<Self, rmp_serde::decode::Error> {
        rmp_serde::from_slice(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_serialization() {
        let frame = SimulationFrame {
            tick: 12450,
            timestamp: Utc::now(),
            crits: vec![
                CritTransform {
                    id: 1,
                    x: 45.23,
                    y: 78.91,
                    vx: 2.15,
                    vy: -0.87,
                    rotation: 1.57,
                },
                CritTransform {
                    id: 2,
                    x: 120.50,
                    y: 34.12,
                    vx: 0.0,
                    vy: 1.42,
                    rotation: 0.0,
                },
            ],
        };

        // Test serialization
        let json_bytes = frame.to_json_bytes().expect("Failed to serialize");
        let json_str = String::from_utf8(json_bytes).expect("Invalid UTF-8");

        // Verify structure
        assert!(json_str.contains("\"tick\":12450"));
        assert!(json_str.contains("\"id\":1"));
        assert!(json_str.contains("\"x\":45.23"));

        // Verify timestamp is ISO 8601 format (string, not number)
        assert!(json_str.contains("\"timestamp\":\""));
        assert!(json_str.contains("T")); // ISO 8601 has T separator
        assert!(json_str.contains("Z")); // UTC indicator

        // Test deserialization
        let deserialized: SimulationFrame =
            serde_json::from_str(&json_str).expect("Failed to deserialize");

        assert_eq!(deserialized.tick, 12450);
        assert_eq!(deserialized.crits.len(), 2);
        assert_eq!(deserialized.crits[0].id, 1);
    }

    #[test]
    fn test_crit_transform_fields() {
        let crit = CritTransform {
            id: 42,
            x: 100.0,
            y: 200.0,
            vx: 5.0,
            vy: -3.0,
            rotation: std::f32::consts::PI,
        };

        assert_eq!(crit.id, 42);
        assert_eq!(crit.x, 100.0);
        assert_eq!(crit.y, 200.0);
        assert_eq!(crit.vx, 5.0);
        assert_eq!(crit.vy, -3.0);
        assert_eq!(crit.rotation, std::f32::consts::PI);
    }

    #[test]
    fn test_msgpack_serialization_uses_struct_map() {
        let frame = SimulationFrame {
            tick: 12345,
            timestamp: Utc::now(),
            crits: vec![CritTransform {
                id: 1,
                x: 10.0,
                y: 20.0,
                vx: 1.0,
                vy: 2.0,
                rotation: 0.5,
            }],
        };

        // Serialize using struct map (field names, not arrays)
        let mut buffer = Vec::new();
        let mut serializer = rmp_serde::Serializer::new(&mut buffer).with_struct_map();
        serde::Serialize::serialize(&frame, &mut serializer).expect("Failed to serialize");

        // Decode as generic Value to inspect structure
        let decoded: serde_json::Value =
            rmp_serde::from_slice(&buffer).expect("Failed to deserialize");

        // CRITICAL: Verify it's an object with field names, not an array
        assert!(
            decoded.is_object(),
            "MessagePack should serialize as object with field names, not array"
        );

        // Verify field names are preserved
        assert_eq!(decoded["tick"], 12345);
        assert!(decoded["timestamp"].is_string());
        assert!(decoded["crits"].is_array());

        // Verify crit has field names too
        let crit = &decoded["crits"][0];
        assert!(crit.is_object(), "Crit should be object, not array");
        assert_eq!(crit["id"], 1);
        assert_eq!(crit["x"], 10.0);
    }
}
