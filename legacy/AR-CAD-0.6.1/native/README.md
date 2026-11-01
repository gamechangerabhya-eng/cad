# AR CAD native engine bridge — 0.5.9

This milestone adds the Android JNI adapter and a serialized Kotlin document-lifecycle wrapper on top of the 0.5.2 CADCraft C ABI. It is still an integration foundation: the Compose editor's active drawing model has **not** been migrated to CADCraft, and the native Rust library has not been built for Android in this environment.

## Layers

1. `native/arcad-ffi` — Rust C ABI around CADCraft 0.15.1 `Session` and `cadcraft-io`.
2. `app/src/main/cpp/arcad_jni.cpp` — JNI adapter; converts Java strings to UTF-8, copies byte arrays at the boundary, maps native errors to Java exceptions, and releases Rust-owned allocations.
3. `app/src/main/java/com/arcad/app/NativeCadBridge.kt` — JNI declarations, lazy library loading, injectable API contract for tests, and `CadCraftDocument` handle owner. Calls on one document are serialized with a lock; close is idempotent.
4. `app/src/main/cpp/CMakeLists.txt` — links the JNI adapter against the per-ABI Rust `libarcad_ffi.so`.

## APIs currently exposed

- Runtime version.
- Create/open/close native document.
- Entity count and CADCraft read diagnostics (`read_notes`).
- Programmatic command execution (`Session::execute`).
- Prompt-driven command start and input (`Session::start` / `Session::input`), returning serialized prompt state after each interaction. Supported input DTO kinds are `point`, `keyword`, `text`, `pick`, `enter` and `cancel`; deferred snap DTOs are not exposed yet.
- Native undo/redo.
- Save to DXF/DWG bytes (format selected by filename extension and CADCraft support).
- Render a bounded model-space PNG preview with `CadCraftDocument.renderPreviewPng(width, height)`.
- Render model space to PNG bytes through CADCraft's raster renderer (16..4096 px per side; maximum 16 MP).

Read diagnostics are exposed so the app can warn about unsupported or converted objects before a user overwrites an original file. The UI must display these notes before enabling destructive overwrite flows.

Interactive input JSON examples:

```json
{"kind":"point","value":{"x":12.5,"y":-3.0}}
{"kind":"keyword","value":"Close"}
{"kind":"text","value":"25.4"}
{"kind":"pick","value":[256,257]}
{"kind":"enter"}
{"kind":"cancel"}
```

`arcad_command_start` returns the current prompt (`prompt`, `promptText`, `commandActive`). Each `arcad_command_input` call returns the next prompt state. Deferred tangent/perpendicular snap inputs are deliberately not accepted until a versioned snap DTO is implemented.

## Build native Android libraries

Requirements:

- Rust/Cargo 1.95+ (the supplied upstream workspace declares Rust 1.95).
- `cargo-ndk` (`cargo install cargo-ndk`).
- Android NDK with `ANDROID_NDK_HOME` set.

Run from the project root:

```bash
./native/build-android.sh
```

This creates:

- `app/src/main/jniLibs/arm64-v8a/libarcad_ffi.so`
- `app/src/main/jniLibs/x86_64/libarcad_ffi.so`

Then open the project in Android Studio and build the `app` configuration. CMake uses a development-only JNI stub when the matching Rust shared library has not been generated, allowing the Android UI to compile while native setup is pending. The Kotlin bridge checks `nativeIsAvailable()` and refuses native operations when the stub is packaged. After `native/build-android.sh`, CMake links the real JNI adapter and Rust engine. The APK packages `libarcad_jni.so` and, when enabled, `libarcad_ffi.so`.

## Verification

Host-side pure-Kotlin tests, including the native-document lifecycle tests:

```bash
./tools/run-dxf-tests.sh
```

Static ABI checks:

```bash
python3 native/check-ffi-contract.py
python3 native/check-jni-contract.py
```

The JNI C++ sources can be syntax-checked on a host with a JDK and Clang:

```bash
clang++ -std=c++17 -fsyntax-only \
  -I"$JAVA_HOME/include" -I"$JAVA_HOME/include/linux" \
  -Inative/include app/src/main/cpp/arcad_jni.cpp
clang++ -std=c++17 -fsyntax-only \
  -I"$JAVA_HOME/include" -I"$JAVA_HOME/include/linux" \
  app/src/main/cpp/arcad_jni_stub.cpp
```

## Ownership and safety

- Each native document handle is owned by one `CadCraftDocument`; the wrapper serializes calls and frees it exactly once.
- Rust-owned strings and buffers are copied into JVM-owned values and released before JNI returns.
- File bytes are capped at 64 MiB at the native bridge boundary; command JSON is capped at 1 MiB.
- Filenames are UTF-8, NUL-free and bounded to 4096 bytes.
- Never call native methods with a handle outside the wrapper or use a handle after `close()`.
- The raw FFI is not internally synchronized. The Kotlin lock is mandatory until the native layer gains an explicit session mutex.

## Not yet completed

- Rust host compilation and unit tests: not run here (Rust/Cargo unavailable).
- Android Rust ABI builds: not run here (Rust/Cargo/NDK unavailable).
- Android CMake/Gradle build and emulator/physical-device tests: not run here (Android SDK/Gradle unavailable).
- Compose screen migration to `CadCraftDocument`: pending. The current editor still uses its interim Kotlin entity model and ASCII DXF codec.
- Native PNG raster output is exposed in the document wrapper, but the Compose canvas does not yet display it; native selection and snap queries remain pending.
- Interactive command APIs are available through the wrapper, but the Compose command bar and drawing gestures have not yet been migrated to feed CADCraft prompts/inputs.
- Full CADCraft read-note UX and safe overwrite confirmation: pending.
- DWG compatibility and data-preservation verification: pending; never silently overwrite the source drawing.
- 3D modelling, Android performance profiling, security review and release signing: pending.

This is not a production release. Do not describe AR CAD as CADCraft-powered at runtime until the native library is built, loaded, and the Compose document flow has been migrated and device-tested.
