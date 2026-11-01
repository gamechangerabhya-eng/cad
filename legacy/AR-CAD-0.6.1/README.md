# AR CAD — Android CAD application

**Brand:** AR CAD
**Source version:** 0.6.1
**Application ID:** `com.arcad.app`

AR CAD is an original Android-first CAD application. It is not an Autodesk product and does not use Autodesk branding. A Rust C-ABI, JNI adapter and Kotlin native-document lifecycle wrapper exist around CADCraft 0.15.1. The active Compose editor still uses an interim Kotlin model and is not yet synchronized with the native `Session`. The 0.6.0 milestone hardens Android release builds so they cannot silently package the JNI stub; 0.6.1 adds source-level native viewport and pointer-input APIs. Native Android compilation and device execution remain unverified.



## 0.6.1 native interaction API

The native bridge now exposes CADCraft viewport sizing and pointer movement/click APIs. Pointer motion delegates to the CADCraft session's native snap/tracking pipeline; clicks delegate to native selection or the active command prompt. Coordinates are passed as double precision world coordinates. The current Compose editor is not yet wired to these methods, so this does not yet remove the separate Kotlin editor document. See `NATIVE-INPUT-API.md`.

## 0.6.0 Android release-gate and integration comparison

- Added `ANDROID-ARCHITECTURE-DECISION.md` comparing the existing Compose/JNI route with the ArtCraft Android GameActivity/egui/wgpu route. The current app keeps one Compose shell; an isolated CADCraft adapter prototype is required before selecting a replacement renderer. No competing render loop is introduced.
- Added `THREE-WAY-PARITY.md` comparing CADCraft desktop, ArtCraft Android and the current AR CAD source. The report records the pinned CADCraft `v0.15.1` release (`660e15c9f4` commit prefix) and ArtCraft Android `v0.3.0` (`9c2af6f` prefix). The ArtCraft main-branch full SHA could not be retrieved because GitHub DNS is unavailable in this environment.
- Release CMake configuration now fails when the real per-ABI Rust `libarcad_ffi.so` is missing. Only Debug explicitly opts into the JNI stub.
- `preReleaseBuild` now depends on a Rust Android build and a release native-library verification task for `arm64-v8a` and `x86_64`.
- Added ELF64/ET_DYN and ABI-machine validation for Rust shared libraries, Gradle/R8 JNI keep rules, and static release-gate regression tests.
- `native/build-android.sh` can discover an NDK through Android SDK environment variables or `local.properties`, generates the FFI lockfile for a development bootstrap and uses `--locked` thereafter. Gradle Release sets `ARCAD_REQUIRE_LOCKFILE=1` and refuses to build until the lockfile is present.
- Created local Git branch `work/ar-cad-1.0` with a baseline commit because the supplied AR CAD folder had no `.git` metadata. The original upstream history could not be reconstructed from the source archive.
- This milestone does not complete the native document migration, 2D parity, 3D, DWG or release build. No APK/AAB is claimed.

## 0.5.9 source audit and build bootstrap

- Added `SOURCE-AUDIT.md` and `DEVELOPMENT-CHECKLIST.md`, based on inspection of the actual 0.5.8 working tree and vendored CADCraft 0.15.1 sources. The report distinguishes the Kotlin editor from the separate Rust native document/preview bridge.
- Fixed the Android Rust build script so a missing FFI crate lockfile does not make the first `cargo ndk build` fail solely because of `--locked`. The first build can generate `native/arcad-ffi/Cargo.lock`; later builds use `--locked`. Commit that generated lockfile before a reproducible production release.
- Added Rust/JNI/Kotlin prompt-driven command APIs (`startCommand` / `inputCommand`) that expose CADCraft interactive prompts and accept bounded point, keyword, text, pick, enter and cancel input DTOs. This is a bridge capability only; the active Compose editor still does not use the native Session.
- Added a mocked build-script test. It validates argument selection only and is not a substitute for an NDK build.

## Implemented in 0.5.8

- Added a request gate for asynchronous native previews. A superseded render cannot replace a newer preview, and a render is discarded if the editor entities or layer list changed while it was running.
- Closing the preview invalidates in-flight results. PNG bytes and decoded dimensions are checked before the image is displayed, including a 16-megapixel ceiling.
- Added host tests for request ordering, snapshot invalidation and explicit cancellation.
- Updated Android metadata to 0.5.8 (versionCode 22).

## Implemented in 0.5.7, 0.5.6 and 0.5.5

- Added `arcad_document_render_png` to the Rust C ABI. It renders model space through CADCraft `cadcraft_io::png` (the upstream display-list and raster pipeline), with per-side 16..4096 pixel bounds and a 16-megapixel ceiling.
- Added JNI `nativeRenderPng`, copying the Rust-owned PNG buffer into a JVM-owned byte array and releasing the native buffer on all normal paths.
- Added `CadCraftDocument.renderPreviewPng(width, height)` with serialized document access and matching dimension limits.
- Added host tests for PNG signature, default/custom sizes, dimension rejection and closed-document rejection.
- Updated Android version metadata to 0.5.7 (versionCode 21).

The toolbar Native PNG action now snapshots the current Kotlin editor drawing, converts its supported entities to an in-memory DXF, opens that DXF in a temporary CADCraft document, renders off the UI thread, validates/decodes the PNG and displays a preview dialog. The active drawing and undo history are not replaced. The round-trip is limited to the current editor entity set; it is a preview bridge, not yet native editing or bidirectional synchronization. Rust/Android native libraries have not been compiled or executed in this environment.

## Implemented in 0.4.2

### 2D model space
- Native Android UI using Kotlin and Jetpack Compose.
- Line, circle, rectangle, three-point arc, regular polygon (3–128 sides) and multi-point polyline creation.
- Selection by geometry hit testing and drag-to-move.
- Copy selected entity by a fixed model-space offset.
- Rotate selected entity by 90 degrees, mirror and 2× scale.
- Delete selected entity.
- Straight-line modification commands: TRIM and EXTEND against intersecting visible straight segments, rectangle edges and polyline edges.
- OFFSET of selected LINE, open/closed straight-segment polyline, rectangle, circle and arc entities by ±10 mm; polyline joins use miters with bevel fallback for excessive spikes, and generated geometry remains on the source layer.
- Polyline corner CHAMFER (5 mm edge setback) and approximate FILLET (5 mm radius, eight straight chord segments), applied to the vertex nearest the last tap. These currently operate on selected polylines only.
- Pan, pinch zoom and fit-to-extents.
- Grid and orthogonal constraint.
- Object snaps for endpoint, midpoint, center, quadrant, intersection and nearest point on supported 2D entities.
- Individual OSNAP mode switches, reset-to-default and a master OSNAP on/off override; disabled modes do not attract the cursor.
- Distance measurement between two tapped points.
- Selected-entity properties: line/polyline length, circle circumference and area, rectangle perimeter and area, closed-polyline perimeter and area, and arc length/radius.
- Regular polygon generation from centre and radius, with configurable side count.
- Layer manager: create layers, set current layer, hide/show layers.
- Layer assignment retained in the AR CAD document and DXF layer table.
- Undo/redo history in memory (up to 100 snapshots).

### Files and interchange
- Versioned AR CAD JSON document format (`.arcad.json`), including entities and layer names.
- Crash-resistant local recovery snapshots with a previous-valid-snapshot fallback; autosave is debounced after document/layer edits.
- Android Storage Access Framework for open/save.
- ASCII DXF import/export for `LINE`, `CIRCLE`, `ARC` and `LWPOLYLINE` entities.
- DXF export includes a layer table and declares millimetres; layer names and supported entity geometry are retained on round-trip.
- Common DXF `$INSUNITS` values are normalized to millimetres on import; unitless files are assumed to be millimetres.
- Import warns before replacing the current drawing if unsupported DXF entity types, block contents or object records are found.
- Input and output guardrails: 25 MB file limit, 100,000 entity limit and 1,000,000 vertices per polyline.

## Important limitations — not production-ready

- CADCraft's `doc`, `engine` and `io` crates are linked into the Rust FFI source crate, but the native Session is **not yet the active Compose editor document**. The current mobile model and DXF codec remain a limited interim implementation.
- DXF support is ASCII only. Binary DXF is not implemented.
- Only `LINE`, `CIRCLE`, `ARC` and `LWPOLYLINE` are imported. Other entity types are skipped and reported. Block contents and object records are reported as omitted; blocks are not preserved.
- DWG import/export is not implemented in the Android app. The CADCraft upstream DWG bridge is not yet cross-compiled or verified for Android.
- No real dimension entities, text/MTEXT, leaders, hatches, blocks, xrefs, splines, ellipses, constraints, layouts, paper space or plotting/printing.
- No 3D solids, extrusion, revolve, Boolean operations or 3D viewport.
- Autosave/recovery stores the current supported 2D model and layer names locally; undo/redo remains in-memory only. Recovery is not a substitute for explicit project saves.
- No cloud sync, collaboration, authentication, subscriptions or Play Store release pipeline.
- The document model currently uses millimetres; the user cannot change drawing units after creation, and precision settings/scale management are not complete.
- No independent AutoCAD compatibility corpus or device performance results yet.

## Build requirements

- Android Studio with Android SDK 35 and Android NDK (NDK version selected by the SDK installation).
- JDK 17 or newer.
- Rust 1.95+, Cargo and `cargo-ndk` for the native CADCraft engine.
- Android Gradle Plugin 8.7.3, Kotlin 2.0.21 and Gradle 8.9-compatible runtime.
- This source snapshot currently has **no Gradle wrapper**; use Android Studio or an installed Gradle 8.9 distribution until a wrapper can be generated and checked in.

### Debug build

The Debug variant explicitly allows the JNI development stub when Rust libraries are absent. In that state the UI may launch, but native CADCraft commands are unavailable. This is not a usable CAD engine build.

### Release build

1. Install Rust 1.95+, `cargo-ndk`, Android SDK and NDK.
2. Run `bash native/build-android.sh` once to generate `native/arcad-ffi/Cargo.lock` and build the ARM64/x86_64 Rust libraries.
3. Review and commit the generated Cargo.lock so dependency resolution is reproducible.
4. Run `gradle :app:assembleRelease` or `gradle :app:bundleRelease` using Gradle 8.9. Release tasks require the lockfile and build the real Rust libraries; CMake fails if a native library is missing. Release signing must be configured separately before distribution.

This environment did not contain Rust/Cargo, cargo-ndk, Gradle, Android SDK/NDK, or a connected emulator/device. **No APK/AAB was built or installed in this revision.**

## Run the verified pure-Kotlin tests

Requires Kotlin compiler (`kotlinc`) and Java:

```bash
./tools/run-dxf-tests.sh
```

The test suite covers ASCII DXF parsing, supported entity mapping, layer table round-trip, unsupported entity reporting, 2D loss reporting for elevated/bulged geometry, UTF-8/file-size limits, malformed/truncated input, 500 deterministic mixed-entity round-trips, arc-angle conversion, recovery-store atomic snapshot/fallback behavior, geometric measurements, and line TRIM/EXTEND/OFFSET operations, polyline corner fillet/chamfer geometry, polyline offset joins/direction/invalid-input cases, object snap candidate priority/intersections/arc sweep/large-drawing filtering, per-mode snap enable/disable behavior, and live dynamic preview metric calculations.

## Upstream CADCraft

The source repository supplied for this project is CADCraft 0.15.1, licensed MIT OR Apache-2.0. Preserve the upstream `LICENSE-MIT`, `LICENSE-APACHE`, `NOTICE` and attribution notices when redistributing its source or derived binaries. The Android integration plan and known blockers are in `CADCRAFT-ANDROID-PORT-STATUS.md`.


## 0.4.4 scope note

Polyline FILLET is represented by an eight-segment chord approximation because the interim mobile entity model does not yet retain bulge values or analytic arc segments inside polylines. It is not a precision/production fillet. CHAMFER creates exact straight setback points. Both are limited to one selected polyline vertex per operation; the user taps near the desired vertex before pressing the toolbar action.


## 0.4.5 scope note

OFFSET now supports straight-segment polylines, rectangles, circles and arcs in addition to LINE entities. Polyline offset uses miter joins and bevels joins exceeding an 8× miter limit. Positive offset follows the left-hand normal of the polyline vertex order; it is not a universal "outward" direction for closed shapes. Complex concave offsets are not topology-cleaned, so self-intersections and collapsed regions must be reviewed and manually edited. Circle/arc offsets are concentric radius changes and reject zero/negative resulting radii.


## Version 0.4.6 — object snapping

Object snapping now supports endpoints, segment and polyline midpoints, circle/arc centers, circle/arc quadrants, nearest points and intersections between supported straight and round entities. Snap tolerance stays approximately 12 screen pixels as the user zooms. Hidden layers are excluded and arc snaps are constrained to the arc sweep. See `QA-STATUS.md` for verified tests and remaining limitations.


## 0.4.8 dynamic input

Coordinate entry accepts absolute `x,y`, relative `@dx,dy`, and polar `@distance<angle` (degrees). Numeric input is exact and bypasses object snap/ortho constraints. Snap result glyphs distinguish endpoint, midpoint, intersection, center, quadrant and nearest.

## 0.5.0 — construction snap tracking
- Added construction tracking for line extensions, perpendicular projections and tangent points to circles/arcs.
- Conventional OSNAP candidates take precedence; tracking is a fallback and can be switched independently with TRACK ON/OFF.
- Construction guides and the tracking target are rendered in model space. Tracking uses the active command's first point or the latest polyline vertex as its anchor.
- Tracking geometry is covered by a separate host-JVM regression suite. The Android Compose screen itself still requires Gradle/device validation.

## Version 0.5.1

Adds a packed immutable AABB/BVH index for visible drawing geometry. The Android OSNAP path queries only geometry whose conservative bounds intersect the cursor tolerance window. The index is rebuilt when the drawing or visible layers change. Construction tracking intentionally remains on the full snapshot until a safe ray-aware spatial query is implemented.

Run host-side verification with:

```bash
./tools/run-dxf-tests.sh
```

This verifies pure Kotlin geometry/codec/recovery modules, not Android Gradle compilation or device runtime. See `QA-STATUS.md` and `RELEASE-GAPS.md` for the exact release state.


## 0.5.2 — native bridge foundation

- Added `native/arcad-ffi`, a Rust C ABI over CADCraft `Session` and `cadcraft-io` for document create/open/save, command execution, undo/redo and entity count.
- Added a C header, bounded input handling, panic containment and matching native memory release functions.
- Vendored upstream CADCraft source with original license and NOTICE files.
- Run `native/check-ffi-contract.py` to verify the C/Rust symbol contract.
- **Not yet runtime-integrated:** Rust/Cargo and Android NDK builds have not been run; JNI and Compose migration remain outstanding. See `native/README.md`, `QA-STATUS.md` and `RELEASE-GAPS.md`.


## 0.5.3 — JNI integration foundation

- Added JNI adapter and Kotlin `CadCraftDocument` handle owner. Calls are serialized and native handles are released exactly once.
- Added native CADCraft read-note retrieval for future data-loss warnings.
- Added CMake target and `native/build-android.sh` to build the Rust library for Android ABIs. If Rust libraries are absent, CMake builds a UI-only stub and the Kotlin bridge reports native CAD unavailable rather than pretending it is integrated.
- Host verification: 14 pure-Kotlin test programs, static C/Rust and Kotlin/JNI contract checks, and host Clang JNI syntax check passed.
- Host CMake build of the UI-only JNI stub passed. Android native/Gradle builds and device runtime remain unverified. The active Compose editor has not yet migrated to the native document engine.


## 0.5.4 native export safety

The Kotlin `CadCraftDocument.save()` wrapper reads CADCraft import notes before invoking the native writer. If the note list is non-empty or the diagnostics response is malformed, saving is blocked unless the caller passes `acknowledgeReadNotes = true` after presenting the warnings to the user. `exportPreviewSvg()` uses the native CADCraft SVG writer and does not overwrite the source drawing.

The Rust FFI copies bounded C strings into owned UTF-8 values and contains panics in document/buffer/string access and release functions. JNI rejects embedded NUL in command JSON and surfaces closed-document operations as exceptions.
