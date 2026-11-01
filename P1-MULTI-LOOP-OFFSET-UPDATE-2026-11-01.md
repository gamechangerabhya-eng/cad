# AR CAD engineering update — 2026-11-01

## Implemented in this update

- Added `cadcraft_geom::region::repair_winding_loops`, which returns every retained positive-area face whose interior has non-zero winding in the source closed straight-segment contour.
- Kept `repair_single_winding_loop` as a compatibility wrapper; it returns a result only when exactly one loop exists, so legacy single-entity callers cannot silently discard extra faces.
- Added a multi-result OFFSET geometry path for lightweight polylines. Self-intersecting closed line-only offsets now use the multi-loop repair API.
- Updated the OFFSET command to create a separate lightweight-polyline entity for every returned loop and return all created handles when there is more than one.
- Preserved constant polyline width on each emitted result.
- Existing ordinary polyline segment generation continues to preserve arc segments through bulges; self-intersecting curved arrangements are still rejected rather than flattened.
- Added a bow-tie multi-face regression test and updated source-contract checks for the new API.

## Validation performed

- `./tools/test-ar-cad.sh`: passed all source acceptance groups and shell/Python syntax checks.
- Rust tests were skipped by the script. Cargo/rustc are not installed in this environment, so the changed Rust code has not been compiled or executed.
- No Android APK was built, installed, or tested.

## Still outstanding

- Robust arrangement handling for overlapping/collinear segments, holes, nested loops, arc-preserving intersections, and complex degeneracies.
- Open-path offset end caps and full open-path cleanup policy.
- Complete FILLET for all line/arc/polyline pairings.
- Formally exact orientation predicates across the full binary64 domain; current expansion implementation is not a formal proof for every extreme case.
- Certified spline root isolation and ellipse intersection error bounds.
- Full fixed-epsilon inventory and replacement/classification.
- Production B-rep kernel, solid modelling operations, Boolean robustness, 3D fillet/chamfer and mass properties.
- Rust/Android build, device test matrix, performance/stability/fuzz testing, compatibility, and release signing validation.
