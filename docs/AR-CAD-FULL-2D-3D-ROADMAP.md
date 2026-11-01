# AR CAD — Full Professional 2D + 3D CAD Roadmap

**Product target:** a professional mobile-first CAD application with complete 2D drafting/documentation and genuine 3D solid, surface and mesh modeling. This is an engineering roadmap, not a claim of AutoCAD parity.

**Baseline:** AR CAD Step 1–10 source tree with CADCraft 0.15.1. The upstream engine describes itself as pre-alpha; its own readiness percentages are estimates and are not AR CAD test results.

## Release principles

1. A visible command is not a completed feature. Every feature needs a working implementation, deterministic tests, file round-trip coverage and Android interaction tests where applicable.
2. No destructive or lossy save is acceptable without a clear warning and a recoverable backup.
3. 2D reliability and file safety come before broad 3D feature count. The app must be dependable for ordinary drawings before promising professional use.
4. 3D means editable model geometry, not merely Z coordinates, 3D polylines or a perspective-looking viewport.
5. Never claim DWG losslessness, full AutoCAD compatibility or production readiness without independent corpus-based verification.
6. All core workflows must work offline on supported Android devices. Cloud/collaboration features are later optional capabilities.

## Prioritized workstreams

### P0 — Establish a trustworthy engineering baseline (blocking)
- Run `cargo fmt --check`, Clippy and the complete relevant Rust test matrix on the exact AR CAD revision.
- Pin and record Rust, NDK, SDK, Gradle and dependency versions; make CI reproducible.
- Build and install a debug ARM64 APK; test startup, lifecycle, file picker, keyboard/IME, touch, save and reopen.
- Add CI artifacts for APK, test reports, checksums and build provenance.
- Establish crash logs, regression corpus and a no-data-loss policy.

**Exit criteria:** reproducible APK build; all required Rust tests pass; physical Android smoke suite passes; no known silent save loss.

### P1 — 2D geometry correctness and precision (first implementation priority)
- Introduce drawing/operation-scoped tolerance contexts; migrate geometry predicates away from global absolute epsilon.
- Exact or bounded-error ellipse/spline intersections and closest-point calculations using subdivision plus safeguarded Newton refinement.
- Apply shared intersection routines to OSNAP, TRIM, EXTEND, BREAK, FILLET and measurement.
- Correct polyline offset: recompute arc bulges, remove collapsed segments, resolve self-intersections and implement OFFSETGAPTYPE.
- Add robust orientation/collinearity predicates and scale-aware degeneracy handling.
- Complete spline fit controls: end tangents, knot parameterization, fit tolerance and periodic curves.
- Add known-answer tests at micro, ordinary, survey (1e6–1e7) and hostile numeric scales.

**Exit criteria:** same geometry answers within declared tolerances across scale; no invalid NaN/Inf geometry; exact/bounded-error results covered by regression tests.

### P2 — Core 2D command depth
- Finish command options and edge cases for LINE, PLINE, SPLINE, OFFSET, TRIM, EXTEND, ROTATE, SCALE, ARRAY, FILLET, CHAMFER, JOIN, BREAK and PEDIT.
- Complete selection modes, crossing/window, grips, property editing, UCS and coordinate entry.
- Add undo/redo transaction tests for every modifying command, including cancel/error paths.
- Add reusable command conformance tests from documented input → expected entity/document changes.

**Exit criteria:** all advertised core commands have tested options, correct cancel semantics and undo/redo.

### P3 — 2D annotation, blocks, layers and hatch
- Full DIMSTYLE manager, dimension overrides and dimension editing.
- MTEXT in-place editing, text fields, font substitution/SHX fallback metrics and multiline fidelity.
- MLEADER styles and editing; table styling, formulas and data links.
- Block editor, attributes/ATTEDIT, groups, XREF attach/reload/detach/bind, path resolution and XCLIP.
- Hatch gap tolerance, curved boundaries, island modes, associative boundary maintenance and expanded pattern support.
- Dynamic blocks/parameters after static block workflows are reliable.

**Exit criteria:** common architectural/mechanical drawings preserve annotation and block semantics through edit/save/reopen.

### P4 — Layouts, plotting and documentation output
- Persistent page setups, paper sizes, printable margins and named viewports.
- CTB/STB parsing, plot style application and lineweight/dash fidelity.
- Android PDF export and print/share flow with deterministic output.
- Batch publish, multi-sheet output and plot preview parity.

**Exit criteria:** known-answer plot/PDF corpus matches expected scale, extents, lineweights and page setup.

### P5 — File compatibility and data integrity
- Build a versioned DXF/DWG corpus from real, licensed, varied drawings (small/large, proxies, xdata, blocks, dimensions, hatch, layouts, fonts and external references).
- Compare open/save/reopen in CADCraft and at least one independent compatible CAD application; use AutoCAD black-box verification where available.
- Preserve unknown classes, objects, xdata, reactors, table records and proxy data; report unsupported entities before save.
- Add chosen-version save controls, backups, recovery, AUDIT/RECOVER and atomic writes.
- Define explicit compatibility matrix; do not market lossless DWG until every required case passes.

**Exit criteria:** no silent data loss in the supported corpus; unsupported content is preserved or clearly reported; crash-safe save verified.

### P6 — Genuine 3D geometry kernel (major subsystem)
- Select and document a permissively licensed, Android-compatible geometry-kernel strategy after legal, ABI, maintenance and performance review. Do not confuse polygon meshes with a boundary-representation solid kernel.
- Introduce 3D model entities and topology: vertices, half-edges/edges, loops, faces, shells and solids; stable identity and adjacency.
- Add robust 3D vector/matrix, plane, line, ray, curve and surface primitives; scale-aware 3D tolerances.
- Add analytic primitives and NURBS curves/surfaces; tessellation with explicit chord/normal error.
- Implement manifold validation, orientation, watertightness, self-intersection diagnostics and topology healing.
- Add serialization/versioning and undo transactions for 3D model entities.

**Exit criteria:** validated editable closed solids and surfaces with deterministic topology tests, not just display meshes.

### P7 — 3D modeling operations
- Box, wedge, cone, cylinder, sphere, torus and pyramid primitives.
- Extrude, revolve, sweep, loft, presspull and planar/ruled/network surfaces.
- Boolean union, subtract and intersect; robust splitting and sectioning.
- Fillet/chamfer edges, shell, thicken, slice, offset faces, move/rotate faces, imprint and separate.
- Mesh primitives, mesh editing, smoothing/subdivision and mesh-to-solid conversion with clear limits.
- Mass properties: volume, surface area, centroid, inertia and principal axes.

**Exit criteria:** modeling operation suite passes analytic reference tests, topology validity and save/reopen checks.

### P8 — 3D viewport, camera and interaction
- GPU-backed 3D renderer, depth buffer, hidden-line/shaded/realistic visual styles.
- Orbit, pan, zoom, perspective/orthographic cameras, standard views and view cube.
- UCS/workplanes, 3D snaps, gizmos, face/edge/vertex selection and section planes.
- Materials, lights, shadows and basic rendering only after geometry and viewport stability.
- Android touch gestures must distinguish orbit from selection and maintain precision input.

**Exit criteria:** stable navigation and selection on target phones with large models; correct depth and selection feedback.

### P9 — 3D exchange and interoperability
- Define supported STEP/IGES and mesh import/export strategy; validate units, colors, names and topology.
- DXF/DWG 3D entity support and solid exchange where licensed formats/kernel allow it.
- Round-trip test solids, surfaces, meshes, assemblies/blocks and metadata.

**Exit criteria:** published import/export matrix with verified corpus and no misleading unsupported-format claims.

### P10 — Android production hardening
- Large drawing performance and memory budgets; incremental rendering/spatial indexing.
- Background/resume, orientation, process death, low-memory recovery and storage permission edge cases.
- Input latency, stylus support where available, external keyboard and accessibility.
- Device matrix for supported Android versions, ARM64 GPUs and screen sizes; thermal/battery testing.
- Crash telemetry only with privacy disclosure and opt-in; offline operation remains supported.

**Exit criteria:** repeated long-session tests, large-file tests, crash-free target sessions and documented device support.

### P11 — Release and maintenance
- Signed reproducible release APK/AAB, versioning, SBOM/license notices, privacy policy and backup guidance.
- Security review of file parsers and untrusted drawing inputs; fuzz parsers and geometry operations.
- Release notes, known limitations, support channel, update/rollback and migration tests.

**Exit criteria:** signed artifact, independent installation test, release checklist and support/maintenance plan.

## Implementation started in this revision

**P1.1 — `ToleranceContext` foundation** added to `crates/geom` and exported from the geometry crate. It derives a linear tolerance from model extent and floating-point coordinate magnitude, plus normalized parameter and angular tolerances. Unit tests cover tiny/ordinary drawings, survey coordinates, degenerate bounds and non-finite inputs.

Important limitation: this is an API foundation only. Existing geometry consumers still use legacy `EPS` and local constants until migrated operation-by-operation. Therefore this does not yet resolve the tolerance issue globally. Rust tests could not be executed in the current environment because the Rust toolchain is absent; the new tests are committed as source and require CI execution.

## Current status ledger

| Workstream | Status |
|---|---|
| P0 build/device baseline | Blocked: no verified APK/device run |
| P1 geometry precision | Started: tolerance context foundation; consumers and curve algorithms remain |
| P2 core 2D commands | Partial upstream implementation; conformance gaps remain |
| P3 annotation/blocks/hatch | Partial; substantial gaps remain |
| P4 layouts/plot | Partial; device/output validation remains |
| P5 file compatibility | Critical data-preservation and corpus gaps remain |
| P6 3D kernel | Not started; architecture/kernel selection required |
| P7 3D modeling | Not started |
| P8 3D viewport | Not started |
| P9 3D exchange | Not started |
| P10 Android hardening | Not verified |
| P11 release | Not started |

## Definition of done

A feature is complete only when implementation, unit/integration tests, regression tests, serialization/round-trip tests, UI interaction tests where relevant, and documented limitations all pass. Source-presence checks alone do not count as feature completion.

### P1 implementation update — 2026-10-31
- Added robust orientation/collinearity predicates and scale-safe Vec2 normalization.
- Three-point circle construction now uses drawing-aware collinearity and a normalized local solve.
- Spline fit tangent and chord parameterization no longer discard valid microscopic geometry due to a fixed 1e-12 floor.
- Added drawing variable `OFFSETGAPTYPE` (0 extend, 1 round connector where supported, 2 chamfer) to the offset command and preview.
- Added regression tests and source-contract checks.
- Status remains **partial**: offset loop repair is not implemented (invalid self-intersections are rejected), some round-join cases fall back to chamfer, the robust predicate is not formally exact for every floating-point input, and Rust/Android execution remains pending. See `P1-IMPLEMENTATION-REPORT.md`.
