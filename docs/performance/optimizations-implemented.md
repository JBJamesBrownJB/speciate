# Implemented Optimizations Log

---

## 2025-11-16: Skip Catatonic Crits in Perception

**Problem:** Perception system computed neighbors for ALL crits, including inactive ones.

**Solution:** Added `BehaviorMode::is_active()` check to skip catatonic crits in AI systems.

**Notes:** Pattern reusable across all AI systems. No archetype thrashing (enum vs marker).

---
