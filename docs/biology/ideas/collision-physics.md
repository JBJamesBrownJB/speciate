# Collision Physics - Mass, Restitution, Damage

**Status:** ❌ NOT IMPLEMENTED

**Current:** Steering-based collision avoidance only (no physics, no damage)

---

## Mass Scaling: Size^2.5

**Formula:** `mass = body_size.length.powf(2.5)`

**Biological rationale:**
- Pure volume (Size^3): Makes large creatures immovable walls
- Pure area (Size^2): Makes small creatures too light (unrealistic)
- **Size^2.5**: Balances geometry with biological allometry

**Real-world evidence:**
- Mass scales ~Size^2.7 in terrestrial mammals
- Creatures have hollow structures (bones, lungs, digestive tracts)
- Mouse (0.03m): ~20g, Wolf (1.2m): ~40kg, Elephant (3m): ~5000kg

**Implementation:**
```rust
pub fn mass_from_size(body_size: &BodySize) -> f32 {
    body_size.length.powf(2.5)
}
```

**Bounds (0.5m - 10m creatures):**
- Minimum: 0.5m → 0.35 mass units
- Maximum: 10m → 316 mass units
- Ratio: ~900:1 (realistic for terrestrial fauna)

---

## Restitution Coefficient: 0.3

**Recommendation:** 0.3 (inelastic), NOT 1.0 (elastic)

**Biological rationale:**
- Muscle tissue deforms and absorbs energy
- Fat layers provide damping
- Joint flexibility dissipates impact

**Real-world coefficients:**
- Human body: 0.2-0.4
- Rugby tackle: ~0.3
- Boxing punch: ~0.25

**1.0 = pinball physics** (wild bouncing)
**0.3 = weighty biological feel** (sticky collisions, energy dissipation)

**Emergent effects:**
- Creatures bunch in dense areas (realistic herd behavior)
- Large collisions slow everyone (energy dissipation)
- Corridor choke points become deadly (pile-ups)

---

## Velocity Threshold: 3.0 m/s

**No damage below 3.0 m/s (~10 km/h jogging speed)**

**Biological rationale:**
- Gentle contact shouldn't cause injury
- Real animals bump constantly without damage
- Prevents "death by standing near each other" bugs

**Speed classifications:**
- Walking (1-2 m/s): No damage
- Jogging (3-5 m/s): Minor damage possible
- Running (8-15 m/s): Significant impact
- Sprinting (15+ m/s): Serious injury

**Implementation:**
```rust
const VELOCITY_THRESHOLD: f32 = 3.0; // m/s

if velocity_along_normal.abs() < VELOCITY_THRESHOLD {
    return; // No damage for gentle contact
}
```

---

## Normalized Collision Vector

**Verdict:** Essential for correct physics

**Why normalization required:**
- Collision normal must be unit vector for impulse direction
- Without normalization, force magnitude scales with distance (wrong!)
- Impulse direction must be perpendicular to contact surface

**Performance optimization:**
```rust
let dist_sq = dx × dx + dy × dy;
if dist_sq < (radius_a + radius_b).powi(2) {
    // Only NOW compute sqrt for narrowphase
    let dist = dist_sq.sqrt();
    let normal = (pos_a - pos_b) / dist;
    // Apply impulse...
}
```

Use squared distance for broadphase, sqrt only when collision confirmed.

---

## Damage Distribution: Mass-Based

**Formula:**
```rust
let total_damage = (impulse - IMPULSE_THRESHOLD) × DAMAGE_MULTIPLIER;
let damage_to_a = total_damage × (mass_b / total_mass);
let damage_to_b = total_damage × (mass_a / total_mass);
```

**Biological validation:**
- Large hits small → small takes most damage (trampling)
- Equal-size collision → damage split evenly
- Matches real stampede/charge injury patterns

**Emergent behaviors:**
- Large predators can trample small prey
- Small creatures evolve high agility OR high armor
- Mid-sized creatures balance speed vs durability

**Collision Force Consequences**
- stun (send them into temporary catatonic phase)
- death it impact high enough

- Animals with teeth can 'lock on' so pray don't bounce away'

- visuals, stars above head
---

## Future Considerations

### Armor DNA Trait
```rust
let armor_factor = 1.0 - creature.dna.armor; // 0.0 to 0.8
let final_damage = base_damage × armor_factor;
```

### Collision Intent System
- Passive collisions (herd movement): No damage
- Aggressive collisions (charging attack): Full damage
- Defensive collisions (panic fleeing): Reduced damage
- Prevents "trampling dominance" where biggest always wins

### Resilience by Size
```rust
let resilience = 1.0 / creature.dna.size.powf(0.5);
```
Small creatures more resilient to falls/impacts (square-cube law).

---

## Implementation Status

### ✅ Currently Implemented
- BodySize component with radius calculation
- Basic collision avoidance (steering force-based)
- Personal space enforcement

### ❌ Not Implemented
- Mass calculation from size (size^2.5)
- Collision impulse response (momentum transfer)
- Restitution coefficient (0.3)
- Velocity threshold for damage (3.0 m/s)
- Normalized collision vectors
- Mass-based damage distribution
- Actual collision physics system

**Current:** DEFAULT_MASS = 65.0 (not scaled by size!)

**Location:** `apps/simulation/src/simulation/core/components.rs`

---

## Trade-Offs

**Every advantage has systemic cost:**
- Large size = High power BUT slow reactions, high energy cost
- Small size = Fast agile BUT vulnerable to trampling
- No "god-tier" combination—physics enforces balance
