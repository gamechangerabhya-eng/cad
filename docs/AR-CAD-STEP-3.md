# AR CAD — Step 3: Android touch-first drafting controls

## Scope

Step 3 adds a dedicated mobile drafting toolbar to the Android CAD workspace. It uses CADCraft's existing command dispatcher and interactive command machines; it does not create a parallel drawing engine or a visual-only mock toolbar.

## Implemented

- Replaced the compact desktop-style Android toolbar with two labelled, horizontally scrollable rows while a drawing is open.
- **Draw row:** Line, Polyline, Circle, Rectangle, Arc, Ellipse and Polygon.
- **Edit row:** Move, Copy, Rotate, Scale, Offset, Trim, Extend, Erase, Undo and Redo.
- Touch targets are 72 × 36 logical points, with a 96-point toolbar region.
- The start page retains its smaller 52-point brand strip; drafting controls appear only after leaving the start page.
- Every button dispatches the corresponding existing CADCraft command through `CadApp::start` and records its accessible control label.
- Desktop toolbar behavior is unchanged.

## Validation

- `python docs/test-step3.py`: 21/21 source-level acceptance checks passed.
- `python docs/test-step2.py`: 18/18 source-level regression checks passed.
- `bash -n` across all shell scripts: passed.
- Engine source inspection confirms every listed draw/edit command exists in the imported CADCraft command registry.

## Not verified

This environment does not contain `rustc`, `cargo`, `cargo-ndk`, Gradle, Android SDK/NDK or ADB. Therefore Rust compilation, Cargo unit tests, APK generation, APK inspection, emulator/device installation and touch interaction tests have **not** been run. The Kotlin compiler alone is not sufficient to build this Rust/C++/Android application.

## Acceptance status

- Step 3 source implementation: complete.
- Step 3 source-level checks: passed.
- Compile/build/device acceptance: pending a compatible Android build environment.

A successful source-level check is not evidence that the Android toolbar has been rendered or tested on a physical phone.
