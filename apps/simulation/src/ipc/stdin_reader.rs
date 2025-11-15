use std::io::{self, Read};
use std::sync::mpsc::Sender;
use std::thread;

use super::Command;

/// Spawn a background thread that reads length-prefixed MessagePack frames from stdin
/// and sends parsed commands to the provided channel.
///
/// Frame format: [4-byte BE length][MessagePack payload]
#[cfg(feature = "dev-tools")]
pub fn spawn_stdin_reader_thread(tx: Sender<Command>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let stdin = io::stdin();
        let mut handle = stdin.lock();
        let mut buffer = Vec::new();

        loop {
            match read_frame(&mut handle, &mut buffer) {
                Ok(command) => {
                    if let Err(e) = tx.send(command) {
                        eprintln!("[StdinReader] Failed to send command: {}", e);
                        break;
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                    // Stdin closed, exit gracefully
                    break;
                }
                Err(e) => {
                    eprintln!("[StdinReader] Error reading frame: {}", e);
                    // Continue reading - don't crash on single bad frame
                }
            }
        }
    })
}

/// Read a single length-prefixed MessagePack frame from the reader
fn read_frame<R: Read>(reader: &mut R, buffer: &mut Vec<u8>) -> io::Result<Command> {
    // Read 4-byte big-endian length prefix
    let mut len_bytes = [0u8; 4];
    reader.read_exact(&mut len_bytes)?;
    let frame_length = u32::from_be_bytes(len_bytes) as usize;

    // Validate frame length (prevent DoS from malicious huge frames)
    if frame_length > 1024 * 1024 {
        // 1MB max frame size
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Frame too large: {} bytes", frame_length),
        ));
    }

    // Read MessagePack payload
    buffer.clear();
    buffer.resize(frame_length, 0);
    reader.read_exact(buffer)?;

    // Deserialize command (accepts both map and array formats)
    Command::from_msgpack(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::sync::mpsc;

    fn create_frame(command: &Command) -> Vec<u8> {
        let payload = rmp_serde::to_vec(command).unwrap();
        let len = payload.len() as u32;
        let mut frame = len.to_be_bytes().to_vec();
        frame.extend_from_slice(&payload);
        frame
    }

    #[test]
    fn test_parse_length_prefixed_msgpack_frame() {
        let command = Command::DevSpawnCreature {
            x: 100.0,
            y: 200.0,
            dna: None,
        };

        let frame = create_frame(&command);
        let mut cursor = Cursor::new(frame);
        let mut buffer = Vec::new();

        let parsed = read_frame(&mut cursor, &mut buffer).unwrap();

        match parsed {
            Command::DevSpawnCreature { x, y, dna } => {
                assert_eq!(x, 100.0);
                assert_eq!(y, 200.0);
                assert!(dna.is_none());
            }
            _ => panic!("Expected DevSpawnCreature"),
        }
    }

    #[test]
    fn test_handle_partial_frames() {
        let command1 = Command::DevSpawnCreature {
            x: 10.0,
            y: 20.0,
            dna: None,
        };
        let command2 = Command::DevLoadTrial {
            template: "test".to_string(),
        };

        // Create two frames concatenated
        let mut data = create_frame(&command1);
        data.extend_from_slice(&create_frame(&command2));

        let mut cursor = Cursor::new(data);
        let mut buffer = Vec::new();

        // Read first frame
        let parsed1 = read_frame(&mut cursor, &mut buffer).unwrap();
        match parsed1 {
            Command::DevSpawnCreature { x, y, .. } => {
                assert_eq!(x, 10.0);
                assert_eq!(y, 20.0);
            }
            _ => panic!("Expected DevSpawnCreature"),
        }

        // Read second frame
        let parsed2 = read_frame(&mut cursor, &mut buffer).unwrap();
        match parsed2 {
            Command::DevLoadTrial { template } => {
                assert_eq!(template, "test");
            }
            _ => panic!("Expected DevLoadTrial"),
        }
    }

    #[test]
    fn test_invalid_length_prefix_returns_error() {
        // Frame with only 2 bytes (incomplete length prefix)
        let data = vec![0x00, 0x01];
        let mut cursor = Cursor::new(data);
        let mut buffer = Vec::new();

        let result = read_frame(&mut cursor, &mut buffer);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::UnexpectedEof);
    }

    #[test]
    fn test_frame_too_large_returns_error() {
        // Create a frame claiming to be 2MB (over 1MB limit)
        let len = 2 * 1024 * 1024u32;
        let mut data = len.to_be_bytes().to_vec();
        data.extend_from_slice(&vec![0u8; 100]); // Add some payload

        let mut cursor = Cursor::new(data);
        let mut buffer = Vec::new();

        let result = read_frame(&mut cursor, &mut buffer);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::InvalidData);
    }

    #[test]
    fn test_invalid_msgpack_returns_error() {
        // Create a frame with invalid MessagePack data
        let invalid_msgpack = vec![0xFF, 0xFF, 0xFF];
        let len = invalid_msgpack.len() as u32;
        let mut frame = len.to_be_bytes().to_vec();
        frame.extend_from_slice(&invalid_msgpack);

        let mut cursor = Cursor::new(frame);
        let mut buffer = Vec::new();

        let result = read_frame(&mut cursor, &mut buffer);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::InvalidData);
    }

    #[test]
    fn test_thread_sends_parsed_commands_to_queue() {
        let (tx, rx) = mpsc::channel();

        let command = Command::DevSpawnCreature {
            x: 50.0,
            y: 75.0,
            dna: None,
        };

        // Create test data
        let frame = create_frame(&command);

        // Simulate stdin by creating a thread that writes to a pipe
        let (pipe_reader, mut pipe_writer) = os_pipe::pipe().unwrap();

        // Spawn reader thread with the pipe as "stdin"
        let handle = thread::spawn(move || {
            let mut buffer = Vec::new();
            let mut reader = pipe_reader;

            match read_frame(&mut reader, &mut buffer) {
                Ok(cmd) => {
                    let _ = tx.send(cmd);
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                }
            }
        });

        // Write frame to pipe
        use std::io::Write;
        pipe_writer.write_all(&frame).unwrap();
        drop(pipe_writer); // Close writer to signal EOF

        // Wait for thread to process
        handle.join().unwrap();

        // Verify command was received
        let received = rx.recv_timeout(std::time::Duration::from_secs(1)).unwrap();
        match received {
            Command::DevSpawnCreature { x, y, .. } => {
                assert_eq!(x, 50.0);
                assert_eq!(y, 75.0);
            }
            _ => panic!("Expected DevSpawnCreature"),
        }
    }
}
