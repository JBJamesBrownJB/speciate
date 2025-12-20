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
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::thread::{self, JoinHandle};
use std::path::Path;
use std::time::Duration;
use std::panic::AssertUnwindSafe;
use crossbeam_channel;
use parking_lot::{Mutex, RwLock};
use log::error;

use crate::ipc::bridge::{DoubleBuffer, NapiApp, TelemetrySnapshot};
#[cfg(feature = "dev-tools")]
use crate::ipc::bridge::PerceptionDebugBuffer;
use crate::config::SaveStateConfig;
use crate::persistence::{SaveStateWorker, SaveType};
use crate::simulation::TickController;

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
    paused: Arc<AtomicBool>,
    time_scale: Arc<AtomicU32>,
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
            paused: Arc::new(AtomicBool::new(false)),
            time_scale: Arc::new(AtomicU32::new(1.0_f32.to_bits())),
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

        // Allocate double buffer (SoA layout: ID, X, Y, Rotation, Size)
        // Pre-allocate for 500K creatures (20 MB, double-buffered = 40 MB total)
        // Sprint 20 upgrade - performance allows larger populations
        const MAX_CREATURES: usize = 500_000;
        let size = MAX_CREATURES * 5;  // 2.5M f32s (ID, X, Y, Rot, Size)
        *self.buffer.lock() = DoubleBuffer::new(size);

        self.running.store(true, Ordering::SeqCst);

        // Clone references for Bevy thread
        let buffer_ref = Arc::clone(&self.buffer);
        let running_ref = Arc::clone(&self.running);
        let paused_ref = Arc::clone(&self.paused);
        let time_scale_ref = Arc::clone(&self.time_scale);
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
                app.set_paused_flag(Arc::clone(&paused_ref));
                app.set_time_scale_flag(Arc::clone(&time_scale_ref));

                // Initialize tick controller (accumulator pattern for timing)
                let mut tick_controller = TickController::new();

                // Save state timing tracking
                let mut last_save = std::time::Instant::now();

                // Custom run loop using accumulator pattern (industry standard)
                // See: "Fix Your Timestep" by Glenn Fiedler
                let mut tick = 0u64;
                while running_ref.load(Ordering::SeqCst) {
                    // 1. Process commands from queue (non-blocking)
                    // ALWAYS process commands even when paused (to receive unpause/time scale)
                    app.process_commands();

                    // 2. Update time scale from atomic (set by SetTimeScale command)
                    let scale = f32::from_bits(time_scale_ref.load(Ordering::SeqCst));
                    tick_controller.set_time_scale(scale);

                    // 3. Check if paused - reset accumulator and sleep
                    let is_paused = paused_ref.load(Ordering::SeqCst);
                    if is_paused {
                        tick_controller.reset(); // Prevent catch-up burst when unpausing
                        thread::sleep(Duration::from_millis(50)); // Don't spin while paused
                        continue;
                    }

                    // 4. Run simulation tick(s) via accumulator pattern
                    // May run 0, 1, or multiple ticks depending on accumulated time
                    let frame_start = std::time::Instant::now();
                    let metrics = tick_controller.tick(|dt| {
                        app.update(dt);
                    });

                    // 5. Only export/swap if we ran at least one tick
                    if metrics.ticks_this_frame > 0 {
                        // Export positions to write buffer (SoA layout)
                        let export_start = std::time::Instant::now();
                        let exported_count = app.export_positions(&buffer_ref);
                        app.record_export_positions_timing(export_start.elapsed().as_micros() as u64);

                        // Read hardware counters (dev-tools only)
                        #[cfg(feature = "dev-tools")]
                        app.read_hardware_counters();

                        // Export perception debug to buffer (dev-tools only)
                        #[cfg(feature = "dev-tools")]
                        app.export_perception_debug(&perception_debug_buffer_ref);

                        // Record total tick timing
                        app.record_total_tick_timing(frame_start.elapsed().as_micros() as u64);

                        // Swap perception debug buffer (dev-tools only)
                        #[cfg(feature = "dev-tools")]
                        perception_debug_buffer_ref.lock().swap();

                        // Swap position buffers after frame completes (lock-free)
                        buffer_ref.lock().swap();

                        // Store count AFTER swap with Release ordering
                        // This ensures JS polling sees consistent count + buffer data
                        // (fixes race condition where poll reads new count but old buffer)
                        buffer_count_ref.store(exported_count as u64, Ordering::Release);

                        tick += metrics.ticks_this_frame as u64;

                        // Calculate tick rate based on time scale
                        // At 1x: 20Hz, at 2x: 40Hz effective, etc.
                        let tick_rate_hz = TARGET_SIMULATION_HZ * scale;

                        // Update telemetry periodically (every ~0.5 seconds at 22Hz)
                        if tick % 11 == 0 {
                            let telemetry = app.get_telemetry(tick, tick_rate_hz);

                            // Write to shared state (for NAPI polling)
                            *telemetry_ref.write() = telemetry.clone();

                            // Also send via callback if registered
                            if let Some(ref tsfn) = telemetry_cb {
                                let telemetry_json = telemetry.to_json().unwrap_or_else(|e| {
                                    eprintln!("⚠️  Failed to serialize telemetry: {}", e);
                                    "{}".to_string()
                                });

                                let _ = tsfn.call(Ok(telemetry_json), ThreadsafeFunctionCallMode::NonBlocking);
                            }
                        }

                        // Periodic save state (if enabled and interval elapsed)
                        if save_state_config.enabled {
                            let save_interval = Duration::from_secs(save_state_config.interval_secs);
                            if last_save.elapsed() >= save_interval {
                                match app.to_save_state() {
                                    Ok(save_state) => {
                                        if let Some(ref worker_ref) = save_state_worker_ref {
                                            worker_ref.lock().save_world_state(save_state, SaveType::Periodic);
                                        }
                                        last_save = std::time::Instant::now();
                                    }
                                    Err(e) => {
                                        error!("Failed to create periodic save state: {}", e);
                                    }
                                }
                            }
                        }
                    }

                    // Log if we dropped time (for debugging performance issues)
                    if metrics.time_dropped > 0.01 {
                        eprintln!("⚠️  Dropped {:.1}ms of simulation time (overrun)", metrics.time_dropped * 1000.0);
                    }
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
    /// [ID₁...IDₙ, X₁...Xₙ, Y₁...Yₙ, Rot₁...Rotₙ, Size₁...Sizeₙ]
    /// ```
    ///
    /// # Example (JavaScript)
    /// ```js
    /// const buffer = sim.getBuffer();
    /// const creatureCount = buffer.length / 5;
    /// const xOffset = creatureCount;
    /// const yOffset = creatureCount * 2;
    /// const rotOffset = creatureCount * 3;
    /// const sizeOffset = creatureCount * 4;
    ///
    /// for (let i = 0; i < creatureCount; i++) {
    ///   sprite.x = buffer[xOffset + i];
    ///   sprite.y = buffer[yOffset + i];
    ///   sprite.scale = buffer[sizeOffset + i];
    /// }
    /// ```
    #[napi]
    pub fn get_buffer(&self) -> Float32Array {
        let buffer = self.buffer.lock();
        let read_slice = buffer.get_read_slice();

        // Get actual creature count (updated by Bevy thread)
        // Use Acquire to synchronize with Release store after buffer swap
        let creature_count = self.buffer_creature_count.load(Ordering::Acquire) as usize;

        // Calculate actual data size (5 f32s per creature: ID, X, Y, Rotation, Size)
        let active_size = creature_count * 5;

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

        // Use Acquire to synchronize with Release store after buffer swap
        let creature_count = self.buffer_creature_count.load(Ordering::Acquire) as usize;
        let active_size = creature_count * 5; // 5 f32s per creature: ID, X, Y, Rot, Size

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

    /// Spawn creature at specific position with optional DNA
    ///
    /// # Arguments
    /// * `x` - X coordinate
    /// * `y` - Y coordinate
    /// * `dna_size_gene` - Optional size gene (0.0-1.0)
    /// * `dna_fov_gene` - Optional FOV gene (0.0-1.0)
    ///
    /// # Errors
    /// * Simulation not started
    /// * Command queue full
    #[napi]
    pub fn spawn_creature_at(&self, x: f64, y: f64, dna_size_gene: Option<f64>, dna_fov_gene: Option<f64>) -> Result<()> {
        use crate::simulation::creatures::dna::Dna;

        let dna = match (dna_size_gene, dna_fov_gene) {
            (Some(size), Some(fov)) => Some(Dna::new(size as f32, fov as f32)),
            _ => None,
        };

        self.command_sender
            .as_ref()
            .ok_or(Error::new(
                Status::GenericFailure,
                "Simulation not started",
            ))?
            .try_send(SimCommand::SpawnAt { x: x as f32, y: y as f32, dna })
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

    /// Set simulation pause state
    ///
    /// # Arguments
    /// * `paused` - true to pause, false to resume
    ///
    /// # Errors
    /// * Simulation not started
    /// * Command queue full
    #[napi]
    pub fn set_paused(&self, paused: bool) -> Result<()> {
        self.command_sender
            .as_ref()
            .ok_or(Error::new(
                Status::GenericFailure,
                "Simulation not started",
            ))?
            .try_send(SimCommand::SetPaused(paused))
            .map_err(|e| {
                Error::new(
                    Status::GenericFailure,
                    format!("Command queue full: {}", e),
                )
            })?;
        Ok(())
    }

    /// Get current pause state
    #[napi]
    pub fn is_paused(&self) -> bool {
        self.paused.load(Ordering::SeqCst)
    }

    /// Set simulation time scale
    ///
    /// # Arguments
    /// * `scale` - Time multiplier (1.0 = normal, 2.0 = 2x speed, 0.5 = half speed)
    ///
    /// # Errors
    /// * Simulation not started
    /// * Command queue full
    #[napi]
    pub fn set_time_scale(&self, scale: f64) -> Result<()> {
        self.command_sender
            .as_ref()
            .ok_or(Error::new(
                Status::GenericFailure,
                "Simulation not started",
            ))?
            .try_send(SimCommand::SetTimeScale(scale as f32))
            .map_err(|e| {
                Error::new(
                    Status::GenericFailure,
                    format!("Command queue full: {}", e),
                )
            })?;
        Ok(())
    }

    /// Get current time scale
    #[napi]
    pub fn get_time_scale(&self) -> f64 {
        f32::from_bits(self.time_scale.load(Ordering::SeqCst)) as f64
    }

    /// Set viewport bounds for culling
    ///
    /// When enabled, export_positions() only sends creatures within these bounds.
    /// The frontend uses ID-based interpolation, so creatures can enter/leave
    /// the viewport without causing visual artifacts.
    ///
    /// # Arguments
    /// * `min_x` - Left edge of viewport in world units
    /// * `min_y` - Bottom edge of viewport in world units
    /// * `max_x` - Right edge of viewport in world units
    /// * `max_y` - Top edge of viewport in world units
    /// * `margin` - Extra padding around viewport (prevents pop-in at edges)
    ///
    /// # Errors
    /// * Simulation not started
    /// * Command queue full
    #[napi]
    pub fn set_viewport_bounds(&self, min_x: f64, min_y: f64, max_x: f64, max_y: f64, margin: f64) -> Result<()> {
        self.command_sender
            .as_ref()
            .ok_or(Error::new(
                Status::GenericFailure,
                "Simulation not started",
            ))?
            .try_send(SimCommand::SetViewportBounds {
                min_x: min_x as f32,
                min_y: min_y as f32,
                max_x: max_x as f32,
                max_y: max_y as f32,
                margin: margin as f32,
            })
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
    /// * `randomize_dna` - If true, each creature gets unique random DNA
    /// * `dna_size_gene` - Optional size gene (0.0-1.0, used when randomize_dna is false)
    /// * `dna_fov_gene` - Optional FOV gene (0.0-1.0, used when randomize_dna is false)
    #[napi]
    pub fn load_trial(&self, template: String, randomize_dna: bool, dna_size_gene: Option<f64>, dna_fov_gene: Option<f64>) -> Result<()> {
        use crate::simulation::creatures::dna::Dna;

        let dna = match (dna_size_gene, dna_fov_gene) {
            (Some(size), Some(fov)) => Some(Dna::new(size as f32, fov as f32)),
            _ => None,
        };

        self.command_sender
            .as_ref()
            .ok_or(Error::new(
                Status::GenericFailure,
                "Simulation not started",
            ))?
            .try_send(SimCommand::LoadTrial { trial_name: template, randomize_dna, dna })
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
        self.buffer_creature_count.load(Ordering::Acquire) as i64
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
        let capacity = buffer.size() / 5;  // 5 f32s per creature: ID, X, Y, Rot, Size
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

        // Wait for Bevy thread FIRST - it creates the shutdown save before exiting
        if let Some(handle) = self.thread_handle.take() {
            eprintln!("🛑 Waiting for Bevy thread to exit (includes shutdown save)...");
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

        // THEN shutdown save state worker (after Bevy thread has queued its shutdown save)
        if let Some(ref worker) = self.save_state_worker {
            eprintln!("🛑 Shutting down SaveStateWorker...");
            worker.lock().shutdown();
            eprintln!("✅ SaveStateWorker stopped");
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
    /// See `ipc/bridge/perception_debug_buffer.rs` for canonical buffer layout.
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

    /// Returns the required buffer size for perception debug data.
    /// Use this to allocate the correct buffer size in JS.
    #[napi]
    pub fn get_perception_debug_buffer_size(&self) -> u32 {
        use crate::ipc::bridge::perception_debug_buffer::BUFFER_SIZE;
        BUFFER_SIZE as u32
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

        // Wait for Bevy thread FIRST - it creates the shutdown save before exiting
        if let Some(handle) = self.thread_handle.take() {
            eprintln!("🧹 Drop: Waiting for Bevy thread to exit (includes shutdown save)...");
            if let Err(e) = handle.join() {
                eprintln!("❌ Bevy thread panicked during shutdown: {:?}", e);
            } else {
                eprintln!("✅ Bevy thread stopped cleanly (Drop)");
            }
        }

        // THEN shutdown save state worker (after Bevy thread has queued its shutdown save)
        if let Some(ref worker) = self.save_state_worker {
            eprintln!("🧹 Drop: Shutting down SaveStateWorker...");
            worker.lock().shutdown();
        }
    }
}
