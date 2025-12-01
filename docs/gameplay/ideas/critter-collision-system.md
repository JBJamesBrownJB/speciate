# Critter Collision System (ECS)

Technical specification for reactive physics collisions between creature entities.

---

## 1. Overview

This document outlines the design for a new ECS-based reactive physics system (`CritterCollisionSystem`).

**Purpose:** Handle physical consequences of creature collisions. This is a *reactive* system that complements existing *proactive* steering behaviors (Separation, Avoidance).

While steering behaviors help creatures avoid each other, they will inevitably fail. This system manages those failures, creating more dynamic, natural, and physically-believable simulations.

---

## 2. Design Goals

### Integration with Force Model

The system outputs **force vectors**, not direct velocity changes. This allows collision "bounce" to be added to the force accumulator, where it competes with steering forces.

**Result:** Emergent "stumbling" or "bracing" visuals as AI corrects path against physical impulse.

### Physical Realism (Simplified)

Models Newton's Third Law (action-reaction) and Conservation of Momentum by distributing collision forces based on creature mass.

### Emergent Behavior

Not just preventing visual overlap - this is a **gameplay-generating system** for unscripted scenarios:

- **Dominance:** Large-mass creatures "shove" through smaller ones, creating natural hierarchies
- **Panic Waves:** Predator striking herd creates physical "bounce" cascading through group
- **Natural Spacing:** Gentle "nudges" create more realistic dynamic spacing than perfect Separation forces

### Performance

Highly performant circle-to-circle checks over complex polygon collision.

---

## 3. System Implementation

### 3.1 Queried Components

The system iterates over entities with:

| Component | Type | Status | Notes |
|-----------|------|--------|-------|
| `Position` | Vector2 | Existing | World position |
| `Velocity` | Vector2 | Existing | Linear velocity |
| `BodySize` | float | Existing | `CollisionRadius = length / 2` |
| `Acceleration` | Vector2 | Existing | Serves as ForceAccumulator |
| `Health` | float | **New** | To be added |

**Derived Values (not stored):**

```rust
let collision_radius = body_size.length / 2.0;
let mass = body_size.length.powf(2.5);  // Allometric scaling
```

See `docs/biology/biology-notes.md` for mass scaling rationale.

---

### 3.2 System Logic (Per-Frame)

**Broadphase:**
Use unified spatial partitioning strategy (see `SPRINTS/spatial-grid/SPRINT_PLAN.md`).

- Initial: O(N²) brute-force (acceptable for <500 creatures)
- Future: 50m spatial hash grid when scaling beyond 500 creatures
- Same structure serves perception and collision queries

**Narrowphase (Iterate on Pairs):**

For each potential pair (A, B):

```rust
let distance = distance(a.position, b.position);
let radii_sum = a.radius + b.radius;

if distance < radii_sum {
    // Collision detected - proceed to response
}
```

---

### 3.3 Collision Response

When collision detected, execute these steps in order:

#### A. Immediate Overlap Resolution

Moves entities apart this frame to fix visual overlap. Applied **before** force calculation.

```rust
// Calculate penetration depth
let penetration = radii_sum - distance;

// Collision normal (points from B to A)
let normal = normalize(a.position - b.position);

// Distribute resolution equally
a.position += normal * (penetration / 2.0);
b.position -= normal * (penetration / 2.0);
```

---

#### B. Impulse Force Calculation

Calculates "bounce" force and adds to accumulator for movement system.

```rust
// Relative velocity
let relative_velocity = a.velocity - b.velocity;

// Velocity component along collision normal
let vel_along_normal = dot(relative_velocity, normal);

// If already separating, no bounce needed
if vel_along_normal > 0.0 {
    return;
}

// Impulse magnitude (inelastic collision)
let restitution = 0.3;  // Biological tissue absorbs energy
let impulse_magnitude = -(1.0 + restitution) * vel_along_normal;

// Distribute force by mass (Conservation of Momentum)
let total_mass = a.mass + b.mass;
let force_on_a = impulse_magnitude * normal * (b.mass / total_mass);
let force_on_b = impulse_magnitude * -normal * (a.mass / total_mass);

// Apply to accumulators
a.acceleration += force_on_a;
b.acceleration += force_on_b;
```

---

#### C. Damage Calculation

Quantizes collision force to apply damage only on significant impacts.

**Constants:**

```rust
const VELOCITY_THRESHOLD: f32 = 3.0;   // m/s (~10 km/h jogging speed)
const IMPULSE_THRESHOLD: f32 = 5.0;    // Minimum force for damage
const DAMAGE_MULTIPLIER: f32 = 0.5;    // Scale damage amount
```

**Logic:**

```rust
// Skip gentle contacts (prevents low-speed damage)
if vel_along_normal.abs() < VELOCITY_THRESHOLD {
    return;  // No damage
}

// Only damage above force threshold
if impulse_magnitude > IMPULSE_THRESHOLD {
    let damage = (impulse_magnitude - IMPULSE_THRESHOLD) * DAMAGE_MULTIPLIER;

    // Distribute damage - receiver of force takes more
    a.health -= damage * (b.mass / total_mass);
    b.health -= damage * (a.mass / total_mass);
}
```

---

## 4. New Components

### Health (Required)

```rust
pub struct Health {
    pub current: f32,
    pub max: f32,
}
```

Initial implementation: Health tracking only, no death system.

### Derived Values (No New Components)

| Value | Formula | Notes |
|-------|---------|-------|
| CollisionRadius | `BodySize.length / 2` | Uses existing BodySize |
| Mass | `BodySize.length.powf(2.5)` | Allometric scaling |

**Why Size^2.5?**
- Size² (area) makes small creatures too light
- Size³ (volume) makes large creatures immovable
- Size^2.5 balances geometry with biological allometry

See `docs/biology/biology-notes.md` for full rationale.

---

## 5. Expected Emergent Behaviors

### Glancing Blows vs Head-On

**Grazing blow:** `relativeVelocity` perpendicular to normal → small `velAlongNormal` → gentle "nudge"

**Head-on collision:** `relativeVelocity` parallel to normal → massive `velAlongNormal` → violent bounce + high damage

### Stumbling

Creature nudged by collision has large force applied. AI's Seek force still points to original direction. Result: "stumble" or "drift" as AI fights physical impulse.

### Physical Dominance

**Example:** Mass 100 vs Mass 1

- Large creature receives ~1% of bounce force
- Small creature receives ~99% of bounce force
- Large creature barely affected, small one violently thrown back
- Creates emergent "shoving" and "trampling" without special AI

---

## 6. Implementation Phases

### 6.1 Minimal First Pass (Recommended Start)

**Goal:** Prove physics model works without death/health systems.

**Components:**
- Add Health component (100.0 default, no death logic)
- Derive CollisionRadius from `BodySize.length / 2`
- Derive Mass from `BodySize.length.powf(2.5)`
- Use Acceleration as force accumulator (existing)

**Systems:**
- O(N²) collision detection (brute force)
- Immediate overlap resolution
- Impulse force calculation
- **Damage logging only:**

```rust
println!("Ooof, that hurt! Entity {:?} took {:.1} damage", entity, damage);
```

**DO NOT** reduce `Health.current` - just record the event.

**Why Logging Only?**
- Validates physics formulas without risk
- Observe damage patterns in console
- No death/despawn system needed yet
- Easy to enable full damage later

**What to Test:**
- Creatures bounce off each other (impulse forces work)
- Large creatures push small ones harder (mass distribution correct)
- No overlap after collision (position resolution works)
- Damage messages for high-speed impacts (threshold working)

---

### 6.2 Future Phases

**Phase 2: Full Damage System**
- Reduce `Health.current` on collision
- Add `Dead` marker component + despawn system
- Add `CollisionDamageEvent` for UI/sound/particles

**Phase 3: Spatial Optimization**
- Implement unified spatial hash grid
- Replace O(N²) with O(N) queries
- Required when creature count > 500

**Phase 4: Advanced Features**
- Armor DNA trait (damage reduction)
- Collision intent system (opt-in damage)
- Resilience scaling (small creatures more durable per mass)

---

## 7. References

- **Spatial partitioning:** `SPRINTS/spatial-grid/SPRINT_PLAN.md`
- **Physics rationale:** `docs/biology/biology-notes.md` (2025-11-16 entry)
- **Behavior engine:** `docs/architecture/behavior-engine.md`
- **DNA-driven design:** `docs/biology/dna-driven-design.md`
