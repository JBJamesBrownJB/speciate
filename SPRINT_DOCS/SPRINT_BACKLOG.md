# Sprint Backlog

## Sprint 16: Basic DNA

### Phase 1: DNA Component (Rust)

| Task | Status | Notes |
|------|--------|-------|
| 1.1 Create `dna/mod.rs` with Dna struct | Pending | size_gene, fov_gene (f32, 0-1) |
| 1.2 Create `dna/expression.rs` | Pending | express_size(), express_fov() |
| 1.3 Export module from creatures/mod.rs | Pending | pub mod dna; pub use dna::* |
| 1.4 Write Dna tests (RED phase) | Pending | clamp, default, serde roundtrip |
| 1.5 Write expression tests (RED phase) | Pending | min/max/default for each gene |

### Phase 2: CritBuilder Integration

| Task | Status | Notes |
|------|--------|-------|
| 2.1 Add `dna` field to CritBuilder | Pending | Option<Dna> |
| 2.2 Add `with_dna()` method | Pending | Derives size/fov from DNA |
| 2.3 Add Dna to CritBundle | Pending | Include in creature bundle |
| 2.4 Write CritBuilder tests (RED phase) | Pending | derives_size, derives_fov, backward_compat |

### Phase 2.5: Perception Allometric Scaling

| Task | Status | Notes |
|------|--------|-------|
| 2.5.1 Add SIZE_ALLOMETRY constants | Pending | EXPONENT=0.35, REFERENCE=0.5 |
| 2.5.2 Update calculate_range() | Pending | Add (size/ref)^0.35 factor |
| 2.5.3 Write allometry tests (RED phase) | Pending | 5m sees ~2.2x farther than 0.5m |

### Phase 3: Command Executor Fix

| Task | Status | Notes |
|------|--------|-------|
| 3.1 Parse DNA JSON in executor | Pending | serde_json::from_value() |
| 3.2 Use CritBuilder instead of minimal spawn | Pending | BUG FIX - missing components |
| 3.3 Write executor tests (RED phase) | Pending | DNA applies, default works |

### Phase 4: Dev-UI Integration

| Task | Status | Notes |
|------|--------|-------|
| 4.1 Add DnaData interface to types.ts | Pending | size_gene, fov_gene |
| 4.2 Add size slider to SpawnForm | Pending | 0-100%, show phenotype |
| 4.3 Add FOV slider to SpawnForm | Pending | 0-100%, show phenotype |
| 4.4 Add "Randomize DNA" checkbox | Pending | Generate random genes on spawn |
| 4.5 Update DevToolsApp handleSpawn | Pending | Pass DNA to command |

---

## Completed Sprints

- Sprint 15: ECS Optimizations (Rayon parallelization, vision refactor)
- Sprint 14: Zero-copy double-buffer architecture
- Sprint 13: IPC refactoring
