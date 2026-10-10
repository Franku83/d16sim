#!/usr/bin/env bash
# Only the isolated probe crate and the requested results directory are written.
# Exit 1 from the final checker means violated contracts, not a harness error.
set -euo pipefail
TASK_ROOT=$(cd "$(dirname "$0")/../.." && pwd)
RESULT_DIR=${1:-"$TASK_ROOT/target/independent-audit-results"}
mkdir -p "$RESULT_DIR"
RESULT_DIR=$(cd "$RESULT_DIR" && pwd)
cd "$TASK_ROOT"
cargo build --manifest-path audit/independent/Cargo.toml --release --locked --offline >"$RESULT_DIR/probes-build.log" 2>&1
for mode in engine subsystems dyno; do
  audit/independent/target/release/d16sim-independent-probes "$mode" >"$RESULT_DIR/probes-$mode.jsonl" 2>"$RESULT_DIR/probes-$mode.stderr"
done
D16_AUDIT_RESULTS="$RESULT_DIR" python3 audit/independent/check_contracts.py >"$RESULT_DIR/independent-contract-tests.log" 2>&1
