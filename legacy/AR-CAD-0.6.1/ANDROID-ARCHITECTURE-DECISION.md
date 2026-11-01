# AR CAD Android architecture decision (ADR-0001)

**Status:** Accepted for the 0.6.0 build-hardening milestone; subject to a native Android prototype before final renderer selection.

## Context

AR CAD 0.5.9 contains a Kotlin/Jetpack Compose application, a Rust `arcad-ffi` crate over CADCraft `doc`, `engine`, and `io`, and a JNI/CMake layer. The active Compose editor still owns a separate Kotlin drawing model. CADCraft desktop is an `eframe`/`egui`/`wgpu` application. The ArtCraft Android builder generates a GameActivity-based Rust launcher and uses patched `winit`, `egui-winit`, `egui-wgpu`, `android-activity`, and Storage Access Framework (SAF) bridges.

## Options

| Criterion | A. Keep Compose + JNI and migrate editor to Rust | B. Adopt ArtCraft CADCraft GameActivity adapter | Assessment |
|---|---|---|---|
| Existing AR CAD work | Preserves current dashboard, Compose shell, JNI and tests | Replaces or splits current editor shell | A has lower immediate migration cost |
| Upstream CADCraft UI parity | Requires a new Android viewport and UI bindings | Reuses upstream eframe application and command/UI implementation | B is stronger for desktop parity |
| GPU rendering | Must implement and own Android surface/wgpu or a high-performance native render bridge | Builder already patches the wgpu/winit Android surface path | B has a demonstrated framework path |
| One authoritative document | Possible only after replacing Kotlin editing model with native `Session` | CADCraft `App` is constructed with `Session::empty()` and `Services::default()` | Both can provide one document; B has a ready upstream path |
| Touch, mouse, keyboard | Custom Compose gesture implementation | Framework includes Android input patches and two-finger scrolling | B has broader existing integration |
| Storage / SAF | Existing SAF document picker, native persistence work needed | Shared SAF bridge, file staging/write-back, intents and printing | B has broader integration |
| Lifecycle / surface recovery | Must be engineered and tested | Framework includes `egui-wgpu` surface recovery and nonblocking callbacks | B has a head start, still needs device verification |
| Compose-native panels | Natural | Not native to eframe; would require separate activity or overlay architecture | A is stronger |
| Build complexity | Existing Gradle/CMake + cargo-ndk | Builder-generated Gradle + cargo-ndk + vendored patches | A fewer new moving parts; B more complete port |
| Maintainability | Own JNI API and Compose UI | Track a separate Android builder and its vendor patches | A has simpler product ownership; B has more upstream parity |

## Decision

**Do not merge two independent render loops or keep two editable document models.** For this milestone, retain Compose + JNI as the current application architecture and make release builds require the actual Rust library. Do not import ArtCraft's full launcher into the same Activity or run its `eframe` loop beside Compose.

For the next renderer spike, evaluate ArtCraft's CADCraft-specific adapter as a separate, isolated branch/prototype. It is the strongest evidence-backed route to upstream `egui`/`wgpu` UI parity, but it must demonstrate:

1. the selected CADCraft revision builds for Android ARM64;
2. `Session` remains the single authoritative document;
3. GameActivity surface create/destroy/recreate works without losing the drawing;
4. SAF open/save and Android back/IME paths work;
5. touch, mouse, keyboard and trackpad events reach CADCraft correctly;
6. no duplicate Compose/egui render loop is active for the same drawing.

If the prototype passes, migrate the drawing workspace to that single native renderer and keep Kotlin/Compose for the project dashboard/settings only. If it does not, continue with a single native Rust render surface hosted by the Compose Activity. Do not decide by screenshots or build success alone.

## Release safety decision

A JNI stub is allowed only for debug builds. Release variants must build the Rust FFI library first and fail configuration if the library for the selected ABI is missing. R8 must preserve the JNI bridge class and methods. This is implemented in 0.6.0 source; Android execution remains unverified until a real SDK/NDK build.

## References

- CADCraft desktop: https://github.com/storytold/cadcraft
- ArtCraft Android: https://github.com/MostafaXS1/artcraft-android
- ArtCraft Android architecture: https://github.com/MostafaXS1/artcraft-android/blob/main/architecture.md
- ArtCraft Android CI: https://github.com/MostafaXS1/artcraft-android/blob/main/docs/ci-workflow.md
