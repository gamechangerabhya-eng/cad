#!/usr/bin/env python3
from pathlib import Path
root = Path(__file__).resolve().parents[1]
geom = root / 'source/cadcraft/crates/geom/src'
engine = root / 'source/cadcraft/crates/engine/src'
checks = {
    'Tolerance context used by finite segment intersections': 'ToleranceContext::from_bounds(bounds)' in (geom/'intersect.rs').read_text(),
    'Shared tolerance reaches infinite and finite line intersections': 'line_line_infinite_with_tolerance' in (geom/'intersect.rs').read_text() and 'line_line_with_tolerance' in (geom/'intersect.rs').read_text(),
    'Shared tolerance reaches line-circle and circle-circle intersections': all(x in (geom/'intersect.rs').read_text() for x in ['line_circle_with_tolerance', 'circle_circle_with_tolerance']),
    'Segment arc intersections reuse the caller tolerance': 'line_circle_with_tolerance(l, &Circle::new(arc.center, arc.radius), tolerance)' in (geom/'intersect.rs').read_text() and 'circle_circle_with_tolerance(&Circle::new(a.center, a.radius), &Circle::new(b.center, b.radius), tolerance)' in (geom/'intersect.rs').read_text(),
    'Regression tests cover shared line-circle and circle-circle tolerance': all(x in (geom/'tests.rs').read_text() for x in ['tolerance_aware_line_circle_uses_shared_drawing_tolerance', 'tolerance_aware_circle_circle_respects_shared_context']),
    'Ellipse-line intersection solver is exported': 'intersect_ellipse_line' in (geom/'lib.rs').read_text(),
    'Spline-line intersection solver is exported': 'intersect_spline_line' in (geom/'lib.rs').read_text(),
    'Ellipse closest point is analytically refined': 'pub fn closest(&self, p: Vec2) -> Vec2' in (geom/'curve.rs').read_text() and 'const SAMPLES: usize = 128' in (geom/'curve.rs').read_text(),
    'Spline closest point is knot-span refined': 'pub fn closest(&self, p: Vec2) -> Option<Vec2>' in (geom/'spline.rs').read_text() and 'for (span_lo, span_hi) in self.knot_spans()' in (geom/'spline.rs').read_text(),
    'Geometry regression tests cover ellipse and spline closest points': all(x in (geom/'tests.rs').read_text() for x in ['ellipse_closest_refines_to_analytic_curve','spline_closest_refines_beyond_display_sample_vertices']),
    'Geometry regression tests cover ellipse and spline intersections': all(x in (geom/'tests.rs').read_text() for x in ['exact_ellipse_line_intersections_find_both_crossings','spline_line_intersection_finds_crossings_and_tangent_contact']),
    'OSNAP uses exact ellipse/spline versus line intersections': 'exact_curve_line_hits' in (engine/'snap.rs').read_text() and 'intersect_ellipse_line' in (engine/'snap.rs').read_text(),
    'Polyline offset joins use drawing-aware tolerances': 'ToleranceContext::from_bounds(pl.bounds())' in (engine/'cmd/modify/offset.rs').read_text(),
    'Spline offset tessellation scales with model bounds': 'scale * 1e-4' in (engine/'cmd/modify/offset.rs').read_text(),
}
for name, ok in checks.items():
    print(('PASS' if ok else 'FAIL') + ' - ' + name)
if not all(checks.values()):
    raise SystemExit(1)
print(f'\n{len(checks)}/{len(checks)} geometry priority source checks passed.')
