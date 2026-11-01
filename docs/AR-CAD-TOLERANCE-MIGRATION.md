# AR CAD — Geometry tolerance migration (implementation 1)

## Scope

This change advances the first engineering task in the full 2D + 3D roadmap: make a single drawing/operation tolerance available to core intersection algorithms instead of leaving line/circle decisions on unrelated fixed epsilons.

## Implemented

- Added `line_line_infinite_with_tolerance` and routed finite line intersections through the shared angular and linear tolerance context.
- Added `line_circle_with_tolerance`; its radial gap classification uses drawing-unit tolerance, and its chord calculation is normalized to avoid the old quadratic subtraction and large-value overflow path.
- Added `circle_circle_with_tolerance`; separation and tangency classification now use the caller's linear tolerance converted to normalized scale.
- Routed line/arc and arc/arc segment intersections through the same caller-provided tolerance context.
- Kept the old public function signatures as compatibility wrappers. They derive a tolerance from their operands.
- Exported the new APIs and added regression tests for near-tangent and outside-tolerance cases.
- Extended the source acceptance script to check that the shared context reaches the intersection implementations.

## Important design notes

- `ToleranceContext` must be created once for a whole document operation (for example a TRIM or OSNAP pass) and passed to every pairwise intersection. Compatibility wrappers derive a local context and therefore cannot guarantee identical classification across an entire large drawing.
- This change does not yet migrate every geometry subsystem. Region construction, curve closest-point routines, fillet construction, constraints, tessellation, and all command-level callers still need a systematic audit.
- The current spline/ellipse line solvers already accept a tolerance context; broad-phase spatial index padding and several command algorithms still contain independent tolerances.

## Validation

- `tools/test-ar-cad.sh`: source acceptance, shell syntax and Python compilation passed; Rust suites were skipped because `cargo` is unavailable in this environment.
- `docs/test-geometry-priority.py`: 14/14 source assertions passed.
- `legacy/AR-CAD-0.6.1/tools/run-dxf-tests.sh`: legacy DXF/native regression script passed. This is not a test of the current CADCraft Rust crate.
- No Rust compile, Cargo unit test, Android build, APK install or physical-device test was possible.

## Next

1. Run `cargo test --locked -p cadcraft-geom -p cadcraft-engine` in Rust-enabled CI.
2. Fix any compile or numerical regression found by those tests.
3. Migrate the shared tolerance context into OSNAP broad phase, TRIM/EXTEND, fillet/chamfer, offset, constraints and region topology.
4. Add property tests at tiny, ordinary, large and far-from-origin coordinate scales.
