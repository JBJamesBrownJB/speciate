# Sprint 9: Trials - Regression Testing System

**Branch:** `feat/sprint-9-trials-regression-testing`
**Start Date:** 2025-11-15
**Status:** 🚀 IN PROGRESS

---

## Sprint Goal

Implement a continuous regression testing system ("Trials") where we can record and replay key scenarios/starting conditions to test all future changes.

---

## Key Outcomes

1. **Scenario Library** - 2 initial trials implemented and runnable:
   - **Current spawning pattern trial** (needs naming - reveals behavior problems in default spawn)
   - **Crowd navigation trial** (creature weaving through obstacle grid, per `docs/testing/trials/crowd-navigation.md`)

2. **Trial Execution System** - Infrastructure to trigger and run trials on demand

3. **Reproducible Scenarios** - Deterministic outcomes using fixed RNG seeds and initial conditions

---

## Constraints

- **TDD Mandatory:** All trial infrastructure must have tests written first, no new features beyond trial system
- **Consult zoologist-tom:** For any biology-related decisions (creature behavior, DNA traits)
- **No Architecture Changes:** Work within existing ECS/IPC architecture, no major refactors

---

## Implementation Phases (Planned)

### Phase 1: Trial Infrastructure
- Core system for defining/running scenarios
- RNG seed control for deterministic outcomes
- Scenario data structures (initial positions, creature properties, target states)

### Phase 2: Spawning Pattern Trial
- Capture current default spawn as baseline trial
- Name the trial based on observed behavior issues
- Document expected vs actual outcomes

### Phase 3: Crowd Navigation Trial
- Implement obstacle grid scenario from `docs/testing/trials/crowd-navigation.md`
- Spawn grid of catatonic creatures (obstacles)
- Spawn seeking creature with target across grid
- Validate successful navigation without collisions

### Phase 4: Documentation & Integration
- Trial authoring guide
- Results logging system
- Integration instructions for future sprints

---

## Success Criteria

- ✅ Both trials can be triggered via command (CLI arg or config)
- ✅ Trials produce deterministic, reproducible results
- ✅ Trial outcomes are clearly documented (pass/fail criteria)
- ✅ All tests passing (100% pass rate)
- ✅ Documentation explains how to author new trials

---

## Out of Scope

- Automated CI/CD integration (future sprint)
- Visual regression testing (future sprint)
- Performance benchmarking (separate concern)
- Additional trial scenarios beyond the 2 specified

---

## Notes

- This sprint builds the **foundation** for continuous regression testing
- Future sprints will expand the trial library and add automation
- Trials should reveal emergent behavior issues (like the current spawning pattern problem)
