# AR CAD roadmap start — validation report

Date: 2026-11-01

## Change made
- Added `docs/AR-CAD-FULL-2D-3D-ROADMAP.md` with P0–P11 workstreams, ordering, exit criteria and a definition of done.
- Added `crates/geom/src/tolerance.rs` with `ToleranceContext` derived from drawing extent and coordinate magnitude.
- Exported `ToleranceContext` from `cadcraft-geom`.
- Added four Rust unit tests for scale behavior, large coordinates, degenerate/invalid bounds and non-finite input handling.

## Validation
- Source structure and API export inspected.
- Python syntax/source checks and legacy source QA are run separately where possible.
- Rust compilation and the new Rust tests were not run: `rustc`, `cargo` and `rustfmt` are not installed in this environment.
- Android build/device tests remain unrun; no APK is claimed.

## Important scope note
The new context is not yet wired into existing intersections, snapping, offsets, trimming or measurement. Existing code still uses the legacy global `EPS` and local constants. This is the first migration foundation, not a completed precision-kernel upgrade.

## Additional checks completed after the initial report
- Re-ran `bash tools/test-ar-cad.sh`: all existing Step 2–10 source acceptance checks passed (18 + 25 + 14 + 17); shell syntax and Python compile checks passed. The runner explicitly skipped current CADCraft Rust tests.
- Re-ran `legacy/AR-CAD-0.6.1/tools/run-dxf-tests.sh`: all legacy DXF/native host tests passed, including deterministic geometry/property cases, recovery, measurement, modify, snap/tracking, spatial-index, document/save/render and FFI/JNI contract checks. These validate the legacy engine subset only, not the current CADCraft 0.15.1 Android runtime.
- Ran 8 focused source assertions for the new tolerance API and its four Rust test cases: 8/8 passed. These assertions do not replace Rust compilation.
