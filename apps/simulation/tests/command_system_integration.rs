/// End-to-end integration test for stdin command system
///
/// Tests the full pipeline:
/// 1. MessagePack frame written to simulated stdin
/// 2. stdin_reader thread reads and parses frame
/// 3. Command sent to queue via channel
/// 4. command_executor_system drains queue and spawns creature
/// 5. Creature appears in ECS World at correct position

use std::io::Write;
use std::sync::{mpsc, Arc, Mutex};

use bevy_ecs::prelude::*;
use speciate::ipc::{Command, CommandReceiver};
use speciate::simulation::core::components::{BodySize, Position};

#[cfg(feature = "dev-tools")]
use speciate::ipc::command_executor_system;

#[test]
#[cfg(feature = "dev-tools")]
fn test_end_to_end_spawn_command() {
    // 1. Create MessagePack frame for spawn command
    let command = Command::DevSpawnCreature {
        x: 456.78,
        y: 901.23,
        dna: None,
    };

    let payload = rmp_serde::to_vec(&command).unwrap();
    let len = payload.len() as u32;
    let mut frame = len.to_be_bytes().to_vec();
    frame.extend_from_slice(&payload);

    // 2. Simulate stdin using os_pipe
    let (pipe_reader, mut pipe_writer) = os_pipe::pipe().unwrap();

    // 3. Set up channel for command queue
    let (tx, rx) = mpsc::channel();

    // 4. Spawn reader thread
    let reader_handle = std::thread::spawn(move || {
        use std::io::Read;
        let mut reader = pipe_reader;
        let mut buffer = Vec::new();

        // Read length prefix
        let mut len_bytes = [0u8; 4];
        reader.read_exact(&mut len_bytes).unwrap();
        let frame_len = u32::from_be_bytes(len_bytes) as usize;

        // Read payload
        buffer.resize(frame_len, 0);
        reader.read_exact(&mut buffer).unwrap();

        // Deserialize and send
        let cmd: Command = rmp_serde::from_slice(&buffer).unwrap();
        tx.send(cmd).unwrap();
    });

    // 5. Write frame to pipe
    pipe_writer.write_all(&frame).unwrap();
    drop(pipe_writer); // Signal EOF

    // 6. Wait for reader to process
    reader_handle.join().unwrap();

    // 7. Create Bevy World with command queue
    let mut world = World::new();
    world.insert_resource(CommandReceiver(Arc::new(Mutex::new(rx))));
    world.insert_resource(speciate::simulation::creatures::systems::NextCreatureId::default());

    // 8. Run executor system
    command_executor_system(&mut world);

    // 9. Verify creature spawned in ECS World
    let mut query = world.query::<(&Position, &BodySize)>();
    let results: Vec<_> = query.iter(&world).collect();

    assert_eq!(results.len(), 1, "Should spawn exactly one creature");

    let (pos, body) = results[0];
    assert_eq!(
        pos.x, 456.78,
        "Creature should spawn at correct X position"
    );
    assert_eq!(
        pos.y, 901.23,
        "Creature should spawn at correct Y position"
    );
    assert_eq!(body.length, 1.0, "Creature should have default body size");
}

#[test]
#[cfg(feature = "dev-tools")]
fn test_multiple_commands_integration() {
    // Test processing multiple spawn commands in sequence
    let commands = vec![
        Command::DevSpawnCreature {
            x: 10.0,
            y: 20.0,
            dna: None,
        },
        Command::DevSpawnCreature {
            x: 30.0,
            y: 40.0,
            dna: None,
        },
        Command::DevLoadTrial {
            template: "test".to_string(),
        },
        Command::DevSpawnCreature {
            x: 50.0,
            y: 60.0,
            dna: None,
        },
    ];

    // Create frames
    let mut all_frames = Vec::new();
    for cmd in &commands {
        let payload = rmp_serde::to_vec(cmd).unwrap();
        let len = payload.len() as u32;
        all_frames.extend_from_slice(&len.to_be_bytes());
        all_frames.extend_from_slice(&payload);
    }

    let (pipe_reader, mut pipe_writer) = os_pipe::pipe().unwrap();
    let (tx, rx) = mpsc::channel();

    // Reader thread processes all frames
    let reader_handle = std::thread::spawn(move || {
        use std::io::Read;
        let mut reader = pipe_reader;
        let mut buffer = Vec::new();

        while {
            let mut len_bytes = [0u8; 4];
            match reader.read_exact(&mut len_bytes) {
                Ok(_) => {
                    let frame_len = u32::from_be_bytes(len_bytes) as usize;
                    buffer.clear();
                    buffer.resize(frame_len, 0);
                    reader.read_exact(&mut buffer).unwrap();

                    let cmd: Command = rmp_serde::from_slice(&buffer).unwrap();
                    tx.send(cmd).unwrap();
                    true
                }
                Err(_) => false, // EOF
            }
        } {}
    });

    pipe_writer.write_all(&all_frames).unwrap();
    drop(pipe_writer);
    reader_handle.join().unwrap();

    // Process all commands
    let mut world = World::new();
    world.insert_resource(CommandReceiver(Arc::new(Mutex::new(rx))));
    world.insert_resource(speciate::simulation::creatures::systems::NextCreatureId::default());
    command_executor_system(&mut world);

    // Should spawn 3 creatures (trial loading is ignored for now)
    let mut query = world.query::<&Position>();
    let positions: Vec<_> = query.iter(&world).collect();

    assert_eq!(
        positions.len(),
        3,
        "Should spawn 3 creatures from 4 commands"
    );

    let xs: Vec<f32> = positions.iter().map(|p| p.x).collect();
    assert!(xs.contains(&10.0));
    assert!(xs.contains(&30.0));
    assert!(xs.contains(&50.0));
}
