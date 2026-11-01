# CADCraft desktop / ArtCraft Android / AR CAD comparison

**Review date:** 2026-10-31. This matrix distinguishes source evidence from Android runtime verification. The current GitHub release remains CADCraft `v0.15.1`; GitHub Actions also showed later same-day `main` activity (`b35f008`), but a full main-branch object ID and source checkout could not be retrieved from this DNS-restricted build environment.

## Reproducible source references

| Project | Selected reference | Revision evidence | Local availability |
|---|---|---|---|
| CADCraft | `storytold/cadcraft`, release `v0.15.1` | Release page identifies commit prefix `660e15c9f4`; vendored workspace manifest says `0.15.1` | Vendored source archive and extracted workspace present; no `.git` metadata |
| ArtCraft Android | `MostafaXS1/artcraft-android`, release `v0.3.0` | Release page identifies commit prefix `9c2af6f`; architecture page reviewed from `main` (61 commits listed) | Not cloned locally; network DNS unavailable in this environment |

The ArtCraft Android `main` branch commit SHA and a full 40-character CADCraft SHA could not be independently fetched in this environment. Do not represent the abbreviated release commit IDs as full hashes. Re-fetch with Git before any source update or cherry-pick.

## Capability matrix

| Capability | CADCraft desktop | ArtCraft Android | Current AR CAD 0.5.9 baseline | Gap / owner |
|---|---|---|---|---|
| CAD engine | Rust workspace, `doc`, `engine`, `geom`, `constraints`, `render`, `io`, `dxf`, `dwg` | Builds official CADCraft source | Vendored CADCraft plus `arcad-ffi` over `doc`/`engine`/`io` | Rust Android integration / AR CAD |
| Drawing engine | Native CADCraft `Session` and document model | Adapter constructs CAD app with `Session::empty()` | Native session plus FFI pointer/viewport APIs; Compose editing remains separate | **P0 architecture defect / AR CAD** |
| Rendering | eframe/egui + wgpu desktop | GameActivity, eframe/egui-wgpu and Android-patched winit surface | Compose canvas plus native PNG preview and source-level pointer/viewport FFI; no live native viewport integration | **P0 renderer choice / Android integration** |
| Android launcher | N/A | GameActivity-based generated launcher or app adapter | Kotlin Activity + Compose + JNI | Android integration |
| 2D drawing | Broad desktop command set, but upstream parity remains pre-alpha | Reuses upstream application functionality; mobile parity not individually verified here | Limited Kotlin editor and native command bridge, not wired to canvas | Upstream + AR CAD |
| 3D modelling | Upstream roadmap puts UCS/3D views then solids after beta; not a complete solid CAD kernel | Same upstream feature ceiling | No integrated 3D solid kernel/viewport | Upstream / geometry kernel |
| DXF | Native CADCraft DXF IO; known fidelity gaps remain | Reuses upstream IO | Kotlin ASCII subset plus native preview import path | Data conversion / AR CAD |
| DWG | `cadcraft-dwg` bridge exists; upstream warns save may drop unmodelled/custom data | Builds upstream CADCraft; Android compatibility must be proven | No verified Android DWG operation | Upstream IO + Android dependency compatibility |
| Undo/redo | CADCraft Session history | Upstream Session | Kotlin undo/redo and native Session API both exist, but separate | Dual-document defect |
| Layers | CADCraft document tables and UI | Upstream UI + SAF environment | Basic Kotlin layers, native sync incomplete | Dual-document / UI |
| Dimensions | Upstream dimension entities and command UI, gaps remain in styles/annotation | Reuses upstream UI | Limited Kotlin model; native Session not authoritative | Upstream / AR CAD |
| Touch | Desktop pointer model | Android input patches, two-finger scroll, long-press right-click | Basic Compose pointer input | Android input |
| Keyboard | Desktop shortcuts/command line | IME bridge, hardware keyboard and modifier support | Compose command UI; native command FFI not fully wired | Android integration |
| Storage | Desktop filesystem and autosave/recovery | SAF staging, save-back, intents, drag/drop | SAF picker and recovery store; no robust catalog/version history | Android storage / AR CAD |
| Lifecycle | Desktop window lifecycle | GameActivity, nonblocking SAF callbacks, GPU surface recovery patch | No device validation; native renderer not integrated | Android lifecycle |

## Findings

1. CADCraft 0.15.1 is not a production-complete AutoCAD substitute. Its own roadmap labels it **pre-alpha** and lists core drafting, modify, annotation, block, plot-style and lossless DWG gaps.
2. ArtCraft Android is a credible integration reference, not a guarantee. It has a GameActivity/eframe/wgpu launcher path, patches for winit/egui-winit/egui-wgpu/android-activity, SAF, IME, mouse/trackpad and surface recovery. Its own release history documents CADCraft crashes fixed during early Android releases.
3. AR CAD has useful Kotlin geometry/test work and a real source-level Rust FFI, but the separate Kotlin editor model is the largest correctness blocker. The current native preview is not proof of a native interactive viewport.
4. The 0.6.0 release gate now prevents a release build from silently shipping the JNI stub. Debug-only stub builds remain explicitly possible.
5. A true 3D solid kernel is not available in the current AR CAD integration. Upstream's own roadmap defers 3D foundations and solids; do not claim 3D parity.

## Source links

- https://github.com/storytold/cadcraft/releases/tag/v0.15.1
- https://github.com/storytold/cadcraft/ROADMAP.md
- https://github.com/storytold/cadcraft/docs/gaps.md
- https://github.com/MostafaXS1/artcraft-android/releases/tag/v0.3.0
- https://github.com/MostafaXS1/artcraft-android/blob/main/architecture.md
- https://github.com/MostafaXS1/artcraft-android/blob/main/docs/ci-workflow.md


## 0.6.1 update

AR CAD now exposes `setViewport`, `pointerMove` and `pointerClick` through Rust FFI, JNI and the serialized Kotlin document wrapper. The pointer move uses CADCraft `Session::snap_cursor`; clicks route into `Session::idle_click`, native selection picks or `Session::input_snapped`. This closes part of the native input bridge gap at the API layer, but the Compose UI has not yet adopted it. Wrapper tests use a fake API and do not count as Rust/Android runtime verification.
