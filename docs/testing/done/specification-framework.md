Spec-Driven Development Implementation Plan
🎯 Objective
Establish a "Source of Truth" testing framework that allows:
Rapid Automated Testing: Headless cargo test runs at maximum CPU speed.
Visual Verification: Interactive cargo run mode to watch tests pass/fail with debug gizmos.

📂 Phase 1: The Spec Architecture
Goal: Create the rigid folder structure and data formats for defining "Truth".
1.1 Folder Structure (root/specs/)
specs/
├── behavior/                    # Theme: AI Behaviors
│   ├── seek-food.toml
│   ├── crowd-navigation.toml
│   └── competing-friends.toml
├── physics/                     # Theme: Physics/Collision
│   └── collision-bounce.toml
└── performance/                 # Theme: Load Testing
    ├── many-wanderers-dense.toml
    └── cycling-brain-stress.toml

1.2 The Spec Schema (.toml)
Defines the Initial State and Success Conditions.

[meta]
name = "Seek Food Basic"
description = "Verify a critter moves towards visible food."
timeout_ticks = 500
seed = 123456  # Deterministic RNG seed

# Optional: Parameterized testing - run spec multiple times with varying values
[variants]
crit_size = { min = 0.5, max = 2.0, steps = 10 }  # 10 tests across size range

# Assertions - evaluated by headless test runner
[[assertions]]
type = "CreatureReachedTarget"
tag = "seeker"  # References tagged spawn below

[[assertions]]
type = "TicksCompleted"
count = 500

[[spawns]]
type = "single"
tag = "seeker"  # Tag for assertion reference
x = -10.0
y = 0.0
size = "$crit_size"  # References variant parameter
creature_type = "seeker"
target_x = 10.0
target_y = 0.0

1.3 Assertion Types
Evaluated by headless test runner (Dev-UI visual mode ignores these):

- NoOverlaps - No creature collisions occurred during trial
- CreatureReachedTarget { tag } - Tagged creature arrived at its target
- CreatureCount { min, max } - Expected creature count at trial end
- TicksCompleted { count } - Ran for N ticks without crash

🎬 Phase 2: The Trial Director System
Goal: A Bevy System capable of managing the lifecycle of a test inside the game loop.
2.1 The TrialDirector Resource
State Machine: Idle -> Loading -> Resetting -> Running -> Finished.
Responsibility:
Intercept: Listens for DevLoadSpec("name") IPC commands.
Hard Reset: Calls world.clear_entities() and resets all Resources (Time, PhysicsConfig) to defaults.
Seed RNG: Re-initializes ChaCha8Rng with the TOML seed.
Spawn: Instantiates entities from the TOML.
Monitor: Runs the specific assertion logic every tick.
Report: Sends SpecResult event back to Dev UI via IPC.
2.2 The ResetOnTrial Trait
To fix the "Dirty World" problem, all simulation Resources must implement this trait:
pub trait ResetOnTrial {
    fn reset(&mut self);
}

The Director iterates registered resources and calls reset() before every trial.
⚡ Phase 3: Automated Test Runner
Goal: Run specs at 1000+ ticks/second without graphics.
3.1 Headless Entry Point
Create a new binary target or test harness (tests/spec_runner.rs) that:
Initializes Bevy with MinimalPlugins (No Render, No Audio).
Sets the App Runner to a custom loop that ignores VSync.
Iterates through all TOML files in specs/.
Runs each trial until Success or Timeout.
Exits with code 0 (Pass) or 1 (Fail).
🎨 Phase 4: Visual Verification
Goal: Allow humans to "see" the math.

Visual verification uses existing force gizmos from Sprint 17. Dev-UI loads specs from specs/ folder and spawns critters normally - no special visualization needed.

