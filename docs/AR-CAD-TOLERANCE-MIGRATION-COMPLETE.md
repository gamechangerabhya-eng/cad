# AR CAD — Shared tolerance migration follow-up

**Scope:** complete the specifically promised P1 follow-up across geometry intersections, curve closest-point refinement, OSNAP, TRIM/EXTEND, FILLET/CHAMFER, OFFSET joins and region topology entry points.

## Implemented in this revision

- OSNAP now derives one `ToleranceContext` from the nearby candidate geometry and reuses it for finite intersections, apparent intersections, and exact ellipse/spline-versus-line intersections. Broad-phase padding is based on the same context (and the local segment context) rather than independent `EPS * coordinate` expressions.
- TRIM and EXTEND derive one operation context from the target and all cutting edges. Line, arc, circle and polyline trimming plus line/arc/polyline extension use the tolerance-aware segment intersection API. Polyline sub-geometry generation has a tolerance-aware internal path.
- Spline EXTEND uses one context for its ray/boundary intersections and rejects results inside the operation tolerance.
- Line FILLET/CHAMFER uses a context for line intersection and degeneracy checks. Curved FILLET uses the new `tangent_circles_with_tolerance` API.
- Polyline OFFSET computes one context for the full polyline and passes it to every adjacent offset-segment join. It no longer derives a new tolerance independently for each join.
- Region graph splitting uses one context for all candidate segment pairs. Context-aware public wrappers were added for endpoint snapping, enclosing-loop, boundary and outline operations.
- Ellipse and spline closest-point solvers now expose `closest_with_tolerance`; existing `closest` signatures remain as compatibility wrappers that derive a local context. Golden-section refinement stops at the requested parameter tolerance while retaining hard iteration caps.
- Added large-coordinate/scale tests and explicit shared-context tests for fillet, region and ellipse/spline closest points.
- Added a dedicated source acceptance suite and connected it to `tools/test-ar-cad.sh`.

## Validation in this environment

`./tools/test-ar-cad.sh` completed successfully:

| Suite | Result |
|---|---:|
| Step 2 source acceptance | 18/18 |
| Step 3 source acceptance | 25/25 |
| Steps 4–6 source acceptance | 14/14 |
| Steps 7–10 source acceptance | 17/17 |
| Earlier geometry priority checks | 14/14 |
| New tolerance migration checks | 20/20 |
| **Total source assertions** | **108/108** |
| Shell syntax | Passed |
| Python compileall | Passed |

**Not executed:** Cargo/Rust compilation, Rust unit tests, rustfmt, Clippy, Android APK build, Android install or physical-device tests. `cargo`, `rustc` and `rustfmt` are not installed in this environment. Source assertions do not prove Rust type-checking or runtime correctness.

## Remaining P1 precision work

This migration does not finish all precision engineering:

1. BREAK/BREAKAT, JOIN, MEASURE, constraints and other geometry consumers still contain independent fixed epsilons or derive per-pair contexts.
2. Several snap tracking, tangent/perpendicular and extension helpers still use fixed angular/distance thresholds.
3. Region gap closure and some public scalar-tolerance callers still accept a caller-supplied scalar; full document-operation context propagation must be completed at all command call sites.
4. Offset for ellipses and splines still uses tessellated approximations; self-intersection resolution, collapsed-segment cleanup and OFFSETGAPTYPE remain incomplete.
5. Robust adaptive orientation/collinearity predicates and comprehensive randomized/property tests across hostile numeric scales remain.
6. The new Rust tests must be compiled and run in CI; any compiler/test failures must be fixed before merging.

## Wider professional CAD gaps (not solved by this migration)

- Full 2D command conformance, grips, UCS, selection modes and transaction/cancel behavior.
- Complete dimensions, text/SHX fidelity, leaders, tables, block editor, attributes, XREF workflows, associative hatch and dynamic blocks.
- Persistent layouts, page setups, CTB/STB, plotting, multi-sheet PDF output.
- DWG fidelity, proxy/xdata/reactor preservation, real-world corpus testing and independent round-trip verification.
- A genuine 3D B-rep/topology kernel, solid/surface primitives, Boolean operations, 3D fillet/shell/loft/sweep, mesh editing and mass properties.
- 3D renderer, depth/selection, camera, UCS/workplanes and touch interaction.
- STEP/IGES/3D exchange and verified import/export.
- Reproducible Android build, device matrix, large-file/performance tests, crash/process-death recovery and signed release.

## Status

P1 tolerance propagation is **substantially advanced, not complete**. The implementation is source-level only until Rust CI and Android tests run. No APK is produced by this revision.
