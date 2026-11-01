# AR CAD — Steps 7–10: reliability, QA, compatibility and release hardening

Date: 2026-10-31

These steps harden the existing CADCraft 0.15.1 integration. They do not claim AutoCAD feature parity and do not replace real Android runtime testing.

## Step 7 — Drawing reliability and recovery

The project now has a repeatable reliability test gate covering the CADCraft tests already included in the imported source:

- DXF parser/writer and round-trip fixtures (`cadcraft-dxf`).
- Autosave scheduling, interrupted-session recovery, lock ownership and failed-write retry (`cadcraft-engine` tests).
- Save backup, Save As export and unsaved-state reporting (`cadcraft-engine` tests).
- UI recovery, recent-file persistence, open-bytes and save-loss behavior (`cadcraft-ui-egui` tests).

The tests are run from the unified `tools/test-ar-cad.sh` gate. This step does not change the underlying DXF format implementation; known upstream DXF/DWG limitations remain.

## Step 8 — Automated quality assurance

- Added one source/QA entry point: `tools/test-ar-cad.sh`.
- It runs the Step 2, Step 3, Step 4–6 and Step 7–10 source acceptance suites, shell syntax checks and Python compilation checks.
- `RUN_RUST_TESTS=1` additionally runs Cargo tests for DXF, geometry, document, I/O, engine and UI crates with `--locked`.
- The Android Actions workflow runs this gate before attempting an APK build. A failed QA gate prevents the APK build step from running.

## Step 9 — Android performance and compatibility constraints

- The current native Android build is ARM64 (`arm64-v8a`) and uses Rust target `aarch64-linux-android`.
- The workflow installs the ARM64 Rust target explicitly and the source acceptance suite checks that the native build remains aligned with the launcher minimum SDK through `MIN_SDK`.
- CADCraft's release profile already enables thin LTO; the QA gate checks this remains configured.
- The current pipeline does not produce an x86_64 Android APK or an Android App Bundle. Emulator compatibility outside ARM64 is not promised.
- No FPS, memory, battery or thermal benchmarks have been measured in this environment. A real device benchmark remains a release gate.

## Step 10 — Release and signing hardening

- Manual workflow builds now default to **debug**, avoiding accidental release builds with throwaway signing keys.
- A release build requires the repository secrets `AR_CAD_KEYSTORE_BASE64` and `AR_CAD_KEYSTORE_PASSWORD`.
- The workflow restores the same keystore to `keys/artcraft-builder.jks` and the password file before Gradle runs. This preserves signing identity across cloud runs if the owner supplies the same keystore every time. The keystore must contain alias `artcraftbuilder`, with the key password matching the store password used by the current builder script.
- The workflow verifies the APK package ID and label, ZIP integrity, and release signature with `apksigner`; it also writes a SHA-256 checksum.
- Never commit the keystore, its password, or signing secrets. Keep an offline backup of the keystore. If it is lost, updates signed with a different key cannot replace the installed app normally.

## Remaining release blockers

1. A successful Android ARM64 debug build in GitHub Actions.
2. Install and launch on a physical ARM64 Android phone.
3. Exercise drawing, touch, command input, undo/redo, save/open, DXF import/export and recovery.
4. Confirm no crash, data loss, unusable touch targets, rendering defect or storage permission failure.
5. Capture startup time, memory use and sustained-use thermal behavior on at least one supported phone.
6. Configure a persistent signing key before creating a distributable release APK.

Until these are complete, AR CAD remains a development build and must not be described as production-ready.
