use bevy_ecs::world::World;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Receiver;

use super::Command;
use crate::simulation::core::components::{Acceleration, BodySize, Position, Velocity};
use crate::simulation::components::{CritId, CreatureState, Rotation};
use crate::simulation::creatures::systems::NextCreatureId;

/// Resource that holds the command receiver from stdin reader thread
/// Wrapped in Arc<Mutex<>> for thread safety (Bevy requires Sync resources)
#[derive(bevy_ecs::system::Resource, Clone)]
pub struct CommandReceiver(pub Arc<Mutex<Receiver<Command>>>);

/// Bevy system that processes queued commands from stdin
#[cfg(feature = "dev-tools")]
pub fn command_executor_system(world: &mut World) {
    // Collect all pending commands first (avoids borrow checker issues)
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

    // Now process commands with mutable access to world
    for cmd in commands {
        match cmd {
            Command::DevSpawnCreature { x, y, dna } => {
                // Get next creature ID
                let mut next_id = world.resource_mut::<NextCreatureId>();
                let creature_id = next_id.generate();

                // Spawn creature at specified position with all required components
                // DNA is ignored for now (placeholder for future sprint)
                world.spawn((
                    CritId(creature_id),
                    Position { x, y },
                    Velocity { vx: 0.0, vy: 0.0 },
                    Acceleration::default(),
                    Rotation::default(),
                    BodySize::default(), // 1.0m default
                    CreatureState::new(), // Default energy (100), age 0
                ));

                if dna.is_some() {
                    eprintln!("[CommandExecutor] DNA parameter not yet implemented, ignoring");
                }
                eprintln!("[CommandExecutor] Spawned creature #{} at ({}, {})", creature_id, x, y);
            }
            Command::DevLoadTrial { template } => {
                eprintln!("[CommandExecutor] Loading trial: {}", template);

                // Try to load trial (will be implemented in Phase 1B)
                #[cfg(feature = "dev-tools")]
                {
                    use crate::trials;
                    match trials::loader::load_trial(world, &template) {
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
                // Query for all entities with CritId (i.e., all creatures)
                use bevy_ecs::query::QueryState;
                use bevy_ecs::entity::Entity;

                let mut query_state: QueryState<(Entity, &CritId)> = world.query();
                let entities: Vec<Entity> = query_state
                    .iter(world)
                    .map(|(entity, _)| entity)
                    .collect();

                let count = entities.len();

                // Despawn all creatures
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
    use std::sync::{mpsc, Arc, Mutex};

    #[test]
    fn test_dev_spawn_creature_spawns_at_position() {
        let (tx, rx) = mpsc::channel();

        // Send spawn command
        tx.send(Command::DevSpawnCreature {
            x: 123.45,
            y: 678.90,
            dna: None,
        })
        .unwrap();

        let mut world = World::new();
        world.insert_resource(CommandReceiver(Arc::new(Mutex::new(rx))));
        world.insert_resource(NextCreatureId::default());

        // Execute system
        command_executor_system(&mut world);

        // Query for spawned creature
        let mut query = world.query::<(&Position, &Velocity, &BodySize)>();
        let results: Vec<_> = query.iter(&world).collect();

        assert_eq!(results.len(), 1, "Should spawn exactly one creature");

        let (pos, vel, body) = results[0];
        assert_eq!(pos.x, 123.45);
        assert_eq!(pos.y, 678.90);
        assert_eq!(vel.vx, 0.0);
        assert_eq!(vel.vy, 0.0);
        assert_eq!(body.length, 1.0); // Default
    }

    #[test]
    fn test_queue_drains_all_commands_in_single_frame() {
        let (tx, rx) = mpsc::channel();

        // Send multiple commands
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

        // Execute system once
        command_executor_system(&mut world);

        // All three creatures should be spawned
        let mut query = world.query::<&Position>();
        let positions: Vec<_> = query.iter(&world).collect();

        assert_eq!(
            positions.len(),
            3,
            "Should process all commands in single frame"
        );

        // Verify positions (order may vary)
        let xs: Vec<f32> = positions.iter().map(|p| p.x).collect();
        assert!(xs.contains(&10.0));
        assert!(xs.contains(&30.0));
        assert!(xs.contains(&50.0));
    }

    #[test]
    fn test_invalid_command_doesnt_crash_system() {
        let (tx, rx) = mpsc::channel();

        // Send valid command, then load trial (stub), then another valid command
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

        // Should not panic
        command_executor_system(&mut world);

        // Two creatures should be spawned (trial loading doesn't spawn yet in Phase 1A)
        let mut query = world.query::<&Position>();
        let count = query.iter(&world).count();
        assert_eq!(
            count, 2,
            "System should continue after trial command (which doesn't spawn yet)"
        );
    }

    #[test]
    fn test_spawn_with_dna_placeholder() {
        let (tx, rx) = mpsc::channel();

        // Send command with DNA (should be ignored for now)
        tx.send(Command::DevSpawnCreature {
            x: 100.0,
            y: 200.0,
            dna: Some(serde_json::json!({"size": 10.0})),
        })
        .unwrap();

        let mut world = World::new();
        world.insert_resource(CommandReceiver(Arc::new(Mutex::new(rx))));
        world.insert_resource(NextCreatureId::default());

        command_executor_system(&mut world);

        // Should spawn creature despite DNA being present
        let mut query = world.query::<(&Position, &BodySize)>();
        let results: Vec<_> = query.iter(&world).collect();

        assert_eq!(results.len(), 1);
        let (pos, body) = results[0];
        assert_eq!(pos.x, 100.0);
        // DNA is ignored, so default body size should be used
        assert_eq!(body.length, 1.0);
    }

    #[test]
    fn test_clear_creatures_removes_all() {
        let (tx, rx) = mpsc::channel();

        // Spawn 3 creatures first
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

        // Execute spawn commands
        command_executor_system(&mut world);

        // Verify 3 creatures exist
        let mut query = world.query::<&CritId>();
        assert_eq!(query.iter(&world).count(), 3, "Should have 3 creatures");

        // Now send clear command
        tx.send(Command::DevClearCreatures).unwrap();

        // Execute clear command
        command_executor_system(&mut world);

        // Verify all creatures removed
        assert_eq!(query.iter(&world).count(), 0, "Should have 0 creatures after clear");
    }

    #[test]
    fn test_clear_creatures_with_empty_world() {
        let (tx, rx) = mpsc::channel();

        // Send clear command to empty world (should not crash)
        tx.send(Command::DevClearCreatures).unwrap();

        let mut world = World::new();
        world.insert_resource(CommandReceiver(Arc::new(Mutex::new(rx))));
        world.insert_resource(NextCreatureId::default());

        // Should not panic
        command_executor_system(&mut world);

        // World should still be empty
        let mut query = world.query::<&CritId>();
        assert_eq!(query.iter(&world).count(), 0);
    }
}
