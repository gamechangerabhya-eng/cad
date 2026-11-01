# AR CAD production release gaps

## Blocking engineering work

1. Build CADCraft FFI for Android `arm64-v8a` and `x86_64` with Rust 1.95+, cargo-ndk and Android NDK.
2. Build/load JNI shim in Android Gradle and test native document create/open/execute/undo/redo/save/close on emulator and physical devices.
3. Migrate Compose document state from the interim Kotlin `CadEntity` list to the authoritative CADCraft `Session` document.
4. Connect the new native PNG preview endpoint to the Compose canvas or replace it with a retained display-list/texture path; verify touch, selection, OSNAP and viewport updates against native geometry.
5. Surface CADCraft `read_notes` in the UI. Require explicit user confirmation before saving over a source file when notes indicate unsupported or converted objects.
6. Add native entity/selection/snap query APIs and synchronize command prompts, selection and cancellation with Compose.
7. Run CADCraft Rust tests and a representative DXF/DWG corpus. Independently open exported files in licensed AutoCAD versions and compare geometry/properties.
8. Add binary DXF, dimensions, text/MTEXT, leaders, blocks, hatches, layouts, plotting and xrefs with round-trip coverage.
9. Implement a separate 3D solid-modelling engine/workflow; upstream CADCraft is not a complete 3D solid modeller.
10. Complete hostile-file, memory, crash-recovery, performance, accessibility, security, privacy, signing and Play Store release checks.

## Current state at 0.5.3

- JNI adapter and Kotlin native document lifecycle wrapper exist in source.
- Static C/Rust and Kotlin/JNI symbol checks pass.
- Host C++ syntax check passes.
- Host pure-Kotlin tests pass.
- Rust, Android ABI, Gradle, emulator/device and runtime JNI tests are not run.
- Active Compose app still uses interim Kotlin model; CADCraft is not yet the runtime engine.

No production-ready claim is permitted until all blocking gates above are closed and verified.


## 0.5.4 additional safeguards

- Native save now requires explicit acknowledgement if CADCraft import diagnostics contain notes; malformed diagnostics fail closed.
- CADCraft SVG preview export is exposed through the existing format-aware native writer.
- Remaining Rust C ABI accessors and destructors contain unwinding; JNI rejects NUL-containing command payloads.
- These safeguards do not complete the engine migration: the Compose editor still uses the interim Kotlin entity model, and native rendering/selection/snap DTOs are still pending.


## 0.5.5 update

A bounded CADCraft model-space PNG renderer is now exposed through Rust FFI, JNI and `CadCraftDocument.renderPreviewPng`. It is not yet consumed by Compose and has not been runtime-tested because Rust/Android toolchains are unavailable.


## 0.5.6 update

- Added a separate Native PNG file-preview action. It reads a selected source CAD file (64 MB limit), opens a temporary CADCraft native document, requests a bounded PNG render on a background dispatcher, closes the document, and displays the decoded preview. It does not alter the active editor drawing.
- This Compose flow has not been compiled with Gradle or exercised on Android. Native library availability, supported input formats, rendering output and memory behavior still require device validation.
- The primary editor remains on the interim Kotlin entity model; native CADCraft session migration and live viewport synchronization remain release blockers.


## 0.5.7 update

- Native PNG now previews a snapshot of the active Kotlin editor drawing through an in-memory DXF conversion and temporary CADCraft document. It no longer asks the user to select a second file.
- This is one-way preview only. Native model ownership, live canvas synchronization, native selection/OSNAP/commands and shared undo/redo remain unimplemented. DXF conversion fidelity and native runtime behavior need Android-device tests.


## 0.5.8 preview hardening

Asynchronous native preview requests are now guarded against stale results and oversized PNG payloads. This improves preview correctness but does not change the primary release blockers: the active editor remains Kotlin-owned, native Rust/Android builds and device runtime are unverified, and native editing/selection/snapping/viewport synchronization are not integrated.


## 0.5.9 audit additions

- The actual 0.5.8 tree and vendored CADCraft source were re-audited. See `SOURCE-AUDIT.md` and `DEVELOPMENT-CHECKLIST.md`.
- The Kotlin editor supports only a limited 2D entity set; text, dimensions, leaders, ellipses, splines, blocks and hatch are not in the active editor model.
- Native Android build bootstrap now tolerates a missing FFI lockfile for the first build, but that first dependency resolution is not reproducible until `native/arcad-ffi/Cargo.lock` is generated and committed.
- The Rust `Session` is not yet the active editor document. Android Rust ABI build, Gradle APK/AAB, native JNI runtime, 3D solid kernel, Android 3D viewport and project catalog remain release blockers.
- 0.5.9 exposes prompt-driven native command start/input, but does not yet expose CADCraft cursor snapping, viewport control, selection query, render display-list DTOs or a Compose command/gesture adapter. The interactive bridge must be Rust-tested and device-tested before production use.


## 0.6.0 release-gate hardening

- Release CMake no longer falls back to `arcad_jni_stub.cpp`. It emits a fatal configuration error if `libarcad_ffi.so` for the selected ABI is absent. Debug opts into the stub explicitly.
- Gradle `preReleaseBuild` depends on the Rust `cargo-ndk` build and a check for the configured ARM64/x86_64 libraries. It requires `native/arcad-ffi/Cargo.lock` to exist before resolving dependencies. R8 keep rules preserve JNI symbol names.
- The build script discovers NDK from SDK environment variables or `local.properties`, generates a missing FFI lockfile and then invokes Cargo with `--locked`.
- Added a verifier for ELF64 little-endian shared libraries and ABI machine IDs.
- Static/CMake source tests pass. Actual Gradle task graph, NDK output and APK contents remain unverified because the required toolchains are absent.

## Architecture decision and upstream comparison

See `ANDROID-ARCHITECTURE-DECISION.md` and `THREE-WAY-PARITY.md`. The supplied project had no `.git` directory; a local `work/ar-cad-1.0` branch and baseline commit were created without changing the source history (there was no original history to preserve). The upstream ArtCraft Android repository could not be cloned because network DNS is unavailable to the build container; its public architecture and release docs were reviewed instead.


## 0.6.1 native interaction bridge update

The Rust FFI/JNI/Kotlin API now includes native viewport sizing, cursor movement through CADCraft `Session::snap_cursor`, and click dispatch to CADCraft selection or interactive point prompts. This is an implementation foundation, not end-to-end editor integration: Compose still owns the active drawing, screen-to-world gesture mapping is not wired to the native session, selection feedback is not native, and the native preview still uses a temporary DXF-loaded session.

Release remains blocked by missing `native/arcad-ffi/Cargo.lock`, unavailable Rust/Android toolchains in this environment, no verified native Android libraries, no APK/AAB and no device test evidence.
