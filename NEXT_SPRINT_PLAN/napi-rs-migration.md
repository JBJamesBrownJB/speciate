SPRINT 13 REBOOT: NAPI-RS Architecture Migration
Context: The current stdio IPC architecture hits a hard performance ceiling at ~27.5k entities. Objective: Pivot apps/simulation to a NAPI-RS Node.js Native Addon. This enables Zero-Copy Shared Memory for positions (solving the bottleneck) and replaces the IPC Command Queue with direct function calls.

Phase 0:

Ensure we have baseline metrics snapshots from before we do the migration.
Name the snapshot pre-napi-re-migration.json - so our team can compare before after and validate the benefits or issues.
PROMPT THE HUMAN USER TO MAKE SURE THIS HAPPENS!!!

Phase 1: Rust Infrastructure (apps/simulation)
1.1 Configuration & Build Setup
File: apps/simulation/Cargo.toml Action: Change crate-type to ["cdylib"]. Action: Add dependencies:
[dependencies] napi = { version = "2.12", features = ["default", "napi4", "serde-json"] } napi-derive = "2.12" serde = { version = "1.0", features = ["derive"] } serde_json = "1.0" lazy_static = "1.4" crossbeam-channel = "0.5"
Keep existing bevy, perfcnt, etc.
[build-dependencies] napi-build = "2.0"
File: apps/simulation/build.rs Action: Create/Update to setup NAPI:
fn main() { napi_build::setup(); }
1.2 The Simulation Engine (Entry Point)
File: Rename apps/simulation/src/main.rs to apps/simulation/src/lib.rs. Action: Implement the SimulationEngine class. Include the Drop trait for clean thread shutdown and assets_path for config loading.
use napi_derive::napi; use napi::bindgen_prelude::; use bevy::prelude::; use std::sync::{Arc, RwLock}; use std::sync::atomic::{AtomicBool, Ordering}; use std::thread;
// Internal enum for commands pub enum SimCommand { Spawn(u32), KillAll, LoadTrial(String), }
#[napi] pub fn init_logger() { // CRITICAL: Forward Rust panics to console so Electron sees them std::panic::set_hook(Box::new(|info| { eprintln!("RUST SIMULATION PANIC: {:?}", info); })); }
#[napi] pub struct SimulationEngine { // The "Firehose" Buffer: [ID, X, Y, Rot, ID, X, Y, Rot...] pub buffer: Arc<RwLock<Vec<f32>>>, pub running: Arc<AtomicBool>, // Command Queue: Send commands (Spawn, Kill) to the Bevy thread pub command_sender: Option<crossbeam_channel::Sender<SimCommand>>, pub telemetry_cb: Option<ThreadsafeFunction<String, ErrorStrategy::Fatal>>, }
#[napi] impl SimulationEngine { #[napi(constructor)] pub fn new() -> Self { Self { buffer: Arc::new(RwLock::new(Vec::new())), running: Arc::new(AtomicBool::new(false)), command_sender: None, telemetry_cb: None, } }
#[napi]
pub fn start(&mut self, count: u32, assets_path: String, callback: JsFunction) -> Result<()> {
    let tsfn = callback.create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))?;
    self.telemetry_cb = Some(tsfn);

    // Channel for commands
    let (tx, rx) = crossbeam_channel::unbounded();
    self.command_sender = Some(tx);

    // Allocate Buffer
    let size = (count * 4) as usize;
    { *self.buffer.write().unwrap() = vec![0.0; size]; }
    self.running.store(true, Ordering::SeqCst);

    // Spawn Bevy Thread
    let buffer_ref = self.buffer.clone();
    let running_ref = self.running.clone();

    thread::spawn(move || {
        // CRITICAL: Set Config Root using assets_path to avoid "File Not Found"
        // Init Bevy App (Headless)
        // Add 'CommandReceiverSystem' (reads rx channel)
        // Add 'ExportPositionSystem' (writes buffer)
        // Add 'TelemetrySystem'
        // Run loop while running_ref is true
    });
    Ok(())
}

#[napi]
pub fn get_buffer(&self) -> Float32Array {
    let guard = self.buffer.read().unwrap();
    Float32Array::new(guard.as_slice())
}

// --- EXPOSED COMMANDS ---

#[napi]
pub fn spawn_creatures(&self, count: u32) {
    if let Some(tx) = &self.command_sender {
        let _ = tx.send(SimCommand::Spawn(count));
    }
}

#[napi]
pub fn kill_all(&self) {
    if let Some(tx) = &self.command_sender {
        let _ = tx.send(SimCommand::KillAll);
    }
}

}
// Prevent Zombie Threads on reload impl Drop for SimulationEngine { fn drop(&mut self) { self.running.store(false, Ordering::SeqCst); println!("Simulation Dropped: Stopping Background Thread"); } }
1.3 The Bevy Systems
export_positions_system: Writes to buffer. telemetry_system: Sends JSON to telemetry_cb. command_receiver_system: Reads SimCommand from the rx channel and executes game logic (spawning/despawning).

Phase 2: Electron Integration (apps/portal)
2.1 Build Config
File: apps/portal/package.json Action: Add asarUnpack for *.node files to avoid load errors.
"build": { "asarUnpack": [ "**/*.node" ], "extraResources": [ { "from": "../simulation/index.node", "to": "simulation.node" } ] }
2.2 The Loader
File: apps/portal/electron/main.cjs Action: Load the native module with error handling. Pass process.resourcesPath (prod) or process.cwd() (dev) to start().
// In sim.start() call: const assetsPath = app.isPackaged ? process.resourcesPath : path.join(__dirname, '../../..'); sim.start(150000, assetsPath, (telemetry) => { ... });

Phase 3: Frontend Logic (apps/portal/src)
3.1 Rendering (apps/portal/src/rendering/PixiApp.ts)
Action: Switch from ipc.on to ticker.add. Optimization: Use a "Sprite Pool" pattern. Do not create new Sprite() inside the loop. Create 150k hidden sprites at startup and toggle visibility.
const buffer = window.simulation.getBuffer(); app.ticker.add(() => { let i = 0; while(i < activeCount) { // Direct memory read spritePool[i].x = buffer[offset+1]; spritePool[i].y = buffer[offset+2]; i++; } });
3.2 Controlling
File: HUDManager.ts (or wherever buttons live) Action: Instead of ipcRenderer.send('spawn'), call window.simulation.spawnCreatures(100).

Phase 4: Dev-UI & Cleanup
Objective: Keep HardwareMetricsPanel working. Action: Ensure Rust serializes HardwareStats to JSON and Electron relays the telemetry-update event. Cleanup: Delete the entire apps/simulation/src/ipc folder to prevent accidental imports of legacy code.

Definition of Done
[ ] npm run build in simulation creates index.node.
[ ] Electron loads without errors (Zombie thread issue fixed via Drop trait).
[ ] Portal: Sprites move (Shared Memory reading).
[ ] Portal: "Spawn" button works (Direct NAPI Call).
[ ] Dev-UI: Graphs update (JSON Callback).
[ ] Validation: Config files load correctly in both Dev and Prod builds.

