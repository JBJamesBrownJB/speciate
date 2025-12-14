#!/bin/bash
# Run headless spec tests
cd "$(dirname "$0")/../apps/simulation" && cargo test --release --features dev-tools --test spec_runner"$@"
