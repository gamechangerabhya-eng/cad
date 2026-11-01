# AR CAD source audit — 2026-10-31

## Scope and method

Audited the checked-out project at `/mnt/data/AR-CAD`, including the Android application sources, native C/JNI bridge, Rust FFI crate, vendored CADCraft 0.15.1 workspace manifests and relevant geometry/document/engine/render/IO source trees. Ran `bash tools/run-dxf-tests.sh` before changes. This is a source audit plus host-side verification; it is **not** an Android or Rust build.

Repository inventory observed:

- 14 Kotlin application source files.
- 18 Kotlin test source files across `app/src/test` and `tools/tests`.
- 821 Rust source files in the vendored CADCraft crates.
- 12 CADCraft crates.
- 408 Rust test source files under the vendored crates (unit and integration test file count; not executed here).
- Rust/JNI bridge consists of `native/arcad-ffi/src/lib.rs`, `native/include/arcad_ffi.h`, `app/src/main/cpp/arcad_jni.cpp` and the unavailable-engine stub.

## Architecture findings

1. The vendored CADCraft workspace is Rust 2024, declares Rust 1.95, and includes document, geometry, engine, rendering, IO, constraints, DXF/DWG and egui UI crates.
2. The upstream desktop shell uses `eframe`/`egui`, `egui-wgpu` and `wgpu`. The Android app does **not** currently host that egui/wgpu UI. It is a separate Kotlin/Jetpack Compose application.
3. The Kotlin Compose editor currently owns the active drawing state (`CadEntity` sealed hierarchy and Compose mutable state). Its current entity model is LINE, CIRCLE, RECT, POLYLINE and ARC.
4. The Rust FFI owns a CADCraft `Session` for a separate native document. It exposes create/open/close, entity count, diagnostics, programmatic command execution, prompt-driven command start/input, undo/redo, save and PNG rendering. The current Compose editor only uses it for an isolated preview created from a temporary DXF snapshot; the new prompt-driven APIs are not yet wired to Compose.
5. At the 0.5.9 baseline, CMake built a JNI stub whenever the per-ABI Rust shared library was absent. In 0.6.0 this is restricted to explicit Debug configuration; Release now fails closed and requires the Rust library.
6. `native/build-android.sh` previously always passed `--locked`, but the project had no `native/arcad-ffi/Cargo.lock`. The upstream workspace lock is in a different workspace and does not satisfy the FFI crate's own lockfile requirement. This was a concrete Android native-build bootstrap blocker and is corrected in 0.5.9; production reproducibility still requires committing the generated FFI lockfile.
7. Android configuration currently targets SDK 35, min SDK 26, and ABI filters `arm64-v8a` and `x86_64`. The manifest has a launcher activity and no runtime permissions declared. No Gradle wrapper is present in the checked-in project tree.

## Current feature assessment

| Area | Evidence in current source | Audit status |
|---|---|---|
| 2D primitives | Kotlin LINE, CIRCLE, RECT, POLYLINE, ARC; polygon tool emits a closed polyline | IMPLEMENTED (limited model) |
| 2D transforms | Single-selection drag move, fixed-offset copy, 90-degree rotate action, mirror and 2x scale | IMPLEMENTED (limited UI actions) |
| Trim / extend | Kotlin line-only operations against straight segment cutters/boundaries | IMPLEMENTED (restricted) |
| Offset | Line, polyline, circle, arc and rectangle paths; polyline self-intersection cleanup is not implied | IMPLEMENTED (restricted) |
| Fillet / chamfer | One selected polyline corner; fillet is approximated with line segments | IMPLEMENTED (restricted; not exact arc topology) |
| Undo / redo | Kotlin entity-list snapshots; native Session undo/redo exists separately | IMPLEMENTED in Kotlin; native path not integrated |
| Precision / input | Double-precision helpers for measurements and modify operations; editor stores coordinates as Float; dynamic coordinate parser and object snaps | PARTIAL |
| Snapping | Endpoint, midpoint, center, quadrant, nearest and intersection candidates; extension/perpendicular/tangent tracking; spatial index | IMPLEMENTED (2D subset) |
| Layers | Layer names, active layer and visibility; no complete persistent CAD layer-property editor | PARTIAL |
| Text / MText / dimensions / leaders | Not represented in the Kotlin active editor model | NOT IMPLEMENTED in active editor |
| Ellipse / spline / blocks / hatch / fill | Not represented in the Kotlin active editor model | NOT IMPLEMENTED in active editor |
| DXF | Kotlin parser/writer supports bounded ASCII subset: LINE, CIRCLE, ARC, LWPOLYLINE and layers; unsupported types are reported/skipped | PARTIAL; binary DXF unsupported by Kotlin codec |
| DWG | CADCraft upstream IO has DWG bridge code; Android ABI build and fidelity are not verified | BLOCKED / NOT VERIFIED |
| Native AR CAD JSON | Versioned JSON serializer/deserializer in MainActivity; user-selected Open/Save | IMPLEMENTED (limited schema) |
| Recovery | Single local autosave snapshot with previous-snapshot fallback and validation | IMPLEMENTED (single active drawing; no project catalog/history) |
| SVG / PDF / JPEG | CADCraft upstream has SVG/PDF/PNG IO; Android Kotlin active editor does not expose these; JPEG not established | NOT IMPLEMENTED in Android UI |
| Native rendering | CADCraft PNG API and temporary-document preview path | SOURCE IMPLEMENTED; Android runtime NOT VERIFIED |
| 3D | CADCraft has 3D vectors and some DXF 3D entities, but no integrated solid-modeling kernel or Android 3D editor in this app | NOT IMPLEMENTED as requested |
| Android lifecycle / runtime | Compose activity and SAF pickers exist; no Android build/device test available | PARTIAL / NOT VERIFIED |
| Project management | Open/Save single drawing via SAF; no in-app catalog, recent list, thumbnails, rename/duplicate/delete, or version history | PARTIAL |

## Build and dependency risks

- `rustc`, `cargo`, Android SDK, Android NDK and Gradle are unavailable in this execution environment. Rust tests, Android Rust cross-compilation, CMake/Gradle builds, JNI load tests and device tests cannot be performed here.
- The FFI crate depends on CADCraft `doc`, `engine` and `io` by local path. These crates bring the upstream workspace dependency graph, including Git-pinned craft-libs/egui components. Android compatibility and transitive native dependencies must be validated by an actual NDK build.
- The upstream eframe/wgpu desktop application is not itself an Android app integration. A deliberate Android renderer/window/surface strategy is required; do not assume the desktop `eframe` shell can simply be embedded in Compose.
- `arcad_jni.cpp` syntax is host-checkable with JDK headers, but a successful syntax check is not proof of Android ABI compatibility or shared-library loading.

## Baseline test result

Before the 0.5.9 change, `bash tools/run-dxf-tests.sh` passed all 17 Kotlin/JVM test programs, FFI/JNI source contracts and CMake source contract checks. The 0.5.9 work adds two C ABI functions and two JNI methods, and the wrapper regression tests now cover prompt-driven calls. This suite does not compile Rust or Android code.

## Audit conclusion

The existing project contains useful 2D drafting work and a substantial upstream Rust CADCraft codebase. However, the requested end state is materially beyond the current Android app: the active editor is not Rust-owned, the Kotlin model is a small 2D subset, Android native execution is not built or tested, and a true 3D solid kernel/editor is absent. Overall production readiness is therefore **not verified and not production-ready**. Historical percentage estimates are not reused as current measurements.


## 0.6.0 upstream comparison and Git safety update

- CADCraft release `v0.15.1` is pinned for this integration review. The upstream release page reports commit prefix `660e15c9f4`; the vendored Cargo workspace version is `0.15.1`. The source was supplied as an archive without `.git`, so full commit metadata cannot be independently recovered from the local checkout.
- ArtCraft Android release `v0.3.0` reports commit prefix `9c2af6f`. Its public `architecture.md` describes a GameActivity/eframe/wgpu launcher, Android-patched winit/egui-winit/egui-wgpu/android-activity, SAF, IME, touch/mouse/trackpad support and GPU surface recovery. The main branch architecture page was reviewed, but its full HEAD SHA could not be retrieved.
- `git ls-remote` and clone attempts for both repositories failed because `github.com` DNS is unavailable from this container. No upstream source update or vendor patch was applied.
- The supplied AR CAD source folder had no `.git` directory. A local repository was initialized on branch `work/ar-cad-1.0` and a baseline commit was created to protect the supplied state before 0.6.0 changes. This does not recreate upstream history.
- See `THREE-WAY-PARITY.md` and `ANDROID-ARCHITECTURE-DECISION.md` for capability comparison and the renderer decision.
- 0.6.0 makes Release CMake fail closed without the Rust shared library, wires the Gradle release task to the Rust build, adds ELF ABI verification and R8 JNI keep rules. Host tests pass; actual Gradle/NDK execution remains unavailable.
