# AR CAD — Step 1: Source integration and project setup

Status: **source integration complete; Android build not yet verified**.

## Integrated sources
- Android build/runtime foundation: supplied ArtCraft Android source archive.
- CAD engine/UI source: supplied CADCraft **0.15.1** source tree.
- Existing AR CAD: preserved unchanged under `legacy/AR-CAD-0.6.1/` for migration and feature retention.
- Android app identity configured by the build wrappers:
  - Display name: `AR CAD`
  - Application ID: `com.arcad.app`
  - Launcher icon: `branding/ar-cad-icon.png`
  - Initial Android ABI target inherited from the ArtCraft CADCraft builder: `arm64-v8a`.

## CADCraft source provenance
- Source version read from workspace Cargo manifest: `0.15.1`.
- Source archive SHA-256: `29d1f3d67257852f38bc64c998c71771fa16ace925f913106fd7b15b8af4321e`
- The archive did not include an upstream Git commit identifier. A local Git import commit and `v0.15.1` tag were created solely so the ArtCraft builder can validate and reuse the source checkout. This local commit is **not** represented as an upstream Storytold commit.
- The local checkout origin is set to the public upstream repository URL for compatibility with the builder's source-origin guard.

## Build commands
- Debug: `./build-ar-cad-debug.sh`
- Release: `./build-ar-cad-release.sh`

The wrappers use the builder's CADCraft adapter and include the CADCraft I/O crate, which is required for native drawing-format registration.

## What is deliberately not claimed
- No APK has been built.
- No Android device/emulator execution has been performed.
- No Rust Android ABI compilation has been performed.
- No claim of full AutoCAD parity or production readiness is made.

## Step 1 acceptance
- [x] ArtCraft Android source unpacked and integrated as project root.
- [x] CADCraft 0.15.1 source installed as the builder's CADCraft workspace.
- [x] Original AR CAD 0.6.1 source preserved for migration.
- [x] AR CAD display name, package ID and custom icon configured in build wrappers.
- [x] Build wrappers and source layout statically checked.
- [ ] Rust / Android SDK / NDK / Gradle build and APK verification — blocked by missing toolchain in this execution environment.
