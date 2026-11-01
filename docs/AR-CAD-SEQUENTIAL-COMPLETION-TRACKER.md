# AR CAD sequential completion tracker

Review date: 2026-11-01
Working tree: `arcad-next/AR-CAD`

## Completion rule

A feature is marked **COMPLETE** only after (1) implementation exists in the active ArtCraft/CADCraft application path, (2) the relevant Rust/Android code compiles, (3) executable regression/integration tests pass, and (4) device-dependent behaviour is tested on a physical ARM64 Android device where applicable. Source-presence checks alone are not completion evidence.

## Step 1 — ArtCraft Android runtime integration

**Source wiring: VERIFIED. Runtime acceptance: BLOCKED. Overall: NOT COMPLETE.**

Evidence:
- `build-ar-cad-debug.sh` routes to the ArtCraft Android builder with display name `AR CAD`, package `com.arcad.app`, and the AR CAD icon.
- `lib/build-common.sh` contains the CADCraft Android launcher generation, Android SDK/NDK setup, native packaging, and APK bridge verification paths.
- The current unified source QA run passed 155/155 source-level assertions, including app identity, Android configuration, touch/input wiring, file actions, CI/release-script contracts, and geometry source checks.
- Shell syntax and Python compile checks passed.

Blocking evidence:
- `./build-ar-cad-debug.sh` exits before generation/compilation with `ERROR: Rust/Cargo is required.`
- `cargo`, `rustc`, `rustfmt`, `gradle`, `adb`, and `sdkmanager` are not installed in this environment.
- Network DNS resolution is unavailable, so the builder cannot bootstrap the missing Rust/Android toolchain here.
- No generated CADCraft Android launcher, APK, installation, or device runtime test is evidenced in this working tree.

Required to close Step 1:
1. Provide a Rust toolchain compatible with the CADCraft workspace (workspace currently declares Rust 1.95).
2. Provide Android SDK platform 35, build tools 37.0.0, NDK 27.2.12479018, JDK 17–23, Gradle wrapper support, and `cargo-ndk`.
3. Run `./build-ar-cad-debug.sh` and preserve the full build log.
4. Install the generated APK on an ARM64 Android device and verify launch, rendering, touch input, keyboard/IME, open/save through SAF, app lifecycle/surface recovery, and drawing persistence.

## Step 2 — 2D drawing and editing

**Source inventory: PRESENT. Runtime acceptance: NOT STARTED. Overall: NOT COMPLETE.**

The active CADCraft engine has draw command modules (`circle`, `ellipse`, `line_pline`, `point`, `polygon`, `solid`, `spline`, `text`) and modify modules (`break_join`, `explode`, `fillet`, `offset`, `select_then`, `stretch`, `transform`, `trim`). Engine and command test source is present. None of those Rust tests were executable in this environment because Cargo is missing.

Required acceptance: compile the active engine; execute drawing/editing command tests; test create/select/modify/undo/redo/save/reopen workflows; cover malformed input, degenerate geometry, mixed curve types, and large drawings.

## Remaining sequential steps (not yet accepted)

3. Layers, properties, snaps, dimensions, annotation, hatch and blocks: audit active command/UI coverage, then compile and run workflow tests.
4. DXF/DWG import/export: validate against a real-file corpus and independent CAD readers/writers; record unsupported/lossy entities.
5. OFFSET: complete mixed line/arc arrangements, open paths, multiple loops, nesting, overlaps, tangencies, arc reconstruction, and executable property tests.
6. FILLET/CHAMFER: complete line-line, line-arc, arc-arc and polyline topology cases; execute regression/fuzz tests.
7. Numerical geometry: independently verify exact predicates; implement and test certified spline/ellipse bounds or explicitly limit guarantees.
8. Tolerance remediation: classify and resolve every production geometric threshold; keep algorithmic/parameter cutoffs with documented rationale.
9. Production 3D: B-rep kernel, solid creation/editing, booleans, fillet/chamfer, shell, volume/area/mass/centroid, and 3D navigation.
10. Android integration: complete generated launcher build and physical-device acceptance.
11. Release: performance, stability, security, compatibility, signing, checksum and reproducible release artifact.

## Latest verification record

Command: `./tools/test-ar-cad.sh`
- Step 2 source assertions: 18/18
- Step 3 source assertions: 25/25
- Steps 4–6 source assertions: 14/14
- Steps 7–10 source assertions: 17/17
- Geometry priority source assertions: 14/14
- Tolerance migration source assertions: 20/20
- Precision safety source assertions: 11/11
- P1 source assertions: 36/36
- Total: 155/155 source-level assertions passed
- Shell syntax checks: passed
- Python compile checks: passed
- Rust tests: skipped (Cargo unavailable)

Command: `./build-ar-cad-debug.sh`
- Result: failed at preflight: `Rust/Cargo is required.`

No APK was built. No physical-device test was performed. AR CAD is not production-ready.
