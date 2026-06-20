# Documentation Standards for /docs

**Category:** 📖 Reference — how docs are written in this repo (the doc taxonomy map lives in [README.md](./README.md)).

**Positioning:** Speciate is a **portfolio showcase** - a high-performance artificial-life engine + visual sandbox demonstrating Rust × JS systems craft. For overall direction (four pillars, NOW/NEXT/DREAM tiers), see `docs/ROADMAP.md`. The standards below govern HOW we write docs, regardless of pillar.

## Core Principle

**Documentation describes WHAT and WHY, not HOW. Code shows HOW.**

Documentation that duplicates code will diverge, become stale, and mislead future readers. Keep docs high-level and reference implementation locations instead.

---

## What Belongs in /docs

### ✅ DO Document

**Feature descriptions:**
- What the feature does (behavior, mechanics)
- Why it exists (biological rationale, gameplay purpose)
- Where it's implemented (file paths, line ranges if specific)
- Key constants and their values (reference actual values, don't duplicate)
- Design decisions and trade-offs

**Biological rationale:**
- Real-world animal behavior parallels
- Physics formulas (Kleiber's law, allometric scaling)
- Ecological principles
- Trade-off explanations (why advantages have costs)

**Gameplay implications:**
- How players interact with the feature
- Strategic considerations
- Emergent behaviors
- Future integration points

### ❌ DON'T Document

**Code blocks:**
- Function signatures (will drift from actual code)
- Implementation details (belongs in source files)
- Algorithm pseudocode (read the actual implementation)
- Data structure definitions (use type signatures in code)

**Exception:** Tiny mathematical formulas are OK if they explain concepts:
```
mass = body_length^2.5  ← This is fine (explains scaling law)
```

But NOT:
```rust
pub fn calculate_mass(size: &BodySize) -> f32 {  ← Don't do this
    size.length.powf(2.5)
}
```

---

## Documentation Taxonomy — the KIND of every doc

**Every doc folder has exactly ONE category. Make it unmistakable.** The authoritative map of the whole tree lives in [docs/README.md](./README.md); this section codifies the convention so future docs follow it.

### Category legend

| | Category | Meaning |
|--|----------|---------|
| 📖 | **Reference** | Stable knowledge. NOT a feature lifecycle. Architecture, contracts, glossaries, retrospectives, archived decisions, evidence/assets. |
| 💡 | **Ideas** | Brainstormed, exploratory, **NOT committed**. `*/ideas/` and freeform concept folders. |
| 🚧 | **In progress (NOW)** | Being built right now — the active NOW pillars (`scale/`, `visuals/`). Cross-link the [Roadmap](./ROADMAP.md) NOW tier. |
| 📋 | **Planned** | Approved/designed but **not started**. `*/todo/` and deferred plans. |
| ✅ | **Done** | Implemented and working. `*/done/`. |
| 🌙 | **Dreamland** | Aspirational north-star. **Not scheduled.** |

### Reference vs Lifecycle — keep them separated

There are two kinds of doc area, and they must not be confused:

- **Reference areas (📖)** hold stable knowledge that does **not** move through a pipeline: `architecture/`, `protocol/`, `process/`, `incidents/`, `archive/`, and the root `GLOSSARY.md`. They have **no** `ideas/todo/done/` subfolders. Evidence/asset folders (`performance/snapshots/`, `performance/history/`, `architecture/diagrams/`) are reference **data**, not prose — never treat a benchmark JSON or a diagram PNG as a "doc."
- **Lifecycle areas** hold features moving `ideas/` 💡 → `todo/` 📋 → `done/` ✅: `biology/`, `gameplay/`, `performance/`, `testing/`. The NOW pillars `scale/` and `visuals/` are 🚧 and cross-link the Roadmap.

### Applying the convention

- **Folder/index level, not every leaf.** Set category at the area `README.md` (banner at top) and at the [docs/README.md](./README.md) map. Do **not** stamp an emoji on every leaf doc; the existing `ideas/todo/done` structure already signals lifecycle stage.
- **Each major area carries a `README.md`** stating its purpose, its category, and (for lifecycle areas) what `ideas/todo/done` mean + a pointer to the [Roadmap](./ROADMAP.md) for what is in progress NOW.
- **Honesty mandate applies to the label too.** An unbuilt item is 📋 Planned even if it is NOW-adjacent. Do not promote ideas to "done."

---

## Folder Structure: ideas/ todo/ done/

### done/

**Implemented and working features.**

Files in `done/` describe features that:
- Exist in the codebase
- Are tested and functional
- Are actively used in the simulation

**Format:**
```markdown
# Feature Name

**Status:** ✅ Implemented (Sprint X)
**Location:** `apps/simulation/src/path/to/file.rs`

## What It Does
[High-level description of behavior]

## Why It Exists
[Biological rationale, gameplay purpose]

## Key Parameters
[Reference actual constants with file:line, don't duplicate code]

## Integration
[How it connects to other systems]

## Future Work
[Known limitations, planned improvements]
```

### todo/

**Approved features for upcoming sprints.**

Files in `todo/` describe features that:
- Have been designed and approved
- Are scheduled for a specific sprint
- Have clear requirements and acceptance criteria

**Format:**
```markdown
# Feature Name

**Status:** 📋 Planned for Sprint X
**Dependencies:** Sprint Y must complete first

## Goal
[What this feature will enable]

## Design
[High-level approach, not implementation details]

## Acceptance Criteria
[How we know it's done]
```

### ideas/

**Brainstorming and future concepts.**

Files in `ideas/` describe:
- Exploratory concepts
- Not-yet-approved features
- Research notes
- Long-term vision items

**No required format** - these are freeform exploration docs.

---

## Reference Style

### ✅ Good: File Path References
```markdown
**Location:** `apps/simulation/src/simulation/creatures/behaviors/seek.rs:32-58`

The seek system calculates edge-to-edge distance by subtracting both radii from
the center distance. See `seek.rs:36-37` for the calculation.
```

### ❌ Bad: Code Duplication
```markdown
The seek system uses this code:
\`\`\`rust
let edge_distance = center_distance - self_radius - target_radius;
if edge_distance < arrival_threshold { ... }
\`\`\`
```

Why? When the code changes, the docs become lies.

---

## Writing Style

**Concise over comprehensive.**
- Prefer bullet points over paragraphs
- One concept per section
- Assume reader can read code if they want details

**Design rationale over implementation.**
- Explain WHY we chose this approach
- Document alternatives considered and rejected
- Highlight biological/physical constraints

**Link to source of truth.**
- Reference code files (implementation)
- Reference specs (live behavior documentation)
- Reference sprint plans (feature requirements)

---

## Examples

### ✅ Good Documentation

```markdown
# Target Radius Seeking

**Status:** ✅ Implemented
**Location:** `apps/simulation/src/simulation/creatures/behaviors/seek.rs`

## What It Does

Creatures stop at the edge of targets rather than centers. A creature seeking a
food patch (radius 2m) will stop when its body edge touches the patch edge, not
when reaching the center point.

## Why It Exists

**Biological realism:** Animals don't seek the center of resources - they stop
when they can access them (drink from water edge, graze at meadow boundary).

**Gameplay foundation:** Enables resource competition. Multiple creatures cannot
occupy the same space, so targets have limited capacity.

## Key Parameters

- Arrival threshold: 0.05m (see `movement/constants.rs:SEEKING.arrival_threshold`)
- Pounce threshold: 0.1m (prevents endless creeping)
- Slow zone multiplier: 30× (see `movement/constants.rs:SLOW_ZONE_MULTIPLIER`)

Edge distance formula: `center_distance - self_radius - target_radius`

## Integration

- Target component has radius field (`perception/components.rs:Target`)
- Seek system uses edge distance for arrival logic
- Future: Occupancy detection will check if target is accessible

## Future Work

**Occupancy detection** (Planned): Detect when target is occupied, transition
to Waiting state, retry or select new target.
```

### ❌ Bad Documentation (Too Much Code)

```markdown
# Target Radius Seeking

**Status:** ✅ Implemented

## Implementation

\`\`\`rust
pub struct Target {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
}

impl Target {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y, radius: 0.0 }
    }
}
\`\`\`

The seek system works like this:

\`\`\`rust
let self_radius = size.radius();
let target_radius = target.radius;
let center_distance = center_distance_sq.sqrt();
let edge_distance = center_distance - self_radius - target_radius;

if edge_distance < arrival_threshold {
    acceleration.ax += -velocity.vx * SEEKING.brake_force;
    acceleration.ay += -velocity.vy * SEEKING.brake_force;
}
\`\`\`
```

Why bad? All that code will drift from reality. Just say "subtracts both radii" and link to the file.

---

## Maintaining Documentation

### When to Update done/

**After implementing a feature:**
1. Create new doc in `done/` describing what was built
2. Keep it concise (1-2 pages max)
3. Focus on design decisions, not code
4. Link to actual implementation files

### When to Update todo/

**When approving a sprint:**
1. Move approved design from `ideas/` to `todo/`
2. Add sprint number and dependencies
3. Keep it high-level (detailed design happens during sprint)

### When Code Changes

**DON'T update docs to match every code change.**

Only update docs when:
- Behavior fundamentally changes (different game mechanics)
- Constants change significantly (affects balance)
- Architecture changes (different integration points)

**DO trust the code as source of truth.**

If docs and code disagree, assume code is correct. Update docs to match reality, don't try to keep them in sync constantly.

---

## Summary

| Question | Answer |
|----------|--------|
| Should I include code blocks? | No (except tiny formulas) |
| Should I document every parameter? | No (reference file:line instead) |
| How detailed should docs be? | High-level: WHAT and WHY, not HOW |
| What if code changes? | Only update docs if behavior changes |
| Where does implementation detail go? | In the code itself (comments if needed) |

**Remember:** Documentation is for understanding design intent. Code is for understanding implementation. Don't duplicate - reference instead.
