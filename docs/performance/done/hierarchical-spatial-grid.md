# Hierarchical Spatial Grid

**Status:** Implemented (ABC Super Sprint - Phase A)
**Location:** `apps/simulation/src/simulation/spatial/`

---

## What It Does

Two-level spatial grid hierarchy for efficient perception at different scales:

- **L0 (20m cells):** Entity scanning, stores PerceptionProxy data
- **L1 (60m cells = 3x3 L0):** Area awareness, stores aggregated BioSignatures

L0 is double-buffered (perception reads front while rebuild writes back). L1 is single-buffered and rebuilt from L0 each tick.

---

## Why It Exists

**Performance:** Instead of checking every entity in range, creatures first scan L1 cells to determine if detailed L0 scanning is worthwhile. Empty or irrelevant L1 cells can be skipped entirely.

**Size domination:** Large creatures ignore tiny entities (below their perception threshold). BioSignature aggregation at L1 enables efficient early-exit when all creatures in a cell are too small to perceive.

---

## Key Components

### BioSignature (L1 Cell Data)

| Field | Purpose |
|-------|---------|
| `total_mass` | Sum of all creature mass in cell |
| `max_size` | Largest creature radius in cell |
| `creature_count` | Number of creatures |

**Location:** `spatial/biosignature.rs`

### L1 Classification

| Classification | Condition | Creature Response |
|----------------|-----------|-------------------|
| EMPTY | No creatures or below threshold | Safe passage |
| THREAT | Contains creature larger than self | Flee/avoid |
| PREY | Contains creatures smaller than self | Hunt/approach |
| CROWDED | Has mass, no clear threat/prey | Avoid (default) |

**Location:** `perception/classification.rs`

---

## Integration

- `HierarchicalGrid::aggregate_l1()` runs each tick after L0 rebuild
- Perception system checks L1 classification before expensive L0 entity scans
- Size domination threshold: `my_mass * 0.05` (5% rule)

---

## Implementation Files

| File | Purpose |
|------|---------|
| `spatial/hierarchical.rs` | HierarchicalGrid resource combining L0/L1 |
| `spatial/coarse_grid.rs` | CoarseGrid (L1) implementation |
| `spatial/biosignature.rs` | BioSignature struct and aggregation |
| `perception/classification.rs` | L1Classification enum and classifier |
