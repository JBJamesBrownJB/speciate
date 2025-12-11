Spec-Driven Development Implementation Plan
🎯 Objective
Establish a "Source of Truth" testing framework that allows:
Rapid Automated Testing: Headless cargo test runs at maximum CPU speed.
Visual Verification: Interactive cargo run mode to watch tests pass/fail with debug gizmos.
Dynamic Configuration: Runtime tweaking of simulation constants without recompiling.
Config editing through Dev-UI: Runtime tweaking of config values through Dev-UI
🏗️ Phase 1: Configuration Extraction
Goal: Decouple hardcoded constants from the binary to allow runtime tuning and trial-specific overrides.
1.1 Architecture
Location: assets/config/
Format: TOML
Files:
physics.toml: Drag, gravity, impulse multipliers.
biology.toml: Metabolism rates, vision ranges, size/mass ratios.
world.toml: Map bounds, grid cell sizes.
1.2 Implementation Details
Crate: Use bevy_common_assets (or toml + serde) to deserializing TOML directly into Bevy Resources (Res<PhysicsConfig>, Res<BiologyConfig>).
Hot Reloading: Enable asset watching so tweaking a value in physics.toml updates the running simulation immediately.
📂 Phase 2: The Spec Architecture
Goal: Create the rigid folder structure and data formats for defining "Truth".
2.1 Folder Structure (root/specs/)
specs/
├── .cursorrules                 # AI Protection Rules (Source of Truth)
├── behavior/                    # Theme: AI Behaviors
│   ├── seek_food.toml           # Scenario Data
│   └── seek_food.rs             # Assertion Logic
├── physics/                     # Theme: Physics Engines
│   ├── collision_bounce.toml
│   └── collision_bounce.rs
└── performance/                 # Theme: Load Testing
    ├── crowd_10k.toml
    └── crowd_10k.rs

2.2 The Spec Schema (.toml)
Defines the Initial State of the world.
[meta]
name = "Seek Food Basic"
description = "Verify a critter moves towards visible food."
timeout_ticks = 500
seed = 123456  # Deterministic RNG seed

[[entities]]
type = "Critter"
pos = [-10.0, 0.0]
components = { "Vision" = { range = 20.0 } }

[[entities]]
type = "Food"
pos = [10.0, 0.0]

2.3 The Assertion Logic (.rs)
Defines Success Conditions.
// A closure or system that returns TrialResult::Pass/Fail/Running
fn check_seek_food(world: &World) -> TrialResult {
    let critter = world.entity(critter_id);
    let food = world.entity(food_id);
    if critter.distance(food) < 1.0 {
        return TrialResult::Pass;
    }
    TrialResult::Running
}

🎬 Phase 3: The Trial Director System
Goal: A Bevy System capable of managing the lifecycle of a test inside the game loop.
3.1 The TrialDirector Resource
State Machine: Idle -> Loading -> Resetting -> Running -> Finished.
Responsibility:
Intercept: Listens for DevLoadSpec("name") IPC commands.
Hard Reset: Calls world.clear_entities() and resets all Resources (Time, PhysicsConfig) to defaults.
Seed RNG: Re-initializes ChaCha8Rng with the TOML seed.
Spawn: Instantiates entities from the TOML.
Monitor: Runs the specific assertion logic every tick.
Report: Sends SpecResult event back to Dev UI via IPC.
3.2 The ResetOnTrial Trait
To fix the "Dirty World" problem, all simulation Resources must implement this trait:
pub trait ResetOnTrial {
    fn reset(&mut self);
}

The Director iterates registered resources and calls reset() before every trial.
⚡ Phase 4: Automated Test Runner
Goal: Run specs at 1000+ ticks/second without graphics.
4.1 Headless Entry Point
Create a new binary target or test harness (tests/spec_runner.rs) that:
Initializes Bevy with MinimalPlugins (No Render, No Audio).
Sets the App Runner to a custom loop that ignores VSync.
Iterates through all TOML files in specs/.
Runs each trial until Success or Timeout.
Exits with code 0 (Pass) or 1 (Fail).
🎨 Phase 5: Visual Verification & Gizmos
Goal: Allow humans to "see" the math.
5.1 The ForceDebug Component
Add a component to critters that stores the raw vectors calculated this frame:
#[derive(Component)]
pub struct ForceDebug {
    pub seek_vector: Vec2,
    pub flee_vector: Vec2,
    pub separation_vector: Vec2,
    pub net_force: Vec2,
}



