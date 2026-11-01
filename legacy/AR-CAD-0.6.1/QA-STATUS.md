# QA status — AR CAD 0.6.1

- `bash tools/run-dxf-tests.sh`: 17 Kotlin/JVM test programs passed.
- FFI static contract: 21 C declarations matched Rust exports.
- JNI static contract: 17 Kotlin external methods matched C++ exports.
- Host Clang C++17 syntax checks passed for real JNI adapter and development stub.
- Native Android build-script and release-gate checks passed.
- Rust/Cargo, Gradle, Android SDK/NDK, APK/AAB and device tests: NOT RUN (toolchains unavailable).

The new pointer/viewport wrapper tests use a fake API. They verify Kotlin validation and forwarding, not native CADCraft runtime. Compose has not yet been connected to these APIs; the active drawing remains Kotlin-owned.

# AR CAD 0.6.0 QA status

## Passed in this environment

- Pure-Kotlin host regression script: **17 test programs passed**, including native document lifecycle, native save-safety and native render-wrapper tests.
- New render-wrapper tests verified PNG signature handling, default 1600x1000 dimensions, custom dimensions, out-of-range rejection and calls after document close.
- C header ↔ Rust exported symbol static contract: **16 functions matched**.
- Kotlin external JNI declaration ↔ C++ JNI symbol static contract: **12 methods matched**.
- Clang C++17 syntax-only checks of `arcad_jni.cpp` and `arcad_jni_stub.cpp` using JDK 21 JNI headers: **passed**.
- Source archive integrity: verified after packaging.
- Both JNI C++ translation units passed Clang C++17 syntax-only checks in the 0.5.8 source pass.

## Implemented in 0.5.8

- Preview request gate tests passed: newest request wins, mismatched editor snapshot is rejected, and explicit invalidation cancels display eligibility.
- Preview result handling checks request freshness after native work and before/after PNG decode; stale decoded bitmaps are recycled.
- PNG input size and decoded dimensions are bounded before display (maximum 4096 per side and 16 megapixels).

## Implemented in 0.5.7 / 0.5.6 / 0.5.5

- 0.5.7 preview source path now renders the current editor snapshot via an in-memory DXF bridge; it does not replace the editor model or enable native edits.

- Added `arcad_document_render_png`, rendering CADCraft model space through `cadcraft_io::png`.
- Enforced dimensions from 16 to 4096 pixels per side and a 16-megapixel ceiling before invoking the renderer.
- Added JNI `nativeRenderPng`; Rust-owned output buffers are copied into JVM-owned arrays and freed.
- Added `CadNativeApi.renderPng` and serialized `CadCraftDocument.renderPreviewPng`.
- Added a dedicated host test for PNG signature, default/custom dimensions, invalid dimensions and closed document rejection.
- Added a Compose Native PNG action that snapshots the active Kotlin entities/layers, converts them to bounded in-memory ASCII DXF, opens a temporary CADCraft document, renders on `Dispatchers.IO`, closes the native document, validates PNG decoding and displays a preview dialog. The active drawing and undo/redo history are not mutated.

## Not run / not passed

- Rust/Cargo compilation and Rust unit tests: **NOT RUN**; `rustc` and `cargo` are unavailable.
- Android Rust ABI cross-compilation: **NOT RUN**; Android NDK and `cargo-ndk` are unavailable.
- Android CMake/Gradle compilation: **NOT RUN**; Android SDK and Gradle are unavailable.
- Emulator or physical-device runtime tests: **NOT RUN**.
- JNI load/link test on Android: **NOT RUN**.
- Actual native PNG bytes from CADCraft: **NOT runtime-tested**; Kotlin host tests use a fake native API and verify the PNG signature contract only.
- Live Compose canvas integration with native PNG/display list and active document synchronization: **NOT IMPLEMENTED**.
- Active-drawing Native PNG Compose workflow Android compile/device runtime: **NOT RUN**.
- Compose document state migration to CADCraft `Session`: **NOT IMPLEMENTED**; the active editor still uses the interim Kotlin entity model.
- DWG import/export fidelity and independent AutoCAD round-trip: **NOT VERIFIED**.
- 3D solid modelling: **NOT IMPLEMENTED**.

## Release decision

**NOT A PRODUCTION RELEASE.** 0.5.8 adds a source-integrated active-drawing-to-native-preview bridge, but it has not been Android-built or device-tested. Do not claim the Android app is running on CADCraft until Rust/Android ABI builds, JNI load tests, Compose migration, file round-trip tests and physical-device QA pass.


## 0.5.9 source-audit, interactive bridge and build-script checks

- Audited current app/native/upstream source and documented findings in `SOURCE-AUDIT.md`.
- Added evidence-based feature tracking in `DEVELOPMENT-CHECKLIST.md`.
- Added prompt-driven command start/input source across Rust FFI, C ABI, JNI and Kotlin wrapper. Host tests verify serialized wrapper calls; source contracts now cover 18 C functions and 14 JNI methods. The Rust command path itself is not compiled or runtime-tested here.
- `python3 tools/check-native-build-script.py`: **passed**. The test uses a fake cargo-ndk executable to confirm first-build behavior without `native/arcad-ffi/Cargo.lock` and locked mode when it exists. It does not compile Rust or Android code.
- `bash -n native/build-android.sh`: **passed**.
- The actual Rust Android build remains **NOT RUN**: no Rust/Cargo, cargo-ndk or Android NDK is installed. No Gradle APK/AAB or device test is claimed.
- Added Rust unit tests for the interactive input DTO parser (supported variants and malformed/bounds cases); these tests are present but **NOT RUN** because `cargo`/`rustc` are unavailable.


## 0.6.0 release-gate and repository comparison checks

- `bash tools/run-dxf-tests.sh`: **passed** all 17 Kotlin/JVM test programs, FFI/JNI static contracts, native build-script tests and release-gate tests.
- FFI static contract: **18 C functions matched** Rust exports.
- JNI static contract: **14 Kotlin native declarations matched** C++ exports.
- `python3 tools/check-native-build-script.py`: **passed** NDK discovery from SDK root, first Cargo.lock generation, locked subsequent build command, mocked ARM64/x86_64 outputs and refusal to resolve an absent lockfile in release mode. This is a fake cargo-ndk harness, not an actual Rust build.
- `python3 tools/check-release-gate.py`: **passed** source-level assertions that Release disables the stub, depends on the Rust build/verification tasks, and preserves JNI symbols through R8.
- `python3 tools/verify-android-elf.py /bin/ls x86_64`: **passed** the ELF parser's x86_64 positive case; a deliberate x86_64-as-ARM64 test was **rejected** as expected.
- C++17 syntax-only checks for both JNI implementations: **passed**.
- CMake debug configuration with explicit stub enabled: **passed**.
- CMake release configuration without Rust library: **failed closed as expected** with the release-native-library error.

## Still not run

- `rustc`, `cargo fmt`, `cargo clippy`, `cargo test`, Rust FFI compile: **NOT RUN**; Rust toolchain unavailable.
- Android NDK cross-compilation and real ELF validation: **NOT RUN**; NDK and cargo-ndk unavailable.
- Gradle sync/build, Debug APK, Release APK, AAB: **NOT RUN**; Gradle and Android SDK unavailable. No Gradle wrapper is currently present.
- JNI load/link on Android, emulator or physical device: **NOT RUN**.
- Active Compose editor to native `Session` migration: **NOT IMPLEMENTED**.
- Live native viewport, touch/OSNAP integration, complete CADCraft parity, DWG round-trip and 3D solids: **NOT VERIFIED / NOT IMPLEMENTED**.

**Release decision: NOT A PRODUCTION RELEASE.** The release gate is implemented and source-tested, but a real release build cannot run until the Rust/NDK/Gradle toolchains are available.
