# CADCraft → AR CAD Android port status

**AR CAD source status:** 0.6.1. Native viewport/pointer APIs are now present in Rust FFI/JNI/Kotlin; the Compose editor has not yet been wired to them. See `NATIVE-INPUT-API.md` and `QA-STATUS.md`.

## Supplied upstream source

- Repository: `storytold/cadcraft` (user-supplied archive)
- Workspace version: 0.15.1
- Minimum Rust version declared by upstream: 1.95
- Licences: MIT OR Apache-2.0; `NOTICE` also identifies the unmodified `acadrust` dependency under MPL-2.0.

## Reusable engine components found

| Crate | Existing responsibility | AR CAD plan |
|---|---|---|
| `cadcraft-geom` | f64 geometry, curves, intersections, offsets, snaps | Reuse as geometry kernel |
| `cadcraft-doc` | Drawing database, entities, symbol tables, blocks, layouts | Make authoritative document model |
| `cadcraft-engine` | Commands, prompts, selection, snaps, undo | Expose command/session API through stable FFI |
| `cadcraft-render` | Display lists and CPU rasterizer | Use for offscreen rendering / render DTOs |
| `cadcraft-io` | DXF/DWG read/write, SVG, PNG and PDF | Replace interim Kotlin DXF codec after JNI validation |
| `cadcraft-ui-egui` | egui desktop interface | Do not embed as the Android UX; retain Compose |
| `apps/cadcraft` | Desktop eframe application | Desktop target only |
| `apps/cadcraft-web` | WebAssembly eframe application | Browser target only |

## Specific Android blockers

1. No Android module, JNI bridge or UniFFI facade exists in the supplied upstream repository.
2. No Rust 1.95/Cargo or Android NDK is available in the current build environment; Android ABI compilation could not be attempted.
3. The workspace includes pinned Git dependencies from `storytold/craft-libs` and an egui fork. The mobile native target must avoid pulling desktop-only UI crates and must verify all dependencies for Android.
4. `cadcraft-dwg` uses `acadrust` for DWG conversion. The upstream code has input/output caps and fail-safe diagnostics, but its Android ABI support and native memory behavior are not verified here.
5. CADCraft itself warns that some unsupported DWG objects can be skipped and that saving may lose unmodelled data. AR CAD must surface read notes and never overwrite an original file silently.
6. The upstream roadmap reports 2% 3D modelling progress; a separate, tested solid-modelling implementation is required.

## Recommended bridge API

Create a Rust `cdylib` with a narrow C ABI/JNI adapter. Keep Rust structs private and pass versioned DTOs/byte buffers only:

- `arcad_core_version()`
- `arcad_document_new(units)` / `arcad_document_free(handle)`
- `arcad_document_open(bytes, format)` / `arcad_document_save(handle, format)`
- `arcad_document_entity_count(handle)` / `arcad_document_entity_at(handle, index)`
- `arcad_command_execute(handle, command, args_json)`
- `arcad_undo(handle)` / `arcad_redo(handle)`
- `arcad_selection_query(handle, bounds)` / `arcad_snap_query(handle, point, modes)`
- `arcad_render_rgba(handle, viewport, output_buffer)`

All FFI entry points must validate handles, lengths, numeric values and thread ownership; errors should return structured status/message pairs. Kotlin should own the lifecycle of the opaque document handle.

## Required integration test gates

- Host Rust tests for geometry/document/engine/io.
- Cross-compile `arm64-v8a` and `x86_64`; add `armeabi-v7a` only if required.
- JNI lifecycle, thread-safety, buffer ownership and cancellation tests.
- DXF corpus round-trip with entity/property loss reports.
- DWG import/export fixtures, opening outputs in licensed AutoCAD versions.
- Large/hostile file tests, memory budget, interruption and crash recovery.
- Physical device testing for touch, stylus, rotation, process death, SAF and long sessions.
- Reproducible signed APK/AAB build and Play Console release checks.

## Current status

The 0.4.0 Android source still uses a limited Kotlin-side model and ASCII DXF codec. The CADCraft crates are analysed but not linked. No native or Android build has been verified.

## 0.5.4 integration safety update

The Kotlin document wrapper now gates native saves on CADCraft import diagnostics. Non-empty or malformed `read_notes` responses block save unless the caller explicitly acknowledges the warnings. SVG preview export uses the native format-aware writer. The Rust C-string helper now returns owned UTF-8 strings, and all remaining pointer access/release exports contain unwinding. JNI rejects NUL-containing command payloads and reports closed-document operations as exceptions.

This is still not a completed engine migration: the active Compose editor has not been migrated to the native CADCraft document, and no Rust or Android ABI build has been performed in this environment.


## 0.5.5 native raster preview API

- Added a bounded C ABI render endpoint using `cadcraft_io::png` over model space.
- Added JNI byte-array transfer with explicit release of the Rust-owned `ArcadBuffer`.
- Added a serialized Kotlin `renderPreviewPng` method and host tests for PNG signature, dimensions and closed handles.
- Preview output is not yet connected to the active Compose canvas; this is an API milestone, not a completed renderer migration.
- Rust compilation, Android ABI build, Gradle build and runtime image rendering remain unverified.


## 0.5.6 native preview UI seam

The Compose toolbar now has an explicit **Native PNG** action. It opens a user-selected source CAD file into a temporary `CadCraftDocument`, renders a 1600 × 1000 PNG on `Dispatchers.IO`, closes the native document and shows the decoded image in a dialog. Input is capped at 64 MB. This is intentionally separate from the active Kotlin editor state, so it does not imply document synchronization. The UI path is source-integrated only; Android compilation, native library loading and on-device rendering remain unverified.


## 0.5.7 active-editor preview bridge

The Native PNG action now snapshots the active Kotlin editor's supported entity types and layer names, converts them to bounded in-memory ASCII DXF, opens the data in a temporary CADCraft native document, renders a 1600 × 1000 preview off the UI thread, and closes the native handle. The preview is therefore based on the current drawing rather than an independently selected file. The active Kotlin document and its undo/redo stacks are not changed.

This is a one-way preview adapter only. It does not migrate editing, selection, snaps, viewport state or undo/redo to CADCraft, and round-trip fidelity is limited by the Kotlin entity types and CADCraft DXF reader. Android and native runtime verification remain outstanding.


## 0.5.8 — Preview request safety

The Compose preview path now binds each asynchronous render to a request token and a snapshot of the current entities/layers. Superseded requests, changed snapshots and dismissed previews are prevented from publishing stale results. PNG byte length and decoded dimensions are checked before display. Host regression coverage includes request ordering, snapshot mismatch and invalidation. This remains a Kotlin-to-DXF-to-temporary-native-document preview bridge, not native editor ownership.


## 0.5.9 source audit and Android build bootstrap

The current project tree was audited on 2026-10-31. See `SOURCE-AUDIT.md` for the full source inventory, architecture, feature matrix and build risks, and `DEVELOPMENT-CHECKLIST.md` for per-feature status. The key architecture finding remains unchanged: the Compose editor owns the active Kotlin drawing; CADCraft `Session` is exposed through JNI but used only by the temporary preview path.

`native/build-android.sh` now uses Cargo `--locked` only when `native/arcad-ffi/Cargo.lock` exists. The first native build can generate that lockfile; it must be committed before a reproducible release. `tools/check-native-build-script.py` tests this decision with a fake cargo-ndk command. It does not verify a real Rust or Android build.


## 0.5.9 interactive command bridge addition

The Rust FFI now exposes CADCraft `Session::start` and `Session::input` through `arcad_command_start` and `arcad_command_input`. Responses include the current serialized prompt, display text and command-active state. Bounded JSON inputs support point, keyword, text, pick, enter and cancel. JNI and the serialized Kotlin `CadCraftDocument` wrapper expose matching methods. This is an important bridge seam for real prompt-driven CAD commands, but it is not yet connected to Compose gestures or the command bar. Deferred snap, cursor tracking, viewport, selection query and render DTOs remain future bridge work.


## 0.6.0 Android architecture and release hardening

- Added an ADR comparing the existing Compose/JNI shell with the ArtCraft Android GameActivity/egui-wgpu integration. The current architecture remains Compose + JNI until an isolated upstream CADCraft Android adapter prototype passes ARM64 build, lifecycle, surface recovery, SAF and input checks. No second render loop is introduced.
- Added `THREE-WAY-PARITY.md` with desktop, ArtCraft Android and AR CAD capability comparisons and source revision evidence.
- Release CMake fails closed when the real Rust FFI shared library is absent; only debug opts into the stub. Gradle release tasks require the FFI Cargo.lock, build Rust and verify both ABI libraries before packaging.
- The Android build script discovers NDK from SDK variables or `local.properties`, creates the FFI Cargo.lock on first build, then uses `--locked`. ELF ABI verification and JNI R8 keep rules were added.
- Host source tests and CMake fail-closed behavior pass. Actual Cargo/NDK/Gradle builds, APK/AAB generation and device testing remain blocked by missing toolchains.


## 0.6.1 native pointer and viewport bridge

Added FFI/JNI/Kotlin methods for setting viewport pixel dimensions, moving a world-coordinate cursor through CADCraft `Session::snap_cursor`, and dispatching clicks to `Session::idle_click`, native selection picks or `Session::input_snapped`. The Rust side applies the session's pick aperture, OSNAP/grid/tracking/ORTHO behavior and serializes the resulting cursor/command/selection DTO. Kotlin serializes calls per document and validates finite f64 coordinates and viewport bounds.

The Compose canvas has not yet been migrated to this API. It remains a separate Kotlin drawing model. The API tests in this environment use a fake native API; Rust compilation and Android runtime integration remain unverified.
