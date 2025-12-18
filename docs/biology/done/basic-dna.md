# Basic DNA

**Status:** ✅ Implemented (Sprint 16)
**Branch:** `feat/sprint-16-basic-dna`

---

## Goal

Implement the foundational DNA component with size and FOV genes. This is the architectural foundation for ALL future genetics.

---

## Design Decisions

### Gene Representation

**Normalized genotype → Scaled phenotype**

```
Genotype: gene_value ∈ [0.0, 1.0]
Phenotype: trait = min + (gene × (max - min))
```

### Gene Bounds

| Gene | Genotype | Phenotype Min | Phenotype Max | Default Gene |
|------|----------|---------------|---------------|--------------|
| `size_gene` | 0.0 - 1.0 | 0.5m | 5.0m | **0.11** (~1.0m) |
| `fov_gene` | 0.0 - 1.0 | 45° | 340° | **0.46** (~180°) |

**Default genes chosen for backward compatibility** - existing 1.0m creatures with 180° FOV.

### Architecture

```
Dna Component (genotype: 0.0-1.0, immutable after birth)
    ↓ express_gene()
Existing Components (phenotype)
    ├── BodySize.length
    └── Perception.fov_angle, Perception.range
```

**Key insight:** BodySize and Perception ARE the phenotype. DNA just provides source values.

### Perception Allometric Scaling

**Range formula with diminishing returns:**

```
base_range = expressed_size × PERCEPTION_MULTIPLIER (10.0)
size_allometry = (expressed_size / SIZE_MIN)^0.35
fov_factor = (180 / expressed_fov)^0.4
range = base_range × size_allometry × fov_factor
```

**Result:** A 5m creature sees ~2.2x farther than a 0.5m creature (not 10x).

---

## Acceptance Criteria

- [x] `Dna` component with `size_gene` and `fov_gene` (f32, 0-1)
- [x] `Dna::default()` = (0.11, 0.46) for backward compatibility
- [x] `express_size(0.11)` ≈ 1.0m, `express_fov(0.46)` ≈ 180°
- [x] `CritBuilder.with_dna(Dna)` method
- [x] Creature size derived from DNA
- [x] Creature FOV derived from DNA
- [x] Perception range uses allometric scaling
- [x] FOV-to-range trade-off preserved
- [x] Dev-UI: size slider with phenotype preview
- [x] Dev-UI: FOV slider with phenotype preview
- [x] Dev-UI: "Randomize DNA" toggle
- [x] Serde round-trip test for DNA

---

## Zoologist Consultation Log

**Date:** 2025-12-18
**Consultant:** zoologist-tom

Key recommendations:
- Normalized 0-1 genotype → scaled phenotype expression
- Continuous f32 for quantitative traits (size, FOV)
- Size bounds: 0.5m - 5.0m (10x, genus-level variation)
- FOV bounds: 45° - 340° (matches existing perception.rs constants)
- Size-perception coupling: Allometric (size^0.35 for diminishing returns)
- Architecture: Flat Dna struct, existing components as phenotype
- Expression: Direct linear mapping (noise/environment deferred to future sprints)

---

## Implementation

See `SPRINT_DOCS/SPRINT_PLAN_sprint-16-basic-dna.md` for detailed implementation plan.
