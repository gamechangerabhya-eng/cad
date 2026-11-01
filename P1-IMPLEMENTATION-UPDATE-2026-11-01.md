# AR CAD P1 implementation update — 2026-11-01

## Implemented in this update

### 1. Scale-normalized adaptive orientation predicate
- `orient2d` now uses a power-of-two coordinate normalization before exact difference expansions.
- Ambiguous cases use `TwoDiff`, FMA-backed `TwoProduct`, and expansion accumulation to preserve determinant sign without relying on a fixed epsilon.
- It detects coordinate/product underflow and uses the conservative filtered fallback where expansion arithmetic cannot preserve all information.
- Added tests for near-collinear large coordinates and very small/large scales.
- This is materially stronger than the prior scaled-FMA-only predicate, but it is not advertised as a formally verified arbitrary-range Shewchuk implementation. Extreme IEEE-754 exponent combinations still need a dedicated adversarial validation suite and a vetted reference implementation comparison.

### 2. Planar repair path for OFFSET
- Added a planar graph face extraction helper that splits straight segments at intersections and traces bounded faces.
- It classifies faces using the original contour's winding number.
- OFFSET now routes self-crossing closed, line-only results through this repair path.
- Because OFFSET currently emits one entity, the repair helper rejects multiple filled faces instead of silently dropping valid geometry. Arc-containing and open self-crossing offsets also remain safely rejected.
- Added regression tests for a bow-tie (ambiguous two-region rejection) and a simple one-face contour.

## Validation performed
- Unified source acceptance runner: 147/147 source-level assertions passed.
- Shell syntax checks passed.
- Python compile checks passed.
- Rust tests were not executed: Cargo and rustc are unavailable in this environment.
- Android APK build and physical-device tests were not performed.

## Remaining P1 work
- Multi-loop OFFSET output and complete topology repair, including arc/bulge preservation, open offsets, overlaps, tangencies, holes and nested loops.
- Complete round-fillet construction for all line/arc/polyline corner configurations.
- Formally validated adaptive exact predicates over adversarial IEEE-754 ranges (including subnormals, overflow, underflow and cancellation).
- Certified ellipse/spline intersection error bounds and complete root isolation; current spline-line solver is sampled/refined and is not certified.
- Full fixed-epsilon audit with documented classification and scale-aware replacement where appropriate.
- Rust workspace compile/tests, Android ARM64 APK build, install, touch workflow, file interoperability, performance and crash-recovery tests.

## Release status

**Not production-ready.** Source-level checks are not substitutes for Rust compilation, executable geometry tests, APK validation or device testing. The full professional 2D/3D feature roadmap also remains incomplete.
