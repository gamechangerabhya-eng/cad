# AR CAD — 2D Geometry Enhancement Update (2026-11-01)

## Scope of this update

APK generation and physical-device validation are intentionally deferred. This update focuses on source-level 2D geometry correctness and tolerance handling.

## Implemented source changes

### OFFSET round joins

- Corrected the signed sweep selection for an outside corner. The source turn and signed offset side now select the sweep together.
- `OFFSETGAPTYPE=1` now fails the operation if a circular connector cannot be constructed. It no longer silently substitutes a straight connector.
- Added an open right-angle polyline regression case that checks for a quarter-circle bulge.

### Spline-line intersection candidate isolation

- Extracted Bernstein sign-variation counting into a dedicated helper.
- Removed the scale-relative epsilon cutoff that previously treated small nonzero coefficients as zero. Such a cutoff can erase sign changes and incorrectly prune a root interval.
- Endpoint root classification now accepts exact zero coefficients only.
- Added regression tests for tiny nonzero coefficients, exact zero coefficients and non-finite coefficients.

This improves candidate retention but does not create a formal interval-arithmetic certificate. Outward-rounded interval bounds are still required for certified root isolation.

### Polyline FILLET tolerance handling

- Threaded a drawing-derived `ToleranceContext` through corner construction and point deduplication.
- Replaced fixed world-unit thresholds in the polyline FILLET helper with drawing-space linear and angular tolerances.
- A nonzero bulge is now treated as an arc rather than being discarded by a fixed `1e-12` threshold.
- Added scale-regression coverage at `1e-9`, `1` and `1e9` drawing scales.

### Bulge-to-arc conversion

- Removed fixed world-unit chord and angle cutoffs.
- Exact zero bulge and coincident endpoints remain degenerate.
- Nonzero arcs are accepted when their radius, centre and angles are representable and finite; unrepresentable cases fail safely.
- Added small-angle and microscopic-chord regression tests.
- Arc-segment OFFSET now rejects only collapsed or non-finite radii rather than applying a fixed world-unit `EPS` floor; added microscopic-radius and overflow regression cases.

## Validation performed

Command: `./tools/test-ar-cad.sh`

- Step 2 source checks: 18/18
- Step 3 source checks: 25/25
- Steps 4–6 source checks: 14/14
- Steps 7–10 source checks: 17/17
- Geometry priority source checks: 14/14
- Tolerance migration source checks: 20/20
- Precision safety source checks: 11/11
- P1 source checks: 49/49
- Total source-level assertions: **168/168**
- Shell syntax checks: passed
- Python compileall: passed

The unified script skips Rust tests unless `RUN_RUST_TESTS=1` is set. Cargo/rustc are not installed in this environment, so the newly added Rust unit tests have not been compiled or executed. This update is therefore source-validated, not runtime-verified.

## Explicitly not completed by this update

- General curved OFFSET self-intersection arrangement and arc reconstruction.
- Full open/closed polyline OFFSET topology for every line/arc combination.
- Complete line/arc/ellipse/spline/polyline FILLET matrix.
- Formally certified spline root isolation and ellipse error bounds.
- Full fixed-epsilon remediation across every production geometry module.
- Production B-rep 3D kernel and solid modelling.
- Android APK build, installation and device tests.

## Release status

AR CAD remains under development and is not production-ready. No APK was built as part of this update, consistent with the current instruction to defer APK work while 2D enhancements continue.
