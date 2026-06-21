use bevy_ecs::world::World;
use std::fs;

use super::{CreatureType, SpawnPattern, SpecConfig, TrialConfig};
use crate::simulation::creatures::builder::CritBuilder;
use crate::simulation::creatures::components::state::BehaviorMode;
use crate::simulation::creatures::components::EntityTag;
use crate::simulation::creatures::dna::{Dna, SizeDistributionParams};
use crate::simulation::creatures::systems::NextCreatureId;

#[cfg(feature = "dev-tools")]
pub fn load_trial(
    world: &mut World,
    template_name: &str,
    randomize_dna: bool,
    override_dna: Option<Dna>,
) -> Result<TrialConfig, String> {
    let cwd =
        std::env::current_dir().map_err(|e| format!("Failed to get current directory: {}", e))?;

    // Try specs/ first (new format: "category/name"), then fall back to trials/ (legacy)
    let spec_path = cwd.join("specs").join(format!("{}.toml", template_name));

    let trial_path = cwd.join("trials").join(format!("{}.toml", template_name));

    let path = spec_path
        .canonicalize()
        .or_else(|_| trial_path.canonicalize())
        .map_err(|e| {
            format!(
                "Spec/trial file not found at {} or {}: {}",
                spec_path.display(),
                trial_path.display(),
                e
            )
        })?;

    let toml_content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read trial file '{}': {}", path.display(), e))?;

    // Try parsing as new SpecConfig format first (has [meta] section)
    let config = if toml_content.contains("[meta]") {
        let spec: SpecConfig = toml::from_str(&toml_content)
            .map_err(|e| format!("Failed to parse spec TOML '{}': {}", path.display(), e))?;

        // Convert SpecConfig to TrialConfig for spawning
        TrialConfig {
            name: spec.meta.name,
            description: spec.meta.description,
            spawns: spec.spawns,
            world: None, // Specs don't use world config
        }
    } else {
        // Legacy TrialConfig format
        toml::from_str(&toml_content)
            .map_err(|e| format!("Failed to parse trial TOML '{}': {}", path.display(), e))?
    };

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
                margin: 10_000.0,
                max_force: 1.0,
            });
        }
    }

    for pattern in &config.spawns {
        spawn_pattern(world, pattern, randomize_dna, override_dna.as_ref());
    }

    Ok(config)
}

fn spawn_pattern(
    world: &mut World,
    pattern: &SpawnPattern,
    randomize_dna: bool,
    override_dna: Option<&Dna>,
) {
    match pattern {
        SpawnPattern::Single {
            tag,
            x,
            y,
            creature_type,
            target_x,
            target_y,
            body_size,
        } => {
            spawn_creature(
                world,
                *x,
                *y,
                *creature_type,
                *target_x,
                *target_y,
                tag.clone(),
                *body_size,
                randomize_dna,
                override_dna,
            );
        }

        SpawnPattern::Grid {
            tag,
            start_x,
            start_y,
            spacing,
            rows,
            cols,
            creature_type,
            grid_offset_y,
            target_x,
            target_y,
            body_size,
        } => {
            for row in 0..*rows {
                for col in 0..*cols {
                    let x = start_x + (col as f32 * spacing);
                    let offset = if col % 2 == 1 {
                        grid_offset_y.unwrap_or(0.0)
                    } else {
                        0.0
                    };
                    let y = start_y + (row as f32 * spacing) + offset;
                    spawn_creature(
                        world,
                        x,
                        y,
                        *creature_type,
                        *target_x,
                        *target_y,
                        tag.clone(),
                        *body_size,
                        randomize_dna,
                        override_dna,
                    );
                }
            }
        }

        SpawnPattern::Circle {
            tag,
            center_x,
            center_y,
            radius,
            count,
            creature_type,
            target_x,
            target_y,
            body_size,
        } => {
            for i in 0..*count {
                let angle = (i as f32 / *count as f32) * 2.0 * std::f32::consts::PI;
                let x = center_x + radius * angle.cos();
                let y = center_y + radius * angle.sin();
                spawn_creature(
                    world,
                    x,
                    y,
                    *creature_type,
                    *target_x,
                    *target_y,
                    tag.clone(),
                    *body_size,
                    randomize_dna,
                    override_dna,
                );
            }
        }
    }
}

fn spawn_creature(
    world: &mut World,
    x: f32,
    y: f32,
    creature_type: CreatureType,
    target_x: Option<f32>,
    target_y: Option<f32>,
    tag: Option<String>,
    body_size: Option<f32>,
    randomize_dna: bool,
    override_dna: Option<&Dna>,
) {
    use crate::simulation::creatures::dna::{DEFAULT_FOV_GENE, SIZE_MAX, SIZE_MIN};

    let mut next_id = world.resource_mut::<NextCreatureId>();
    let creature_id = next_id.generate();

    let mut builder = match creature_type {
        CreatureType::Catatonic => CritBuilder::new()
            .at(x, y)
            .in_behavior(BehaviorMode::Catatonic),

        CreatureType::Seeker => {
            let target_x = target_x.unwrap_or(0.0);
            let target_y = target_y.unwrap_or(0.0);
            CritBuilder::new().at(x, y).as_seeker(target_x, target_y)
        }

        CreatureType::Wanderer => CritBuilder::new()
            .at(x, y)
            .in_behavior(BehaviorMode::Wandering),
    };

    // Apply tag if present
    if let Some(tag_str) = &tag {
        builder = builder.with_tag(tag_str.clone());
    }

    // Apply DNA priority: body_size > override_dna > random_dna > default
    if let Some(size) = body_size {
        let size_gene = ((size - SIZE_MIN) / (SIZE_MAX - SIZE_MIN)).clamp(0.0, 1.0);
        builder = builder.with_dna(Dna::new(size_gene, DEFAULT_FOV_GENE));
    } else if randomize_dna {
        builder = builder.with_dna(Dna::random_realistic_seeded(
            &mut rand::thread_rng(),
            &SizeDistributionParams::realistic(),
        ));
    } else if let Some(dna) = override_dna {
        builder = builder.with_dna(dna.clone());
    }

    let bundle = builder.build(creature_id);
    let entity = world.spawn(bundle).id();

    // Insert EntityTag component separately (not part of CritBundle)
    if let Some(tag_str) = tag {
        world.entity_mut(entity).insert(EntityTag(tag_str));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::core::components::{Acceleration, BodySize, Position, Velocity};
    use crate::simulation::creatures::components::{CreatureState, Target};

    #[test]
    fn test_spawn_single_catatonic() {
        let mut world = World::new();
        world.insert_resource(NextCreatureId::default());

        let pattern = SpawnPattern::Single {
            x: 123.45,
            y: 678.90,
            creature_type: CreatureType::Catatonic,
            target_x: None,
            target_y: None,
            tag: None,
            body_size: None,
        };

        spawn_pattern(&mut world, &pattern, false, None);

        let mut query = world.query::<(&Position, &CreatureState)>();
        let results: Vec<_> = query
            .iter(&world)
            .filter(|(_, state)| state.behavior == BehaviorMode::Catatonic)
            .collect();

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
            target_x: None,
            target_y: None,
            tag: None,
            body_size: None,
        };

        spawn_pattern(&mut world, &pattern, false, None);

        let mut query = world.query::<(&Position, &Target, &CreatureState)>();
        let results: Vec<_> = query.iter(&world).collect();

        assert_eq!(results.len(), 1);
        let (pos, target, state) = results[0];
        assert_eq!(pos.x, 100.0);
        assert_eq!(pos.y, 200.0);
        assert_eq!(target.x, 0.0);
        assert_eq!(target.y, 0.0);

        assert!(matches!(state.behavior, BehaviorMode::Seeking));
    }

    #[test]
    fn test_spawn_seeker_with_custom_target() {
        let mut world = World::new();
        world.insert_resource(NextCreatureId::default());

        let pattern = SpawnPattern::Single {
            x: 20.0,
            y: 0.0,
            creature_type: CreatureType::Seeker,
            target_x: Some(-10.0),
            target_y: Some(5.0),
            tag: None,
            body_size: None,
        };

        spawn_pattern(&mut world, &pattern, false, None);

        let mut query = world.query::<(&Position, &Target, &CreatureState)>();
        let results: Vec<_> = query.iter(&world).collect();

        assert_eq!(results.len(), 1);
        let (pos, target, state) = results[0];
        assert_eq!(pos.x, 20.0);
        assert_eq!(pos.y, 0.0);
        assert_eq!(target.x, -10.0);
        assert_eq!(target.y, 5.0);

        assert!(matches!(state.behavior, BehaviorMode::Seeking));
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
            grid_offset_y: None,
            tag: None,
            target_x: None,
            target_y: None,
            body_size: None,
        };

        spawn_pattern(&mut world, &pattern, false, None);

        let mut query = world.query::<&Position>();
        let positions: Vec<_> = query.iter(&world).collect();
        assert_eq!(positions.len(), 12);

        let first = positions.iter().find(|p| p.x == 0.0 && p.y == 0.0);
        assert!(first.is_some());

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
            target_x: None,
            target_y: None,
            tag: None,
            body_size: None,
        };

        spawn_pattern(&mut world, &pattern, false, None);

        let mut query = world.query::<&Position>();
        let positions: Vec<_> = query.iter(&world).collect();
        assert_eq!(positions.len(), 8);

        for pos in positions {
            let dx = pos.x - 100.0;
            let dy = pos.y - 100.0;
            let dist = (dx * dx + dy * dy).sqrt();
            assert!(
                (dist - 50.0).abs() < 0.01,
                "Distance {} should be ~50.0",
                dist
            );
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
            grid_offset_y: None,
            tag: None,
            target_x: None,
            target_y: None,
            body_size: None,
        };

        spawn_pattern(&mut world, &pattern, false, None);

        let mut query = world.query::<&Position>();
        let positions: Vec<_> = query.iter(&world).collect();

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
            target_x: None,
            target_y: None,
            tag: None,
            body_size: None,
        };

        spawn_pattern(&mut world, &pattern, false, None);

        let mut query = world.query::<&Position>();
        let positions: Vec<_> = query.iter(&world).collect();

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

        spawn_creature(
            &mut world,
            0.0,
            0.0,
            CreatureType::Catatonic,
            None,
            None,
            None,
            None,
            false,
            None,
        );
        spawn_creature(
            &mut world,
            10.0,
            10.0,
            CreatureType::Seeker,
            None,
            None,
            None,
            None,
            false,
            None,
        );
        spawn_creature(
            &mut world,
            20.0,
            20.0,
            CreatureType::Wanderer,
            None,
            None,
            None,
            None,
            false,
            None,
        );

        let catatonic_count = world
            .query::<&CreatureState>()
            .iter(&world)
            .filter(|state| state.behavior == BehaviorMode::Catatonic)
            .count();
        let seeker_count = world
            .query::<&CreatureState>()
            .iter(&world)
            .filter(|state| state.behavior == BehaviorMode::Seeking)
            .count();
        let wanderer_count = world
            .query::<&CreatureState>()
            .iter(&world)
            .filter(|state| state.behavior == BehaviorMode::Wandering)
            .count();
        let total_count = world.query::<&Position>().iter(&world).count();

        assert_eq!(catatonic_count, 1);
        assert_eq!(seeker_count, 1);
        assert_eq!(wanderer_count, 1);
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
            grid_offset_y: None,
            tag: None,
            target_x: None,
            target_y: None,
            body_size: None,
        };

        spawn_pattern(&mut world, &pattern, false, None);

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
            target_x: None,
            target_y: None,
            tag: None,
            body_size: None,
        };

        spawn_pattern(&mut world, &pattern, false, None);

        let mut query = world.query::<&Position>();
        let positions: Vec<_> = query.iter(&world).collect();

        assert!(positions
            .iter()
            .any(|p| (p.x - 100.0).abs() < 0.01 && p.y.abs() < 0.01));
        assert!(positions
            .iter()
            .any(|p| p.x.abs() < 0.01 && (p.y - 100.0).abs() < 0.01));
        assert!(positions
            .iter()
            .any(|p| (p.x + 100.0).abs() < 0.01 && p.y.abs() < 0.01));
        assert!(positions
            .iter()
            .any(|p| p.x.abs() < 0.01 && (p.y + 100.0).abs() < 0.01));
    }

    #[test]
    fn test_circle_seekers_with_shared_target() {
        let mut world = World::new();
        world.insert_resource(NextCreatureId::default());

        let pattern = SpawnPattern::Circle {
            center_x: 0.0,
            center_y: 0.0,
            radius: 50.0,
            count: 4,
            creature_type: CreatureType::Seeker,
            target_x: Some(0.0),
            target_y: Some(0.0),
            tag: None,
            body_size: None,
        };

        spawn_pattern(&mut world, &pattern, false, None);

        let mut query = world.query::<(&Target, &CreatureState)>();
        let results: Vec<_> = query.iter(&world).collect();

        assert_eq!(results.len(), 4);
        for (target, state) in results {
            assert_eq!(target.x, 0.0);
            assert_eq!(target.y, 0.0);
            assert!(matches!(state.behavior, BehaviorMode::Seeking));
        }
    }

    #[test]
    fn test_grid_with_column_offset() {
        let mut world = World::new();
        world.insert_resource(NextCreatureId::default());

        let pattern = SpawnPattern::Grid {
            start_x: 0.0,
            start_y: 0.0,
            spacing: 10.0,
            rows: 2,
            cols: 4,
            creature_type: CreatureType::Catatonic,
            grid_offset_y: Some(5.0),
            tag: None,
            target_x: None,
            target_y: None,
            body_size: None,
        };

        spawn_pattern(&mut world, &pattern, false, None);

        let mut query = world.query::<&Position>();
        let positions: Vec<_> = query.iter(&world).collect();

        assert!(positions.iter().any(|p| p.x == 0.0 && p.y == 0.0));
        assert!(positions.iter().any(|p| p.x == 10.0 && p.y == 5.0));
        assert!(positions.iter().any(|p| p.x == 20.0 && p.y == 0.0));
        assert!(positions.iter().any(|p| p.x == 30.0 && p.y == 5.0));

        assert!(positions.iter().any(|p| p.x == 0.0 && p.y == 10.0));
        assert!(positions.iter().any(|p| p.x == 10.0 && p.y == 15.0));
        assert!(positions.iter().any(|p| p.x == 20.0 && p.y == 10.0));
        assert!(positions.iter().any(|p| p.x == 30.0 && p.y == 15.0));
    }

    #[test]
    fn test_grid_without_offset_backward_compatible() {
        let mut world = World::new();
        world.insert_resource(NextCreatureId::default());

        let pattern = SpawnPattern::Grid {
            start_x: 0.0,
            start_y: 0.0,
            spacing: 10.0,
            rows: 2,
            cols: 3,
            creature_type: CreatureType::Wanderer,
            grid_offset_y: None,
            tag: None,
            target_x: None,
            target_y: None,
            body_size: None,
        };

        spawn_pattern(&mut world, &pattern, false, None);

        let mut query = world.query::<&Position>();
        let positions: Vec<_> = query.iter(&world).collect();

        assert!(positions.iter().any(|p| p.x == 0.0 && p.y == 0.0));
        assert!(positions.iter().any(|p| p.x == 10.0 && p.y == 0.0));
        assert!(positions.iter().any(|p| p.x == 20.0 && p.y == 0.0));
        assert!(positions.iter().any(|p| p.x == 0.0 && p.y == 10.0));
        assert!(positions.iter().any(|p| p.x == 10.0 && p.y == 10.0));
        assert!(positions.iter().any(|p| p.x == 20.0 && p.y == 10.0));
    }

    #[test]
    fn test_grid_seekers_with_shared_target() {
        let mut world = World::new();
        world.insert_resource(NextCreatureId::default());

        let pattern = SpawnPattern::Grid {
            start_x: -50.0,
            start_y: -50.0,
            spacing: 25.0,
            rows: 2,
            cols: 2,
            creature_type: CreatureType::Seeker,
            grid_offset_y: None,
            tag: Some("grid-seekers".to_string()),
            target_x: Some(100.0),
            target_y: Some(100.0),
            body_size: None,
        };

        spawn_pattern(&mut world, &pattern, false, None);

        let mut query = world.query::<(&Target, &CreatureState)>();
        let results: Vec<_> = query.iter(&world).collect();

        assert_eq!(results.len(), 4);
        for (target, state) in results {
            assert_eq!(target.x, 100.0);
            assert_eq!(target.y, 100.0);
            assert!(matches!(state.behavior, BehaviorMode::Seeking));
        }
    }
}
