#!/bin/bash
# Run only behavior spec tests (excludes performance specs)
cd "$(dirname "$0")/../apps/simulation" && SPEC_CATEGORY=behavior cargo test --release --features dev-tools --test spec_runner "$@"
