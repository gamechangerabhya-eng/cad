#!/usr/bin/env bash
# AR CAD quality gate: deterministic source checks plus optional CADCraft Rust test suites.
set -Eeuo pipefail
ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

RUN_RUST_TESTS="${RUN_RUST_TESTS:-0}"

printf '\n== AR CAD source acceptance ==\n'
python3 docs/test-step2.py
python3 docs/test-step3.py
python3 docs/test-steps4-6.py
python3 docs/test-steps7-10.py
python3 docs/test-geometry-priority.py
python3 docs/test-tolerance-migration-complete.py
python3 docs/test-precision-safety-followup.py
python3 docs/test-p1-completion.py
python3 docs/test-critical-cad-gaps.py
python3 docs/test-ci-workflow.py

printf '\n== Shell and Python syntax ==\n'
while IFS= read -r -d '' script; do
  bash -n "$script"
  printf 'PASS - shell syntax: %s\n' "$script"
done < <(find . -path './source/cadcraft' -prune -o -type f -name '*.sh' -print0)
python3 -m compileall -q lib docs
printf 'PASS - Python compileall\n'

if [[ "$RUN_RUST_TESTS" == "1" ]]; then
  command -v cargo >/dev/null 2>&1 || { echo 'ERROR: RUN_RUST_TESTS=1 but cargo is unavailable.' >&2; exit 2; }
  printf '\n== CADCraft reliability and regression suites ==\n'
  cd "$ROOT/source/cadcraft"
  cargo test --locked -p cadcraft-dxf -p cadcraft-geom -p cadcraft-doc -p cadcraft-io -p cadcraft-engine -p cadcraft-ui-egui --no-fail-fast -- --skip gpu_viewport_stencil::clipped_viewport_contents_draw_only_inside_their_outline
else
  printf '\nSKIP - Rust tests (set RUN_RUST_TESTS=1 in a Rust-enabled environment)\n'
fi

printf '\nAR CAD quality gate completed.\n'
