# Changelog

## 0.6.1 — Native viewport and CAD pointer input bridge

- Added Rust FFI endpoints to update the CADCraft viewport dimensions, move the world-space cursor through CADCraft's native OSNAP/grid/polar/ORTHO/object-snap-tracking pipeline, and submit pointer clicks to native selection or active command prompts.
- Native click handling now routes selection prompts through CADCraft's pick index and point prompts through `Session::snap_cursor` / `Session::input_snapped`; idle clicks use `Session::idle_click`.
- Added JNI and Kotlin document-wrapper methods for viewport size, pointer move and pointer click. World coordinates remain `f64` across Kotlin/JNI/C/Rust; Android callers supply world coordinates after applying their view transform.
- Added input validation for finite world coordinates, monotonic timestamps and bounded viewport dimensions; document operations remain serialized by the Kotlin document lock.
- Versioned all Rust bridge JSON response envelopes with `schemaVersion: 1` for forward-compatible DTO evolution.
- Extended the fake-native API regression test for pointer/viewport forwarding and invalid-input rejection.

**Not completed:** the existing Compose editor has not yet been switched to this API. UI gesture mapping, native selection highlighting, live native canvas rendering and removal of the Kotlin entity store remain outstanding. Rust compilation and Android runtime tests were unavailable.

### Verification
- `bash tools/run-dxf-tests.sh`: 17 Kotlin/JVM test programs passed; FFI/JNI contracts, native build-script tests and release-gate checks passed.
- Both JNI C++ translation units passed Clang C++17 syntax-only checks.
- Rust/Cargo compilation, Gradle build, Android ABI linking, APK/AAB generation and device tests were not run.


## 0.6.0 — Android release gate and architecture comparison

- Added an architecture decision record comparing Compose + JNI against the ArtCraft Android GameActivity/egui/wgpu integration.
- Added a three-way feature parity report for CADCraft desktop, ArtCraft Android and AR CAD, with source revision evidence and explicit fetch limitations.
- Made the JNI stub debug-only by default; release CMake configuration fails if the actual CADCraft Rust library is missing.
- Made `preReleaseBuild` and Release CMake/external-native tasks depend on the Rust Android build and verify both configured ABIs before release packaging.
- Added strict ELF64, ET_DYN and ABI machine checks for `arm64-v8a` and `x86_64` shared libraries.
- Added R8 keep rules for JNI class/method names.
- Improved NDK discovery from Android SDK environment variables and `local.properties`.
- Changed the native bootstrap to generate `native/arcad-ffi/Cargo.lock` for development once, then build with `--locked`; Gradle Release requires the lockfile to exist before it starts.
- Added release-gate and ELF/build-script regression tests.
- Initialized a local `work/ar-cad-1.0` Git branch with a safety baseline because the provided source tree had no Git metadata.

**Not completed:** Rust compilation, Android Gradle build, APK/AAB generation, native Session ownership migration, live native viewport, full 2D parity, DWG validation, 3D solid modelling and device testing.

ative preview race and image safety

- Added `NativePreviewGate` to associate each background render with its immutable entity/layer snapshot and a monotonically increasing request token.
- Newer preview requests supersede older work; edits made while a render is in progress cause the result to be discarded instead of showing stale geometry.
- Closing the preview invalidates pending results. PNG byte length and decoded dimensions are validated before display, with a 16-megapixel limit.
- Added host tests for current request acceptance, supersession, snapshot mismatch and invalidation.
- Updated Android metadata to 0.5.8 (versionCode 22).

### Verification
- `bash tools/run-dxf-tests.sh`: 17 Kotlin/JVM test programs passed; FFI/JNI static contract checks passed.
- Both JNI C++ translation units passed Clang C++17 syntax-only checks.
- Rust compilation, Android Gradle build, Android NDK linking and device rendering remain unverified.

## 0.5.7 — Native preview of the active editor drawing

- Replaced the separate-file Native PNG picker with a preview of the **current editor drawing**. The UI snapshots the current entities/layers, converts supported Kotlin entities to an in-memory ASCII DXF, and opens that buffer in a temporary CADCraft native document.
- Native rendering remains on `Dispatchers.IO`; the temporary native document is closed in `finally`, and the active drawing plus undo/redo history are left untouched.
- Preview status reports CADCraft import-conversion diagnostics when present. Empty drawings and DXF preview-size overflow are rejected before native open.
- This is a one-way preview bridge through DXF. Native editing, bidirectional state synchronization, and exact fidelity for unsupported CADCraft/DXF entities are not claimed.
- Updated Android metadata to 0.5.7 (versionCode 21).

### Verification
- Host Kotlin/JVM regression suite and C++/FFI/JNI source checks are rerun for this milestone; see `QA-STATUS.md` for the actual result.
- Android Gradle/Compose compilation, Rust compilation, Android ABI build and device rendering remain unverified because required toolchains are unavailable.

## 0.5.6 — Native CADCraft preview entry point

- Added a dedicated **Native PNG** action to the Compose toolbar. It opens a source CAD file independently and requests a 1600 × 1000 PNG from `CadCraftDocument.renderPreviewPng`.
- Added bounded 64 MB input reading, background-thread file read/native render, deterministic native document close, PNG decode validation, and a preview dialog.
- Kept the native preview separate from the active Kotlin editor document: previewing a file does not replace or mutate the current drawing.
- Native library absence or import/render failure is reported in the editor status message; the existing editor remains usable.
- Updated Android app metadata to 0.5.6 (versionCode 20).

### Verification
- `./tools/run-dxf-tests.sh`: all 16 Kotlin/JVM test programs passed; FFI/JNI static contracts passed.
- Android Gradle/Compose compilation and native rendering runtime were **NOT RUN** because Android SDK/Gradle/Rust toolchains are unavailable. The new Compose flow is source-integrated but not device-verified.

### Remaining integration gap
- This is a separate native file-preview workflow, not a migration of the active editor's in-memory drawing to CADCraft. Live canvas synchronization, native editing commands and device validation remain outstanding.

## 0.5.5 — CADCraft native raster preview API

- Added `arcad_document_render_png` to the Rust C ABI, rendering model space with CADCraft's own `cadcraft_io::png` pipeline.
- Added strict preview dimensions (16..4096 per side) and a 16-megapixel pixel budget before rendering.
- Added JNI `nativeRenderPng`, with Rust buffer ownership released after copying into a JVM byte array.
- Added `CadNativeApi.renderPng` and `CadCraftDocument.renderPreviewPng`, serialized with all other document operations.
- Added host-side tests for PNG signature, default/custom dimensions, invalid sizes and closed-document rejection.
- Updated Android app metadata to 0.5.5 (versionCode 19).

### Verification
- `./tools/run-dxf-tests.sh`: all 16 Kotlin/JVM test programs passed; FFI and JNI static contracts passed.
- Host Clang C++17 syntax-only checks for real JNI adapter and development stub passed with JDK 21 JNI headers.
- Rust/Cargo compilation, Android NDK/Gradle build, native runtime, and device rendering remain **NOT RUN** (toolchains unavailable).

### Not implemented in this milestone
- The active Compose canvas does not yet display this PNG; the native preview is exposed through the document wrapper only.
- CADCraft is not yet the active document model in the Compose editor.
- No production APK or device-tested release is claimed.

## 0.4.9 — 2026-10-31

- Added live single-pointer model-space cursor tracking without consuming drawing taps or drag gestures.
- Added live OSNAP resolution and snap glyph updates while moving the pointer, not only after tapping.
- Added dynamic drafting preview for LINE, RECTANGLE, CIRCLE, POLYGON, ARC, MEASURE and the next POLYLINE segment.
- Added live ΔX, ΔY, distance and angle readout from the active first point.
- Cached converted snap geometry between pointer events; cache invalidates when the drawing entities or hidden-layer set changes.
- Added host-side tests for dynamic preview metrics, axis-aligned, diagonal, zero-length and invalid-coordinate cases.

# Changelog

## 0.4.8 — 2026-10-31

- Added locale-independent dynamic coordinate entry: absolute `x,y`, relative `@dx,dy`, and relative polar `@distance<angle` in degrees.
- Added numeric point placement for Line, Circle, Rectangle, Polyline, Polygon, Arc and Measure workflows.
- Explicit numeric coordinates bypass OSNAP and ORTHO so entered values remain exact.
- Added visible on-canvas OSNAP glyphs for endpoint, midpoint, intersection, center, quadrant and nearest candidates, plus a snap-kind readout.
- Added parser validation for malformed, non-finite and out-of-range coordinates.
- Added host-side regression tests for absolute, relative Cartesian, relative polar, negative-angle and invalid-input cases.

## 0.4.7
- Added independent object snap mode controls and reset-to-default.

## 0.5.0 — construction snap tracking
- Added host-testable construction tracking candidates for segment extension rays, perpendicular projections and circle/arc tangent points.
- Added explicit construction guide geometry for visual feedback.
- Integrated tracking as a fallback after conventional OSNAP, anchored to the current command point or last polyline vertex.
- Added TRACK ON/OFF control and dashed guide/target rendering in model space.
- Added regression tests for forward/reverse extension rays, perpendicular foot bounds, exact circle tangency, arc sweep filtering and invalid inputs.

## 0.5.1 — Persistent spatial snap index

- Added immutable packed AABB/BVH spatial index for visible snap geometry.
- Index is rebuilt only when the drawing or visible-layer set changes; pointer events query a local model-space window.
- Integrated indexed candidates into the Android OSNAP resolver while preserving the existing full-geometry list for construction tracking, whose extension rays can originate beyond the cursor window.
- Conservative bounds include segments, polylines, circles and arcs (arcs use full-circle bounds to avoid missing valid sweep candidates).
- Added a 10,004-entity host test covering local pruning, empty queries, entity bounds and indexed-vs-full snap equivalence.
- Android version updated to 0.5.1 (versionCode 15).

## 0.5.2 — CADCraft native bridge foundation

- Vendored the complete supplied CADCraft 0.15.1 source tree alongside the original upstream archive, retaining MIT/Apache license files and NOTICE.
- Added `native/arcad-ffi`, a Rust `cdylib`/`staticlib` crate exposing opaque document lifecycle, CADCraft file open/save, command execution, undo/redo and entity-count functions.
- Added bounded C ABI inputs, panic containment, structured JSON command responses, explicit buffer/string deallocation and a C header for a future JNI adapter.
- Added a static C-header/Rust-export contract checker and verified the vendored `doc`, `engine` and `io` crate paths.
- Updated Android app metadata to 0.5.2 (versionCode 16).
- IMPORTANT: the bridge is a source-level integration foundation only. Cargo/Rust 1.95 and Android NDK builds were unavailable, so the bridge has not been compiled. Compose still uses the interim Kotlin document model; no native `.so`, APK or device runtime is claimed.

## 0.5.3 — JNI adapter and Kotlin native document lifecycle

- Added `arcad_jni.cpp`, a JNI adapter for the Rust C ABI. It converts Java UTF-16 strings to UTF-8, copies file byte arrays, returns JVM-owned byte arrays/strings, and releases Rust-owned buffers/strings.
- Native creation/open/save failures are surfaced as Java `IllegalStateException`s instead of silently returning a null handle/buffer.
- Added Kotlin `NativeCadBridge` with lazy native library loading, injectable `CadNativeApi`, and `CadCraftDocument` ownership wrapper. Document operations are serialized; close is idempotent; calls after close are rejected.
- Added native read-diagnostics API so CADCraft `Drawing.read_notes` can be retrieved before any overwrite decision.
- Added CMake JNI target and Android ABI filters (`arm64-v8a`, `x86_64`), plus `native/build-android.sh` using cargo-ndk to generate the per-ABI Rust shared libraries. CMake now uses an explicit development-only stub if the Rust library is absent; Kotlin detects the stub and blocks native operations, so the interim Android UI can still compile.
- Added lifecycle tests and Kotlin ↔ JNI symbol contract checks.

### Verification
- `./tools/run-dxf-tests.sh`: passed all existing 13 test programs plus `CadNativeDocumentTest` (14 programs total); the script also ran the C/Rust and Kotlin/JNI static contract checks.
- `native/check-ffi-contract.py`: passed (15 C declarations ↔ Rust exports).
- `native/check-jni-contract.py`: passed (10 Kotlin native methods ↔ C++ JNI exports).
- Host Clang syntax checks of `arcad_jni.cpp` and `arcad_jni_stub.cpp`: passed using JDK 21 JNI headers.
- Host CMake configure/build of the UI-only JNI stub target: passed; Android NDK cross-build remains unverified.
- Android manifest and Gradle configuration were source-inspected only.

### Not verified
- Rust/Cargo compilation and Rust unit tests: not run; Rust/Cargo are unavailable.
- Android Rust ABI builds: not run; Android NDK and cargo-ndk are unavailable.
- CMake/Gradle Android build, emulator or physical-device tests: not run; Android SDK/Gradle are unavailable.
- Compose still uses the interim Kotlin drawing model. JNI bridge source exists but is not yet invoked by the active editor.

### Release decision
NOT production-ready. This milestone implements the JNI boundary and safe Kotlin handle lifecycle, but it does not complete native engine migration, runtime validation, DWG data-preservation checks, or 3D functionality.


## 0.5.4 — Native save safety and export hardening

- Added a fail-closed import-diagnostics gate before native save. A drawing with CADCraft read notes cannot be saved unless the caller explicitly acknowledges the notes.
- Added a native SVG preview export path using CADCraft's format-aware `cadcraft_io::write` implementation.
- Removed an unnecessary unconstrained lifetime from the FFI C-string helper; C strings are now validated and copied into owned UTF-8 strings before use.
- Added panic containment to remaining Rust C ABI buffer/document/string access and release functions so Rust unwinds do not cross the C ABI boundary.
- JNI rejects embedded NUL in command/parameter strings and reports closed-document calls as Java exceptions rather than ambiguous zero/JSON values.
- Added native save-safety tests for clean drawings, non-empty diagnostics, explicit acknowledgement, malformed diagnostics and SVG export.

## 0.5.9 — Source audit and native Android build bootstrap

- Audited the actual 0.5.8 working tree and the vendored CADCraft 0.15.1 workspace. Added `SOURCE-AUDIT.md` with architecture findings, verified source inventory, current feature boundaries, build risks and evidence-based readiness conclusion.
- Added `DEVELOPMENT-CHECKLIST.md` to track each requested feature with implementation evidence, explicit status and remaining verification.
- Fixed a concrete native build bootstrap issue: `native/build-android.sh` no longer unconditionally passes Cargo `--locked` when `native/arcad-ffi/Cargo.lock` is absent. Cargo can now resolve and generate the FFI lockfile on the first build; once the lockfile exists, subsequent builds use `--locked`.
- Added a mocked build-script regression test for both missing-lock bootstrap and locked mode. This tests script argument selection only; it does not compile Rust or Android binaries.
- Extended the Rust/JNI/Kotlin bridge with prompt-driven command start and input APIs. The bridge returns CADCraft prompt state after each interaction and accepts bounded point, keyword, text, pick, enter and cancel inputs. This creates the necessary command-line interaction seam for a native Android editor; the current Compose editor has not yet been migrated to use it.
- Added host wrapper tests for interactive command lifecycle calls and extended static ABI checks to 18 C functions and 14 JNI methods.
- Updated Android metadata to 0.5.9 (versionCode 23).

### Verification
- `bash tools/run-dxf-tests.sh`: all 17 Kotlin/JVM test programs and source contract checks passed.
- `python3 tools/check-native-build-script.py`: missing-lock bootstrap and locked-mode command selection passed using a fake cargo-ndk executable.
- Rust/Cargo, Android NDK, Gradle, JNI runtime and device tests remain NOT RUN in this environment.
