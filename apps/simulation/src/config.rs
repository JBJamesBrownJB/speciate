//! Centralized configuration for the simulation world
//!
//! This module contains all configurable parameters for the simulation,
//! making it easy to tweak world settings without modifying code.

use serde::{Deserialize, Serialize};

/// World configuration parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldConfig {
    /// World boundaries
    pub world: WorldBoundaries,

    /// Initial creature spawning parameters
    pub spawning: SpawningConfig,

    /// Simulation timing parameters
    pub timing: TimingConfig,

    /// Network configuration
    pub network: NetworkConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldBoundaries {
    /// World width (in world coordinates)
    pub width: f32,
    /// World height (in world coordinates)
    pub height: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawningConfig {
    /// Number of creatures to spawn at startup
    pub initial_population: usize,

    /// Minimum creature size
    pub min_size: f32,

    /// Maximum creature size
    pub max_size: f32,

    /// Spawn area - minimum X coordinate (screen space)
    pub spawn_x_min: f32,

    /// Spawn area - maximum X coordinate (screen space)
    pub spawn_x_max: f32,

    /// Spawn area - minimum Y coordinate (screen space)
    pub spawn_y_min: f32,

    /// Spawn area - maximum Y coordinate (screen space)
    pub spawn_y_max: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingConfig {
    /// Target simulation tick rate in Hz (e.g., 60 for 60 ticks per second)
    pub target_tick_rate: u32,

    /// Tick timing window size (number of samples for rolling average)
    pub timing_window_size: usize,

    /// Report timing stats every N ticks
    pub timing_report_interval: u64,

    /// Log creature count every N seconds (real time)
    pub creature_count_log_interval_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// WebSocket server bind address
    pub bind_address: String,

    /// WebSocket server port
    pub port: u16,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            world: WorldBoundaries {
                width: 180.0,
                height: 130.0,
            },
            spawning: SpawningConfig {
                initial_population: 200,
                min_size: 0.5,
                max_size: 2.0,
                spawn_x_min: 40.0,
                spawn_x_max: 140.0,
                spawn_y_min: 30.0,
                spawn_y_max: 100.0,
            },
            timing: TimingConfig {
                target_tick_rate: 20,
                timing_window_size: 100,
                timing_report_interval: 200,
                creature_count_log_interval_secs: 1,
            },
            network: NetworkConfig {
                bind_address: "0.0.0.0".to_string(),
                port: 8080,
            },
        }
    }
}

impl WorldConfig {
    /// Create a new WorldConfig with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the tick duration for the target tick rate
    /// Get the WebSocket bind address with port
    pub fn ws_bind_address(&self) -> String {
        format!("{}:{}", self.network.bind_address, self.network.port)
    }

    /// Get the WebSocket URL for logging
    pub fn ws_url(&self) -> String {
        format!("ws://localhost:{}/ws", self.network.port)
    }

    /// Get the health check URL for logging
    pub fn health_url(&self) -> String {
        format!("http://localhost:{}/health", self.network.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = WorldConfig::default();
        assert_eq!(config.world.width, 180.0);
        assert_eq!(config.world.height, 130.0);
        assert_eq!(config.spawning.initial_population, 100);
        assert_eq!(config.timing.target_tick_rate, 20);
        assert_eq!(config.network.port, 8080);
    }

    #[test]
    fn test_network_urls() {
        let config = WorldConfig::default();
        assert_eq!(config.ws_bind_address(), "0.0.0.0:8080");
        assert_eq!(config.ws_url(), "ws://localhost:8080/ws");
        assert_eq!(config.health_url(), "http://localhost:8080/health");
    }
}
