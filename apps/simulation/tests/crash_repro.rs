
    use speciate::ipc::bridge::{NapiApp, DoubleBuffer};
    use speciate::ipc::SimCommand;
    use std::sync::Arc;
    use parking_lot::Mutex;
    use std::time::{Duration, Instant};
    use std::thread;
    use crossbeam_channel::bounded;

    #[test]
    fn test_stress_spawn_crash() {
        // Setup similar to SimulationEngine::start
        let (tx, rx) = bounded(128);
        let buffer = Arc::new(Mutex::new(DoubleBuffer::new(200_000 * 4)));
        let running = Arc::new(std::sync::atomic::AtomicBool::new(true));
        
        let buffer_ref = buffer.clone();
        let running_ref = running.clone();
        
        let handle = thread::spawn(move || {
            let mut app = NapiApp::new(rx, 100, ".".to_string());
            let delta_time = 0.016;
            
            while running_ref.load(std::sync::atomic::Ordering::SeqCst) {
                app.process_commands();
                app.update(delta_time);
                app.export_positions(&buffer_ref);
                buffer_ref.lock().swap();
                
                // Simulate tick rate
                thread::sleep(Duration::from_millis(10));
            }
        });

        // Simulate "Load Trial" spam
        for i in 0..150 {
            println!("Load iteration {}", i);
            // Send spawn command (2500 creatures)
            tx.send(SimCommand::Spawn(2500)).unwrap();
            
            thread::sleep(Duration::from_millis(50));
            
            // Read buffer like JS would
            let buf = buffer.lock();
            let _slice = buf.get_read_slice();
            // Simulate copy
            let _vec = _slice.to_vec();
        }

        running.store(false, std::sync::atomic::Ordering::SeqCst);
        handle.join().unwrap();
    }
