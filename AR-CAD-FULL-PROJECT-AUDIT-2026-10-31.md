# AR CAD — Full Project Revalidation

**Audit date:** 2026-10-31  
**Scope:** AR CAD source archive currently at `AR-CAD-Step1`, including CADCraft 0.15.1, ArtCraft Android build integration, Steps 1–10 source changes, test scripts and release workflow.

## Executive decision

**AR CAD is not yet a complete CAD application and is not release-ready.** The current project is a mobile-oriented shell and workflow layer around CADCraft 0.15.1. It contains meaningful 2D CAD functionality, but the upstream engine's own documentation identifies it as pre-alpha / approximately 38% ready for real work (the estimate is not an independent benchmark). Its generated menu breadth is reported as 298/491 (61%) in the 2026-10-30 parity checklist; menu presence is not feature depth or functional verification.

The previous “Steps 1–10 completed” wording must be interpreted narrowly: source changes and static source checks were prepared. It does **not** mean all CAD requirements, Android build, runtime, or release acceptance are complete. The 74/74 number is not 74 functional CAD tests; most checks search for source strings, constants, command names, files or workflow settings.

## Revalidation performed in this environment

| Validation | Result | Meaning |
|---|---|---|
| `bash tools/test-ar-cad.sh` | Passed source-only mode | Step 2–10 Python acceptance scripts, shell syntax and Python compile checks passed |
| Step 2 source assertions | 18/18 | Branding and mobile start-page source patterns present |
| Step 3 source assertions | 25/25 | Toolbar labels and command registration strings present; not interactive command tests |
| Steps 4–6 source assertions | 14/14 | Mobile UI/control source patterns present; not touch/file picker runtime tests |
| Steps 7–10 source assertions | 17/17 | Test-file presence and workflow/configuration strings present; not proof those Rust tests ran |
| Legacy AR CAD DXF/native test script | Passed | Tests the legacy Kotlin/C++/Rust FFI subset under `legacy/AR-CAD-0.6.1`, not the current CADCraft 0.15.1 Android app |
| Current CADCraft Rust tests | **Not run** | `rustc`, `cargo`, `rustup` unavailable |
| Android APK build | **Not run** | Android SDK/NDK, Gradle and cargo-ndk unavailable |
| APK signature/package inspection | **Not run** | No APK exists from this audit |
| Physical Android device test | **Not run** | No installed build or ADB connection |
| Performance/memory/thermal tests | **Not run** | No Android build or device benchmark |

Tool availability observed: `java` and `kotlinc` are present; `rustc`, `cargo`, `cargo-ndk`, `gradle`, `adb` are absent. The required Android SDK/NDK environment variables/directories were not established.

## Step-by-step correction

| Step | What is actually in source | Correct status |
|---|---|---|
| 1 — Android integration | ArtCraft build scripts, CADCraft 0.15.1 source, app identity and ARM64 build workflow | Source prepared; compile/build unverified |
| 2 — Mobile start/branding | AR CAD title/icon/package, responsive start page, recent drawings and Android-first defaults | Source assertions pass; visual/device verification pending |
| 3 — Drawing toolbar | File/Draw/Edit rows with command dispatcher calls for selected core commands | Source assertions pass; not all CAD commands are exposed and buttons not runtime-tested |
| 4 — Drafting/navigation | Grid/Snap/Ortho/Polar/OSnap/Dynamic Input status controls, Zoom Fit and existing pinch/pan code paths | Source assertions pass; touch/gesture behavior unverified on Android |
| 5 — Layers/properties | Existing layer and properties palette resized for mobile | UI adaptation only; full workflows unverified |
| 6 — File/command input | New/Open/Save/Save As toolbar, larger command surface; builder has generated CADCraft SAF picker/bridge recipe | Source integration exists; Android open/save, URI permissions, overwrite and persistence unverified |
| 7 — Reliability | CADCraft engine/UI test source files for autosave/recovery/save backup/DXF roundtrip are present | Tests not executed; source test script checks file presence |
| 8 — QA automation | Unified runner and GitHub Actions Rust-test gate | Runner source checks pass; current Rust suites not executed locally or in Actions |
| 9 — Performance/compatibility | ARM64 target and release LTO configuration | Configuration only; no device performance or compatibility evidence |
| 10 — Release preparation | Debug default, persistent signing-secret checks, package/signature/checksum steps | Workflow prepared; no actual APK/release artifact verified |
| 11 — Deploy | No verified signed APK, Play Store/internal distribution or deployment | Not complete |

## CAD capability audit

### 2D drafting and editing

**Present in engine:** lines, polylines, circles, arcs, rectangles, ellipses, splines, polygons, text, a broad set of modification commands, selection, grips, layers, object snaps, polar/ortho and object-snap tracking. The latest UI-parity notes describe tracking, extension and parallel paths as implemented. Some older roadmap summaries still say these are missing; the project documentation is inconsistent and should be reconciled.

**Important gaps:** ellipse/spline intersections and several related operations use tessellated approximations; spline fit/edit options remain incomplete; polyline offsets do not fully recompute arc bulges or clean loops; tolerances are largely absolute and may behave poorly at survey-scale or tiny geometry. Some command options and edge cases remain incomplete. These issues matter for precision/production drafting.

### Annotation and documentation

**Present:** core dimension commands, TEXT/MTEXT, leaders, tables and annotative-scale functionality exist in varying depth.

**Gaps:** no full DIMSTYLE manager; MTEXT in-place editing and fields are incomplete; leader styling/editing remains limited; table formulas and custom styling/data-link behavior are not fully interoperable. SHX fonts are not read, which can alter text metrics and appearance.

### Layers, blocks, references and hatching

**Present:** layers/properties, static blocks, block insertion, attributes, static Block Editor and substantial XREF attach/path operations are documented.

**Gaps:** no dynamic-block authoring; XREF editing in place and XCLIP are missing; some dynamic block behavior is read/partially editable only. Hatch pattern catalog and curved boundary fidelity are limited; HPGAPTOL, boundary modes and associative edge cases remain.

### Layouts, plotting and output

**Present:** layouts, viewports, page setups, plot dialog/preview, CTB/STB plot styles, PUBLISH and PDF/vector output are documented.

**Gaps:** binary PC3/CTB/STB handling and system printer integration are incomplete; some margins and plot settings do not persist or behave identically across formats. Plot output still needs a real-file/device verification matrix.

### File compatibility and data safety

**Present:** substantial ASCII/binary DXF and DWG read/write support, version handling, and DXF preservation of many unknown entities/objects and foreign data are documented.

**Critical gaps:** the engine's own gap list says DWG round-trips can still drop custom-class entities/objects, xdata on table records/blocks, colour-book names and reactors. Real-world DWG/DXF corpus coverage and black-box verification in AutoCAD are missing. A documented 10 MB DWG throughput/failure issue remains. QSAVE warnings and `.bak` reduce risk but do not make DWG round-tripping lossless.

### 3D CAD

**Not implemented to a useful 3D modeling level.** The engine documentation reports approximately 2% 3D parity and explicitly says there are no 3D solids, surfaces, meshes, ACIS/B-rep regions, 3D operations, visual styles, materials, lights or rendering. Stored Z values and 3D polylines do not constitute a 3D CAD modeler.

### Android-specific acceptance

The mobile toolbar, touch sizing, start page, status strip and generated SAF bridge are in source. They have not been validated on an Android device. In particular, verify keyboard/IME behavior, multi-touch selection vs pan/zoom, touch accuracy, file picker cancellation, URI read/write grants, Save As, large files, app background/resume, orientation changes and low-memory recovery.

## Release blockers (must close before claiming production-ready)

1. Build the exact source revision using the checked-in workflow; preserve the complete CI log and toolchain versions.
2. Run all current CADCraft Rust tests (`cargo test --locked` for the selected crates or the upstream full CI target) and formatting/lint checks; fix every failure.
3. Produce and inspect the ARM64 APK; verify package ID, launcher activity, ABI, native library, permissions, signing identity and checksum.
4. Install on a physical Android phone and run scripted smoke tests for new/open/save/save-as, draw/edit, undo/redo, layers/properties, coordinate entry, snaps, pan/zoom, keyboard, background/resume and crash recovery.
5. Test DXF and DWG with a licensed real-world drawing corpus, save/reopen in both directions, and verify files in AutoCAD/compatible CAD software. Never claim lossless DWG until this passes.
6. Fix or explicitly scope 2D precision and data-loss gaps; define the minimum CAD feature set AR CAD promises.
7. If the product promise includes 3D AutoCAD-like modeling, implement a genuine 3D geometry kernel, solid/surface/mesh tools, 3D viewport/navigation and file interoperability. This is a separate major workstream, not a deployment task.
8. Benchmark on target Android hardware: startup, 10k/100k/1M entities where feasible, memory, frame rate, battery/thermal behavior and large file load/save.
9. Complete signing, release notes, privacy/license notices, support/backup policy and distribution testing.

## Recommended scope decision

- **AR CAD 2D first release:** possible only after core 2D drafting, precision, file safety, plotting and Android runtime gates are proven. Label known DWG limitations honestly and avoid production-critical promises until resolved.
- **AR CAD full AutoCAD-like 2D + 3D:** not close to completion. The 3D kernel and modeling stack are absent, and major 2D interoperability/precision gaps remain.

## Final verdict

The ten development steps describe staged source/integration work, not a complete CAD feature checklist. The prior 74/74 count is a source-assertion score, not a CAD functional test score. Current verified state is: **source-level mobile integration prepared; legacy tests pass; current Rust tests, APK build, Android runtime, production 2D readiness and 3D CAD are unverified or incomplete.** Do not mark Step 11 as the only remaining work. Feature development and validation must continue before deployment.
