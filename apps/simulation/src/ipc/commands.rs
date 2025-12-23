use serde::{Deserialize, Serialize};
use std::io;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Command {
    #[cfg(feature = "dev-tools")]
    DevSpawnCreature {
        x: f32,
        y: f32,
        #[serde(default)]
        dna: Option<serde_json::Value>,
    },

    #[cfg(feature = "dev-tools")]
    DevLoadTrial { template: String },

    #[cfg(feature = "dev-tools")]
    DevClearCreatures,
}

impl Command {
    pub fn to_msgpack(&self) -> io::Result<Vec<u8>> {
        let mut buf = Vec::new();
        self.serialize(&mut rmp_serde::Serializer::new(&mut buf).with_struct_map())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok(buf)
    }

    pub fn from_msgpack(bytes: &[u8]) -> io::Result<Self> {
        rmp_serde::from_slice(bytes).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rmp_serde;

    #[test]
    fn test_deserialize_dev_spawn_creature() {
        let json = r#"{"type": "dev_spawn_creature", "x": 100.5, "y": 200.5}"#;
        let value: serde_json::Value = serde_json::from_str(json).unwrap();
        let msgpack = rmp_serde::to_vec(&value).unwrap();

        let command: Command = rmp_serde::from_slice(&msgpack).unwrap();

        match command {
            Command::DevSpawnCreature { x, y, dna } => {
                assert_eq!(x, 100.5);
                assert_eq!(y, 200.5);
                assert!(dna.is_none());
            }
            _ => panic!("Expected DevSpawnCreature"),
        }
    }

    #[test]
    fn test_deserialize_dev_spawn_creature_with_dna() {
        let json = r#"{"type": "dev_spawn_creature", "x": 50.0, "y": 75.0, "dna": {"size": 1.5}}"#;
        let value: serde_json::Value = serde_json::from_str(json).unwrap();
        let msgpack = rmp_serde::to_vec(&value).unwrap();

        let command: Command = rmp_serde::from_slice(&msgpack).unwrap();

        match command {
            Command::DevSpawnCreature { x, y, dna } => {
                assert_eq!(x, 50.0);
                assert_eq!(y, 75.0);
                assert!(dna.is_some());
                let dna_value = dna.unwrap();
                assert_eq!(dna_value.get("size").unwrap().as_f64().unwrap(), 1.5);
            }
            _ => panic!("Expected DevSpawnCreature"),
        }
    }

    #[test]
    fn test_deserialize_dev_load_trial() {
        let json = r#"{"type": "dev_load_trial", "template": "flocking_test"}"#;
        let value: serde_json::Value = serde_json::from_str(json).unwrap();
        let msgpack = rmp_serde::to_vec(&value).unwrap();

        let command: Command = rmp_serde::from_slice(&msgpack).unwrap();

        match command {
            Command::DevLoadTrial { template } => {
                assert_eq!(template, "flocking_test");
            }
            _ => panic!("Expected DevLoadTrial"),
        }
    }

    #[test]
    fn test_deserialize_dev_clear_creatures() {
        let json = r#"{"type": "dev_clear_creatures"}"#;
        let value: serde_json::Value = serde_json::from_str(json).unwrap();
        let msgpack = rmp_serde::to_vec(&value).unwrap();

        let command: Command = rmp_serde::from_slice(&msgpack).unwrap();

        match command {
            Command::DevClearCreatures => {}
            _ => panic!("Expected DevClearCreatures"),
        }
    }

    #[test]
    fn test_command_enum_uses_snake_case_type_field() {
        let json = r#"{"type": "dev_spawn_creature", "x": 0.0, "y": 0.0}"#;
        let value: serde_json::Value = serde_json::from_str(json).unwrap();
        let msgpack = rmp_serde::to_vec(&value).unwrap();

        let result: Result<Command, _> = rmp_serde::from_slice(&msgpack);
        assert!(
            result.is_ok(),
            "Should deserialize with snake_case type field"
        );

        let json_pascal = r#"{"type": "DevSpawnCreature", "x": 0.0, "y": 0.0}"#;
        let value_pascal: serde_json::Value = serde_json::from_str(json_pascal).unwrap();
        let msgpack_pascal = rmp_serde::to_vec(&value_pascal).unwrap();

        let result_pascal: Result<Command, _> = rmp_serde::from_slice(&msgpack_pascal);
        assert!(
            result_pascal.is_err(),
            "Should fail with PascalCase type field"
        );
    }

    #[test]
    fn test_invalid_command_returns_error() {
        let json = r#"{"type": "dev_spawn_creature", "x": 100.5}"#;
        let value: serde_json::Value = serde_json::from_str(json).unwrap();
        let msgpack = rmp_serde::to_vec(&value).unwrap();

        let result: Result<Command, _> = rmp_serde::from_slice(&msgpack);
        assert!(
            result.is_err(),
            "Should fail when missing required field 'y'"
        );

        let json_unknown = r#"{"type": "unknown_command", "data": "test"}"#;
        let value_unknown: serde_json::Value = serde_json::from_str(json_unknown).unwrap();
        let msgpack_unknown = rmp_serde::to_vec(&value_unknown).unwrap();

        let result_unknown: Result<Command, _> = rmp_serde::from_slice(&msgpack_unknown);
        assert!(
            result_unknown.is_err(),
            "Should fail with unknown command type"
        );
    }

    #[test]
    fn test_roundtrip_serialization() {
        let original = Command::DevSpawnCreature {
            x: 123.45,
            y: 678.90,
            dna: Some(serde_json::json!({"test": "value"})),
        };

        let msgpack = rmp_serde::to_vec(&original).unwrap();
        let deserialized: Command = rmp_serde::from_slice(&msgpack).unwrap();

        assert_eq!(original, deserialized);
    }
}
