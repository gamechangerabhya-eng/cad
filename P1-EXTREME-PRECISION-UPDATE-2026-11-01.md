# AR CAD precision update — 2026-11-01

## Implemented in this update

`source/cadcraft/crates/geom/src/predicates.rs`

- Expanded the power-of-two normalization range in `orient2d` from a +/-500 exponent clamp to the complete usable binary64 power-of-two scaling range (+/-1023).
- This addresses a concrete overflow path: coordinates near `f64::MAX` previously remained around `1e157` after normalization, so determinant products could overflow before the expansion arithmetic ran.
- Added Rust regression cases for near-maximum coordinates, the smallest positive subnormal, and opposite-sign extreme coordinates.

## Validation

- `./tools/test-ar-cad.sh`: all existing source-level checks passed (141/141), plus shell syntax and Python compile checks.
- The new Rust regression tests have **not** been compiled or executed. Cargo/rustc are not installed in this environment.
- This change is not a proof that `orient2d` is formally exact over every binary64 input. The expansion path still has documented conservative fallback cases; a vetted adaptive predicate with proven IEEE-754 assumptions and exhaustive adversarial testing remains required.

## Remaining release blockers

- OFFSET still lacks complete multi-loop output, robust arc-preserving planar repair, and general open/complex curve handling.
- FILLET still needs full line/arc/polyline combinations and geometric continuity tests.
- Spline-line root isolation is sampled/bracketed, not certified interval isolation; ellipse intersections do not yet expose certified residual bounds.
- Fixed-epsilon review remains incomplete.
- No production B-rep kernel or complete solid modelling feature set is present in this workspace.
- Android build, installation, device testing, performance/stability matrix, and signed release validation have not been completed.

This update is a narrow numerical robustness improvement, not a production-readiness declaration.
