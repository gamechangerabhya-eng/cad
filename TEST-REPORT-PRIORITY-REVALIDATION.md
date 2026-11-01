# AR CAD — Priority implementation and revalidation report

Date: 2026-11-01

## Implemented in this revision

1. BREAK/BREAKATPOINT now derives a geometry-scoped tolerance from the selected entity bounds. Line endpoint clipping uses normalized parameter tolerance; zero-length lines are rejected; arc and polyline break decisions use the shared operation tolerance.
2. JOIN now derives one tolerance from the selected segment bounds and uses it for endpoint chaining and closed-chain detection instead of a fixed absolute `1e-6` threshold.
3. Polyline OFFSET now rejects non-finite output vertices, collapsed edges and non-adjacent finite-segment self-intersections. This prevents invalid topology from being written. It does not yet reconstruct all valid offset loops after a self-intersection; it fails safely instead.
4. Added 11 source-contract checks and wired them into `tools/test-ar-cad.sh`.
5. Added a priority implementation/revalidation document with the remaining blockers.

## Current source gate results

| Suite | Result |
|---|---:|
| Step 2 acceptance | 18/18 |
| Step 3 acceptance | 25/25 |
| Steps 4–6 acceptance | 14/14 |
| Steps 7–10 acceptance | 17/17 |
| Geometry priority source checks | 14/14 |
| Tolerance migration source checks | 20/20 |
| New BREAK/JOIN/OFFSET source checks | 11/11 |
| **Total source assertions** | **119/119** |
| Shell syntax checks | Passed |
| Python compileall | Passed |
| Legacy DXF/native regression script | Previously passed; this revision's unified run does not re-run it |

These are source/static checks, not Rust tests.

## Build/test attempts

- `RUN_RUST_TESTS=1 ./tools/test-ar-cad.sh` was attempted and exited 2 because `cargo` is unavailable.
- `./build-ar-cad-debug.sh` was attempted and exited 1 at preflight because Rust/Cargo is unavailable.
- `rustc`, `cargo`, `gradle`, `adb` and `aapt` are unavailable in this environment.
- No APK was produced. No Android device/emulator tests were performed.

## Remaining before a diagnostic APK build

**Required immediate blockers:**
- Run Rust formatting, compilation, Clippy and CADCraft unit/integration tests on this exact source revision.
- Fix any compile/test failures.
- Build a debug ARM64 APK in a Rust + Android SDK/NDK environment.

**Required before production/release:**
- Install and test on physical Android devices, including lifecycle, file picker, IME, touch, save/reopen, undo/redo and recovery.
- Test large drawings, memory/performance and crash recovery.
- Validate DXF/DWG data preservation against a licensed real-world corpus and independent CAD implementation.
- Complete offset self-intersection loop reconstruction; current implementation rejects unsafe output rather than healing it.
- Complete remaining professional 2D features: annotation/dimension editing, blocks/attributes/XREF, associative hatch, layouts/page setup/plot and command conformance.
- Select and integrate a licensed, Android-compatible 3D B-rep kernel; implement and test 3D topology, solids, booleans, modeling operations, viewport, materials and STEP/IGES exchange.
- Security/fuzz tests for untrusted drawings, signed release, SBOM/license notices and privacy/release documentation.

## Decision

**Do not call this release-ready.** A cloud diagnostic debug APK build may start once a Rust/Android runner is connected, but only after the current Rust workspace tests compile and pass. A production APK must wait for physical-device and data-integrity validation.
