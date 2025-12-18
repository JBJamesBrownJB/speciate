use bevy_ecs::world::World;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Receiver;

use super::Command;
use crate::simulation::core::WorldBounds;
use crate::simulation::creatures::builder::CritBuilder;
use crate::simulation::creatures::components::CritId;
use crate::simulation::creatures::dna::Dna;
use crate::simulation::creatures::systems::NextCreatureId;

#[derive(bevy_ecs::system::Resource, Clone)]
pub struct CommandReceiver(pub Arc<Mutex<Receiver<Command>>>);

#[cfg(feature = "dev-tools")]
pub fn command_executor_system(world: &mut World) {

    let commands: Vec<Command> = {
        let receiver = match world.get_resource::<CommandReceiver>() {
            Some(r) => r,
            None => {
                eprintln!("[CommandExecutor] CommandReceiver resource not found");
                return;
            }
        };

        let rx = receiver.0.lock().unwrap();
        let mut cmds = Vec::new();
        while let Ok(cmd) = rx.try_recv() {
            cmds.push(cmd);
        }
        cmds
    };


    for cmd in commands {
        match cmd {
            Command::DevSpawnCreature { x, y, dna } => {
                let mut next_id = world.resource_mut::<NextCreatureId>();
                let creature_id = next_id.generate();

                // Parse DNA from JSON or use default
                let creature_dna = match dna {
                    Some(json) => {
                        let size_gene = json
                            .get("size_gene")
                            .and_then(|v| v.as_f64())
                            .map(|v| v as f32)
                            .unwrap_or(crate::simulation::creatures::dna::DEFAULT_SIZE_GENE);
                        let fov_gene = json
                            .get("fov_gene")
                            .and_then(|v| v.as_f64())
                            .map(|v| v as f32)
                            .unwrap_or(crate::simulation::creatures::dna::DEFAULT_FOV_GENE);
                        Dna::new(size_gene, fov_gene)
                    }
                    None => Dna::default(),
                };

                // Get world bounds for wanderer initialization
                let world_bounds = world
                    .get_resource::<WorldBounds>()
                    .cloned()
                    .unwrap_or_else(|| WorldBounds::from_dimensions(1000.0, 1000.0));

                // Build creature using CritBuilder for full component bundle
                let bundle = CritBuilder::new()
                    .at(x, y)
                    .with_dna(creature_dna)
                    .as_wanderer(&world_bounds)
                    .build(creature_id);

                world.spawn(bundle);

                eprintln!(
                    "[CommandExecutor] Spawned creature #{} at ({}, {}) with DNA({:.2}, {:.2})",
                    creature_id, x, y, creature_dna.size_gene, creature_dna.fov_gene
                );
            }
            Command::DevLoadTrial { template } => {
                eprintln!("[CommandExecutor] Loading trial: {}", template);


                #[cfg(feature = "dev-tools")]
                {
                    use crate::trials;
                    // Legacy stdio command doesn't support randomize_dna or custom DNA
                    match trials::loader::load_trial(world, &template, false, None) {
                        Ok(config) => {
                            eprintln!(
                                "[CommandExecutor] ✓ Loaded trial: {} ({} spawn patterns)",
                                config.name,
                                config.spawns.len()
                            );
                        }
                        Err(e) => {
                            eprintln!(
                                "[CommandExecutor] ✗ Failed to load trial '{}': {}",
                                template, e
                            );
                        }
                    }
                }
            }
            Command::DevClearCreatures => {

                use bevy_ecs::query::QueryState;
                use bevy_ecs::entity::Entity;

                let mut query_state: QueryState<(Entity, &CritId)> = world.query();
                let entities: Vec<Entity> = query_state
                    .iter(world)
                    .map(|(entity, _)| entity)
                    .collect();

                let count = entities.len();


                for entity in entities {
                    world.despawn(entity);
                }

                eprintln!("[CommandExecutor] Cleared {} creatures", count);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::core::components::{BodySize, Position, Velocity};
    use std::sync::{mpsc, Arc, Mutex};

    #[test]
    fn test_dev_spawn_creature_spawns_at_position() {
        let (tx, rx) = mpsc::channel();

        tx.send(Command::DevSpawnCreature {
            x: 123.45,
            y: 678.90,
            dna: None,
        })
        .unwrap();

        let mut world = World::new();
        world.insert_resource(CommandReceiver(Arc::new(Mutex::new(rx))));
        world.insert_resource(NextCreatureId::default());
        world.insert_resource(crate::simulation::core::WorldBounds::from_dimensions(1000.0, 1000.0));

        command_executor_system(&mut world);

        let mut query = world.query::<(&Position, &Velocity, &BodySize)>();
        let results: Vec<_> = query.iter(&world).collect();

        assert_eq!(results.len(), 1, "Should spawn exactly one creature");

        let (pos, vel, body) = results[0];
        assert_eq!(pos.x, 123.45);
        assert_eq!(pos.y, 678.90);
        assert_eq!(vel.vx, 0.0);
        assert_eq!(vel.vy, 0.0);
        // Default DNA expresses to approximately 1.0m (within 5%)
        assert!(
            (body.length - 1.0).abs() < 0.05,
            "Default creature size should be ~1.0m, got {}",
            body.length
        );
    }

    #[test]
    fn test_queue_drains_all_commands_in_single_frame() {
        let (tx, rx) = mpsc::channel();

        tx.send(Command::DevSpawnCreature {
            x: 10.0,
            y: 20.0,
            dna: None,
        })
        .unwrap();
        tx.send(Command::DevSpawnCreature {
            x: 30.0,
            y: 40.0,
            dna: None,
        })
        .unwrap();
        tx.send(Command::DevSpawnCreature {
            x: 50.0,
            y: 60.0,
            dna: None,
        })
        .unwrap();

        let mut world = World::new();
        world.insert_resource(CommandReceiver(Arc::new(Mutex::new(rx))));
        world.insert_resource(NextCreatureId::default());
        world.insert_resource(crate::simulation::core::WorldBounds::from_dimensions(1000.0, 1000.0));

        command_executor_system(&mut world);

        let mut query = world.query::<&Position>();
        let positions: Vec<_> = query.iter(&world).collect();

        assert_eq!(
            positions.len(),
            3,
            "Should process all commands in single frame"
        );

        let xs: Vec<f32> = positions.iter().map(|p| p.x).collect();
        assert!(xs.contains(&10.0));
        assert!(xs.contains(&30.0));
        assert!(xs.contains(&50.0));
    }

    #[test]
    fn test_invalid_command_doesnt_crash_system() {
        let (tx, rx) = mpsc::channel();

        tx.send(Command::DevSpawnCreature {
            x: 1.0,
            y: 2.0,
            dna: None,
        })
        .unwrap();
        tx.send(Command::DevLoadTrial {
            template: "test".to_string(),
        })
        .unwrap();
        tx.send(Command::DevSpawnCreature {
            x: 3.0,
            y: 4.0,
            dna: None,
        })
        .unwrap();

        let mut world = World::new();
        world.insert_resource(CommandReceiver(Arc::new(Mutex::new(rx))));
        world.insert_resource(NextCreatureId::default());
        world.insert_resource(crate::simulation::core::WorldBounds::from_dimensions(1000.0, 1000.0));

        command_executor_system(&mut world);

        let mut query = world.query::<&Position>();
        let count = query.iter(&world).count();
        assert_eq!(
            count, 2,
            "System should continue after trial command (which doesn't spawn yet)"
        );
    }

    #[test]
    fn test_spawn_with_dna_applies_genes() {
        use crate::simulation::creatures::dna::{Dna, SIZE_MAX};

        let (tx, rx) = mpsc::channel();

        // DNA with max size gene (1.0) should produce SIZE_MAX (5.0m) creature
        tx.send(Command::DevSpawnCreature {
            x: 100.0,
            y: 200.0,
            dna: Some(serde_json::json!({"size_gene": 1.0, "fov_gene": 0.5})),
        })
        .unwrap();

        let mut world = World::new();
        world.insert_resource(CommandReceiver(Arc::new(Mutex::new(rx))));
        world.insert_resource(NextCreatureId::default());
        world.insert_resource(crate::simulation::core::WorldBounds::from_dimensions(1000.0, 1000.0));

        command_executor_system(&mut world);

        let mut query = world.query::<(&Position, &BodySize, &Dna)>();
        let results: Vec<_> = query.iter(&world).collect();

        assert_eq!(results.len(), 1);
        let (pos, body, dna) = results[0];
        assert_eq!(pos.x, 100.0);
        assert_eq!(dna.size_gene, 1.0);
        assert_eq!(dna.fov_gene, 0.5);
        assert_eq!(body.length, SIZE_MAX, "DNA size_gene=1.0 should produce SIZE_MAX creature");
    }

    #[test]
    fn test_spawn_without_dna_uses_default() {
        use crate::simulation::creatures::dna::{Dna, DEFAULT_SIZE_GENE, DEFAULT_FOV_GENE};

        let (tx, rx) = mpsc::channel();

        tx.send(Command::DevSpawnCreature {
            x: 50.0,
            y: 50.0,
            dna: None,
        })
        .unwrap();

        let mut world = World::new();
        world.insert_resource(CommandReceiver(Arc::new(Mutex::new(rx))));
        world.insert_resource(NextCreatureId::default());
        world.insert_resource(crate::simulation::core::WorldBounds::from_dimensions(1000.0, 1000.0));

        command_executor_system(&mut world);

        let mut query = world.query::<&Dna>();
        let results: Vec<_> = query.iter(&world).collect();

        assert_eq!(results.len(), 1);
        let dna = results[0];
        assert_eq!(dna.size_gene, DEFAULT_SIZE_GENE);
        assert_eq!(dna.fov_gene, DEFAULT_FOV_GENE);
    }

    #[test]
    fn test_spawn_includes_full_bundle_components() {
        use crate::simulation::creatures::components::{Brain, CanSeek, CanFlee, Target};
        use crate::simulation::perception::Perception;

        let (tx, rx) = mpsc::channel();

        tx.send(Command::DevSpawnCreature {
            x: 0.0,
            y: 0.0,
            dna: None,
        })
        .unwrap();

        let mut world = World::new();
        world.insert_resource(CommandReceiver(Arc::new(Mutex::new(rx))));
        world.insert_resource(NextCreatureId::default());
        world.insert_resource(crate::simulation::core::WorldBounds::from_dimensions(1000.0, 1000.0));

        command_executor_system(&mut world);

        // Verify full bundle components are present
        let mut query = world.query::<(&Brain, &CanSeek, &CanFlee, &Perception, &Target)>();
        let count = query.iter(&world).count();
        assert_eq!(count, 1, "Creature should have full bundle components (Brain, CanSeek, etc.)");
    }

    #[test]
    fn test_clear_creatures_removes_all() {
        let (tx, rx) = mpsc::channel();

        tx.send(Command::DevSpawnCreature {
            x: 10.0,
            y: 20.0,
            dna: None,
        })
        .unwrap();
        tx.send(Command::DevSpawnCreature {
            x: 30.0,
            y: 40.0,
            dna: None,
        })
        .unwrap();
        tx.send(Command::DevSpawnCreature {
            x: 50.0,
            y: 60.0,
            dna: None,
        })
        .unwrap();

        let mut world = World::new();
        world.insert_resource(CommandReceiver(Arc::new(Mutex::new(rx))));
        world.insert_resource(NextCreatureId::default());
        world.insert_resource(crate::simulation::core::WorldBounds::from_dimensions(1000.0, 1000.0));

        command_executor_system(&mut world);

        let mut query = world.query::<&CritId>();
        assert_eq!(query.iter(&world).count(), 3, "Should have 3 creatures");

        tx.send(Command::DevClearCreatures).unwrap();

        command_executor_system(&mut world);

        assert_eq!(query.iter(&world).count(), 0, "Should have 0 creatures after clear");
    }

    #[test]
    fn test_clear_creatures_with_empty_world() {
        let (tx, rx) = mpsc::channel();


        tx.send(Command::DevClearCreatures).unwrap();

        let mut world = World::new();
        world.insert_resource(CommandReceiver(Arc::new(Mutex::new(rx))));
        world.insert_resource(NextCreatureId::default());


        command_executor_system(&mut world);


        let mut query = world.query::<&CritId>();
        assert_eq!(query.iter(&world).count(), 0);
    }
}
