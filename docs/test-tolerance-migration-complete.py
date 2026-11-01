#!/usr/bin/env python3
"""Source acceptance checks for shared-tolerance migration follow-up."""
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
checks = [
    ("source/cadcraft/crates/engine/src/snap.rs", "ToleranceContext::from_bounds(tolerance_bounds)", "OSNAP derives one tolerance from nearby drawing geometry"),
    ("source/cadcraft/crates/engine/src/snap.rs", "intersect_segments_with_tolerance(&fix_circle(a), &fix_circle(b), false, &snap_tolerance)", "OSNAP exact segment intersections share the operation tolerance"),
    ("source/cadcraft/crates/engine/src/snap.rs", "intersect_segments_with_tolerance(a, b, true, &snap_tolerance)", "OSNAP apparent intersections share the operation tolerance"),
    ("source/cadcraft/crates/engine/src/snap.rs", "near_prims.get(i), near_prims.get(j), extend, &snap_tolerance", "Ellipse/spline OSNAP intersections share the operation tolerance"),
    ("source/cadcraft/crates/engine/src/cmd/modify/trim.rs", "fn modify_tolerance(target: &EntityKind, cut: &[Segment])", "TRIM/EXTEND derive one tolerance from target and cutting edges"),
    ("source/cadcraft/crates/engine/src/cmd/modify/trim.rs", "intersect_segments_with_tolerance(a, b, false, tolerance)", "TRIM/EXTEND use tolerance-aware intersections"),
    ("source/cadcraft/crates/engine/src/cmd/modify/fillet.rs", "line_line_infinite_with_tolerance", "Line FILLET/CHAMFER use shared tolerance for corner intersections"),
    ("source/cadcraft/crates/engine/src/cmd/modify/fillet.rs", "radius <= tolerance.linear", "Line FILLET uses drawing-aware degeneracy checks"),
    ("source/cadcraft/crates/engine/src/cmd/modify2/curve_edits.rs", "tangent_circles_with_tolerance", "Curved FILLET uses an explicit shared tolerance"),
    ("source/cadcraft/crates/engine/src/cmd/modify2/curve_edits.rs", "intersect_segments_with_tolerance(&ray, c, false, &tolerance)", "Spline EXTEND uses a shared operation tolerance"),
    ("source/cadcraft/crates/engine/src/cmd/modify/offset.rs", "intersect_segments_with_tolerance(a, b, extend, tolerance)", "OFFSET polyline joins use tolerance-aware intersections"),
    ("source/cadcraft/crates/geom/src/fillet.rs", "pub fn tangent_circles_with_tolerance", "Geometry kernel exposes a tolerance-aware fillet solver"),
    ("source/cadcraft/crates/geom/src/region.rs", "pub fn boundary_at_with_tolerance", "Region boundary operations expose a shared-tolerance API"),
    ("source/cadcraft/crates/geom/src/region.rs", "pub fn snap_ends_with_tolerance", "Region endpoint snapping exposes a shared-tolerance API"),
    ("source/cadcraft/crates/geom/src/tolerance.rs", "tolerance_remains_finite_across_extreme_coordinate_scales", "Extreme coordinate scale regression test is present"),
    ("source/cadcraft/crates/geom/src/fillet.rs", "explicit_tolerance_api_preserves_fillet_solution", "Tolerance-aware fillet regression test is present"),
    ("source/cadcraft/crates/geom/src/region.rs", "region_operations_accept_one_shared_tolerance_context", "Region shared-tolerance regression test is present"),
    ("source/cadcraft/crates/geom/src/curve.rs", "pub fn closest_with_tolerance", "Ellipse closest-point refinement accepts shared tolerance"),
    ("source/cadcraft/crates/geom/src/spline.rs", "pub fn closest_with_tolerance", "Spline closest-point refinement accepts shared tolerance"),
    ("source/cadcraft/crates/geom/src/tests.rs", "ellipse_and_spline_closest_accept_shared_operation_tolerance", "Large-coordinate curve closest-point regression is present"),
]
failed = 0
for rel, needle, label in checks:
    if needle in (ROOT / rel).read_text():
        print(f"PASS - {label}")
    else:
        print(f"FAIL - {label}")
        failed += 1
print(f"\n{len(checks)-failed}/{len(checks)} tolerance migration source checks passed.")
raise SystemExit(1 if failed else 0)
