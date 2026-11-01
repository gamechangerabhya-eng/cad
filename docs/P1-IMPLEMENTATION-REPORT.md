# AR CAD P1 — 2D Geometry Correctness and Precision

## Implemented in this revision

### Robust predicates and scale-aware vector math
- Added `orient2d`, using scaled coordinate differences and fused multiply-add residuals to make orientation decisions more reliable near collinearity.
- Added `segments_properly_cross` for strict segment crossings; endpoint touches and collinear overlaps are intentionally handled by the caller's distance/intersection policy.
- Added `collinear_with_tolerance`, expressing collinearity in drawing units rather than raw cross-product area.
- Exported the predicates from `cadcraft-geom` and used them in offset self-crossing validation and three-point circle construction.
- Changed `Vec2::normalized` to preserve valid microscopic vectors and avoid overflow when normalizing very large vectors.
- Reworked three-point circle construction to solve in a normalized local frame and apply a drawing-derived collinearity tolerance.

### Spline fit controls
- Removed the fixed `1e-12` chord-distance floor from chord and square-root chord parameterization.
- Tiny nonzero fit tangents are now normalized and preserved rather than silently dropped.
- Existing `FitOptions` continues to support chord, square-root chord and uniform knots, start/end tangents, fit tolerance, and periodic closed splines.

### Polyline OFFSET gap behavior
- Added the drawing variable `OFFSETGAPTYPE` with validated values 0, 1 and 2.
- `0`: preserve the existing extended/miter join behavior.
- `1`: add a circular fillet connector when the two offset endpoints lie on the expected offset radius; where a circular fillet cannot be safely constructed, use a straight connector rather than emitting invalid geometry.
- `2`: add a straight chamfer connector.
- The interactive command, preview and JSON command path all read the drawing's setting.
- Existing collapsed-segment, non-finite-vertex and self-intersection guards remain in place.

### Regression coverage added
- Orientation winding and collinearity tests.
- Scale/translation tests at tiny, ordinary and large coordinate magnitudes.
- Proper crossing versus endpoint-touch/collinear-overlap tests.
- Vec2 normalization tests for microscopic and very large vectors.
- Spline tiny-fit-spacing and tiny-tangent tests.
- OFFSETGAPTYPE tests for all three modes and invalid values.

## Validation performed

The unified source QA runner completed successfully:
- Step 2: 18/18
- Step 3: 25/25
- Steps 4–6: 14/14
- Steps 7–10: 17/17
- Geometry priority: 14/14
- Tolerance migration: 20/20
- Precision safety: 11/11
- New P1 source contracts: 16/16

Total: **135/135 source-level assertions passed**. Shell syntax checks and Python compileall also passed.

## Not verified

- Rust compilation, rustfmt and Rust unit/integration tests were not run: `cargo`, `rustc` and `rustfmt` are not installed in this environment.
- Android APK compilation and physical-device tests were not run.
- The new engine OFFSETGAPTYPE integration tests are written but not executed.

## Remaining P1 gaps — do not mark P1 complete yet

1. **Offset self-intersection resolution:** current code detects and safely rejects self-intersecting offset results; it does not yet polygonize/repair all valid loops and islands.
2. **OFFSETGAPTYPE fillet completeness:** circular fillets are constructed for supported radius geometry; unsupported joins currently fall back to a straight connector. More arc/polyline corner cases need dedicated treatment and tests.
3. **Predicate scope:** the new robust orientation predicate is a floating-point compensated predicate, not a formally exact arbitrary-precision predicate for every possible IEEE-754 input.
4. **Curve solver error bounds:** ellipse/spline solvers need an explicit documented maximum-error contract and dense adversarial reference corpus, beyond current numerical regression tests.
5. **Full operation migration:** all remaining legacy fixed-epsilon uses in the 2D command/geometry code still need a targeted audit, especially dimension/measurement, PEDIT, hatch boundaries and constraints.
6. **End-to-end validation:** all new Rust tests must compile and pass in CI before these changes can be considered functionally verified.

P1 is therefore **substantially implemented, but not complete**. Continue with self-intersection repair, fillet coverage, remaining fixed-epsilon audit and Rust CI before moving its status to complete.
