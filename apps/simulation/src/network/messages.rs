//! WebSocket message types

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EntityState {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SimulationStateMessage {
    pub tick: u64,
    pub entity: EntityState,
    pub server_time: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_state_creation() {
        let state = EntityState {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        };
        assert_eq!(state.x, 1.0);
    }

    #[test]
    fn test_message_serialization() {
        let msg = SimulationStateMessage {
            tick: 42,
            entity: EntityState {
                x: 10.5,
                y: 20.3,
                z: 30.1,
            },
            server_time: 1234567890,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"tick\":42"));
    }
}
