# AR CAD — Steps 4, 5 and 6

Date: 2026-10-31

These steps continue from the Step 3 Android-first CAD workspace. The changes are conditional on `target_os = android`; desktop layout and controls retain their existing path.

## Step 4 — Mobile drafting controls and navigation

- Added a 44-point Android status strip with horizontally scrollable, labelled controls for Grid, Snap, Ortho, Polar, Object Snap and Dynamic Input.
- The controls invoke CADCraft's existing transparent commands; they do not maintain a second set of settings.
- Added a Zoom Fit action backed by the existing `Session::zoom_extents` implementation.
- Preserved the existing canvas pinch-zoom path, which zooms around the gesture focal point, and the existing pan navigation implementation.

## Step 5 — Mobile layers and properties

- Reduced the optional Android right palette width from 300 to 260 logical points to leave more drawing area available.
- Raised the scoped minimum interactive size in the palette to 40 × 40 logical points and increased button padding.
- Kept the existing Layers and Properties sections and their command-backed editing paths. No duplicate property model was introduced.

## Step 6 — File workflow and command input

- Added a horizontally scrollable FILE row to the Android drafting toolbar: New, Open, Save and Save As.
- These buttons dispatch the existing CADCraft commands through `CadApp::start`.
- Expanded the Android command bar to nearly the full canvas width and 38 logical points high.
- Increased the hit region for prompt keywords on Android to 30 logical points so point/keyword workflows are easier to use by touch.

## Validation performed

- `python docs/test-step2.py`: 18/18 source regression checks passed.
- `python docs/test-step3.py`: 25/25 source checks passed (updated for the File row and 134-point toolbar).
- `python docs/test-steps4-6.py`: 14/14 source acceptance checks passed.
- `bash -n` on every shell script: passed.
- Manual source review confirmed Android-specific changes are guarded and the desktop status-bar branch remains in place.

## Not verified

This environment still has no Rust compiler/Cargo, Gradle, Android SDK/NDK, cargo-ndk, ADB, emulator or physical-device bridge. Therefore these steps have **not** been compiled, APK-built, installed or tested on a phone. The checks above validate source structure and wiring only. Runtime correctness, touch behavior, file picker integration and drawing persistence remain pending a real Android build and device test.
