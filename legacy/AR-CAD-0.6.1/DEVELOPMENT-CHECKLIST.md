# AR CAD development checklist

Statuses are evidence-based. `VERIFIED` requires successful verification in the target environment, not source inspection alone.

| Feature / workstream | Existing implementation | Changes required | Status | Test evidence / remaining issues |
|---|---|---|---|---|
| Preserve CADCraft Rust core | Vendored CADCraft 0.15.1 workspace; Rust FFI uses `Session` | Keep Rust document/geometry/engine authoritative | IMPLEMENTED | Source present; Rust compilation unavailable |
| Native Android build bootstrap | Rust FFI crate and cargo-ndk script | Discover NDK, generate lockfile once, build locked ARM64/x86_64 libraries | TESTED (script only) | Mocked cargo-ndk test and ELF verifier pass; real Rust/NDK build BLOCKED |
| JNI contract | Kotlin external methods mapped to C++ exports | Runtime load/link, lifecycle and exception tests on Android | TESTED (source) | Static contract and host C++ syntax pass; runtime not run |
| Native viewport and pointer bridge | CADCraft `Session` supports viewport size, cursor snap, idle click and selection picks | Wire Compose gesture mapping, native selection highlighting and native render loop | IMPLEMENTED | Fake API wrapper tests, 21-function C ABI / 17-method JNI contracts, C++ syntax pass; Rust runtime unverified |
| Interactive native command bridge | Rust FFI exposes `Session::start` and `Session::input`; Kotlin wrapper serializes calls and returns prompt/state JSON | Wire Compose command bar and gestures to the persistent native session | IMPLEMENTED | Host wrapper tests; Rust runtime unverified |
| Active document ownership | Kotlin editor state; temporary CADCraft document for preview | Migrate create/open/save/edit to one native `Session` and remove duplicate Kotlin state after parity tests | IN PROGRESS | Native pointer/viewport APIs now exist, but Compose still owns active drawing and preview still uses temporary DXF document |
| 2D primitives | LINE/CIRCLE/RECT/POLYLINE/ARC and polygon tool | Add ellipse, spline, point, construction line, text, MText, dimensions, leaders, blocks, hatch | IN PROGRESS | Existing limited features covered by host tests; missing types not implemented |
| 2D editing | Single selection, drag move, copy, rotate, mirror, scale, delete; restricted trim/extend/offset/fillet/chamfer | Multi-select, robust native commands, join/explode/group and exact fillet topology | IN PROGRESS | Kotlin operation tests exist; native Session integration absent |
| Precision and snapping | Coordinates, dynamic input, orthographic, polar/object snaps and spatial index | Keep double precision end-to-end; complete polar/grid/units/settings and native snaps | IN PROGRESS | Host tests for input/snaps/index; UI model currently Float |
| Layers | Names, active layer, visibility | Persist colors, linetype, lineweight, lock/freeze and native layer sync | IN PROGRESS | Basic UI exists; full CAD layer properties absent |
| Native project format | Versioned JSON drawing schema | Versioned migration, metadata, robust save-as and file validation | IN PROGRESS | Current schema limited to five entity kinds |
| Recovery / autosave | Single recovery snapshot with backup fallback | Per-project autosave, crash recovery UX, version history | IMPLEMENTED | Recovery-store tests pass; project catalog absent |
| DXF | Bounded ASCII subset in Kotlin; CADCraft upstream broader IO | Route imports/exports through native IO and report fidelity loss | IN PROGRESS | Kotlin round-trip tests; binary and broad entity fidelity absent |
| DWG | Upstream converter source | Android build, licensing/dependency review and independent round-trip QA | BLOCKED | No Android build or compatibility verification |
| SVG / PDF / PNG / JPEG | Native PNG preview source path; upstream SVG/PDF APIs | Expose native export workflows; assess JPEG and document limitations | IN PROGRESS | PNG source path exists; no Android runtime verification |
| 3D geometry kernel | Some 3D vectors and 3D DXF entities upstream | Select/implement B-rep or mesh kernel, topology, booleans and feature ops | NOT STARTED | No 3D solid modeling in app |
| 3D viewport and tools | None in Android app | GPU viewport, camera, picking, face/edge/vertex selection, materials | NOT STARTED | No 3D app integration |
| Android Compose UX | Compose drawing canvas, toolbars, SAF file pickers | Native CAD viewport, touch UX, lifecycle and accessibility polish | IN PROGRESS | No APK build/device run |
| Project browser | Open/Save via SAF only | Catalog, recent files, thumbnails, rename, duplicate, delete, search, backup/restore | NOT STARTED | Not present |
| Performance / reliability | Bounded parsers, spatial index, preview gate and recovery limits | Large drawing profiling, memory/ANR and GPU recovery tests | IN PROGRESS | Host tests only; device profiling absent |
| Release packaging gate | Gradle app module and CMake config | Build Rust before release, forbid stub, preserve JNI with R8, verify ELF ABIs | TESTED (source) | CMake release fails closed; Gradle task graph only statically asserted; no Gradle execution |
| Release artifacts | No wrapper or built APK/AAB | Add Gradle wrapper, signing, APK/AAB and device QA | BLOCKED | Gradle/SDK/NDK unavailable; no APK/AAB generated |
| Upstream comparison | CADCraft 0.15.1 vendored; ArtCraft Android source not cloned | Re-fetch pinned tags and validate Android adapter prototype | IN PROGRESS | Public docs reviewed; GitHub DNS blocked in container; see `THREE-WAY-PARITY.md` |


## 0.6.0 work completed

- Release CMake gate: TESTED by host CMake; debug stub configuration succeeds only when explicitly enabled, release configuration without Rust library fails as expected.
- Release Gradle wiring/R8 JNI keep rules: IMPLEMENTED; only source-level regression assertions are available until Gradle is installed.
- Android NDK discovery, one-time Cargo.lock generation, locked builds and ELF ABI verifier: TESTED with mocked cargo-ndk and synthetic ELF fixtures. Real NDK outputs remain BLOCKED.
- CADCraft desktop / ArtCraft Android / AR CAD comparison and architecture decision: IMPLEMENTED in `THREE-WAY-PARITY.md` and `ANDROID-ARCHITECTURE-DECISION.md`.
- Active native Session ownership: NOT STARTED; this remains the highest-priority product architecture gap.
