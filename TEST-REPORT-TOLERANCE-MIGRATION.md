# Test report — shared geometry tolerance migration

Date: 2026-10-31

## Implemented source changes

- `crates/geom/src/intersect.rs`: shared tolerance variants for infinite-line, line-circle and circle-circle intersections; finite segment/arc dispatch now reuses the supplied context.
- `crates/geom/src/lib.rs`: exported the new tolerance-aware APIs.
- `crates/geom/src/tests.rs`: added two regression tests for tolerance-aware line-circle and circle-circle tangency classification.
- `docs/test-geometry-priority.py`: expanded source assertions from 10 to 14 checks.
- `docs/AR-CAD-TOLERANCE-MIGRATION.md`: recorded scope, limitations and next validation steps.

## Results

| Check | Result |
|---|---:|
| Geometry-priority source assertions | 14/14 passed |
| Unified AR CAD source acceptance | Passed; 18 + 25 + 14 + 17 + 14 assertions |
| Shell syntax checks | Passed |
| Python compileall | Passed |
| Legacy DXF/native regression script | Passed |
| Current CADCraft Rust compile/tests | Not run — Cargo/Rust unavailable |
| Android APK build/device test | Not run |

## Status

Source implementation is present. Rust compilation and the two new Rust tests are **unverified** until executed in Rust-enabled CI. This work must not be marked production-ready based on static checks alone.
