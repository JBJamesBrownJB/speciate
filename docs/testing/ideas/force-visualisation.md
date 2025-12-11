# Force Vector Visualization

**Status:** 💡 IDEA
**Category:** Developer Tools / Debugging
**Prerequisites:** Selected creature system, dev-tools toggle infrastructure

---

## Goal

Visualize the steering forces driving a selected creature's behavior. Answer "Why is it turning left?" by showing force vectors graphically and numerically.

**Visual:** Net force vector rendered as colored line from creature center
**Panel:** Force breakdown displayed in CreatureInfoPanel

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                         RUST BACKEND                                │
│                                                                     │
│  Behavior Systems (avoidance, seeking, fleeing, wandering)          │
│       │                                                             │
│       ▼                                                             │
│  ForceDebug Component ← Populated when entity is selected           │
│  (seek, flee, separation, wander, net_force vectors)                │
│       │                                                             │
│       ▼                                                             │
│  Serialization (only for selected entity, bandwidth efficient)      │
└─────────────────────────────────────────────────────────────────────┘
                              │
                              ▼ sim:state channel (EntitySnapshot)
┌─────────────────────────────────────────────────────────────────────┐
│                         PORTAL FRONTEND                             │
│                                                                     │
│  ForceOverlay (PixiJS Graphics)                                     │
│  - Renders net_force vector from creature center                    │
│  - Color-coded, scaled for visibility                               │
│  - Toggle via dev-tools menu                                        │
│       │                                                             │
│  CreatureInfoPanel (HTML)                                           │
│  - Shows force breakdown: seek, flee, separation, wander            │
│  - Magnitudes and directions                                        │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Phase 1: Rust Backend Instrumentation

**Outcome:** ForceDebug component populated for selected creature

### 1.1 ForceDebug Component

**Location:** `apps/simulation/src/simulation/creatures/components.rs`

```rust
#[derive(Component, Default, Clone)]
pub struct ForceDebug {
    pub seek: Vec2,
    pub flee: Vec2,
    pub separation: Vec2,
    pub wander: Vec2,
    pub net_force: Vec2,
}
```

### 1.2 Selected Entity Tracking

**Location:** `apps/simulation/src/simulation/core/resources.rs`

```rust
#[derive(Resource, Default)]
pub struct SelectedEntity(pub Option<Entity>);
```

Frontend sends selected entity ID via NAPI command when player clicks a creature.

### 1.3 Behavior System Updates

Modify each steering system to populate ForceDebug when entity matches SelectedEntity:

**Avoidance** (`behaviors/avoidance/systems.rs`):
- After computing separation force, write to `force_debug.separation`

**Seeking** (`behaviors/seeking.rs`):
- After computing seek force, write to `force_debug.seek`

**Fleeing** (`behaviors/fleeing.rs`):
- After computing flee force, write to `force_debug.flee`

**Wandering** (`behaviors/wandering.rs`):
- After computing wander force, write to `force_debug.wander`

**Movement** (`movement/systems.rs`):
- Before integration, copy Acceleration.0 to `force_debug.net_force`

### 1.4 Conditional Compilation

All ForceDebug code behind `--dev-tools` feature flag:

```rust
#[cfg(feature = "dev-tools")]
if let Some(selected) = selected_entity.0 {
    if entity == selected {
        if let Ok(mut debug) = force_debug_query.get_mut(entity) {
            debug.separation = separation_force;
        }
    }
}
```

---

## Phase 2: IPC Pipeline

**Outcome:** ForceDebug data reaches frontend via existing sim:state channel

### 2.1 EntitySnapshot Extension

**Location:** `apps/simulation/src/napi_addon/serialization.rs`

Add optional force_debug field to EntitySnapshot (only populated for selected entity):

```rust
pub struct EntitySnapshot {
    // existing fields...

    #[cfg(feature = "dev-tools")]
    pub force_debug: Option<ForceDebugSnapshot>,
}

#[cfg(feature = "dev-tools")]
pub struct ForceDebugSnapshot {
    pub seek: [f32; 2],
    pub flee: [f32; 2],
    pub separation: [f32; 2],
    pub wander: [f32; 2],
    pub net_force: [f32; 2],
}
```

### 2.2 Selection Command

**Location:** `apps/simulation/src/napi_addon/commands.rs`

```rust
pub fn set_selected_entity(entity_id: Option<u64>) {
    // Update SelectedEntity resource
}
```

Called from Portal when player clicks creature or clears selection.

---

## Phase 3: Frontend Rendering

**Outcome:** Visual force vector and panel breakdown

### 3.1 ForceOverlay Class

**Location:** `apps/portal/src/rendering/ForceOverlay.ts`

Follow PerceptionOverlay pattern:

```typescript
export class ForceOverlay {
    private graphics: Graphics;
    private enabled: boolean = false;

    constructor(container: Container) {
        this.graphics = new Graphics();
        container.addChild(this.graphics);
    }

    setEnabled(enabled: boolean): void {
        this.enabled = enabled;
        this.graphics.visible = enabled;
    }

    update(snapshot: EntitySnapshot | undefined): void {
        this.graphics.clear();
        if (!this.enabled || !snapshot?.forceDebug) return;
        this.render(snapshot);
    }

    private render(snapshot: EntitySnapshot): void {
        const { forceDebug } = snapshot;
        const pos = snapshot.position;

        // Net force vector (blue, thick)
        this.drawVector(pos, forceDebug.netForce, 0x3366ff, 3);
    }

    private drawVector(
        origin: Vec2,
        force: Vec2,
        color: number,
        width: number
    ): void {
        const SCALE = 10; // Visualization multiplier
        const endX = origin.x + force.x * SCALE;
        const endY = origin.y + force.y * SCALE;

        this.graphics.lineStyle(width, color, 0.8);
        this.graphics.moveTo(origin.x, origin.y);
        this.graphics.lineTo(endX, endY);

        // Arrowhead
        this.drawArrowhead(origin, { x: endX, y: endY }, color);
    }
}
```

### 3.2 CreatureInfoPanel Extension

**Location:** `apps/portal/src/ui/CreatureInfoPanel.ts`

Add force breakdown section:

```typescript
private updateForceSection(forceDebug: ForceDebugData | undefined): void {
    if (!forceDebug) {
        this.forceSection.style.display = 'none';
        return;
    }

    this.forceSection.style.display = 'block';
    this.forceSection.innerHTML = `
        <div class="section-header">Forces</div>
        <div class="force-row">
            <span class="force-label">Seek:</span>
            <span class="force-value">${this.formatForce(forceDebug.seek)}</span>
        </div>
        <div class="force-row">
            <span class="force-label">Flee:</span>
            <span class="force-value">${this.formatForce(forceDebug.flee)}</span>
        </div>
        <div class="force-row">
            <span class="force-label">Separation:</span>
            <span class="force-value">${this.formatForce(forceDebug.separation)}</span>
        </div>
        <div class="force-row">
            <span class="force-label">Wander:</span>
            <span class="force-value">${this.formatForce(forceDebug.wander)}</span>
        </div>
        <div class="force-row total">
            <span class="force-label">Net:</span>
            <span class="force-value">${this.formatForce(forceDebug.netForce)}</span>
        </div>
    `;
}

private formatForce(vec: Vec2): string {
    const mag = Math.sqrt(vec.x * vec.x + vec.y * vec.y);
    const angle = Math.atan2(vec.y, vec.x) * (180 / Math.PI);
    return `${mag.toFixed(1)}N @ ${angle.toFixed(0)}°`;
}
```

### 3.3 Dev-Tools Toggle

**Location:** `apps/portal/src/ui/DevToolsMenu.ts`

Add toggle alongside existing FOV and grid toggles:

```typescript
{
    label: 'Show Force Vectors',
    key: 'forceVectors',
    default: false,
    onChange: (enabled) => this.forceOverlay.setEnabled(enabled)
}
```

---

## Phase 4: Integration

### 4.1 Selection Flow

1. Player clicks creature → SelectionManager captures entity ID
2. Portal sends `setSelectedEntity(entityId)` via NAPI
3. Rust updates SelectedEntity resource
4. Behavior systems populate ForceDebug for selected entity only
5. Serialization includes ForceDebug in that entity's snapshot
6. Portal renders force vector and updates panel

### 4.2 Deselection Flow

1. Player clicks empty space → SelectionManager clears selection
2. Portal sends `setSelectedEntity(null)` via NAPI
3. Rust clears SelectedEntity resource
4. No ForceDebug data serialized (saves bandwidth)
5. ForceOverlay clears, panel hides force section

---

## Visual Design

### Color Coding

| Force | Color | Hex | Meaning |
|-------|-------|-----|---------|
| Net Force | Blue | `0x3366ff` | Final steering direction |
| Seek | Green | `0x33cc33` | Attraction toward target |
| Flee | Red | `0xff3333` | Repulsion from threat |
| Separation | Orange | `0xff9933` | Personal space enforcement |
| Wander | Purple | `0x9933ff` | Random exploration |

### Scaling

Raw force vectors are small (typically 0-50 N). Multiply by visualization constant (10×) for visible lines on screen.

```typescript
const FORCE_SCALE = 10; // 1 Newton = 10 pixels
```

### Rendering Order

```
1. World terrain
2. Resources
3. Creatures
4. Force vectors (ForceOverlay)  ← Above creatures
5. Perception range (PerceptionOverlay)
6. UI panels
```

---

## Performance Considerations

### Bandwidth

Only selected entity sends ForceDebug data:
- 5 vectors × 2 floats × 4 bytes = 40 bytes per tick
- Negligible vs full entity stream

### CPU

Force calculation happens regardless (steering behavior). Only additional work:
- One entity comparison per behavior system
- One component write if match

Estimated overhead: <0.01ms

### GPU

One additional Graphics object with 1-5 line segments:
- Negligible draw call overhead

---

## Implementation Files

**New:**
- `apps/portal/src/rendering/ForceOverlay.ts`
- `apps/simulation/src/simulation/core/selected_entity.rs`

**Modified:**
- `apps/simulation/src/simulation/creatures/components.rs` (ForceDebug)
- `apps/simulation/src/simulation/creatures/behaviors/avoidance/systems.rs`
- `apps/simulation/src/simulation/creatures/behaviors/seeking.rs`
- `apps/simulation/src/simulation/creatures/behaviors/fleeing.rs`
- `apps/simulation/src/simulation/creatures/behaviors/wandering.rs`
- `apps/simulation/src/simulation/movement/systems.rs`
- `apps/simulation/src/napi_addon/serialization.rs`
- `apps/simulation/src/napi_addon/commands.rs`
- `apps/portal/src/ui/CreatureInfoPanel.ts`
- `apps/portal/src/ui/DevToolsMenu.ts`

---

## Success Criteria

- [ ] ForceDebug component added behind dev-tools feature flag
- [ ] Behavior systems populate ForceDebug for selected entity
- [ ] Selected entity ID communicated via NAPI
- [ ] ForceOverlay renders net force vector from creature center
- [ ] CreatureInfoPanel shows force breakdown (seek, flee, separation, wander, net)
- [ ] Toggle in dev-tools menu enables/disables force visualization
- [ ] Force vectors scaled appropriately for visibility (10× multiplier)
- [ ] No performance regression (verified via benchmarks)
- [ ] All existing tests pass

---

## Future Extensions

- **Component forces:** Show individual vectors in overlay (not just net)
- **Force history:** Trail showing force changes over time
- **Comparative view:** Side-by-side force comparison of two creatures
- **Force magnitude graph:** Time-series chart in dev-ui

---

## See Also

- `apps/portal/src/rendering/PerceptionOverlay.ts` - Existing visualization pattern
- `apps/portal/src/ui/CreatureInfoPanel.ts` - Panel pattern
- `docs/biology/done/movement-physics.md` - Force accumulation architecture
- `docs/biology/done/avoidance-behavior.md` - Avoidance force calculation

---

**Last Updated:** 2025-12-11
