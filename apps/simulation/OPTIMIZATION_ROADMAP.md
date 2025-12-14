# Optimization Roadmap: Sub-20ms Tick @ 200K Creatures

## Quick Reference
- **Current:** 22-24ms/tick @ 200K creatures
- **Target:** < 20ms/tick
- **Primary Issue:** Perception component cache locality (192 bytes!)
- **Expected Gain:** 3.5-7ms total reduction

See `PERFORMANCE_ANALYSIS.md` for full technical analysis.

---

## Phase 1: Perception Component Split (CRITICAL)
**Expected: 2-4ms gain**
**Risk: LOW** (isolated change, well-tested access pattern)
**Effort: 4-6 hours**

### Files to Modify
1. `/home/dev/dev/speciate/apps/simulation/src/simulation/perception/components.rs`
2. `/home/dev/dev/speciate/apps/simulation/src/simulation/perception/systems.rs`
3. `/home/dev/dev/speciate/apps/simulation/src/simulation/creatures/behaviors/avoidance/systems.rs`
4. `/home/dev/dev/speciate/apps/simulation/src/simulation/creatures/builder.rs` (add NeighborCache init)

### Changes

#### 1. Split Component (components.rs)

**BEFORE:**
```rust
#[derive(Component, Debug, Clone)]
pub struct Perception {
    pub fov_angle: f32,
    pub range: f32,
    pub cos_half_fov_sq: f32,
    neighbor_count: u8,
    neighbors: [NeighborData; MAX_PERCEIVED_NEIGHBORS],
}
```

**AFTER:**
```rust
// Hot data: 16 bytes (read every tick for radius queries, FOV checks)
#[derive(Component, Debug, Clone, Copy)]
pub struct Perception {
    pub fov_angle: f32,        // 4 bytes
    pub range: f32,            // 4 bytes
    pub cos_half_fov_sq: f32,  // 4 bytes
    pub neighbor_count: u8,    // 1 byte + 3 padding
}

// Cold data: 168 bytes (read only when iterating neighbors in avoidance)
#[derive(Component, Debug, Clone)]
pub struct NeighborCache {
    neighbors: [NeighborData; MAX_PERCEIVED_NEIGHBORS],
}

impl NeighborCache {
    pub fn new() -> Self {
        Self {
            neighbors: [NeighborData::EMPTY; MAX_PERCEIVED_NEIGHBORS],
        }
    }

    pub fn iter(&self, count: usize) -> impl Iterator<Item = NeighborData> + '_ {
        self.neighbors[..count].iter().copied()
    }

    pub fn set(&mut self, index: usize, data: NeighborData) {
        self.neighbors[index] = data;
    }
}
```

#### 2. Update Perception Methods (components.rs)

```rust
impl Perception {
    // ... existing new(), from_body_size() unchanged ...

    pub fn has_neighbors(&self) -> bool {
        self.neighbor_count > 0
    }

    pub fn neighbor_count(&self) -> usize {
        self.neighbor_count as usize
    }

    pub fn clear(&mut self) {
        self.neighbor_count = 0;
    }

    pub fn set_neighbor_count(&mut self, count: usize) {
        self.neighbor_count = count.min(MAX_PERCEIVED_NEIGHBORS) as u8;
    }

    // REMOVED: add_neighbor(), iter_neighbors(), contains()
    // These now operate on NeighborCache directly
}
```

#### 3. Update Perception System (systems.rs)

**BEFORE:**
```rust
pub fn update_perception_system(
    grid: Res<DoubleBufferedSpatialGrid>,
    mut query: Query<(Entity, &Position, &Rotation, &BodySize, &mut Perception, &CreatureState)>,
```

**AFTER:**
```rust
pub fn update_perception_system(
    grid: Res<DoubleBufferedSpatialGrid>,
    mut query: Query<(
        Entity,
        &Position,
        &Rotation,
        &BodySize,
        &mut Perception,
        &mut NeighborCache,  // ADD THIS
        &CreatureState
    )>,
```

**Inside parallel loop (line 55-142), replace:**
```rust
// BEFORE
perception.clear();
// ... collect candidates ...
for (_, neighbor) in candidates.iter().take(k) {
    perception.add_neighbor(*neighbor);
}

// AFTER
perception.clear();
// ... collect candidates ...
for (i, (_, neighbor)) in candidates.iter().take(k).enumerate() {
    neighbor_cache.set(i, *neighbor);
}
perception.set_neighbor_count(k);
```

#### 4. Update Avoidance System (avoidance/systems.rs)

**BEFORE:**
```rust
pub fn avoidance_system(
    mut query: AvoidanceQuery,
```

**AFTER:**
```rust
pub fn avoidance_system(
    mut query: Query<(
        Entity,
        &Position,
        &Velocity,
        &BodySize,
        &mut Acceleration,
        &Perception,
        &NeighborCache,  // ADD THIS
        &AvoidanceBehavior,
        &CreatureState,
    ), With<CanAvoidObstacles>>,
```

**Inside parallel loop (line 46-95), replace:**
```rust
// BEFORE
for neighbor in perception.iter_neighbors() {

// AFTER
for neighbor in neighbor_cache.iter(perception.neighbor_count()) {
```

#### 5. Update Query Type Alias (queries.rs)

```rust
pub type AvoidanceQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Position,
        &'static Velocity,
        &'static BodySize,
        &'static mut Acceleration,
        &'static Perception,
        &'static NeighborCache,  // ADD THIS
        &'static AvoidanceBehavior,
        &'static CreatureState,
    ),
    With<CanAvoidObstacles>,
>;
```

#### 6. Update CritBuilder (builder.rs)

```rust
// Add to bundle:
.insert(NeighborCache::new())
```

### Validation

#### Tests to Update
1. `perception/systems.rs::tests` - Add `NeighborCache` to test entities
2. `avoidance/systems.rs::tests` - Add `NeighborCache` to test entities

#### Performance Measurement
```bash
# Before
cargo build --release
./scripts/perf_baseline.sh 200000 10 > before.txt

# After changes
cargo build --release
./scripts/perf_baseline.sh 200000 10 > after.txt

# Compare
diff before.txt after.txt
# Look for:
# - L1 miss rate reduction (target: -20% or more)
# - IPC increase (target: +10% or more)
```

---

## Phase 2: Thread-Local Allocation Optimization
**Expected: 0.5-1ms gain**
**Risk: LOW** (Rayon feature, well-documented)
**Effort: 2-3 hours**

### Files to Modify
1. `/home/dev/dev/speciate/apps/simulation/src/simulation/perception/systems.rs`

### Changes

**BEFORE (lines 22-28, 73-80):**
```rust
thread_local! {
    static CELL_SCRATCH: RefCell<Vec<(f32, usize)>> = RefCell::new(Vec::with_capacity(256));
}

thread_local! {
    static NEIGHBOR_CANDIDATES: RefCell<Vec<(f32, NeighborData)>> =
        RefCell::new(Vec::with_capacity(256));
}

// Usage:
CELL_SCRATCH.with(|scratch| {
    NEIGHBOR_CANDIDATES.with(|candidates_cell| {
        let mut cells = scratch.borrow_mut();
        let mut candidates = candidates_cell.borrow_mut();
        // ...
    });
});
```

**AFTER:**
```rust
// Remove thread_local! blocks entirely

struct PerceptionScratch {
    cells: Vec<(f32, usize)>,
    candidates: Vec<(f32, NeighborData)>,
}

impl Default for PerceptionScratch {
    fn default() -> Self {
        Self {
            cells: Vec::with_capacity(512),     // Increased from 256
            candidates: Vec::with_capacity(512),
        }
    }
}

// In parallel loop:
entities.par_iter_mut()
    .for_each_init(PerceptionScratch::default, |scratch, (entity, pos, rot, ...)| {
        scratch.cells.clear();
        scratch.candidates.clear();

        grid_ref.collect_cells_sorted(x, y, query_radius, facing_x, facing_y, &mut scratch.cells);

        for &(sort_key, cell_idx) in scratch.cells.iter() {
            // ... use scratch.candidates instead of candidates
        }
    });
```

### Validation
- Run existing tests (should pass unchanged)
- Check thread scalability: `RAYON_NUM_THREADS=4 cargo test` vs `RAYON_NUM_THREADS=16`

---

## Phase 3: Component Size Audit
**Expected: TBD (depends on findings)**
**Risk: MEDIUM** (may require data structure changes)
**Effort: 4-8 hours**

### Investigation Steps

1. **Run size analysis:**
   ```bash
   ./scripts/analyze_component_sizes.sh
   ```

2. **Check actual component sizes in code:**
   ```rust
   // Add to simulation startup (dev-tools only):
   #[cfg(feature = "dev-tools")]
   {
       use std::mem::size_of;
       log::info!("Component sizes:");
       log::info!("  Position: {} bytes", size_of::<Position>());
       log::info!("  Velocity: {} bytes", size_of::<Velocity>());
       log::info!("  CreatureState: {} bytes", size_of::<CreatureState>());
       log::info!("  WanderState: {} bytes", size_of::<WanderState>());
       log::info!("  Brain: {} bytes", size_of::<Brain>());
   }
   ```

3. **Identify shrink targets:**
   - Components > 32 bytes that are read every tick
   - Enum variants with large size differences (use `Box` for rare variants)
   - Unnecessary padding (reorder fields for better packing)

### Example Optimization (if CreatureState is bloated)

**BEFORE (hypothetical):**
```rust
struct CreatureState {
    behavior: BehaviorMode,  // 1 byte
    energy: f32,             // 4 bytes
    max_speed: f32,          // 4 bytes
    // ... other fields
    // PADDING: 23 bytes to next alignment
}
```

**AFTER:**
```rust
// Reorder fields for tighter packing
struct CreatureState {
    energy: f32,             // 4 bytes
    max_speed: f32,          // 4 bytes
    behavior: BehaviorMode,  // 1 byte
    // ... other fields aligned
    // PADDING: reduced
}
```

---

## Phase 4: Advanced (Optional)
**Only pursue if Phases 1-3 don't reach < 20ms**

### 4a. Query Iterator Reuse
**Risk: HIGH** (may violate Bevy borrowing rules)
**Expected: 1-2ms IF feasible**

Test with prototype:
```rust
#[derive(Resource)]
struct EntityCollector {
    perception_buf: Vec</* entity refs */>,
    movement_buf: Vec</* entity refs */>,
}
```

If Bevy's iter_mut() returns non-'static refs, this won't compile. Fallback: Accept allocation cost.

### 4b. Parallel Behavior Systems
**Risk: MEDIUM** (depends on Bevy scheduler)
**Expected: 0-2ms**

```rust
schedule.add_systems((
    territory_wandering_system,
    flee_system,
    seek_system,
    behaviors::avoidance_system,
).chain().after(perception::update_perception_system))
```

Change to:
```rust
schedule.add_systems((
    (
        territory_wandering_system,
        flee_system,
        seek_system,
        behaviors::avoidance_system,
    ),  // No .chain() = parallel!
).after(perception::update_perception_system))
```

---

## Measurement Protocol

### Before Each Phase
```bash
cargo build --release
perf stat -r 3 -e instructions,cycles,L1-dcache-load-misses,LLC-load-misses \
    timeout 10s ./target/release/simulation > before_phase_N.txt
```

### After Each Phase
```bash
cargo build --release
perf stat -r 3 -e instructions,cycles,L1-dcache-load-misses,LLC-load-misses \
    timeout 10s ./target/release/simulation > after_phase_N.txt

# Extract key metrics
grep "instructions" after_phase_N.txt
grep "cycles" after_phase_N.txt
grep "L1-dcache-load-misses" after_phase_N.txt
```

### Success Criteria
- Phase 1: L1 miss reduction > 20%, tick time reduction > 2ms
- Phase 2: Allocation count = 0 (check with `heaptrack`), tick time reduction > 0.5ms
- Phase 3: Component memory < 64 bytes for all hot components
- Overall: **Tick time < 20ms @ 200K creatures**

---

## Rollback Plan

Each phase is isolated. If performance regresses:

1. **Revert commit:** `git revert HEAD`
2. **Analyze:** `perf diff before.data after.data`
3. **Debug:** Check if Bevy archetype changed (component add/remove breaks SoA)

---

## Expected Timeline

- **Week 1:** Phase 1 (Perception split) + validation
- **Week 2:** Phase 2 (Thread-local) + Phase 3 (Audit)
- **Week 3:** Phase 4 IF needed, otherwise polish + documentation

**Deliverables:**
- Tick time < 20ms @ 200K creatures
- Perf report showing before/after
- Updated documentation in `docs/architecture/performance.md`
