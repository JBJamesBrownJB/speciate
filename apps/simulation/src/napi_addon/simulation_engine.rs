//! SimulationEngine - NAPI Bridge for Bevy ECS
//!
//! This is the core NAPI interface that replaces stdio IPC. It provides:
//! - Zero-copy buffer access via DoubleBuffer
//! - Direct function calls (no MessagePack serialization)
//! - Custom Bevy run loop (clean shutdown, no blocking)
//! - Panic handling (Rust panics don't crash Electron)
//! - Error boundaries (all methods return Result<()>)
//!
//! **Architecture:**
//! - Main thread: JavaScript calls NAPI methods
//! - Bevy thread: Runs simulation loop, writes to DoubleBuffer
//! - Atomic swap: Lock-free synchronization between threads

use napi_derive::napi;
use napi::bindgen_prelude::*;
use napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread::{self, JoinHandle};
use std::path::Path;
use std::time::{Duration, Instant};
use std::collections::VecDeque;
use std::panic::AssertUnwindSafe;
use crossbeam_channel;
use parking_lot::{Mutex, RwLock};
use log::error;

use crate::ipc::bridge::{DoubleBuffer, NapiApp, TelemetrySnapshot};
#[cfg(feature = "dev-tools")]
use crate::ipc::bridge::PerceptionDebugBuffer;
use crate::config::SaveStateConfig;
use crate::persistence::{SaveStateWorker, SaveType};

/// Target simulation tick rate (Hz)
///
/// This is the SINGLE SOURCE OF TRUTH for simulation speed.
/// All sleep intervals are calculated from this constant.
const TARGET_SIMULATION_HZ: f32 = 20.0;

use crate::ipc::SimCommand;


/// Initialize panic handler (MUST be called before creating SimulationEngine)
///
/// This ensures Rust panics are logged to stderr so Electron can see them.
/// Without this, panics would be silent and difficult to debug.
#[napi]
pub fn init_logger() {
    std::panic::set_hook(Box::new(|info| {
        eprintln!("🔥 RUST SIMULATION PANIC: {:?}", info);
    }));
}

/// SimulationEngine - The NAPI bridge to Bevy ECS
///
/// **Lifecycle:**
/// 1. `new()` - Create engine (allocates resources)
/// 2. `start(count, path, callback)` - Spawn Bevy thread
/// 3. `get_buffer()` - Read positions (zero-copy, called every frame)
/// 4. `spawn_creatures()`, `kill_all()` - Send commands
/// 5. Drop - Clean shutdown (waits for thread to exit)
///
/// **Thread Safety:**
/// - DoubleBuffer: Lock-free atomic swap (zero contention)
/// - Commands: Bounded channel (prevents overflow)
/// - Telemetry: ThreadsafeFunction (JavaScript callback)
#[napi]
pub struct SimulationEngine {
    buffer: Arc<Mutex<DoubleBuffer>>,
    running: Arc<AtomicBool>,
    command_sender: Option<crossbeam_channel::Sender<SimCommand>>,
    telemetry_cb: Option<ThreadsafeFunction<String>>,
    thread_handle: Option<JoinHandle<()>>,
    telemetry: Arc<RwLock<TelemetrySnapshot>>,
    buffer_creature_count: Arc<AtomicU64>,
    save_state_worker: Option<Arc<Mutex<SaveStateWorker>>>,
    save_state_config: SaveStateConfig,
    #[cfg(feature = "dev-tools")]
    perception_debug_buffer: Arc<Mutex<PerceptionDebugBuffer>>,
}

#[napi]
impl SimulationEngine {
    /// Create new SimulationEngine (does not start simulation yet)
    #[napi(constructor)]
    pub fn new() -> Self {
        // Read SaveStateConfig from environment (allows testing with fast intervals)
        let save_state_config = SaveStateConfig {
            enabled: std::env::var("SAVE_STATE_ENABLED")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(true),
            interval_secs: std::env::var("SAVE_STATE_INTERVAL_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(300),
            keep_last_n: std::env::var("SAVE_STATE_KEEP_LAST_N")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),
            save_dir: std::path::PathBuf::from("save-states"),
        };

        // Initialize SaveStateWorker if enabled (Arc<Mutex<>> for thread sharing)
        let save_state_worker = if save_state_config.enabled {
            Some(Arc::new(Mutex::new(SaveStateWorker::start(save_state_config.clone()))))
        } else {
            None
        };

        Self {
            buffer: Arc::new(Mutex::new(DoubleBuffer::new(0))),
            running: Arc::new(AtomicBool::new(false)),
            command_sender: None,
            telemetry_cb: None,
            thread_handle: None,
            telemetry: Arc::new(RwLock::new(TelemetrySnapshot::default())),
            buffer_creature_count: Arc::new(AtomicU64::new(0)),
            save_state_worker,
            save_state_config,
            #[cfg(feature = "dev-tools")]
            perception_debug_buffer: Arc::new(Mutex::new(PerceptionDebugBuffer::new())),
        }
    }

    /// Start simulation with initial creature count
    ///
    /// # Arguments
    /// * `count` - Initial creature count
    /// * `assets_path` - Path to config directory (must exist)
    /// * `callback` - JavaScript function for telemetry updates
    /// * `save_state_path` - Optional path to save state file (loads if exists)
    ///
    /// # Errors
    /// * Assets path does not exist
    /// * Simulation already running
    ///
    /// # Example (JavaScript)
    /// ```js
    /// const sim = new SimulationEngine();
    /// // Start fresh with 150K creatures
    /// sim.start(150000, '/path/to/assets', (telemetry) => { }, null);
    ///
    /// // Or resume from save state
    /// sim.start(0, '/path/to/assets', (telemetry) => { }, 'save-states/latest.msgpack');
    /// ```
    #[napi]
    pub fn start(
        &mut self,
        count: u32,
        assets_path: String,
        callback: JsFunction,
        save_state_path: Option<String>,
    ) -> Result<()> {
        // Validate assets_path exists
        let path = Path::new(&assets_path);
        if !path.exists() {
            return Err(Error::new(
                Status::InvalidArg,
                format!("Assets path does not exist: {}", assets_path),
            ));
        }

        // Check if already running
        if self.running.load(Ordering::SeqCst) {
            return Err(Error::new(
                Status::GenericFailure,
                "Simulation already running",
            ));
        }

        // Create telemetry callback (bounded queue to prevent memory issues)
        let tsfn: ThreadsafeFunction<String> = callback
            .create_threadsafe_function(10, |ctx| {
                Ok(vec![ctx.value])
            })?;
        self.telemetry_cb = Some(tsfn);

        // Bounded command queue (size: 128 - prevents overflow)
        let (tx, rx) = crossbeam_channel::bounded(128);
        self.command_sender = Some(tx);

        // Allocate double buffer (SoA layout: ID, X, Y, Rotation)
        // Pre-allocate for 200K creatures (6.4 MB, double-buffered = 12.8 MB total)
        // Supports Sprint 11 goal (150K-200K) with reasonable headroom
        const MAX_CREATURES: usize = 200_000;
        let size = MAX_CREATURES * 4;  // 800K f32s (ID, X, Y, Rot)
        *self.buffer.lock() = DoubleBuffer::new(size);

        self.running.store(true, Ordering::SeqCst);

        // Clone references for Bevy thread
        let buffer_ref = Arc::clone(&self.buffer);
        let running_ref = Arc::clone(&self.running);
        let telemetry_cb = self.telemetry_cb.clone();
        let telemetry_ref = Arc::clone(&self.telemetry);
        let buffer_count_ref = Arc::clone(&self.buffer_creature_count);
        let assets_path_owned = assets_path.clone();
        let save_state_config = self.save_state_config.clone();
        let save_state_worker_ref = self.save_state_worker.clone();
        #[cfg(feature = "dev-tools")]
        let perception_debug_buffer_ref = Arc::clone(&self.perception_debug_buffer);

        // Spawn Bevy thread with JoinHandle (for clean shutdown)
        let handle = thread::spawn(move || {
            // Panic handling wrapper (prevents Electron crash)
            let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
                // Set working directory to assets path
                if let Err(e) = std::env::set_current_dir(&assets_path_owned) {
                    eprintln!("❌ Failed to set working directory: {}", e);
                    return;
                }

                // Initialize Bevy App (with optional save state path)
                let mut app = NapiApp::new(rx, count, assets_path_owned.clone(), save_state_path);
                let delta_time = 1.0 / TARGET_SIMULATION_HZ;

                // Frame timing tracking for tick rate measurement
                let mut frame_times: VecDeque<Instant> = VecDeque::with_capacity(30);

                // Save state timing tracking
                let mut last_save = Instant::now();

                // Custom run loop (NOT App::run() which blocks forever)
                let mut tick = 0u64;
                while running_ref.load(Ordering::SeqCst) {
                    let frame_start = Instant::now();
                    // 1. Process commands from queue (non-blocking)
                    app.process_commands();

                    // 2. Update Bevy ECS (one frame)
                    app.update(delta_time);

                    // 3. Export positions to write buffer (SoA layout)
                    let exported_count = app.export_positions(&buffer_ref);
                    buffer_count_ref.store(exported_count as u64, Ordering::Relaxed);

                    // 4. Read hardware counters (dev-tools only)
                    #[cfg(feature = "dev-tools")]
                    app.read_hardware_counters();

                    // 4b. Export perception debug to buffer (dev-tools only)
                    #[cfg(feature = "dev-tools")]
                    app.export_perception_debug(&perception_debug_buffer_ref);

                    // 5. Record total tick timing (always enabled - negligible overhead ~1μs)
                    app.record_total_tick_timing(frame_start.elapsed().as_micros() as u64);

                    // 5b. Swap perception debug buffer (dev-tools only)
                    #[cfg(feature = "dev-tools")]
                    perception_debug_buffer_ref.lock().swap();

                    // 6. Swap position buffers after frame completes (lock-free)
                    buffer_ref.lock().swap();

                    // 6. Update frame timing history
                    frame_times.push_back(frame_start);
                    if frame_times.len() > 30 {
                        frame_times.pop_front();
                    }

                    // 7. Calculate actual tick rate from last 30 frames
                    let tick_rate_hz = if frame_times.len() >= 2 {
                        // Safe to unwrap: len() >= 2 guarantees both front() and back() exist
                        let elapsed = frame_times.back()
                            .expect("frame_times.back() exists (len >= 2)")
                            .duration_since(*frame_times.front()
                                .expect("frame_times.front() exists (len >= 2)"));
                        (frame_times.len() - 1) as f32 / elapsed.as_secs_f32()
                    } else {
                        30.0 // Default until enough samples
                    };

                    // 7. Update telemetry (once per second)
                    if tick % 30 == 0 {
                        let telemetry = app.get_telemetry(tick, tick_rate_hz);

                        // Write to shared state (for NAPI polling)
                        *telemetry_ref.write() = telemetry.clone();

                        // Also send via callback if registered (send FULL telemetry)
                        if let Some(ref tsfn) = telemetry_cb {
                            let telemetry_json = telemetry.to_json().unwrap_or_else(|e| {
                                eprintln!("⚠️  Failed to serialize telemetry: {}", e);
                                "{}".to_string()
                            });

                            let _ = tsfn.call(Ok(telemetry_json), ThreadsafeFunctionCallMode::NonBlocking);
                        }
                    }

                    // 8. Periodic save state (if enabled and interval elapsed)
                    if save_state_config.enabled {
                        let save_interval = Duration::from_secs(save_state_config.interval_secs);
                        if last_save.elapsed() >= save_interval {
                            match app.to_save_state() {
                                Ok(save_state) => {
                                    if let Some(ref worker_ref) = save_state_worker_ref {
                                        worker_ref.lock().save_world_state(save_state, SaveType::Periodic);
                                    }
                                    last_save = Instant::now();
                                }
                                Err(e) => {
                                    error!("Failed to create periodic save state: {}", e);
                                }
                            }
                        }
                    }

                    tick += 1;

                    // Sleep to maintain target tick rate (subtract work time from target)
                    let work_time = frame_start.elapsed();
                    let target_frame_time = Duration::from_secs_f32(1.0 / TARGET_SIMULATION_HZ);

                    if let Some(sleep_time) = target_frame_time.checked_sub(work_time) {
                        thread::sleep(sleep_time);
                    }
                    // If work took longer than target, skip sleep (running slow)
                }

                // Shutdown save (if enabled)
                if save_state_config.enabled {
                    if let Some(ref worker_ref) = save_state_worker_ref {
                        eprintln!("💾 Creating shutdown save state...");
                        match app.to_save_state() {
                            Ok(shutdown_save) => {
                                {
                                    let worker = worker_ref.lock();
                                    worker.save_world_state(shutdown_save, SaveType::Shutdown);
                                } // Lock released here

                                eprintln!("✅ Shutdown save state queued");

                                // CRITICAL: Give worker time to write before thread exits
                                eprintln!("⏳ Waiting for save state to write to disk...");
                                thread::sleep(std::time::Duration::from_millis(500));
                                eprintln!("✅ Save state worker completed");
                            }
                            Err(e) => {
                                error!("Failed to create shutdown save state: {}", e);
                                eprintln!("⚠️  Shutdown save state failed: {}", e);
                            }
                        }
                    }
                }
            }));

            // Handle panic
            if let Err(panic_info) = result {
                eprintln!("💥 Bevy thread panicked: {:?}", panic_info);

                // Send panic to telemetry
                if let Some(tsfn) = telemetry_cb {
                    let panic_json = serde_json::json!({
                        "type": "panic",
                        "message": format!("{:?}", panic_info)
                    })
                    .to_string();

                    let _ = tsfn.call(
                        Ok(panic_json),
                        ThreadsafeFunctionCallMode::NonBlocking,
                    );
                }
            }
        });

        self.thread_handle = Some(handle);

        Ok(())
    }

    /// Get read-only buffer for zero-copy access from JavaScript
    ///
    /// **Performance:** Instant (just returns Float32Array view)
    ///
    /// **Layout (SoA):**
    /// ```
    /// [ID₁, ID₂, ..., IDₙ, X₁, X₂, ..., Xₙ, Y₁, Y₂, ..., Yₙ, Rot₁, Rot₂, ..., Rotₙ]
    /// ```
    ///
    /// # Example (JavaScript)
    /// ```js
    /// const buffer = sim.getBuffer();
    /// const creatureCount = buffer.length / 4;
    /// const xOffset = creatureCount;
    /// const yOffset = creatureCount * 2;
    ///
    /// for (let i = 0; i < creatureCount; i++) {
    ///   sprite.x = buffer[xOffset + i];
    ///   sprite.y = buffer[yOffset + i];
    /// }
    /// ```
    #[napi]
    pub fn get_buffer(&self) -> Float32Array {
        let buffer = self.buffer.lock();
        let read_slice = buffer.get_read_slice();

        // Get actual creature count (updated by Bevy thread)
        let creature_count = self.buffer_creature_count.load(Ordering::Relaxed) as usize;

        // Calculate actual data size (4 f32s per creature: ID, X, Y, Rotation)
        let active_size = creature_count * 4;

        // Simple allocation pattern - let NAPI handle memory management
        // Complex Vec reuse patterns can interfere with NAPI's GC integration
        if active_size > 0 && active_size <= read_slice.len() {
            Float32Array::new(read_slice[0..active_size].to_vec())
        } else {
            Float32Array::new(vec![])
        }
    }

    /// Fill a JS-owned buffer with creature data (zero-allocation)
    ///
    /// **MEMORY FIX:** JS creates buffer once, Rust fills it every poll.
    /// This avoids the per-call Float32Array allocation that V8 doesn't GC properly.
    ///
    /// # Arguments
    /// * `buffer` - JS-owned Float32Array to fill (must be large enough)
    ///
    /// # Returns
    /// Number of creatures written (buffer layout: [IDs, Xs, Ys, Rotations])
    ///
    /// # Safety
    /// Uses as_mut() which is safe because JS polling is single-threaded and
    /// we only access the buffer during this synchronous call.
    #[napi]
    pub fn fill_buffer(&self, mut buffer: Float32Array) -> i32 {
        let src_buffer = self.buffer.lock();
        let read_slice = src_buffer.get_read_slice();

        let creature_count = self.buffer_creature_count.load(Ordering::Relaxed) as usize;
        let active_size = creature_count * 4;

        let dest = buffer.as_mut();
        if active_size == 0 || active_size > read_slice.len() || active_size > dest.len() {
            return 0;
        }

        dest[..active_size].copy_from_slice(&read_slice[..active_size]);
        creature_count as i32
    }

    /// Get target simulation tick rate (Hz)
    ///
    /// JavaScript should query this to calculate appropriate polling rates.
    /// Recommended: Poll at 2x this rate to ensure no frames are missed.
    #[napi]
    pub fn get_target_hz(&self) -> f32 {
        TARGET_SIMULATION_HZ
    }

    /// Resize buffer to accommodate new creature count
    ///
    /// **Use case:** If entity count changes significantly
    ///
    /// # Arguments
    /// * `new_count` - New creature count
    #[napi]
    pub fn resize_buffer(&mut self, new_count: u32) -> Result<()> {
        let new_size = (new_count * 4) as usize;
        *self.buffer.lock() = DoubleBuffer::new(new_size);
        Ok(())
    }

    /// Spawn creatures at random positions
    ///
    /// # Arguments
    /// * `count` - Number of creatures to spawn
    ///
    /// # Errors
    /// * Simulation not started
    /// * Command queue full (128 capacity)
    ///
    /// # Example (JavaScript)
    /// ```js
    /// sim.spawnCreatures(100); // Spawn 100 creatures
    /// ```
    #[napi]
    pub fn spawn_creatures(&self, count: u32) -> Result<()> {
        self.command_sender
            .as_ref()
            .ok_or(Error::new(
                Status::GenericFailure,
                "Simulation not started",
            ))?
            .try_send(SimCommand::Spawn(count))
            .map_err(|e| {
                Error::new(
                    Status::GenericFailure,
                    format!("Command queue full: {}", e),
                )
            })?;
        Ok(())
    }

    /// Spawn creature at specific position
    ///
    /// # Arguments
    /// * `x` - X coordinate
    /// * `y` - Y coordinate
    ///
    /// # Errors
    /// * Simulation not started
    /// * Command queue full
    #[napi]
    pub fn spawn_creature_at(&self, x: f64, y: f64) -> Result<()> {
        self.command_sender
            .as_ref()
            .ok_or(Error::new(
                Status::GenericFailure,
                "Simulation not started",
            ))?
            .try_send(SimCommand::SpawnAt { x: x as f32, y: y as f32 })
            .map_err(|e| {
                Error::new(
                    Status::GenericFailure,
                    format!("Command queue full: {}", e),
                )
            })?;
        Ok(())
    }

    /// Despawn all creatures
    ///
    /// # Errors
    /// * Simulation not started
    /// * Command queue full
    #[napi]
    pub fn kill_all(&self) -> Result<()> {
        self.command_sender
            .as_ref()
            .ok_or(Error::new(
                Status::GenericFailure,
                "Simulation not started",
            ))?
            .try_send(SimCommand::KillAll)
            .map_err(|e| {
                Error::new(
                    Status::GenericFailure,
                    format!("Command queue full: {}", e),
                )
            })?;
        Ok(())
    }

    /// Load trial configuration
    ///
    /// # Arguments
    /// * `template` - Trial template name (e.g., "flocking_test")
    #[napi]
    pub fn load_trial(&self, template: String) -> Result<()> {
        self.command_sender
            .as_ref()
            .ok_or(Error::new(
                Status::GenericFailure,
                "Simulation not started",
            ))?
            .try_send(SimCommand::LoadTrial { trial_name: template })
            .map_err(|e| {
                Error::new(
                    Status::GenericFailure,
                    format!("Command queue full: {}", e),
                )
            })?;
        Ok(())
    }

    /// Select a creature for perception debug visualization
    ///
    /// When a creature is selected, the simulation will include detailed
    /// perception data (position, range, neighbors) in telemetry updates.
    ///
    /// # Arguments
    /// * `creature_id` - The creature ID to select, or None to clear selection
    ///
    /// # Example (JavaScript)
    /// ```js
    /// // Select creature 123 for debug visualization
    /// sim.selectCreatureDebug(123);
    ///
    /// // Clear selection
    /// sim.selectCreatureDebug(null);
    /// ```
    #[napi]
    pub fn select_creature_debug(&self, creature_id: Option<u32>) -> Result<()> {
        self.command_sender
            .as_ref()
            .ok_or(Error::new(
                Status::GenericFailure,
                "Simulation not started",
            ))?
            .try_send(SimCommand::SelectCreatureDebug(creature_id))
            .map_err(|e| {
                Error::new(
                    Status::GenericFailure,
                    format!("Command queue full: {}", e),
                )
            })?;
        Ok(())
    }

    /// Get full telemetry snapshot (all 45+ metrics)
    ///
    /// **Performance:** 3-8µs per call (negligible overhead)
    ///
    /// # Returns
    /// JSON string with complete telemetry data:
    /// - tick, creatureCount (always included)
    /// - systemTimings (always included)
    /// - hardwareMetrics (dev-tools only)
    /// - parallelizationMetrics (dev-tools only)
    ///
    /// # Example (JavaScript)
    /// ```js
    /// const telemetryJson = sim.getTelemetry();
    /// const data = JSON.parse(telemetryJson);
    /// console.log('Tick:', data.tick);
    /// console.log('Creatures:', data.creatureCount);
    /// console.log('Movement system:', data.systemTimings.movementUs, 'μs');
    /// ```
    #[napi]
    pub fn get_telemetry(&self) -> Result<String> {
        let snapshot = self.telemetry.try_read()
            .ok_or_else(|| Error::new(
                Status::GenericFailure,
                "Telemetry lock poisoned",
            ))?;

        snapshot.to_json()
            .map_err(|e| Error::new(
                Status::GenericFailure,
                format!("Failed to serialize telemetry: {}", e),
            ))
    }

    /// Get current simulation tick
    ///
    /// # Returns
    /// Current tick number (0 if not started or error)
    ///
    /// # Example (JavaScript)
    /// ```js
    /// const tick = sim.getTick();
    /// console.log('Current tick:', tick);
    /// ```
    #[napi]
    pub fn get_tick(&self) -> i64 {
        self.telemetry.try_read()
            .map(|snapshot| snapshot.tick as i64)
            .unwrap_or(0)
    }

    /// Get simulation tick rate (Hz)
    ///
    /// # Returns
    /// Tick rate in Hz (default: 30.0)
    ///
    /// # Example (JavaScript)
    /// ```js
    /// const tickRate = sim.getTickRate();
    /// console.log('Tick rate:', tickRate, 'Hz');
    /// ```
    #[napi]
    pub fn get_tick_rate(&self) -> f32 {
        30.0  // Fixed for now, will be DNA-driven in future
    }

    /// Get current creature count
    ///
    /// # Returns
    /// Number of living creatures (0 if not started or error)
    ///
    /// # Example (JavaScript)
    /// ```js
    /// const count = sim.getCreatureCount();
    /// console.log('Creature count:', count);
    /// ```
    #[napi]
    pub fn get_creature_count(&self) -> i64 {
        self.telemetry.try_read()
            .map(|snapshot| snapshot.creature_count as i64)
            .unwrap_or(0)
    }

    /// Get the actual number of creatures in the current buffer
    ///
    /// This returns the exact count of creatures that were exported to the buffer
    /// on the last tick. Unlike getCreatureCount() which reads from telemetry
    /// (updated every 30 ticks), this is always current.
    ///
    /// # Returns
    /// Number of creatures in buffer (updated every tick)
    ///
    /// # Example (JavaScript)
    /// ```js
    /// const bufferCount = sim.getBufferCreatureCount();
    /// console.log('Buffer creature count:', bufferCount);
    /// ```
    #[napi]
    pub fn get_buffer_creature_count(&self) -> i64 {
        self.buffer_creature_count.load(Ordering::Relaxed) as i64
    }

    /// Get buffer capacity statistics
    ///
    /// Returns capacity, used count, and utilization percentage for monitoring
    ///
    /// # Returns
    /// JSON string with buffer stats: { capacity, used, utilizationPct }
    ///
    /// # Example (JavaScript)
    /// ```js
    /// const stats = JSON.parse(sim.getBufferStats());
    /// console.log(`Buffer: ${stats.used}/${stats.capacity} (${stats.utilizationPct}%)`);
    /// ```
    #[napi]
    pub fn get_buffer_stats(&self) -> String {
        let buffer = self.buffer.lock();
        let capacity = buffer.size() / 4;  // 4 f32s per creature
        let used = self.get_creature_count() as usize;
        let utilization_pct = if capacity > 0 {
            (used * 100) / capacity
        } else {
            0
        };

        serde_json::json!({
            "capacity": capacity,
            "used": used,
            "utilizationPct": utilization_pct,
        })
        .to_string()
    }

    /// Stop simulation gracefully (with optional shutdown save)
    ///
    /// **Note:** Also called automatically when SimulationEngine is dropped
    #[napi]
    pub fn stop(&mut self) -> Result<()> {
        self.running.store(false, Ordering::SeqCst);

        if let Some(handle) = self.thread_handle.take() {
            eprintln!("🛑 Waiting for Bevy thread to exit...");
            handle
                .join()
                .map_err(|e| {
                    Error::new(
                        Status::GenericFailure,
                        format!("Thread join failed: {:?}", e),
                    )
                })?;
            eprintln!("✅ Bevy thread stopped cleanly");
        }

        Ok(())
    }

    /// Shutdown simulation with final save state
    ///
    /// Sets the running flag to false, which triggers the thread to:
    /// 1. Exit the run loop
    /// 2. Create a shutdown save (if enabled, lines 293-301)
    /// 3. Clean exit
    ///
    /// Should be called before process exit for proper state preservation.
    #[napi]
    pub fn shutdown(&mut self) -> Result<()> {
        // Signal thread to stop (shutdown save happens in thread before exit)
        self.stop()
    }
}

/// Dev-tools only NAPI methods
///
/// These methods are only available when compiled with `--features dev-tools`.
/// They provide developer-facing debugging APIs that should NOT be shipped
/// in production builds.
#[cfg(feature = "dev-tools")]
#[napi]
impl SimulationEngine {
    /// Get perception debug buffer
    ///
    /// Returns Float32Array with perception debug data for selected creature.
    ///
    /// **Layout:**
    /// - [0]: has_data (1.0 = valid, 0.0 = no selection)
    /// - [1]: target_id
    /// - [2]: target_x
    /// - [3]: target_y
    /// - [4]: perception_range
    /// - [5]: neighbor_count
    /// - [6..6+64]: neighbor_ids
    /// - [6+64..6+128]: neighbor_xs
    /// - [6+128..6+192]: neighbor_ys
    ///
    /// # Example (JavaScript)
    /// ```js
    /// const debug = simulation.getPerceptionDebug();
    /// if (debug[0] > 0.5) { // has_data
    ///   const targetX = debug[2];
    ///   const targetY = debug[3];
    ///   const range = debug[4];
    ///   const neighborCount = debug[5];
    ///   // Draw circle at (targetX, targetY) with radius `range`
    ///   // Draw lines to neighbors at indices 6+i, 70+i, 134+i
    /// }
    /// ```
    #[napi]
    pub fn get_perception_debug(&self) -> Float32Array {
        let buffer = self.perception_debug_buffer.lock();
        let read_slice = buffer.get_read_slice();

        // Simple allocation pattern - let NAPI handle memory management
        Float32Array::new(read_slice.to_vec())
    }

    /// Fill a JS-owned buffer with perception debug data (zero-allocation)
    ///
    /// **MEMORY FIX:** JS creates buffer once, Rust fills it every poll.
    /// This avoids the per-call Float32Array allocation that V8 doesn't GC properly.
    ///
    /// # Arguments
    /// * `buffer` - JS-owned Float32Array to fill (must be 605+ elements)
    ///
    /// # Returns
    /// true if has_data flag is set (creature selected), false otherwise
    ///
    /// # Safety
    /// Uses as_mut() which is safe because JS polling is single-threaded and
    /// we only access the buffer during this synchronous call.
    #[napi]
    pub fn fill_perception_debug(&self, mut buffer: Float32Array) -> bool {
        let src_buffer = self.perception_debug_buffer.lock();
        let read_slice = src_buffer.get_read_slice();

        let dest = buffer.as_mut();
        if dest.len() < read_slice.len() {
            return false;
        }

        dest[..read_slice.len()].copy_from_slice(read_slice);
        dest[0] > 0.5 // has_data flag
    }
}

/// CRITICAL: Wait for thread to actually stop (prevents zombie threads)
///
/// This is called automatically when JavaScript releases the SimulationEngine.
/// Hot reload scenario:
/// 1. Electron renderer reloads
/// 2. SimulationEngine reference dropped
/// 3. Drop trait runs → thread.join() waits for clean exit
/// 4. New SimulationEngine can be created safely
impl Drop for SimulationEngine {
    fn drop(&mut self) {
        self.running.store(false, Ordering::SeqCst);

        if let Some(handle) = self.thread_handle.take() {
            eprintln!("🧹 Drop: Waiting for Bevy thread to exit...");
            if let Err(e) = handle.join() {
                eprintln!("❌ Bevy thread panicked during shutdown: {:?}", e);
            } else {
                eprintln!("✅ Bevy thread stopped cleanly (Drop)");
            }
        }
    }
}
