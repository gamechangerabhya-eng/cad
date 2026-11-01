# AR CAD — Priority Implementation and Revalidation

## This revision's source changes

### BREAK / BREAKATPOINT
- Derives a `ToleranceContext` from the selected entity's bounds.
- Rejects zero-length lines using the operation's linear tolerance.
- Uses normalized parameter tolerance for line cut endpoints.
- Uses angular tolerance for arc endpoint classification.
- Uses drawing-unit tolerance for polyline path-length classification and single-point break detection.

### JOIN
- Derives one tolerance from all selected segment bounds.
- Uses that shared tolerance for endpoint chaining and closed-loop detection instead of a fixed `1e-6` drawing-unit threshold.

### Polyline OFFSET safety
- Checks generated offset segments for collapsed/near-zero edges.
- Checks non-adjacent generated segments for self-intersections.
- Rejects a collapsed or self-intersecting result instead of writing invalid polyline topology.
- This is a safety guard, not a full self-intersection resolution algorithm. The operation still needs a robust arrangement/loop reconstruction implementation to trim and return all valid offset loops.

## Validation performed in this environment

- Existing source acceptance suites: 18 + 25 + 14 + 17 + 14 + 20 checks, previously recorded in the supplied source tree.
- Added 10 source-contract assertions for this revision.
- Shell syntax and Python compilation checks are run again by `tools/test-ar-cad.sh`.
- Legacy DXF/native regression suite remains a separate older-code test suite.

## Validation not performed

- Rust compilation, rustfmt, Clippy or current CADCraft Rust unit/integration tests: Rust toolchain is unavailable in this environment.
- Android Gradle/NDK build, APK install, runtime, touch/stylus, lifecycle and storage tests: Android toolchain and device access are unavailable.
- Full feature conformance, DWG round-trip corpus, 3D kernel/modeling and 3D viewport tests: not complete.

## Release decision

**Do not start a production/release APK build yet.** It is reasonable to start a CI debug-build attempt once a Rust/Android cloud runner is available, but that is a diagnostic build, not release readiness. The immediate blocker is to compile and run the exact Rust revision, fix any compiler/test failures, and then install the debug APK on a physical ARM64 Android device.
