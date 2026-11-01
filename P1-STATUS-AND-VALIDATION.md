# AR CAD P1 — final source review and validation status

Date: 2026-11-01

## Work completed in this revision

- OFFSET rejects non-finite/non-positive distances and non-finite side points before geometry construction.
- Circle and arc OFFSET radius validity is based on a drawing-scale `ToleranceContext`, removing the fixed `1e-12` world-unit cutoff from those paths.
- Round OFFSET connector degeneracy uses the operation's shared angular tolerance instead of a fixed angular epsilon.
- Spline fit derivative scaling and kink-piece weighting preserve valid microscopic nonzero geometry using `f64::MIN_POSITIVE` rather than a `1e-12` drawing-unit floor.
- Added Rust regression tests for microscopic circle OFFSET and invalid numeric OFFSET inputs.
- Expanded P1 source-contract checks from 16 to 22.

## Validation actually performed

- Unified AR CAD source QA: **141/141 source-level assertions passed**.
  - Step 2: 18/18
  - Step 3: 25/25
  - Steps 4–6: 14/14
  - Steps 7–10: 17/17
  - Geometry priority: 14/14
  - Tolerance migration: 20/20
  - Precision safety: 11/11
  - P1 contracts: 22/22
- Shell syntax checks passed.
- Python compileall passed.
- Legacy DXF/native regression script passed, including its legacy Kotlin geometry, DXF, snap, recovery, FFI/JNI contract and native stub tests.
- ZIP integrity verified after packaging.

## Verification blockers

- `./build-ar-cad-debug.sh` was attempted and stopped at preflight: `Rust/Cargo is required`.
- `cargo test`, `rustfmt`, current CADCraft compilation and current-engine Rust unit/integration tests were not run because Rust tooling is not installed in this environment.
- Android APK compilation and physical-device tests were not run.

## P1 remains incomplete — do not mark complete

1. Offset self-intersection repair/polygonization is not implemented. The current engine fails closed and rejects invalid self-crossing results; this prevents corrupt output but does not reconstruct all valid loops/islands.
2. `OFFSETGAPTYPE=1` does not construct circular fillets for every possible arc/polyline join. Unsupported joins fall back to a straight connector, so full round-join semantics are not guaranteed.
3. `orient2d` is a scaled, FMA-compensated floating-point predicate, not a formally exact adaptive expansion predicate for every IEEE-754 input.
4. Ellipse/spline intersection and closest-point solvers still need a documented numerical error bound and a dense adversarial reference corpus.
5. A complete semantic audit of remaining fixed-epsilon decisions across the active geometry/engine commands is not finished. Some occurrences are tests, unit conversions or dimensionless parameter tolerances and should not be blindly replaced.
6. New Rust tests and current CADCraft code must compile and pass in a Rust-enabled CI environment before P1 can be declared functionally complete.

## Release decision

P1 is **substantially implemented but not complete**. Do not treat this archive as a verified production geometry engine and do not use it to claim a release-ready APK. Continue P1 closure work, then run Rust tests/build in CI and Android device QA.
