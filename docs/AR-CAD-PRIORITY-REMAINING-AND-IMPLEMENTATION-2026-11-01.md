# AR CAD — Priority implementation update and remaining work

**Revision:** 2026-11-01
**APK:** deliberately deferred.
**Important:** this report does not declare the priority programme complete. It records the source changes made in this increment and the remaining acceptance gates.

## Changes implemented in this increment

### OVERKILL closed-polyline seam cleanup
- Extended `OVERKILL` polyline optimization to inspect the cyclic `(penultimate, last, first)` triplet on closed polylines.
- A redundant straight vertex at the closing seam is now removed when both adjacent segments are straight, same-direction, collinear within the requested tolerance, and have zero widths.
- The operation preserves the original polyline start point and does not alter curved/width-bearing segments.
- Added an engine integration regression test for a closed contour with a redundant seam vertex and a source acceptance assertion.
- This improves one OVERKILL case; it does not implement AutoCAD's full Geometry Cleanup/Geometry Repair workflow or preview-based repair UI.

### OVERKILL line-direction normalization seam
- Fixed a cyclic-angle grouping gap in collinear line cleanup: line directions immediately on either side of the internal normalization seam are now considered neighboring candidates within angular tolerance.
- Wrapped groups align their projection axes before collinear overlap analysis, preventing a false offset split caused solely by opposite vector orientation.
- The logic is applied independently per entity-property bucket so ignored-property settings remain respected.
- Added an engine integration regression case using nearly collinear, oppositely directed segments across the seam.
- This is a targeted OVERKILL correctness improvement, not a claim of complete AutoCAD Geometry Cleanup parity.

### 3D grouped-facet integrity
- Strengthened `ARCAD_BREP` XDATA parsing: duplicate/malformed version, group, index or count metadata is rejected.
- Group discovery now checks consistent expected face count, unique face indices, index range and complete contiguous index coverage.
- `SOLIDINFO` reports `metadataValid` separately from geometric shell validity.
- `SOLIDEDITDELETE` now refuses deletion when group metadata is corrupt/incomplete or the faceted shell is not a closed, consistently oriented manifold.
- Corrected a stale Rust regression expectation: EXTRUDE emits `grouped-faceted-3dface-shell`, not `faceted-3dface-shell`. This was a latent test failure that source-presence checks had not previously detected.
- Strengthened B-rep topology validation so every loop must be owned exactly once by a face; orphaned or multiply referenced face loops are rejected.

### QA improvements
- Added source checks for unique group indices, malformed XDATA, safe deletion, exact EXTRUDE representation label and face-loop ownership.
- Re-ran the complete source acceptance suite.

## Priority status matrix

| Priority | Workstream | Current implementation | Remaining acceptance criteria | Status |
|---|---|---|---|---|
| P0 | DWG/DXF integrity | DWG export fails closed for known unsupported entities/metadata and partial reads; DXF export refuses known incomplete reads; save UI blocks lossy overwrite | Compile/run I/O tests; build a versioned DWG/DXF corpus; independently round-trip in AutoCAD and ODA; fix bridge preservation or keep specific cases blocked; verify XDATA, reactors, classes, colour books, ACIS and layouts | **OPEN — NOT LOSSLESS-CERTIFIED** |
| P0 | Rust executable validation | Regression tests are in source; source/shell/Python checks run | Install/use Rust toolchain; `cargo test --locked -p cadcraft-dxf -p cadcraft-geom -p cadcraft-doc -p cadcraft-io -p cadcraft-engine -p cadcraft-ui-egui --no-fail-fast`; fix all compiler/test failures | **BLOCKED — NO CARGO/RUSTC HERE** |
| P1 | Persistent editable 3D B-rep | Polygon-prism topology foundation; EXTRUDE produces validated grouped 3DFACE shells; SOLIDINFO and SOLIDEDITDELETE; metadata and shell checks hardened | Native document-level B-rep entity, save/load persistence, grips/selection/properties, transformations, arbitrary plane/UCS profiles, holes/curved surfaces, booleans, fillet/chamfer, healing and mass properties; compile/runtime tests | **PARTIAL — FACETED OUTPUT ONLY** |
| P1 | 3D viewport and interaction | 3D face entities can be represented by existing drawing engine | Orbit/pan/zoom, depth-aware selection, grips, shading modes, hidden-line, visual styles and device interaction tests | **OPEN** |
| P1 | Geometry Cleanup | AUDIT/RECOVER exists; audit shortcut exposed on mobile | Implement guided detection for gaps, overshoots, undershoots, duplicate/overlapping segments and misaligned angles; previews, per-finding repair, undo and tests | **OPEN — AUDIT IS NOT GEOMETRY CLEANUP** |
| P1 | Complex 2D geometry | Robust predicates, tolerance migration, some multi-loop OFFSET and spline/ellipse intersection work | Complete curved offset arrangements and open-path joins/caps; complete line/arc/polyline FILLET matrix; certified spline/ellipse bounds; broad randomized/adversarial regression | **PARTIAL** |
| P1 | Xrefs and dynamic blocks | Common Xref attach/load and static block editing | XCLIP, in-place Xref editing, persistence/override behavior, dynamic block authoring and parameter/action evaluation | **PARTIAL** |
| P1 | Android command discoverability | Four-row toolbar exposes file, draw, edit, measurement, annotation and inspection shortcuts | Rust compile, install/debug runtime, touch/IME/SAF/lifecycle and small/large-screen acceptance tests | **SOURCE UPDATED — RUNTIME BLOCKED** |
| P2 | Trace, markup and collaboration | No complete equivalent found | Local trace overlays, markup import/review, sharing, conflict handling and multi-user workflows | **MISSING** |
| P2 | Drawing-aware AI and Smart Blocks | External MCP/agent control is present | In-app assistant with document context, safe command preview/approval, block recognition/insertion and audit trail | **MISSING** |
| P2 | Plotter and specialist file compatibility | PDF export and internal plot config paths | CTB/STB/PC3/PMP interoperability and print validation against reference output | **PARTIAL** |

## Validation actually performed

Command: `./tools/test-ar-cad.sh`

- Step 2: 18/18 source checks
- Step 3: 37/37 source checks
- Steps 4–6: 14/14 source checks
- Steps 7–10: 17/17 source checks
- Geometry priority: 14/14 source checks
- Tolerance migration: 20/20 source checks
- Precision safety: 11/11 source checks
- P1 source checks: 49/49
- Critical-gap checks: 34/34
- **Total: 218/218 source-level assertions passed**
- Shell syntax: passed
- Python compileall: passed

Explicit Rust test attempt: `RUN_RUST_TESTS=1 ./tools/test-ar-cad.sh` exited 2 with `cargo is unavailable`. `rustc`, `rustfmt`, Gradle and ADB are also unavailable. No Rust test, Android build, APK, device test or AutoCAD/ODA round-trip was executed.

## Completion decision

**The priority programme is not complete.** This increment closes a real test expectation mismatch, strengthens 3D group corruption handling, and fixes closed-polyline seam optimization in OVERKILL. It does not turn the DWG bridge into a lossless codec or turn faceted 3DFACE groups into native editable B-rep solids. P0 DWG integrity and executable Rust validation remain release blockers. APK creation remains deferred as requested.

## R6 verification and CI correction (2026-11-01)

### Additional issue found and fixed
- The GitHub Actions workflow was triggered by version tags (`v*.*.*`), but its three build-mode expressions defaulted to `debug` whenever the run was not manually dispatched. This meant a release tag could silently produce a debug APK instead of a release APK.
- Updated all three mode selections so manual dispatch still respects `inputs.mode`, version tags select `release`, and ordinary branch pushes remain `debug`.
- Kept the release signing-secret requirement and APK signature verification in place.
- Added `docs/test-ci-workflow.py` with six focused acceptance checks and wired it into `tools/test-ar-cad.sh`.

### Re-verification
- Step 2: 18/18 source checks
- Step 3: 37/37 source checks
- Steps 4–6: 14/14 source checks
- Steps 7–10: 17/17 source checks
- Geometry priority: 14/14 source checks
- Tolerance migration: 20/20 source checks
- Precision safety: 11/11 source checks
- P1 source checks: 49/49
- Critical-gap checks: 43/43
- CI workflow checks: 6/6
- **Total: 229/229 source-level assertions passed**
- Shell syntax and Python compileall passed.
- GitHub workflow YAML parsed successfully with the available YAML parser.

### Environment verification
- `tools/ar-cad-preflight.sh` was run and correctly failed because Cargo, rustc, rustup, Android SDK and Android NDK are unavailable.
- `RUN_RUST_TESTS=1 ./tools/test-ar-cad.sh` was attempted and exited 2 with the explicit message that Cargo is unavailable.
- The normal source gate exits 0 but intentionally skips Rust tests unless `RUN_RUST_TESTS=1` is set.
- A Python startup warning from the environment's spreadsheet runtime warmup appeared on stderr; it did not cause the source gate to fail and is unrelated to AR CAD code.

### Release status
The CI tag-mode defect is fixed and source-verified. The project is still **not runtime-verified or production-ready**: no Rust compile/test, Android APK build/install, device test, or independent DWG/DXF interoperability test has been executed in this environment.
