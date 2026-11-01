# AR CAD — Five-stage production release status

**Assessment date:** 2026-11-01  
**Baseline:** latest available `AR-CAD-P1-Curve-Intersection-Update` source tree  
**Release decision:** **NOT READY — production release blocked**

This assessment separates source implementation, source-level assertions, executable tests, native builds, and real-device acceptance. A source-presence check is not a functional test.

## Evidence collected in this session

| Gate | Result | Evidence |
|---|---:|---|
| Unified source acceptance | 209/209 passed (latest review run) | `tools/test-ar-cad.sh` with Rust stage disabled; source-only, not Rust execution |
| Shell syntax checks | Passed | same run |
| Python compile checks | Passed | same run |
| Android Step 3 source checks | 37/37 passed | includes declaration-level check for inspection tools and `AUDIT` registration |
| Geometry epsilon inventory | 434 occurrences across 16 Rust files | `GEOMETRY-FIXED-EPSILON-AUDIT-CURRENT.md` |
| Explicit Rust test run | Blocked, exit 2 | `validation/2026-11-01/rust-test-attempt.log` |
| Android debug build | Blocked, exit 1 | `validation/2026-11-01/android-build-attempt.log` |
| APK install / device tests | Not run | no SDK, emulator, ADB, or APK in this environment |

The 155 passing checks are source-level acceptance assertions. They do not certify geometry execution or Android behaviour.

## Stage 1 — Geometry foundation

**Status: IMPLEMENTED IN SOURCE, NOT EXECUTION-VERIFIED**

Present in the source:
- Shared scale-aware `ToleranceContext` used by several geometry operations.
- Integer-based exact orientation determinant path for finite binary64 coordinates, with a floating-point filter.
- Regression test cases for mixed exponents, subnormal and extreme finite inputs.
- Bernstein-form spline/line candidate isolation based on homogeneous Bézier decomposition and recursive subdivision.
- Normalized ellipse/line intersection calculation with residual checking.

Still required:
- Compile and execute the Rust tests.
- Independently compare the predicate against an exact oracle over randomized/adversarial inputs.
- Formal interval/outward-rounded certificates for spline and ellipse roots; current implementations explicitly do not provide these certificates.
- Profiling of the exact predicate on large drawings.

## Stage 2 — 2D geometry

**Status: PARTIAL**

Present in the source:
- Multi-loop face extraction for closed, line-only self-crossing contours.
- OFFSET command path capable of emitting multiple polyline entities for the supported line-only repair case.
- Bulge-based arc representation for non-self-intersecting polyline offsets.
- Round/chamfer gap connector modes and signed sweep checks.
- FILLET implementations for line/arc/circle entities, curve-edit paths for ellipse/spline cases, and straight-segment polyline fillets.

Still required:
- Arc-preserving splitting and reconstruction for self-intersecting curved offsets.
- General planar arrangement handling for overlaps, tangencies, nested loops and mixed line/arc inputs.
- Open-path crossing resolution and cap/join policies.
- Complete FILLET coverage for all adjacent line/arc polyline segment combinations, closed polylines, and complex multi-segment cases.
- End-to-end tests that assert output geometry, not only source structure.

## Stage 3 — 3D CAD

**Status: PARTIAL FOUNDATION ADDED; NOT A PRODUCTION SOLID MODELLER**

The new `cadcraft-geom::brep` module adds explicit vertices, edges, half-edges, loops, planar faces, shells and solids; topology validation; simple polygon-prism extrusion; volume and planar surface-area properties. Source regression tests cover rectangular and concave prisms, negative extrusion, microscopic profiles, large coordinate offsets and invalid profiles.

Still required:
- Compile and execute the new Rust tests.
- Integrate B-rep entities with the document model, rendering, selection, grips, undo/redo and Android commands.
- General profile planes/UCS, holes, curved surfaces, analytic/NURBS surface representation and curve/surface intersections.
- REVOLVE, SWEEP, LOFT and robust Boolean union, subtraction and intersection.
- Shell, 3D fillet/chamfer, healing and topology repair.
- Centroid, centre of mass, inertia tensor and material mass properties.
- STEP/IGES and reliable 3D DWG/ACIS exchange, plus real-world model corpus and fuzzing.

The current increment is a topology and extrusion foundation only. It is not evidence of a production B-rep or Boolean modeller.

## DWG/DXF integrity — critical release requirement

**Status: FAIL-CLOSED LOSS PROTECTION IMPLEMENTED; FULL INTEROPERABILITY CERTIFICATION PENDING**

The I/O layer now refuses DWG output when known unsupported entities, bridge-loss metadata, custom-class objects, or incomplete source reads are detected. The UI no longer offers a lossy overwrite bypass. DXF export also refuses parser-reported partial reads instead of presenting an incomplete drawing as a complete export. DXF remains the preservation path for content successfully read. This is a safe-refusal strategy, not a lossless DWG codec. Rust runtime tests and independent AutoCAD/ODA corpus validation remain mandatory. See `docs/AR-CAD-POINT-3-FAIL-CLOSED-DWG-DXF-UPDATE-2026-11-01.md`.

## Stage 4 — Android application integration

**Status: SOURCE AND BUILD WORKFLOW PRESENT; APK NOT BUILT**

Present in the source:
- AR CAD Android package identity and app branding.
- Android-oriented Compose/editor integration and build scripts.
- ARM64 Android build workflow with Rust target, NDK, Gradle and APK metadata verification.

Blocked or unverified:
- This environment has no `cargo`, `rustc`, `gradle`, Android SDK, emulator, or `adb`.
- Debug APK not produced.
- No installation, launch, JNI loading, touch, file access, save/reopen, lifecycle, crash-recovery, or long-session tests were run on a device.

## Stage 5 — Release engineering

**Status: PIPELINE SCAFFOLDING PRESENT; RELEASE ACCEPTANCE NOT MET**

Present in the source:
- CI workflow for Rust regression tests and Android APK packaging.
- Release signing secret checks, APK identity checks, signature verification and SHA-256 artifact generation.

Still required:
- Successful CI run on the repository with all Rust tests and Android build completed.
- APK installation and end-to-end device tests on supported Android versions and ARM64 devices.
- Performance and memory profiling on representative small, medium and large drawings.
- Fuzzing, crash/recovery, file-format robustness and security review.
- Compatibility matrix, key management validation, reproducible release and signed artifact verification.

## Numeric tolerance audit

The generated current inventory reports 434 numeric-threshold occurrences in 16 geometry Rust source files. This includes test-only values and numerical/parameterisation guards; these are not all errors and must not be replaced indiscriminately. The inventory still contains production decisions marked for manual review, including geometric decisions, curve degeneracy, FILLET convergence, planar-arrangement grid policy, NURBS guards, spline guards, matrix conditioning and vector normalization.

Exit criteria for this workstream:
- Every production occurrence has a semantic disposition and owner.
- Drawing-distance decisions use a shared operation tolerance or document why they cannot.
- Parameter-space tolerances are dimensionless and conditioning-aware.
- Floating-point underflow, singularity and convergence guards are justified independently of drawing units.
- Test tolerances stay in tests.
- No production occurrence remains without a documented decision.

## Release gate

AR CAD may be called production-ready only after all five stages pass their executable acceptance criteria. A green source-level assertion suite alone is insufficient. On this assessment, the release is blocked by unexecuted Rust tests, no Android APK, no device validation, incomplete 2D geometry coverage, incomplete numerical certification, and the absence of a production 3D B-rep kernel.


### DWG metadata-only loss protection follow-up — 2026-11-01

The pre-save dialog now labels metadata loss as well as unsupported objects. Added UI regression coverage showing that a metadata-only lossy DWG save is held until explicit consent and Cancel preserves the original file bytes. This improves safety but does not establish lossless DWG interoperability. Point 3 remains PENDING / CRITICAL.


## 2026-11-01 follow-up — first document-integrated 3D command

Added the `EXTRUDE` engine command. It accepts a closed straight-segment lightweight polyline handle or explicit simple polygon points plus height, builds and validates the B-rep prism, triangulates concave cap faces, and adds a faceted native `3DFACE` shell to the drawing. Each generated face now carries a shared `ARCAD_BREP` XDATA group identity with version, face index and expected face count. `SOLIDINFO` inspects the group and checks shell closure; `SOLIDEDITDELETE` removes a complete group and refuses deletion if the group is incomplete. Volume and surface area are reported by EXTRUDE. This is persistent grouped faceted geometry, not an editable B-rep solid, and it does not retain the original analytic surfaces or constraints. Full 3D viewport and Boolean operations remain open. Rust tests are still unverified because Cargo is unavailable.

Full source QA at the prior increment: 185/185 assertions passed. The latest grouped-solid increment adds persistent XDATA grouping, SOLIDINFO and SOLIDEDITDELETE; latest full source QA is 192/192 assertions, including 24 critical-gap checks. Point 3 (lossless DWG/DXF interoperability) remains PENDING / CRITICAL.


## Point 3 fail-closed implementation — 2026-11-01

The DWG I/O boundary now blocks conversion when known bridge-loss categories are present, including unknown entities, entity/table/block metadata, custom-class objects and incomplete reads. The UI's Save Anyway path was removed. DXF export is also blocked for parser-reported partial reads, because skipped records cannot be recovered. Direct I/O regression tests cover unsupported entities, metadata, incomplete reads and clean DWG exports. Source QA: 197/197 assertions passed, including 29/29 critical-gap checks. Rust tests remain unexecuted because Cargo/rustc are unavailable. Full Point 3 certification remains pending independent AutoCAD/ODA round-trips against a licensed DWG/DXF corpus; the bridge is not claimed lossless.


## Autodesk 2027 parity review follow-up — 2026-11-01

A full feature matrix is maintained in `AR-CAD-AUTODESK-ARTCRAFT-CADCRAFT-PARITY-REVIEW-2026-11-01.md`. Autodesk 2027 product highlights now include Geometry Cleanup, Forma Data Management Checkout, Connected References, Connected Support Files and Autodesk Assistant. These capabilities are not currently complete in AR CAD. In particular, the existing CADCraft `AUDIT` command checks drawing database integrity and is not an implementation of Autodesk's guided geometric cleanup workflow.

### Android drafting toolbar source update

Added an explicit `ANDROID_INSPECT_TOOLS` constant and exposed `AUDIT` alongside Distance, Area, List, Layers, Properties and Blocks. This fixes an unresolved source reference found during the comparison review. `docs/test-step3.py` was strengthened to require the constant declaration and each exact action. Step 3 source checks: 37/37.

### Latest validation from 2026-11-01

- Unified source-level QA: **209/209 assertions passed** (18 + 37 + 14 + 17 + 14 + 20 + 11 + 49 + 29).
- Shell syntax checks: passed.
- Python compile checks: passed.
- Rust compile/tests: not run; Cargo/Rust compiler unavailable.
- Android APK/device validation: deferred as requested; not run.
- DWG/DXF independent interoperability with AutoCAD/ODA: not run.

**Release decision remains NOT READY.** Point 3 remains critical pending lossless/independent format validation; the production 3D kernel, complete 2D precision matrix, guided geometry cleanup, trace/markup/collaboration and Android runtime evidence also remain open.
