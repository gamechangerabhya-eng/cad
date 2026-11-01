#!/usr/bin/env python3
"""Source contract checks for scale-aware BREAK/JOIN and safe polyline OFFSET."""
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BREAK_JOIN = (ROOT / "source/cadcraft/crates/engine/src/cmd/modify/break_join.rs").read_text()
OFFSET = (ROOT / "source/cadcraft/crates/engine/src/cmd/modify/offset.rs").read_text()

checks = {
    "BREAK derives an operation tolerance from source geometry": "ToleranceContext::from_bounds(operation_bounds)" in BREAK_JOIN,
    "BREAK rejects degenerate lines using drawing tolerance": 'ln.len() <= tolerance.linear' in BREAK_JOIN,
    "BREAK line endpoints use normalized parameter tolerance": "parameter_tolerance" in BREAK_JOIN,
    "BREAK arc endpoint classification uses angular tolerance": "tolerance.angular" in BREAK_JOIN,
    "BREAK polyline cut uses drawing-unit tolerance": "tolerance.linear" in BREAK_JOIN,
    "JOIN derives tolerance from selected segment bounds": "ToleranceContext::from_bounds(chain_bounds)" in BREAK_JOIN,
    "JOIN endpoint matching uses shared tolerance": "let tol = tolerance.linear" in BREAK_JOIN,
    "OFFSET validates generated segment lengths": "segment.len() <= tolerance.linear" in OFFSET,
    "OFFSET excludes adjacent segment pairs from crossing test": "let adjacent = j == i + 1" in OFFSET,
    "OFFSET rejects non-finite generated vertices": "!vertex.p.is_finite() || !vertex.bulge.is_finite()" in OFFSET,
    "OFFSET detects non-adjacent crossings and routes closed line-only contours to planar repair": "intersect_segments_with_tolerance(&result_segments[i], &result_segments[j], false, &tolerance)" in OFFSET and "repair_winding_loops(&points, tolerance.linear)" in OFFSET,
}

for label, ok in checks.items():
    print(f"{'PASS' if ok else 'FAIL'} - {label}")

print(f"\n{sum(checks.values())}/{len(checks)} precision safety source checks passed.")
if not all(checks.values()):
    raise SystemExit(1)
