# 📖 Archive — Reference / ADRs (What We Tried and Abandoned)

> **Category: 📖 REFERENCE.** This folder is the project's architectural
> decision record (ADR) graveyard: approaches that were **designed, often built,
> and then abandoned or superseded.** Nothing here is current. It is preserved
> on purpose — the *why we stopped* is as valuable to a reading engineer as the
> *what we shipped*.
>
> For the documentation category legend and the reference-vs-lifecycle split,
> see [`../documentation-standards.md`](../documentation-standards.md). For the current architecture, see
> [`../architecture/`](../architecture/).

**Legend:** 📖 REFERENCE · 💡 IDEAS · 🚧 IN PROGRESS (NOW) · 📋 PLANNED · ✅ DONE · 🌙 DREAMLAND

---

## Purpose

When Speciate replaces an architecture, the old one is not deleted — it is moved
here with a short post-mortem. This keeps the honesty mandate intact: a reader
can see what was attempted, what it cost, and what replaced it. Each subfolder
documents one abandoned/superseded decision.

---

## Contents

| Subfolder | Status | What it records |
|-----------|--------|-----------------|
| [`stdio/`](./stdio/) | Superseded (Sprint 13) | The legacy stdout/stdin MessagePack IPC, replaced by the zero-copy NAPI-RS bridge. Contains the old design doc plus the retired Rust source (`hooks.rs`, `stdin_reader.rs`, `main.rs.bak`). |
| [`dual-tick/`](./dual-tick/) | Abandoned (Sprint 11) | The dual-tick (physics-fast / AI-slow) scheduling experiment. Abandoned because sequential single-thread execution still pays the worst-case combined cost — no benefit without true parallelism. Frontend interpolation solved the real problem instead. |
| [`perception-skip/`](./perception-skip/) | Removed (Sprint 16) | Dynamic neighbour perception-skipping, removed in favour of always-fresh perception (eliminated stale-neighbour-data concerns). |
| [`superseded-by-phase-c/`](./superseded-by-phase-c/) | Superseded | Earlier perception-spreading / time-slicing designs replaced by a later phase. No subfolder index; two short design notes. |
| `visuals/` | Empty / vestigial | Placeholder folder, currently empty. Retained only to avoid breaking links; no archived content yet. |

> Each non-empty subfolder above (except `superseded-by-phase-c/`) carries its
> own `README.md` with the full post-mortem. Start there for detail.

---

## Reading these

- **Treat everything here as historical.** If an archive doc and the code under
  `apps/` disagree, the code is correct and the archive is simply old.
- **The lesson is the point.** Each post-mortem ends with what was learned (e.g.
  "tick separation buys nothing on a single thread"; "if targeting Electron, use
  NAPI from day one"). That is why these are kept rather than deleted.

---

## Related

- 📖 [`../architecture/`](../architecture/) — the architectures that *won* and are current.
- 📖 [`../process/`](../process/) and [`../incidents/`](../incidents/) — lessons and post-mortems from production incidents (distinct from abandoned-design records here).
