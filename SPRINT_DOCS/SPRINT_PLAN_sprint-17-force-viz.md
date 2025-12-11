# Sprint 17: Force Vector Visualization

**Branch:** `feat/sprint-17-force-viz`
**Started:** 2025-12-11
**Design Doc:** `docs/testing/ideas/force-visualisation.md`

---

## Goal

Implement force vector visualization to debug creature steering behavior. Answer "Why is it turning left?" by showing force vectors graphically and numerically.

---

## Key Outcomes

1. **ForceDebug component** in Rust backend, populated for selected creature only
2. **ForceOverlay** rendering net force vector in Portal (behind dev-tools toggle)
3. **Force breakdown** (seek, flee, separation, wander, net) displayed in CreatureInfoPanel

---

## Constraints

- All code behind `--dev-tools` feature flag (not shipped to players)
- Minimal bandwidth impact (only selected entity sends force data)
- Follow existing patterns (PerceptionOverlay, CreatureInfoPanel)

---

## Implementation Phases

### Phase 1: Rust Backend Instrumentation

- [ ] Create `ForceDebug` component with vector fields (seek, flee, separation, wander, net_force)
- [ ] Create `SelectedEntity` resource to track player selection
- [ ] Modify behavior systems to populate ForceDebug when entity matches selection
- [ ] All code behind `#[cfg(feature = "dev-tools")]`

### Phase 2: IPC Pipeline

- [ ] Extend EntitySnapshot with optional `force_debug` field
- [ ] Add `set_selected_entity` NAPI command
- [ ] Wire selection from Portal to Rust

### Phase 3: Frontend Rendering

- [ ] Create `ForceOverlay.ts` following PerceptionOverlay pattern
- [ ] Extend `CreatureInfoPanel.ts` with force breakdown section
- [ ] Add toggle to dev-tools menu

### Phase 4: Integration & Polish

- [ ] Selection flow: click creature → set_selected_entity → ForceDebug populated
- [ ] Deselection flow: click empty → clear selection → ForceOverlay clears
- [ ] Visual scaling and color coding
- [ ] Verify no performance regression

### Phade 5: Sprint close and summary

- [ ] Move docs/testing/ideas/force-visualisation.md -> docs/testing/done/force-visualisation.md
- [ ] Carry out /sprint-end tasks

---

## Success Criteria

- [ ] ForceDebug component added behind dev-tools feature flag
- [ ] Behavior systems populate ForceDebug for selected entity
- [ ] Selected entity ID communicated via NAPI
- [ ] ForceOverlay renders net force vector from creature center
- [ ] CreatureInfoPanel shows force breakdown
- [ ] Toggle in dev-tools menu enables/disables visualization
- [ ] All existing tests pass (230 unit + 10 spec + 309 portal)

---

## Files to Create/Modify

**New:**
- `apps/portal/src/rendering/ForceOverlay.ts`
- `apps/simulation/src/simulation/core/selected_entity.rs`

**Modified:**
- `apps/simulation/src/simulation/creatures/components.rs`
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

## References

- Design doc: `docs/testing/ideas/force-visualisation.md`
- Pattern: `apps/portal/src/rendering/PerceptionOverlay.ts`
- Pattern: `apps/portal/src/ui/CreatureInfoPanel.ts`
