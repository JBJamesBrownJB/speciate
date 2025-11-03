//! Creature spawning logic
//!
//! This module handles the creation and spawning of creatures into the simulation world.

use crate::config::SpawningConfig;
use crate::simulation::Simulation;
use log::info;
use rand::Rng;

/// Spawn initial creatures based on configuration
pub fn spawn_initial_creatures(simulation: &mut Simulation, config: &SpawningConfig) {
    let mut rng = rand::thread_rng();

    info!(
        "Spawning {} initial creatures...",
        config.initial_population
    );

    for _ in 0..config.initial_population {
        // Generate random size within configured range
        let size_val = rng.gen_range(config.min_size..config.max_size);

        // Generate random position within configured spawn area
        let x = rng.gen_range(config.spawn_x_min..config.spawn_x_max);
        let y = rng.gen_range(config.spawn_y_min..config.spawn_y_max);

        // Spawn creature with width = size * 2, height = size
        simulation.spawn_creature(x, y, size_val * 2.0, size_val);
    }

    let count = simulation.creature_count();
    info!("✅ Total creatures spawned: {}", count);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SpawningConfig;
    use crate::simulation::Simulation;

    #[test]
    fn test_spawn_initial_creatures() {
        let mut simulation = Simulation::new();
        let config = SpawningConfig {
            initial_population: 10,
            min_size: 0.5,
            max_size: 2.0,
            spawn_x_min: 40.0,
            spawn_x_max: 140.0,
            spawn_y_min: 30.0,
            spawn_y_max: 100.0,
        };

        spawn_initial_creatures(&mut simulation, &config);

        assert_eq!(simulation.creature_count(), 10);
    }

    #[test]
    fn test_spawn_zero_creatures() {
        let mut simulation = Simulation::new();
        let config = SpawningConfig {
            initial_population: 0,
            min_size: 0.5,
            max_size: 2.0,
            spawn_x_min: 40.0,
            spawn_x_max: 140.0,
            spawn_y_min: 30.0,
            spawn_y_max: 100.0,
        };

        spawn_initial_creatures(&mut simulation, &config);

        assert_eq!(simulation.creature_count(), 0);
    }
}
