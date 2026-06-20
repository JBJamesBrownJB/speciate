<!-- See ../AGENTS.md (and root /AGENTS.md) for global rules. -->

# specs/ — Specification Tests (point-of-edit reminder)

`specs/{behavior,physics,performance}/*.toml` are specification tests. **Changing ANY file here requires explicit human approval** — never edit, relax, or delete a spec to make a build pass.

Run: `cargo test --release --features dev-tools --test spec_runner -- --nocapture` (or `scripts/run-specs.sh` from repo root). The `dev-tools` feature is mandatory or no specs run (`tests/spec_runner.rs:7`). Filter a category with `SPEC_CATEGORY=physics`. Perf budget: tick < 50 ms.
