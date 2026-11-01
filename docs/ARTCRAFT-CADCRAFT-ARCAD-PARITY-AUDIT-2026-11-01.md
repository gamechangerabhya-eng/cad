# ArtCraft Android + CADCraft + AR CAD: implementation parity audit

**Review date:** 2026-11-01  
**Reviewed local project:** `/mnt/data/arcad-next/AR-CAD`  
**Purpose:** Determine whether the capabilities in the ArtCraft Android builder and upstream CADCraft repository have all been implemented and integrated in AR CAD.

## Executive finding

**No. AR CAD does not yet contain everything offered by the two upstream projects, and it is not a completed professional 2D/3D CAD application.**

The active build path is an ArtCraft Android builder `v0.4.1` / generator `61` wrapper that builds upstream CADCraft `0.15.1` with AR CAD's display name, package ID and icon. It uses the generic CADCraft launcher-generation recipe (`CadApp::new(Session::empty(), Services::default())`) and the CADCraft IO installer. There is no `adapters/cadcraft/` custom adapter in the project.

The earlier standalone AR CAD Compose/JNI implementation is preserved under `legacy/AR-CAD-0.6.1`; it is not the active app implementation used by `build-ar-cad-debug.sh` or `build-ar-cad-release.sh`. Its Kotlin drawing model and the current CADCraft `Session` must not be described as one unified document model.

Consequently, AR CAD currently gets much of its CAD functionality from CADCraft and much of its Android platform integration from ArtCraft Android. The AR-specific work primarily covers product branding, package identity, build entry points and the accumulated source modifications in the vendored CADCraft checkout. This is a valid integration direction, but it is not equivalent to implementing every feature from both projects or reaching full professional CAD parity.

## 1. Repository identity and integration

| Layer | Local evidence | Finding |
|---|---|---|
| ArtCraft Android builder | `lib/build-common.sh`: `BUILDER_VERSION="0.4.1"`, `GENERATOR_VERSION = "61"` | Builder/runtime integration is present in source. |
| CAD engine | `source/cadcraft/Cargo.toml`: version `0.15.1`, Rust `1.95` | Upstream CADCraft source is included. |
| AR CAD identity | `build-ar-cad-debug.sh`, `build-ar-cad-release.sh`, `AR-CAD-INTEGRATION.json` | App title `AR CAD`, package `com.arcad.app`, AR CAD icon. |
| Android launcher | `lib/build-common.sh` CADCraft recipe and generated launcher mechanism | Generic launcher is generated during build; no checked-in generated `cadcraft-android` crate in this checkout. |
| CADCraft adapter | `adapters/` contains PhotoCraft, VectorCraft and SubCraft only | No CADCraft-specific custom adapter. CADCraft uses the generic generated path. |
| Earlier AR CAD Compose app | `legacy/AR-CAD-0.6.1` | Preserved legacy project, not the active ArtCraft/CADCraft launch path. |
| APK output | `output/` contains no APK | No built artifact in this workspace. |

### Active build flow

`build-ar-cad-debug.sh` calls `build-debug.sh -cadcraft` with AR CAD name, package ID and icon. The generic CADCraft generator creates an Android launcher around the upstream `CadApp` and `Session`; the builder installs CADCraft IO handlers. This reuses the upstream CADCraft UI and command engine rather than merging the old Compose editor into the CADCraft document model.

### Android platform capabilities inherited from ArtCraft Android

The builder source documents and implements shared Android runtime features including GameActivity, egui/wgpu/winit adaptation, SAF file picker and staged save-back, intent/file handoff, touch/mouse/trackpad handling, keyboard modifiers, IME, clipboard, system fonts, lifecycle and surface-recovery patches, theming and update checks. These are Android host/runtime facilities, not CAD geometry or CAD feature implementations. Their CADCraft-specific runtime behaviour still needs a real Android build and device tests.

## 2. CADCraft feature coverage inherited by AR CAD

The local CADCraft `README.md` documents a broad 2D CAD command set: drawing primitives, modification commands, snaps, layers/properties, dimensions, text, hatch, blocks, DXF/DWG IO, layouts, plotting, constraints, grips and automation. These are inherited upstream features, not independently implemented AR CAD features.

The upstream `ROADMAP.md` labels CADCraft **pre-alpha**, estimates approximately **38% ready for real work**, and lists **3D modelling/rendering at 2%**. It also estimates substantial remaining work for 2D geometry precision, modify operations, dimensions, text, blocks/xrefs, layouts and file formats.

The upstream `docs/gaps.md` and `docs/geometry-parity.md` explicitly identify important remaining issues, including DWG data loss on save, lack of real-file/AutoCAD interoperability corpus, ellipse/spline geometry relying on tessellation in some operations, polyline offset bulge/cleanup gaps, incomplete dynamic blocks/xrefs and missing 3D modelling.

## 3. Current AR CAD source modifications checked directly

The current `source/cadcraft` tree contains real local changes beyond simple branding. The following is the precise scope visible in source:

| Area | Present in current AR CAD source | Limit / unresolved issue |
|---|---|---|
| Exact orientation | `crates/geom/src/predicates.rs` contains an integer-significand/exponent exact determinant path for finite binary64 coordinates, preceded by a floating-point filter. | Rust code and tests have not been compiled/executed in this environment; independent oracle/property validation is still needed. Non-finite inputs return `Equal`. |
| Multiple-loop OFFSET | `offset_polyline_with_gap_type_multi` and `repair_winding_loops` can emit multiple result polylines for a closed, line-only self-crossing contour. The command creates each returned loop as an entity. | Not general curved arrangement repair. Open self-crossing paths and mixed line/arc self-intersections still fail closed. Nested/overlapping/tangent/near-degenerate cases need broader handling and tests. |
| Arc preservation | Existing offset segments represent circular arcs and convert them back to polyline bulges when the result remains a valid single contour. | If a curved result self-intersects, the current repair path rejects it rather than splitting/reconstructing arcs. Thus arc preservation through intersections is not implemented. |
| OFFSET open paths | Ordinary open polyline offset generation is present. | It skips the first open join and does not run the multi-loop repair for open paths. Complex crossing/collapse and cap/join policies are not generally resolved. |
| OFFSET joins | `OFFSETGAPTYPE` 0/1/2 and a signed round-join bulge helper exist. | A failed round-join construction falls back to a straight connector (`unwrap_or(0.0)`); complete round-fillet geometry is not guaranteed. |
| FILLET | Upstream CADCraft includes line/arc/circle and some ellipse/spline/polyline fillet paths; tolerance-aware solver interfaces and local tolerance improvements exist. | Not every line/arc/polyline segment pairing, closed-polyline case, multi-segment topology and tangency/trim edge case is covered. No complete end-to-end runtime evidence. |
| Spline-line intersection | Homogeneous Bézier decomposition, Bernstein sign-variation subdivision and safeguarded Newton refinement are present. | The source comments explicitly state that this is **not** a formal interval-arithmetic certificate; floating-point coefficient/subdivision rounding and cutoff heuristics can still affect root isolation. |
| Ellipse-line intersection | Normalized analytic calculation with residual checks is present. | No outward-rounded interval enclosure or certified error bound for all ill-conditioned cases. |
| Fixed-epsilon audit | `GEOMETRY-FIXED-EPSILON-AUDIT-CURRENT.md` inventories 434 occurrences across 16 Rust files. | 27 geometric decision thresholds and other solver/NURBS/fillet/planar-arrangement thresholds remain classified for review. Audit is not fully remediated. |
| 3D solids | Existing 3DFACE and simple SOLID/TRACE entity types exist. | They are not a manifold B-rep kernel. No production solid modelling/booleans/3D fillet/chamfer/mass properties. |

## 4. Android build and runtime parity

ArtCraft Android provides a credible Android porting architecture, but source presence is not proof of working runtime integration.

Observed local state:
- `source/cadcraft/apps/cadcraft-android` is not present before the builder generates it.
- `output/` contains zero APKs.
- The current environment has Java and CMake, but no `cargo`, `rustc`, `gradle` or `adb` available on PATH.
- The unified `tools/test-ar-cad.sh` run passed **155/155 source-level assertions**, plus shell syntax and Python compile checks. Its Rust tests are skipped by default.
- Therefore no current Rust workspace test, Android APK build, installation, JNI runtime check or physical-device test is evidenced by this review.

## 5. Current source QA run

Command: `./tools/test-ar-cad.sh`

Results:
- Step 2 source checks: 18/18
- Step 3 source checks: 25/25
- Steps 4–6 source checks: 14/14
- Steps 7–10 source checks: 17/17
- Geometry priority checks: 14/14
- Tolerance migration checks: 20/20
- Precision safety checks: 11/11
- P1 checks: 36/36
- Total: **155/155 source-level assertions passed**
- Shell syntax checks: passed
- Python compile checks: passed
- Rust tests: skipped because Cargo is unavailable

These checks mostly inspect source contracts and test-file presence. They are not equivalent to running CAD geometry tests or exercising the Android app.

## 6. Reconciled answer: have we implemented everything from both repositories in AR CAD?

**No.** The accurate description is:

1. AR CAD currently uses the ArtCraft Android builder/runtime as its Android porting layer.
2. AR CAD currently uses CADCraft 0.15.1 as its underlying CAD application and engine.
3. AR-specific branding and build wrappers are present; several additional geometry source changes are present.
4. The older Compose/JNI AR CAD implementation is retained as legacy and is not unified with the active CADCraft `Session`.
5. CADCraft itself documents pre-alpha status and major remaining feature gaps, especially 3D.
6. The latest source checks pass, but Rust runtime tests, Android build/device acceptance and production release gates have not passed.

## 7. Work required before saying the upstream capabilities are implemented in AR CAD

- Decide and document one authoritative active architecture: the current ArtCraft-generated CADCraft app, or a separately maintained custom AR CAD UI. Do not keep two competing drawing/document models.
- Pin exact upstream ArtCraft Android and CADCraft commit hashes; the current import manifest records a local version/tag label, not a full reproducible upstream commit SHA.
- Build the generated Android launcher with the pinned Rust/NDK/Gradle toolchain and test SAF, lifecycle, IME, input and rendering on physical ARM64 devices.
- Complete and runtime-test OFFSET arrangements for line/arc mixtures, open paths, multiple loops, nested regions and tangencies.
- Complete and runtime-test FILLET across supported curve and polyline combinations.
- Validate exact orientation against an independent exact arithmetic oracle and fuzz finite binary64 inputs.
- Use outward-rounded interval arithmetic or a proven equivalent for certified spline/ellipse intersection bounds; otherwise label the solvers robust numerical approximations, not certified.
- Finish the fixed-epsilon disposition for every production occurrence and add scale/conditioning tests.
- Implement or integrate a properly licensed production B-rep kernel and the required solid operations if full 3D CAD remains a product requirement.
- Validate DWG/DXF interoperability with a real drawing corpus and independent CAD software; document any known loss.
- Run performance, memory, fuzz, crash recovery, compatibility and signed release tests.

## Sources

- Local source: `AR-CAD-INTEGRATION.json`, `build-ar-cad-debug.sh`, `build-ar-cad-release.sh`, `lib/build-common.sh`, `source/cadcraft/README.md`, `source/cadcraft/ROADMAP.md`, `source/cadcraft/docs/gaps.md`, `source/cadcraft/docs/geometry-parity.md`, `source/cadcraft/crates/engine/src/cmd/modify/offset.rs`, `source/cadcraft/crates/geom/src/predicates.rs`, `source/cadcraft/crates/geom/src/intersect.rs`, `GEOMETRY-FIXED-EPSILON-AUDIT-CURRENT.md`.
- Upstream ArtCraft Android: https://github.com/MostafaXS1/artcraft-android
- Upstream ArtCraft Android architecture: https://github.com/MostafaXS1/artcraft-android/blob/main/architecture.md
- Upstream CADCraft: https://github.com/storytold/cadcraft
- Upstream CADCraft roadmap: https://github.com/storytold/cadcraft/blob/main/ROADMAP.md
- Upstream CADCraft gaps: https://github.com/storytold/cadcraft/blob/main/docs/gaps.md
- CADCraft v0.15.1 release: https://github.com/storytold/cadcraft/releases/tag/v0.15.1

## Follow-up: critical DWG/DXF and 3D foundation — 2026-11-01

A subsequent AR CAD increment extended the DWG pre-save loss report to cover known bridge-loss metadata (entity reactors/colour-book names, table/block XDATA/reactors/colour-book names, and retained custom-class objects). This is loss detection only; the current acadrust bridge still does not guarantee preservation of those items.

The geometry crate now contains an initial B-rep topology module with half-edge validation, simple polygon-prism extrusion, volume and planar surface-area properties. This is not integrated into the CADCraft document/Android editing workflow and is not a production 3D modeller. Boolean operations, curved surfaces, revolve/sweep/loft, shell/fillet/chamfer, healing and 3D exchange remain open.

Revalidation: 185/185 source-level assertions passed, including 17 critical-gap assertions. Rust tests and Android builds remain unverified because the required toolchains are unavailable. See `DWG-DXF-AND-3D-CRITICAL-GAPS-UPDATE-2026-11-01.md`.


## Follow-up: Point 3 fail-closed DWG/DXF hardening — 2026-11-01

The I/O boundary now refuses DWG conversion for known unsupported entities, bridge-loss metadata, custom-class objects and incomplete source reads. DXF export refuses parser-reported partial reads. The UI no longer offers Save Anyway, and direct I/O tests cover the refusal paths. This implements a non-destructive safe-refusal strategy for known gaps, not a lossless DWG codec. Source QA passed 197/197 assertions (29/29 critical-gap checks); Rust tests and independent AutoCAD/ODA corpus checks remain pending because the Rust toolchain and external CAD test environment are unavailable.
