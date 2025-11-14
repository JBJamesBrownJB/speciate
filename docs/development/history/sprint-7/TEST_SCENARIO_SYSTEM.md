# Test Scenario System - On-Demand Spawning

**Status:** Planned for future sprint
**Priority:** Medium (Developer experience improvement)
**Effort:** 6-9 hours

## Problem

Need to quickly test different creature configurations during development. Current hardcoded `spawn_initial_creatures()` requires code changes and recompile for each test setup.

**Critical bug found:** Current obstacle avoidance demo (1 obstacle + 3 seekers) reveals pathfinding issues that need visual testing.

## Solution: Dev UI with Runtime Scenario Spawning

Enable on-demand spawning of test scenarios while app is running, similar to old admin-ui with buttons like "Clear", "Spawn 10 Wanderers", "Obstacle Test", etc.

## Architecture

### Backend: Scenario Registry Pattern

```rust
// src/scenarios/mod.rs
pub struct ScenarioRegistry {
    scenarios: HashMap<String, Box<dyn Fn(&mut Simulation)>>,
}

impl ScenarioRegistry {
    pub fn register(&mut self, name: &str, scenario_fn: impl Fn(&mut Simulation) + 'static) {
        self.scenarios.insert(name.to_string(), Box::new(scenario_fn));
    }

    pub fn spawn(&self, name: &str, sim: &mut Simulation) -> Result<(), String> {
        // Execute scenario function
    }

    pub fn list(&self) -> Vec<ScenarioMetadata> {
        // Return list for UI
    }
}
```

### Scenario Presets (src/scenarios/presets.rs)

```rust
// Adding a new scenario is trivial:
pub fn obstacle_avoidance(sim: &mut Simulation) {
    // Spawn 1 catatonic obstacle at (0, 0)
    sim.spawn_crit(
        CritBuilder::new()
            .at(0.0, 0.0)
            .in_behavior(BehaviorMode::Catatonic)
    );

    // Spawn 3 seekers from different angles
    sim.spawn_crit(CritBuilder::new().at(20.0, 0.0).as_seeker(-10.0, 0.0));
    sim.spawn_crit(CritBuilder::new().at(-20.0, 0.0).as_seeker(10.0, 0.0));
    sim.spawn_crit(CritBuilder::new().at(0.0, 20.0).as_seeker(-10.0, -10.0));
}

pub fn spawn_swarm(sim: &mut Simulation) {
    // 100+ wanderers for stress testing
    for i in 0..100 {
        let angle = (i as f32) * 0.0628; // Radial distribution
        let radius = 50.0 + (i as f32 % 10.0) * 5.0;
        sim.spawn_crit(
            CritBuilder::new()
                .at(angle.cos() * radius, angle.sin() * radius)
                .with_wandering()
                .in_behavior(BehaviorMode::Wandering)
        );
    }
}

pub fn empty_world(_sim: &mut Simulation) {
    // No-op (clear handled separately)
}

// Register at startup:
fn init_registry() -> ScenarioRegistry {
    let mut registry = ScenarioRegistry::new();
    registry.register("empty", empty_world);
    registry.register("obstacle_avoidance", obstacle_avoidance);
    registry.register("swarm", spawn_swarm);
    // Easy to add more!
    registry
}
```

### Tauri IPC Commands

```rust
// portal/src-tauri/src/tauri_commands.rs

#[tauri::command]
fn clear_simulation(sim_state: State<Arc<RwLock<Simulation>>>) -> Result<(), String> {
    let mut sim = sim_state.write().unwrap();
    // Despawn all entities (iterate and despawn)
    Ok(())
}

#[tauri::command]
fn spawn_scenario(
    name: String,
    registry: State<Arc<ScenarioRegistry>>,
    sim_state: State<Arc<RwLock<Simulation>>>
) -> Result<ScenarioInfo, String> {
    let mut sim = sim_state.write().unwrap();
    registry.spawn(&name, &mut sim)?;
    Ok(ScenarioInfo {
        name,
        creature_count: sim.creature_count()
    })
}

#[tauri::command]
fn get_available_scenarios(
    registry: State<Arc<ScenarioRegistry>>
) -> Vec<ScenarioMetadata> {
    registry.list()
}
```

### Frontend: DevTools Panel

```typescript
// apps/portal/src/devtools/DevToolsPanel.tsx
import { invoke } from '@tauri-apps/api/tauri';

export function DevToolsPanel() {
  const [visible, setVisible] = useState(false);
  const [scenarios, setScenarios] = useState<ScenarioMeta[]>([]);

  // Toggle with Ctrl+Shift+D
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.ctrlKey && e.shiftKey && e.key === 'D') {
        setVisible(v => !v);
      }
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, []);

  // Load scenarios on mount
  useEffect(() => {
    invoke<ScenarioMeta[]>('get_available_scenarios').then(setScenarios);
  }, []);

  if (!visible) return null;

  return (
    <div className="devtools-panel">
      <h3>Test Scenarios</h3>

      <button
        className="clear-btn"
        onClick={() => invoke('clear_simulation')}
      >
        Clear All
      </button>

      {scenarios.map(scenario => (
        <button
          key={scenario.name}
          onClick={() => invoke('spawn_scenario', { name: scenario.name })}
        >
          {scenario.display_name}
        </button>
      ))}
    </div>
  );
}
```

**UI Mockup:**
```
┌─────────────────────────┐
│  Test Scenarios    [x]  │  ← Floating panel, top-right
├─────────────────────────┤
│  [Clear All]            │  ← Red/prominent
│  [Empty World]          │
│  [Obstacle Avoidance]   │  ← Current hardcoded demo
│  [Swarm (100+)]         │
│  [Seeking Test]         │
│  [Territory Test]       │
└─────────────────────────┘

Toggle: Ctrl+Shift+D
```

## Implementation Plan

### Phase 1: Backend (3-4h)
1. Create `src/scenarios/` module
2. Implement `ScenarioRegistry` and `presets.rs`
3. Add 5-7 useful presets
4. Add Tauri commands (`clear_simulation`, `spawn_scenario`, `get_available_scenarios`)
5. Wire up in `portal/src-tauri/src/main.rs`

### Phase 2: Frontend (2-3h)
1. Create `DevToolsPanel.tsx` component
2. Add keyboard shortcut handler (Ctrl+Shift+D)
3. Fetch and render scenario buttons
4. Style as floating dev panel (distinct from game UI)

### Phase 3: Migration (1-2h)
1. Extract current `spawn_initial_creatures()` → `scenarios::obstacle_avoidance()`
2. Change default spawn to empty world
3. Test all scenarios + clear cycle
4. Write README for adding scenarios

## Benefits

### Easy to Add New Scenarios
```rust
// Just add function + 1 line of registration = done!
pub fn my_test(sim: &mut Simulation) {
    sim.spawn_crit(CritBuilder::new().at(0.0, 0.0).as_seeker(100.0, 0.0));
}

// In init_registry():
registry.register("my_test", my_test);
```

**No UI changes needed** - button appears automatically.

### Clean Architecture
- **Scenarios** = what to spawn (pure functions)
- **Registry** = discovery/execution (infrastructure)
- **Tauri commands** = IPC bridge (thin)
- **DevTools UI** = trigger (generic)

### No Breaking Changes
- Existing spawning API unchanged
- Can keep TOML system for future config-based scenarios
- Tests can reuse scenarios

## Initial Scenario Library

1. **empty** - Clear world (no spawn)
2. **obstacle_avoidance** - Current demo (reveals pathfinding bug)
3. **swarm** - 100+ wanderers (stress test)
4. **seeking_test** - 5 seekers with distant targets
5. **territory_test** - Overlapping wanderer territories
6. **sparse** - 10 wanderers, large world
7. **dense** - 50 wanderers, small world

## Testing Strategy

### Manual
1. Launch: `cargo tauri dev`
2. Press Ctrl+Shift+D → DevTools appear
3. Click scenarios → verify spawning
4. Click Clear → verify despawning
5. Test: clear → spawn → clear → spawn (memory safety)

### Automated
```rust
#[test]
fn test_scenario_spawns_correct_count() {
    let mut sim = SimulationBuilder::new().build();
    scenarios::obstacle_avoidance(&mut sim);
    assert_eq!(sim.creature_count(), 4); // 1 obstacle + 3 seekers
}
```

## Future Enhancements (Not MVP)
- Parameterized scenarios (slider: spawn N creatures)
- Save custom scenarios to disk
- Scenario descriptions/tooltips in UI
- Keyboard shortcuts for quick access (Ctrl+Shift+1-9)
- Scenario categories/tags

## Why Rust Presets (Not TOML)?

For **runtime on-demand spawning**, Rust functions are better:
- **Flexible**: Can use loops, conditionals, randomization
- **Type-safe**: Compiler catches errors
- **Fast to add**: Just a function + 1 line registration
- **No parsing overhead**: Direct execution

TOML is still useful for **startup configuration** (keep for future).

## Related Files

### New
- `src/scenarios/mod.rs` - Registry
- `src/scenarios/presets.rs` - Preset functions
- `src/scenarios/README.md` - "How to add scenario"
- `apps/portal/src/devtools/DevToolsPanel.tsx` - UI

### Modified
- `src/lib.rs` - Add `pub mod scenarios;`
- `portal/src-tauri/src/tauri_commands.rs` - Add commands
- `portal/src-tauri/src/main.rs` - Initialize registry
- `apps/portal/src/main.ts` - Render DevToolsPanel

---

**Key Decision:** Runtime spawning (not just startup config) enables rapid visual testing workflow.
