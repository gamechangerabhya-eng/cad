# AR CAD — Android CAD workspace

AR CAD is an Android-first CAD application built around the imported CADCraft 0.15.1 engine and the ArtCraft Android runtime integration.

## Current implementation stages

- Step 1: AR CAD identity and Android build integration.
- Step 2: Responsive mobile start page and Android-first defaults.
- Step 3: Touch-first File, Draw and Edit toolbar.
- Step 4: Mobile drafting status controls, Zoom Fit and existing pinch/pan navigation.
- Step 5: Narrower touch-friendly Layers and Properties palette.
- Step 6: Mobile New/Open/Save/Save As controls and larger command input surface.

## Build status

Source-level checks have passed through Step 6. A native Android APK has **not** yet been built or tested in this environment because Rust/Cargo, Gradle and Android SDK/NDK are unavailable. Do not treat this source archive as an installable APK or production release.

See `TEST-REPORT-STEP1.md`, `TEST-REPORT-STEP2.md`, `TEST-REPORT-STEP3.md` and `TEST-REPORT-STEP4-6.md` for the validation boundaries and results.


## Steps 7–10 status — reliability and release hardening

- Step 7: Added a repeatable test gate for DXF round-trips, save backups, autosave and drawing recovery.
- Step 8: Added `tools/test-ar-cad.sh` and wired it into the Android Actions workflow before APK compilation.
- Step 9: Recorded and checked the current ARM64-only Android build target and release LTO configuration. No performance benchmark has yet been measured.
- Step 10: Cloud builds default to debug. Release mode requires a persistent signing keystore supplied through GitHub repository secrets; APK signature and SHA-256 checks are included.

**Release status:** not production-ready until the Android APK is built, installed and exercised on a physical device, including file persistence and crash recovery. See `docs/AR-CAD-STEPS-7-10.md` and `docs/AR-CAD-RELEASE-CHECKLIST.md`.
