# AR CAD — Production Readiness Gate (2026-11-01)

## Decision

**NOT PRODUCTION READY. Do not publish a production APK.** The source acceptance suite passes, but the current CADCraft Rust workspace has not been compiled or tested in this environment, no Android APK has been built, and no device test has been performed. The P1 geometry work is not complete.

This report deliberately distinguishes source-presence checks from executable correctness. A test that searches source for a function or assertion does not prove that the implementation compiles or behaves correctly.

## Validation performed for this review

Command: `./tools/test-ar-cad.sh`

- Step 2 source checks: 18/18
- Step 3 source checks: 25/25
- Steps 4–6 source checks: 14/14
- Steps 7–10 source checks: 17/17
- Geometry priority source checks: 14/14
- Tolerance migration source checks: 20/20
- Precision safety source checks: 11/11
- P1 source checks: 22/22
- Total source assertions: **141/141**
- Shell syntax checks: passed
- Python `compileall`: passed
- Rust tests: skipped by default

Explicit Rust gate attempt: `RUN_RUST_TESTS=1 ./tools/test-ar-cad.sh`

Result: exit code 2, `ERROR: RUN_RUST_TESTS=1 but cargo is unavailable.` This environment also has no `rustc`, `adb`, or Android `sdkmanager`. `build-ar-cad-debug.sh` stops at preflight with `Rust/Cargo is required.`

## P1 blockers confirmed by source inspection

### 1. Automatic OFFSET self-intersection repair — NOT IMPLEMENTED

`source/cadcraft/crates/engine/src/cmd/modify/offset.rs` detects non-adjacent intersections in the generated offset polyline and returns `None`. This is safe rejection, not repair. A production repair must build a planar arrangement, split edges at all intersections, classify faces against the source offset/winding rule, stitch valid loops, preserve arc segments where possible, and reject ambiguous or numerically unstable cases with a clear diagnostic. It needs tests for bow-ties, concave collapse, multiple loops, narrow necks, holes, arcs, and reversed winding.

### 2. Complete round-fillet support — PARTIAL

`round_join_bulge` only returns a bulge when both connector endpoints lie within a radius tolerance of the original vertex. When it returns `None`, the caller uses a zero bulge, i.e. a straight connector. This is not complete round-join support for every supported line/arc/polyline combination. Need explicit convex/concave and orientation classification, arc-aware joins, sweep selection, tangency continuity checks, and regression tests for every supported pair type and both windings.

### 3. Formally exact adaptive orientation — NOT IMPLEMENTED

`crates/geom/src/predicates.rs::orient2d` scales differences and uses FMA product residuals. Its own module comment says it is not a general arbitrary-precision arithmetic package and describes equality only at implementation precision. This is a more robust floating-point predicate, not a formally exact adaptive predicate. A production implementation needs a vetted adaptive expansion predicate (or a proven equivalent), documented floating-point assumptions, and adversarial tests including near-collinear, subnormal, huge-coordinate, opposite-sign, and overflow/underflow cases. Do not relabel the current function as formally exact.

### 4. Ellipse/spline numerical error bounds — NOT IMPLEMENTED

Ellipse-line intersection is analytic in parameter space, but no certified residual/error bound is returned or checked for all conditioning cases. Spline-line intersection samples 48 points per knot span, brackets sign changes, and searches sampled local minima for tangencies. This can miss multiple roots or narrow features between samples; it is a heuristic numerical solver, not a certified root isolator. Need derivative/interval bounds or Bézier subdivision with conservative convex-hull rejection, interval root isolation, residual bounds, multiplicity/tangency handling, and adversarial tests.

### 5. Fixed-epsilon audit — INCOMPLETE

A grep of `crates/geom/src` still finds world/parameter constants and floors in production code, including `EPS` checks in `curve.rs`, fixed `1e-12` bulge/closure checks, fixed `1e-9` polygon endpoint checks, spline knot/fit thresholds, `1e-300` denominator guards, and fixed grid cell floors in `region.rs`. Some constants are intentional algorithmic/parameter safeguards or test tolerances, but they have not been individually classified and proven safe. Each production occurrence needs classification as: (a) dimensionless algorithm threshold, (b) drawing-space tolerance to migrate, or (c) unavoidable floating-point guard with an error rationale. No blanket text replacement is safe.

### 6. Rust compilation and tests — BLOCKED

No Cargo or rustc is installed in this environment. The current source tree therefore has not been compiled and no current CADCraft Rust unit/integration tests have executed here. This is a hard release blocker, not a documentation gap.

## Additional product-wide production blockers

- Android debug APK has not been produced, installed, or run on a physical device.
- Save/open/reopen, crash recovery, Android storage permissions, lifecycle/surface recreation, keyboard/IME, touch gestures, and large drawings have not been verified on device.
- Full professional 2D scope remains incomplete: advanced dimensions/annotation, block attributes/dynamic blocks, XREF, associative hatch, layouts/viewports/page setup/plotting, and robust DWG round-trip/data preservation need feature-by-feature verification.
- Full professional 3D scope is not implemented: production B-rep kernel, robust booleans, extrusion/revolve/sweep/loft, shell/fillet/chamfer, surface/mesh editing, mass properties, 3D viewport/navigation, and STEP/IGES exchange.
- Performance, memory, crash recovery, fuzzing, security review, and compatibility matrix have not been completed.
- Release signing and distribution verification have not been exercised end-to-end.

## Required release gates

1. Build in a pinned Rust/Android CI image with the exact lockfile; compile all workspace crates.
2. Run `cargo test --locked -p cadcraft-dxf -p cadcraft-geom -p cadcraft-doc -p cadcraft-io -p cadcraft-engine -p cadcraft-ui-egui --no-fail-fast` and fix every failure.
3. Add the exact/adaptive predicate and certified curve-intersection work; execute adversarial/property/fuzz tests.
4. Implement and verify OFFSET arrangement cleanup and complete supported round joins; never silently degrade a requested round join to a straight join.
5. Finish the fixed-epsilon audit with a reviewed inventory and scale-based tests.
6. Build an ARM64 debug APK; verify package id/signature and install on real Android hardware.
7. Test critical 2D workflows end-to-end: create/edit/select/snap/trim/extend/offset/fillet/chamfer/dimension/layers/hatch/blocks; save, close, reopen, DXF import/export, recovery.
8. Complete the defined 3D scope and format interoperability before claiming full 2D+3D professional CAD.
9. Run long-session, large-file, memory, crash/recovery, and device compatibility tests.
10. Only then sign and publish a production release.

## Honest status

The 141 source-level assertions passing is useful progress. It is not equivalent to 141 functional CAD tests. No statement that AR CAD is production-ready, fully implemented, or APK-tested is supported by the evidence currently available.

## Follow-up implementation session (2026-11-01)

A targeted OFFSETGAPTYPE=1 improvement was made in `source/cadcraft/crates/engine/src/cmd/modify/offset.rs`:
- Round-join sweep direction is now selected from the source polyline's signed turn and the signed offset distance, rather than blindly accepting `atan2`'s principal sweep.
- The connector rejects ambiguous zero-direction cases and sweeps at/above a half-turn, and retains radial endpoint validation.
- Three focused Rust unit tests were added for counter-clockwise joins, clockwise joins, and invalid/ambiguous geometry.

Validation status: source edited and test cases added; Rust compilation and execution remain unverified because `cargo`/`rustc` are not installed in this environment. This does not complete OFFSET self-intersection repair, the exact-predicate requirement, certified curve root isolation, or the wider production-readiness gates.

## Five-stage revalidation — 2026-11-01

The latest available curve-intersection source baseline was rechecked. The unified source-level suite passes **155/155**, plus shell syntax and Python compile checks. The current geometry threshold scanner records **434 occurrences in 16 Rust files**; multiple production classes remain under manual review.

An explicit `RUN_RUST_TESTS=1 ./tools/test-ar-cad.sh` run exits 2 because Cargo is unavailable. `./build-ar-cad-debug.sh` exits 1 at its preflight because Rust/Cargo is unavailable. No APK was built, installed or tested. The environment also has no Gradle, Android SDK, emulator or ADB.

The full 3D kernel remains absent: existing `SOLID`/`TRACE` and `3DFACE` entities are simple corner/face primitives, not a manifold B-rep model. See `docs/AR-CAD-FIVE-STAGE-RELEASE-STATUS-2026-11-01.md` for the per-stage acceptance criteria and verified evidence.

## 2D geometry enhancement follow-up — 2026-11-01

The subsequent enhancement session made the following source changes:

- Corrected OFFSETGAPTYPE=1 outside-corner sweep selection using signed source turn and signed offset side.
- Changed round OFFSET connector construction to fail closed instead of silently degrading an invalid requested fillet into a straight connector.
- Removed epsilon-based pruning of nonzero Bernstein coefficients in spline-line root candidate isolation; endpoint classification now uses exact zero only.
- Threaded a drawing-derived tolerance context through polyline FILLET corner construction and point deduplication.
- Removed fixed world-unit/angle cutoffs from bulge-to-arc conversion, retaining only exact degeneracy and finite/representability checks.
- Removed the fixed `EPS` radius floor from arc-segment OFFSET; collapsed, non-finite and overflowed radii are rejected explicitly.
- Added Rust regression cases for open-path round OFFSET, small Bernstein coefficients, scale-invariant polyline FILLET, and tiny-angle/microscopic bulge arcs.

Re-run of `./tools/test-ar-cad.sh`: **168/168 source-level assertions passed**, shell syntax passed, Python compileall passed. Rust tests were not executed because Cargo/rustc are unavailable. Android build remains intentionally deferred in this work session.

These changes improve source behavior but do not complete general curved OFFSET topology, every FILLET combination, certified curve solvers, all fixed-epsilon remediation, or the production 3D kernel.


## Critical DWG/DXF and 3D implementation increment — 2026-11-01

The DWG pre-save loss report now includes known bridge-loss metadata categories (entity reactors/colour-book names, table/block XDATA/reactors/colour-book names, and retained custom-class objects). This detects more potential data loss but does not make the DWG bridge lossless. The critical DWG/DXF gap remains open.

A first B-rep geometry foundation was added in `source/cadcraft/crates/geom/src/brep.rs`: manifold half-edge topology structures, validation, simple polygon-prism extrusion, volume and planar surface-area properties. This is not integrated into the CAD document/Android command model and does not include booleans, revolve, sweep, loft, curved surfaces or 3D editing. The 3D production gap remains open.

Revalidation: `tools/test-critical-cad-gaps.py` passed 17/17 source checks; full `tools/test-ar-cad.sh` passed 185/185 source-level assertions, shell syntax and Python compile checks. Rust tests remain skipped because Cargo/rustc are unavailable. No APK or DWG interoperability claim is made. See `DWG-DXF-AND-3D-CRITICAL-GAPS-UPDATE-2026-11-01.md` for the complete change list and remaining work.


### DWG metadata-only loss guard — 2026-11-01

The interactive DWG save warning now explicitly covers metadata-only loss (entity reactors, colour-book names, table/block XDATA and retained custom-class objects). A UI regression test verifies QSAVE is held before writing and Cancel preserves the original file bytes. This is a destructive-save safety improvement only; Point 3 remains pending until the bridge preserves the data or independent round-trip tests prove the required behavior. Rust tests have not run in this environment.


## 2026-11-01 follow-up — EXTRUDE document integration

The engine now registers `EXTRUDE`, validates a B-rep polygon prism and emits its outer shell as native 3DFACE entities. Concave caps are triangulated; volume and surface area are returned. This is a faceted surface shell, not a persistent editable solid. Validation: 185/185 source-level assertions pass. Cargo is unavailable, so Rust compile/runtime tests remain unverified. DWG/DXF Point 3 remains critical and pending.


## 3D grouped-facet increment — 2026-11-01

EXTRUDE now tags each generated 3DFACE with persistent ARCAD_BREP XDATA group metadata. SOLIDINFO validates group completeness and closed-shell topology; SOLIDEDITDELETE removes a complete group and refuses incomplete groups. This provides basic group-level lifecycle operations across supported XDATA round-trips, but does not create an editable analytic B-rep solid. Rust compile/runtime verification and DXF/DWG external round-trip tests remain required.


### Point 3 — fail-closed DWG/DXF data integrity — 2026-11-01

The I/O layer now refuses DWG conversion for known unsupported entities and bridge-loss metadata, custom-class objects and incomplete source reads. DXF export refuses parser-reported partial reads. The UI no longer provides a Save Anyway bypass. Direct I/O regression tests cover the refusal paths and a clean modelled DWG export. The current `acadrust` bridge remains a conversion bridge and is not certified lossless. Latest source QA: 197/197 assertions, including 29/29 critical-gap checks; shell syntax and Python compileall passed. Cargo/rustc are unavailable, so Rust tests are not run. Point 3 remains pending real-file corpus and independent AutoCAD/ODA validation.


## 2026-11-01 incremental OVERKILL improvement

`OVERKILL` now checks the cyclic closing seam of closed lightweight polylines and removes a redundant same-direction, collinear, zero-width straight vertex while preserving the original start vertex. A focused engine integration regression test was added. This is a bounded cleanup improvement, not completion of AutoCAD Geometry Cleanup. The Rust test is present but has not been compiled or executed in this environment because Cargo/rustc are unavailable.


## 2026-11-01 — OVERKILL tolerance-boundary follow-up

Arc/circle candidate matching in `source/cadcraft/crates/engine/src/cmd/overkill.rs` now checks neighboring spatial buckets and confirms actual center, radius and Z coordinate deltas against the drawing tolerance. This avoids false negatives caused by rounded quantization-cell boundaries. Added a focused engine integration regression test. The OVERKILL duplicate path also had a redundant assignment removed.

Latest source gate: **216/216 source-level assertions passed** (including 36/36 critical-gap checks), shell syntax passed, and Python compileall passed. Rust compilation and engine tests remain unverified because Cargo/rustc are unavailable. No Android APK/device validation or AutoCAD/ODA format interoperability test has been performed. This change does not complete Geometry Cleanup, full curved OFFSET/FILLET, editable B-rep, or the P0 DWG interoperability gate.


## 2026-11-01 incremental engineering note

- OVERKILL now groups collinear line candidates cyclically across its internal direction-normalization seam, independently per property bucket, and aligns wrapped projection axes before overlap processing.
- Added an engine integration regression case for nearly collinear, oppositely directed segments crossing that seam.
- Source acceptance suite: 218/218 assertions passed (including 38 critical-gap assertions).
- Rust tests, Android build/device validation and independent DWG/DXF interoperability remain unverified; release readiness is not established.

## R5 continuation — 2026-11-01

- Standard line/arc/circle FILLET now routes through the shared drawing-tolerance-aware tangent solver rather than the legacy fixed-EPS TTR solver.
- The curve adapter no longer rejects every line shorter than a fixed 1e-12 world-unit threshold; it accepts finite nonzero line geometry for operation-level validation.
- Added a microscopic-scale line-fillet geometry regression and three source acceptance checks.
- Latest available source gate: 223/223 assertions, shell syntax and Python compile checks pass.
- Explicit Rust test gate remains blocked: `RUN_RUST_TESTS=1 ./tools/test-ar-cad.sh` reports Cargo unavailable. Android SDK/NDK/Gradle and device testing are not available in this execution environment. No APK is claimed.
- Release decision unchanged: NOT production-ready until Rust CI, Android debug/device acceptance and independent DWG/DXF interoperability gates pass.


## R6 verification update — 2026-11-01

- Fixed CI release-tag selection: `v*.*.*` tags now select release mode; manual dispatch preserves the chosen mode; branch pushes remain debug.
- Added six CI workflow assertions and integrated them into the source gate.
- Latest available source verification: 229/229 assertions, shell syntax and Python compileall passed; workflow YAML parsed successfully.
- Rust tests remain unexecuted because Cargo/rustc are unavailable. Android SDK/NDK are also unavailable. No APK or device test and no independent DWG/DXF round-trip have been performed.
- Production status remains blocked pending executable Rust validation, Android validation and independent file-format verification.
