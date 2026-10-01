#!/usr/bin/env bash
# T2-BOOT-06 budget harness. Builds the release profile, then measures the four
# Fase-0-measurable REQ-02 metrics on the empty app: binary size, cold start, and RSS of
# the main process plus every descendant WebView process. Writes bench-report.json
# (machine-readable) next to a human-readable summary on stdout.
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

echo "=== T2-BOOT-06 bench harness ===" >&2
echo "[1/3] cargo build --release --workspace" >&2
cargo build --release --workspace

CATERMCTL="target/release/caframeworkctl"
if [ -f "target/release/caframeworkctl.exe" ]; then
    CATERMCTL="target/release/caframeworkctl.exe"
fi

echo "[2/3] caframeworkctl bench run --json -> bench-report.json" >&2
"$CATERMCTL" bench run --json | tee "$PROJECT_ROOT/bench-report.json"

echo "" >&2
echo "[3/3] caframeworkctl bench run (ringkasan terbaca manusia)" >&2
"$CATERMCTL" bench run
