# AR CAD P1 curve-intersection update

## Changes in this revision

- Added `Spline::homogeneous_bezier_pieces()` to expose exact homogeneous Bézier span controls for positive-weight splines.
- Replaced fixed 48-sample spline/line root discovery with Bernstein-form coefficient sign-variation pruning and recursive de Casteljau subdivision. This catches multiple crossings within one knot span and even-multiplicity/tangent candidates without depending on display tessellation.
- Added safeguarded Newton refinement and drawing-tolerance residual checks for isolated candidates.
- Added a regression case with three roots in one cubic Bézier span.
- Reworked ellipse/line calculations into a normalized coordinate frame, added coefficient roundoff slack, and residual-checked the resulting points.
- Added source acceptance checks for the new solver paths.

## Numerical guarantees and limits

The spline solver is a substantially stronger root-isolation method, but it is **not yet a formal certificate**. Coefficients and subdivision are evaluated in ordinary binary64 arithmetic, without outward-rounded interval bounds. It also rejects rational splines with non-positive or non-finite weights because the convex-hull/sign-variation argument requires positive weights.

The ellipse solver controls overflow and checks residuals against a computed tolerance, but it is **not a formally certified interval solver**. A rigorous certificate needs interval arithmetic with directed rounding and explicit conditioning bounds for the ellipse parameter recovery.

## Still outstanding

- Curved and open OFFSET topology, preservation of arc segments through planar arrangement splitting, and general overlapping/nested intersection cases.
- Full FILLET coverage across all line/arc/polyline pairings and multi-segment corner cases.
- Formal interval-certified spline and ellipse root enclosures.
- Remaining production fixed-epsilon decisions in curve, spline-fit, region, and NURBS code. Some constants are algorithmic/parameter thresholds or test tolerances; each needs a documented classification before changing it safely.
- Rust workspace compilation and runtime tests. Cargo is unavailable in the current environment.

## Validation performed

`./tools/test-ar-cad.sh` completed successfully with source-level acceptance checks, shell syntax checks, and Python compile checks. It skipped Rust tests by default. An explicit `RUN_RUST_TESTS=1` attempt exited with code 2 because Cargo is unavailable. No Rust compilation, Android build, APK installation, or device test is claimed.
