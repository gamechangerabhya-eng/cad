# AR CAD — Steps 7–10 Validation Report

Date: 2026-10-31

## Source-level acceptance

| Suite | Result |
|---|---:|
| Step 2 regression | 18/18 passed |
| Step 3 acceptance | 25/25 passed |
| Steps 4–6 acceptance | 14/14 passed |
| Steps 7–10 acceptance | 17/17 passed |
| Unified QA runner | Passed in source-only mode |
| Shell syntax checks | Passed for discovered shell scripts |
| Python compile checks | Passed |
| Workflow YAML parse | Passed with PyYAML (GitHub's `on` key is interpreted as a boolean by YAML 1.1 parsers; GitHub Actions uses its workflow parser) |
| Legacy DXF/native host regression script | Passed all listed codec, geometry, recovery, snap, spatial-index, FFI/JNI and release-gate checks |

## Not run / not proven

- Current CADCraft 0.15.1 Rust/Cargo unit and integration tests: not run; `cargo` and `rustc` are not installed in this environment.
- Android native ARM64 build and Gradle APK packaging: not run; Android SDK/NDK, Gradle and cargo-ndk are unavailable.
- APK package/signature inspection: not run because no APK was produced.
- Installation and runtime testing on a physical Android device: not run.
- Performance, memory, battery and thermal benchmarks: not run.

## Important release hardening

- Manual cloud builds now default to debug.
- Release mode requires stable GitHub repository secrets `AR_CAD_KEYSTORE_BASE64` and `AR_CAD_KEYSTORE_PASSWORD` so successive releases can use the same signing identity.
- The workflow verifies APK package ID/label and ZIP integrity, verifies release signatures, and emits SHA-256 checksums.

## Status

Steps 7–10 source and workflow changes are prepared and source-validated. They are **not** a production release. The full Rust QA gate and Android build must run in GitHub Actions, followed by physical-device verification using `docs/AR-CAD-RELEASE-CHECKLIST.md`.
