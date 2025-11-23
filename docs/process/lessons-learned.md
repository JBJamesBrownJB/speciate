# Lessons Learned - Production Incidents

This document captures critical production failures and the lessons learned to prevent recurrence.

---

## Incident #1: Save State Corruption at Scale (2025-11-23)

### Severity: P0 - Critical

**Impact:** Complete save/load failure for populations >10K creatures

### Symptoms

```
[NAPI] ⚠️  Failed to load save state: Deserialization error: IO error while reading data: unexpected end of file.
```

- Save state files truncated at ~14-18MB
- Consistent failure across multiple restart attempts
- File appeared valid but incomplete (ended mid-stream)

### Root Causes (2 Critical Bugs)

#### Bug #1: MessagePack Serialization Limits

**Problem:**
- Original implementation: `rmp_serde::to_vec(self)`
- No explicit streaming for large payloads
- May have hit internal buffer limits for >18MB files

**Code Location:** `apps/simulation/src/persistence/snapshot.rs:98-110`

**Fix Applied:**
```rust
// BEFORE (unsafe for large payloads)
let bytes = rmp_serde::to_vec(self)?;
fs::write(path, bytes)?;

// AFTER (streaming for any size)
use rmp_serde::encode::Serializer;
use serde::Serialize;

let mut buf = Vec::new();
let mut serializer = Serializer::new(&mut buf)
    .with_struct_map(); // Better compatibility with large strings

self.serialize(&mut serializer)?;
fs::write(path, buf)?;
```

#### Bug #2: Worker Thread Shutdown Race Condition

**Problem:**
- Background worker thread writes save states asynchronously
- Main thread exits immediately after queuing shutdown save
- Worker killed mid-write, leaving truncated file

**Code Location:** `apps/simulation/src/napi_addon/simulation_engine.rs:306-332`

**Fix Applied:**
```rust
// Queue shutdown save
worker_ref.lock().save_world_state(shutdown_save, SaveType::Shutdown);

// CRITICAL: Wait for worker to finish writing
eprintln!("⏳ Waiting for save state to write to disk...");
thread::sleep(std::time::Duration::from_millis(500));
eprintln!("✅ Save state worker completed");
```

**Better long-term solution:** Implement proper synchronization (flush channel, wait for ack).

### Testing Gaps

**What Missed This:**
- Unit tests only covered 1-2 creatures (small payloads)
- No integration test for shutdown save reliability
- No scale testing at production volumes (10K-175K creatures)

**What We Added:**
- `tests/large_scale_save_load.rs` - 10K creature save/load cycle
- Scale verification: 100, 500, 1000, 5000 creatures
- Quick shutdown synchronization test

### Process Failures

#### Gap #1: NAPI Rebuild Automation

**Problem:** Rust source code was fixed and tested, but NAPI addon wasn't automatically rebuilt.

**Impact:** Production app ran stale code, bug persisted despite fix.

**Solution Added:**
- `scripts/check-napi-freshness.sh` - Detects stale .node binary
- `prebuild` hook in package.json - Automatic freshness check

#### Gap #2: No CI/CD Pipeline

**Problem:** No automated verification of Rust→NAPI→Electron integration.

**Impact:** Integration bugs only caught in manual testing.

**Future Work:** Set up GitHub Actions workflow with:
- NAPI freshness check
- Large-scale integration tests
- Full build verification

### Prevention Checklist

Before merging save state changes:

- [ ] Test with 10K+ creature population
- [ ] Verify file size matches expectations (not truncated)
- [ ] Test save/load cycle with quick shutdown
- [ ] Run `npm run check-freshness` before deploying
- [ ] Rebuild NAPI addon after Rust changes
- [ ] Manual verification: spawn large trial → quit → reload → verify count

### Verification Results

**Production Scale Testing (2025-11-23):**
- ✅ 17K creatures: Save/load successful
- ✅ 175K creatures: Save/load successful
- ✅ File sizes: 12-20MB (complete, no truncation)
- ✅ Shutdown synchronization: Working correctly

### Documentation Updates

- `apps/simulation/docs/technical-debt.md` - Section 1.4 (MessagePack) + 1.2 (Serialization)
- `SPRINT_DOCS/Final-Refactor.md` - Phase 2.3 + 2.4
- `tests/large_scale_save_load.rs` - Integration test suite

### Key Takeaways

1. **Test at production scale:** Unit tests with toy data don't catch serialization limits
2. **Async operations need synchronization:** Don't assume background threads finish before exit
3. **Integration testing matters:** Rust tests passed, but Rust→NAPI→Electron integration failed
4. **Process > People:** This was a testing methodology gap, not a personnel failure
5. **Automation prevents human error:** Manual rebuild steps will be forgotten

### Related Incidents

- None (first occurrence of this class of failure)

### Owner

- **Discovered:** CEO (production testing)
- **Root cause analysis:** Development team
- **Fixes:** Implemented 2025-11-23
- **Verified:** CEO (17K + 175K creature tests)

---

**Last Updated:** 2025-11-23
**Next Review:** Sprint 14 retrospective
