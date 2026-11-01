# AR CAD engineering update — 2026-11-01

## Changes made in this update

### Polyline FILLET tolerance migration

File: `source/cadcraft/crates/engine/src/cmd/modify2/fillet.rs`

- Derives one `ToleranceContext` from the selected polyline's bounds.
- Uses the operation's angular tolerance when deciding whether a corner is degenerate or effectively straight.
- Uses the drawing-unit linear tolerance for radius eligibility, cutback fit checks, and deciding whether a cutback is meaningful.
- Uses the angular tolerance for distinguishing line segments from bulged arc segments in the straight-corner FILLET path.
- Rejects negative and non-finite radii at the geometry operation boundary.

### Regression tests added

File: `source/cadcraft/crates/engine/src/cmd/modify2/tests.rs`

- Adds a scale-invariance test for closed-polyline FILLET at drawing scales 1e-9, 1 and 1e9.
- Adds a negative-radius rejection test.

These Rust tests have been added but could not be executed in this environment.

## Validation actually performed

- `./tools/test-ar-cad.sh`: exit 0; all source-level checks passed.
- Acceptance suites: 18/18, 25/25, 14/14, 17/17, 14/14, 20/20, 11/11 and 31/31 (150 source-level assertions total).
- Shell syntax checks passed.
- Python compileall passed.
- `RUN_RUST_TESTS=1 ./tools/test-ar-cad.sh`: exit 2 because Cargo is unavailable.
- `./build-ar-cad-debug.sh`: stopped at preflight because Rust/Cargo is required.

## Tooling limitations

The environment has CMake and Java, but does not have `cargo`, `rustc`, `rustfmt`, `gradle`, `adb` or `sdkmanager`. Therefore Rust compilation, Android APK generation, installation and physical-device testing are not verified.

## Status of the 11 requested workstreams

1. OFFSET multiple loops: source path exists for closed straight-segment self-crossings; curved and complex arrangements remain incomplete.
2. OFFSET arc preservation: ordinary bulge arcs are retained in supported cases; arc-aware arrangement repair remains incomplete.
3. OFFSET open paths and complex intersections: ordinary open offsets exist; topology repair for complex cases remains incomplete.
4. FILLET: line/arc/circle and ellipse/spline dispatch exists; polyline fillet remains limited to straight-segment corners. This update migrates its thresholds but does not complete arc-polyline corner fillets.
5. Exact predicates: expansion-based orientation implementation exists but is not formally verified for every IEEE-754 input; adversarial proof/validation remains outstanding.
6. Spline intersections: current method is sampled/bracketed and tangent-minimum refined; not certified root isolation.
7. Ellipse intersections: analytic parameter-space solver exists; certified residual/error bounds are not implemented.
8. Fixed-epsilon audit: still incomplete; thresholds need classification and operation-specific migration, not blanket replacement.
9. Android build/device test: blocked by missing Android/Rust toolchain and device bridge.
10. Production B-rep/solid kernel: not implemented; existing SOLID/TRACE/FACE3D entities are not a production B-rep modelling kernel.
11. Release validation/signing: CI/release gates exist in source, but no reproducible signed build, device matrix, performance/stability run or release verification has been completed.

No production-readiness claim is made by this update.
