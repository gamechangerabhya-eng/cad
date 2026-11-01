#!/usr/bin/env python3
"""Source-contract checks for P1 precision and offset gap behavior."""
from pathlib import Path
ROOT = Path(__file__).resolve().parents[1]
GEOM = ROOT / "source/cadcraft/crates/geom/src"
ENGINE = ROOT / "source/cadcraft/crates/engine/src"
PRED = (GEOM / "predicates.rs").read_text()
VEC = (GEOM / "vec.rs").read_text()
CURVE = (GEOM / "curve.rs").read_text()
SPLINE = (GEOM / "spline_fit.rs").read_text()
REGION = (GEOM / "region.rs").read_text()
OFFSET = (ENGINE / "cmd/modify/offset.rs").read_text()
FILLET_PL = (ENGINE / "cmd/modify2/fillet_pline.rs").read_text()
SYSVARS = (ENGINE / "sysvars/table.rs").read_text()
TESTS = (ENGINE / "cmd/modify2/tests.rs").read_text()
INTERSECT = (GEOM / "intersect.rs").read_text()
NURBS = (GEOM / "nurbs.rs").read_text()
GEOM_TESTS = (GEOM / "tests.rs").read_text()

checks = {
    "robust orientation predicate uses an exact binary64 integer determinant": "pub fn orient2d" in PRED and "struct ExactSigned" in PRED and "struct ExactProduct" in PRED,
    "proper segment crossing predicate is available": "pub fn segments_properly_cross" in PRED,
    "distance-based collinearity predicate is available": "pub fn collinear_with_tolerance" in PRED,
    "geometry crate exports robust predicates": "pub use predicates::{collinear_with_tolerance, orient2d, segments_properly_cross}" in (GEOM / "lib.rs").read_text(),
    "2D vector normalization no longer uses fixed EPS cutoff": "pub fn normalized(self) -> Vec2" in VEC and "let scale = self.x.abs().max(self.y.abs())" in VEC and "if l > crate::EPS" not in VEC[VEC.index("pub fn normalized(self) -> Vec2"):VEC.index("pub fn perp(self)")],
    "circle through three points uses drawing-space collinearity": "collinear_with_tolerance(p1, p2, p3, tolerance.linear)" in CURVE,
    "circle through three points solves in a normalized frame": "let un = u / scale" in CURVE,
    "spline fit tangent sanitization accepts tiny nonzero directions": "v.x != 0.0 || v.y != 0.0" in SPLINE,
    "spline knot parameterization no longer imposes a 1e-12 distance floor": "d.max(f64::MIN_POSITIVE)" in SPLINE and "d.sqrt().max(f64::MIN_POSITIVE)" in SPLINE,
    "OFFSETGAPTYPE exposes 0, 1 and 2": '("OFFSETGAPTYPE", Kind::Choice(&[0, 1, 2]), Def::Int(0))' in SYSVARS,
    "OFFSET gap style uses finite intersections for fillet/chamfer": "gap_type == 0, &tolerance" in OFFSET,
    "round gap connector computes a signed bulge": "round_join_bulge(\n                    source_vertex,\n                    segment_end,\n                    next_start" in OFFSET and "sweep_direction" in OFFSET,
    "chamfer gap connector is represented as a straight segment": "let connector_bulge = if gap_type == 1" in OFFSET,
    "OFFSET command reads the drawing's gap setting": "offset_kind_with_gap_type(&e.kind, dist.abs(), side, gap_type)" in OFFSET and "offset_gap_type(s)" in OFFSET,
    "OFFSET tests cover all three gap modes": '"OFFSETGAPTYPE", "value": 0' in TESTS and '"OFFSETGAPTYPE", "value": 1' in TESTS and '"OFFSETGAPTYPE", "value": 2' in TESTS,
    "OFFSET tests reject invalid gap mode": "offset_gaptype_rejects_values_outside_supported_range" in TESTS,
    "circle and arc OFFSET no longer use a fixed world-unit radius cutoff": "r > tolerance.linear" in OFFSET,
    "OFFSET rejects non-finite distances and side points before geometry construction": "!dist.is_finite() || dist <= 0.0 || !side.is_finite()" in OFFSET,
    "round OFFSET connector uses the shared angular tolerance": "sweep.abs() <= angular_tolerance" in OFFSET,
    "microscopic circle OFFSET has a regression test": "circle_offset_preserves_valid_microscopic_geometry" in OFFSET,
    "OFFSET rejects non-finite numeric inputs in regression tests": "offset_rejects_non_finite_or_non_positive_distance" in OFFSET,
    "spline fit length scaling preserves tiny nonzero geometry": "chord_length(fit).max(f64::MIN_POSITIVE)" in (GEOM / "spline_fit.rs").read_text(),
    "orientation uses exact significand alignment and integer products": "fn decompose_f64" in PRED and "fn multiply(a: &ExactSigned, b: &ExactSigned)" in PRED,
    "orientation fast filter falls back on overflow and underflow": "products_safe" in PRED and "f64::MIN_POSITIVE" in PRED,
    "orientation has adversarial mixed-exponent regression tests": "exact_integer_predicate_handles_mixed_subnormal_and_maximum_exponents" in PRED and "exact_integer_predicate_resolves_ulp_separation_at_max_scale" in PRED,
    "region kernel exposes single-winding loop repair": "pub fn repair_single_winding_loop" in REGION,
    "OFFSET routes closed line-only crossings through planar repair": "repair_winding_loops(&points, tolerance.linear)" in OFFSET,
    "planar repair rejects ambiguous multiple filled faces": "(loops.len() == 1).then(|| loops.pop()).flatten()" in REGION,
    "planar repair regression tests cover bow-tie rejection and simple loops": "repairs_a_single_winding_self_crossing_loop" in REGION and "keeps_a_simple_closed_loop_unchanged_as_one_face" in REGION,
    "planar repair exposes multi-loop face extraction": "pub fn repair_winding_loops" in REGION and "repair_winding_loops_returns_all_bow_tie_faces" in REGION,
    "OFFSET has a multi-result polyline path": "offset_polyline_with_gap_type_multi" in OFFSET and "repair_winding_loops(&points, tolerance.linear)" in OFFSET,
    "OFFSET command emits every repaired loop": "for vertices in loops" in OFFSET and '"handles"' in OFFSET,
    "spline intersections use homogeneous Bezier decomposition": "homogeneous_bezier_pieces" in INTERSECT and "pub fn homogeneous_bezier_pieces" in NURBS,
    "spline intersections use Bernstein sign variation and recursive subdivision": "fn bernstein_sign_variations(values: &[f64])" in INTERSECT and "bernstein_sign_variations(&normalized)" in INTERSECT and "fn isolate(values: &[f64]" in INTERSECT and "split_bernstein" in INTERSECT,
    "spline intersection regression covers three roots in one span": "spline_line_bernstein_isolation_finds_three_roots_in_one_bezier_span" in GEOM_TESTS,
    "ellipse line solver normalizes coefficients and checks residual": "coordinate_scale" in INTERSECT and "residual_bound" in INTERSECT,
    "round OFFSET fails closed instead of silently degrading to a straight join": "round_join_bulge(" in OFFSET and "tolerance.angular,\n                )?" in OFFSET,
    "round OFFSET sweep direction accounts for signed offset side": "-(source_vertex - before).cross(after - source_vertex) * d" in OFFSET,
    "open polyline round OFFSET has a geometric regression case": "open_polyline_round_gap_type_emits_a_circular_join" in OFFSET,
    "round OFFSET rejects undefined sweep instead of emitting a straight connector": "round_gap_type_rejects_a_corner_without_a_defined_sweep" in OFFSET,
    "Bernstein root pruning preserves every finite nonzero coefficient sign": "let sign = if *value > 0.0" in INTERSECT and "cutoff" not in INTERSECT[INTERSECT.index("fn bernstein_sign_variations"):INTERSECT.index("/// Intersections of a rational or non-rational B-spline")],
    "Bernstein sign variation has small-coefficient regression coverage": "small_nonzero_coefficients_are_not_erased_by_an_epsilon_cutoff" in INTERSECT,
    "polyline FILLET uses drawing-aware corner and dedup tolerances": "ToleranceContext::from_bounds" in FILLET_PL and "tolerance.linear" in FILLET_PL and "push_point(&mut out" in FILLET_PL,
    "polyline FILLET keeps nonzero small bulges as real arcs": "if inner.bulge != 0.0" in FILLET_PL,
    "polyline FILLET has multi-scale corner regression coverage": "polyline_corner_fillet_scales_with_drawing_units" in FILLET_PL,
    "polyline FILLET rejects negative radius": "polyline_corner_fillet_rejects_negative_radius" in FILLET_PL,
    "bulge conversion avoids fixed world-unit and angle cutoffs": "bulge == 0.0" in CURVE and "a.near(b, EPS)" not in CURVE[CURVE.index("pub fn bulge_to_arc"):CURVE.index("/// Bulge for an arc")],
    "bulge conversion has tiny-angle and microscopic-chord regression coverage": "bulge_conversion_preserves_small_angles_and_small_drawing_units" in GEOM_TESTS,
    "arc segment offset accepts microscopic radii and rejects overflow": "r.is_finite() && r > 0.0" in CURVE and "arc_segment_offset_preserves_small_radii_and_rejects_unrepresentable_results" in GEOM_TESTS,
}
for name, ok in checks.items():
    print(f"{'PASS' if ok else 'FAIL'} - {name}")
print(f"\n{sum(checks.values())}/{len(checks)} P1 source checks passed.")
if not all(checks.values()):
    raise SystemExit(1)
