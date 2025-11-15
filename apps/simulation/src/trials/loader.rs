//! Trial loader - reads TOML files and spawns creatures into ECS World

use bevy_ecs::world::World;
use std::fs;

use super::{CreatureType, SpawnPattern, TrialConfig};
use crate::simulation::core::components::{
    Acceleration, BodySize, Catatonic, Position, Velocity,
};
use crate::simulation::components::{CritId, CreatureState, Rotation};
use crate::simulation::creatures::components::Target;
use crate::simulation::creatures::systems::NextCreatureId;

/// Load a trial from TOML template file
///
/// # Arguments
/// * `world` - ECS World to spawn creatures into
/// * `template_name` - Name of trial template (e.g., "default-spawn-baseline")
///
/// # Returns
/// * `Ok(TrialConfig)` - Successfully loaded and spawned trial
/// * `Err(String)` - Error message if file not found or invalid TOML
///
/// # Example
/// ```no_run
/// use bevy_ecs::world::World;
/// use speciate::trials::loader::load_trial;
///
/// let mut world = World::new();
/// let config = load_trial(&mut world, "crowd-navigation").unwrap();
/// println!("Loaded trial: {}", config.name);
/// ```
#[cfg(feature = "dev-tools")]
pub fn load_trial(world: &mut World, template_name: &str) -> Result<TrialConfig, String> {
    // Resolve trial path relative to binary location, not CWD
    // When run from Electron, CWD is apps/portal but binary is in apps/simulation/target/debug
    let exe_dir = std::env::current_exe()
        .map_err(|e| format!("Failed to get executable path: {}", e))?
        .parent()
        .ok_or_else(|| "Failed to get executable directory".to_string())?
        .to_path_buf();

    // Detect if running from test (target/debug/deps) vs normal (target/debug)
    let trials_relative_path = if exe_dir.ends_with("deps") {
        "../../../trials" // test binary in target/debug/deps
    } else {
        "../../trials" // normal binary in target/debug or target/release
    };

    let trial_path = exe_dir
        .join(trials_relative_path)
        .join(format!("{}.toml", template_name));

    let path = trial_path.canonicalize()
        .map_err(|e| format!("Trial file not found at {}: {}", trial_path.display(), e))?;

    // Read TOML file
    let toml_content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read trial file '{}': {}", path.display(), e))?;

    // Parse TOML into TrialConfig
    let config: TrialConfig = toml::from_str(&toml_content)
        .map_err(|e| format!("Failed to parse trial TOML '{}': {}", path.display(), e))?;

    // Apply world configuration overrides (if present)
    if let Some(world_config) = &config.world {
        if let Some(dt) = world_config.delta_time {
            world.insert_resource(crate::simulation::core::components::DeltaTime(dt));
        }

        if let Some(boundary) = &world_config.boundary {
            world.insert_resource(crate::simulation::core::components::BoundaryConfig {
                min_x: boundary.min_x,
                max_x: boundary.max_x,
                min_y: boundary.min_y,
                max_y: boundary.max_y,
                margin: 10_000.0, // Default margin
                max_force: 1.0,   // Default max force
            });
        }
    }

    // Spawn creatures for each pattern
    for pattern in &config.spawns {
        spawn_pattern(world, pattern);
    }

    Ok(config)
}

/// Spawn creatures according to a spawn pattern
fn spawn_pattern(world: &mut World, pattern: &SpawnPattern) {
    match pattern {
        SpawnPattern::Single {
            x,
            y,
            creature_type,
        } => {
            spawn_creature(world, *x, *y, *creature_type);
        }

        SpawnPattern::Grid {
            start_x,
            start_y,
            spacing,
            rows,
            cols,
            creature_type,
        } => {
            for row in 0..*rows {
                for col in 0..*cols {
                    let x = start_x + (col as f32 * spacing);
                    let y = start_y + (row as f32 * spacing);
                    spawn_creature(world, x, y, *creature_type);
                }
            }
        }

        SpawnPattern::Circle {
            center_x,
            center_y,
            radius,
            count,
            creature_type,
        } => {
            for i in 0..*count {
                let angle = (i as f32 / *count as f32) * 2.0 * std::f32::consts::PI;
                let x = center_x + radius * angle.cos();
                let y = center_y + radius * angle.sin();
                spawn_creature(world, x, y, *creature_type);
            }
        }
    }
}

/// Spawn a single creature with appropriate components based on type
fn spawn_creature(world: &mut World, x: f32, y: f32, creature_type: CreatureType) {
    // Get next creature ID
    let mut next_id = world.resource_mut::<NextCreatureId>();
    let creature_id = next_id.generate();

    match creature_type {
        CreatureType::Catatonic => {
            world.spawn((
                CritId(creature_id),
                Position { x, y },
                Velocity { vx: 0.0, vy: 0.0 },
                Acceleration::default(),
                Rotation::default(),
                BodySize::default(),
                CreatureState::new(),
                Catatonic,
            ));
        }

        CreatureType::Seeker => {
            world.spawn((
                CritId(creature_id),
                Position { x, y },
                Velocity { vx: 0.0, vy: 0.0 },
                Acceleration::default(),
                Rotation::default(),
                BodySize::default(),
                CreatureState::new(),
                Target { x: 0.0, y: 0.0 }, // Default target at origin
            ));
        }

        CreatureType::Wanderer => {
            world.spawn((
                CritId(creature_id),
                Position { x, y },
                Velocity { vx: 0.0, vy: 0.0 },
                Acceleration::default(),
                Rotation::default(),
                BodySize::default(),
                CreatureState::new(),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_single_catatonic() {
        let mut world = World::new();
        world.insert_resource(NextCreatureId::default());

        let pattern = SpawnPattern::Single {
            x: 123.45,
            y: 678.90,
            creature_type: CreatureType::Catatonic,
        };

        spawn_pattern(&mut world, &pattern);

        // Verify creature spawned
        let mut query = world.query::<(&Position, &Catatonic)>();
        let results: Vec<_> = query.iter(&world).collect();

        assert_eq!(results.len(), 1);
        let (pos, _) = results[0];
        assert_eq!(pos.x, 123.45);
        assert_eq!(pos.y, 678.90);
    }

    #[test]
    fn test_spawn_single_seeker() {
        let mut world = World::new();
        world.insert_resource(NextCreatureId::default());

        let pattern = SpawnPattern::Single {
            x: 100.0,
            y: 200.0,
            creature_type: CreatureType::Seeker,
        };

        spawn_pattern(&mut world, &pattern);

        // Verify seeker has Target component
        let mut query = world.query::<(&Position, &Target)>();
        let results: Vec<_> = query.iter(&world).collect();

        assert_eq!(results.len(), 1);
        let (pos, target) = results[0];
        assert_eq!(pos.x, 100.0);
        assert_eq!(pos.y, 200.0);
        assert_eq!(target.x, 0.0); // Default target
        assert_eq!(target.y, 0.0);
    }

    #[test]
    fn test_spawn_grid_pattern() {
        let mut world = World::new();
        world.insert_resource(NextCreatureId::default());

        let pattern = SpawnPattern::Grid {
            start_x: 0.0,
            start_y: 0.0,
            spacing: 10.0,
            rows: 3,
            cols: 4,
            creature_type: CreatureType::Wanderer,
        };

        spawn_pattern(&mut world, &pattern);

        // Verify correct count (3 rows × 4 cols = 12 creatures)
        let mut query = world.query::<&Position>();
        let positions: Vec<_> = query.iter(&world).collect();
        assert_eq!(positions.len(), 12);

        // Verify first creature at (0, 0)
        let first = positions.iter().find(|p| p.x == 0.0 && p.y == 0.0);
        assert!(first.is_some());

        // Verify last creature at (30, 20) - (cols-1)*spacing, (rows-1)*spacing
        let last = positions.iter().find(|p| p.x == 30.0 && p.y == 20.0);
        assert!(last.is_some());
    }

    #[test]
    fn test_spawn_circle_pattern() {
        let mut world = World::new();
        world.insert_resource(NextCreatureId::default());

        let pattern = SpawnPattern::Circle {
            center_x: 100.0,
            center_y: 100.0,
            radius: 50.0,
            count: 8,
            creature_type: CreatureType::Wanderer,
        };

        spawn_pattern(&mut world, &pattern);

        // Verify count
        let mut query = world.query::<&Position>();
        let positions: Vec<_> = query.iter(&world).collect();
        assert_eq!(positions.len(), 8);

        // Verify all creatures are approximately 50m from center
        for pos in positions {
            let dx = pos.x - 100.0;
            let dy = pos.y - 100.0;
            let dist = (dx * dx + dy * dy).sqrt();
            assert!((dist - 50.0).abs() < 0.01, "Distance {} should be ~50.0", dist);
        }
    }

    #[test]
    fn test_grid_positions_correct() {
        let mut world = World::new();
        world.insert_resource(NextCreatureId::default());

        let pattern = SpawnPattern::Grid {
            start_x: 10.0,
            start_y: 20.0,
            spacing: 5.0,
            rows: 2,
            cols: 3,
            creature_type: CreatureType::Wanderer,
        };

        spawn_pattern(&mut world, &pattern);

        let mut query = world.query::<&Position>();
        let positions: Vec<_> = query.iter(&world).collect();

        // Expected positions:
        // Row 0: (10, 20), (15, 20), (20, 20)
        // Row 1: (10, 25), (15, 25), (20, 25)
        assert!(positions.iter().any(|p| p.x == 10.0 && p.y == 20.0));
        assert!(positions.iter().any(|p| p.x == 15.0 && p.y == 20.0));
        assert!(positions.iter().any(|p| p.x == 20.0 && p.y == 20.0));
        assert!(positions.iter().any(|p| p.x == 10.0 && p.y == 25.0));
        assert!(positions.iter().any(|p| p.x == 15.0 && p.y == 25.0));
        assert!(positions.iter().any(|p| p.x == 20.0 && p.y == 25.0));
    }

    #[test]
    fn test_circle_first_creature_at_angle_zero() {
        let mut world = World::new();
        world.insert_resource(NextCreatureId::default());

        let pattern = SpawnPattern::Circle {
            center_x: 0.0,
            center_y: 0.0,
            radius: 100.0,
            count: 4,
            creature_type: CreatureType::Wanderer,
        };

        spawn_pattern(&mut world, &pattern);

        let mut query = world.query::<&Position>();
        let positions: Vec<_> = query.iter(&world).collect();

        // First creature should be at angle 0 (radius along X axis)
        let first = positions
            .iter()
            .find(|p| (p.x - 100.0).abs() < 0.01 && p.y.abs() < 0.01);
        assert!(
            first.is_some(),
            "Should have creature at (100, 0) for angle 0"
        );
    }

    #[test]
    fn test_spawn_creature_types() {
        let mut world = World::new();
        world.insert_resource(NextCreatureId::default());

        spawn_creature(&mut world, 0.0, 0.0, CreatureType::Catatonic);
        spawn_creature(&mut world, 10.0, 10.0, CreatureType::Seeker);
        spawn_creature(&mut world, 20.0, 20.0, CreatureType::Wanderer);

        // Count each type
        let catatonic_count = world.query::<&Catatonic>().iter(&world).count();
        let seeker_count = world.query::<&Target>().iter(&world).count();
        let total_count = world.query::<&Position>().iter(&world).count();

        assert_eq!(catatonic_count, 1);
        assert_eq!(seeker_count, 1);
        assert_eq!(total_count, 3);
    }

    #[test]
    fn test_all_creatures_have_physics_components() {
        let mut world = World::new();
        world.insert_resource(NextCreatureId::default());

        let pattern = SpawnPattern::Grid {
            start_x: 0.0,
            start_y: 0.0,
            spacing: 10.0,
            rows: 2,
            cols: 2,
            creature_type: CreatureType::Wanderer,
        };

        spawn_pattern(&mut world, &pattern);

        // All creatures should have Position, Velocity, Acceleration, BodySize
        let pos_count = world.query::<&Position>().iter(&world).count();
        let vel_count = world.query::<&Velocity>().iter(&world).count();
        let accel_count = world.query::<&Acceleration>().iter(&world).count();
        let body_count = world.query::<&BodySize>().iter(&world).count();

        assert_eq!(pos_count, 4);
        assert_eq!(vel_count, 4);
        assert_eq!(accel_count, 4);
        assert_eq!(body_count, 4);
    }

    #[test]
    fn test_circle_evenly_distributed() {
        let mut world = World::new();
        world.insert_resource(NextCreatureId::default());

        let pattern = SpawnPattern::Circle {
            center_x: 0.0,
            center_y: 0.0,
            radius: 100.0,
            count: 4,
            creature_type: CreatureType::Wanderer,
        };

        spawn_pattern(&mut world, &pattern);

        let mut query = world.query::<&Position>();
        let positions: Vec<_> = query.iter(&world).collect();

        // With 4 creatures, angles should be 0°, 90°, 180°, 270°
        // Positions: (100, 0), (0, 100), (-100, 0), (0, -100)
        assert!(positions.iter().any(|p| (p.x - 100.0).abs() < 0.01
            && p.y.abs() < 0.01));
        assert!(positions.iter().any(|p| p.x.abs() < 0.01
            && (p.y - 100.0).abs() < 0.01));
        assert!(positions.iter().any(|p| (p.x + 100.0).abs() < 0.01
            && p.y.abs() < 0.01));
        assert!(positions.iter().any(|p| p.x.abs() < 0.01
            && (p.y + 100.0).abs() < 0.01));
    }
}
