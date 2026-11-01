# AR CAD — DWG/DXF integrity and 3D kernel implementation update

**Date:** 2026-11-01  
**Scope:** First implementation increment for the critical DWG/DXF data-integrity gap and the major missing 3D solid-modelling capability.

## Executive status

Both workstreams have source changes and regression tests added. Neither is production-complete. Rust compilation and runtime tests remain blocked because this environment has no `cargo`, `rustc` or `rustfmt`.

## Workstream A — DWG/DXF integrity (critical)

### Implemented in this increment

- Extended `engine::cmd::file::not_saved_as` so the DWG pre-save loss report includes known bridge-loss metadata categories in addition to unmodelled graphical entity types.
- Added counts for entity persistent reactors and entity colour-book names.
- Added counts for table/block XDATA, reactors and colour-book names.
- Added detection for retained custom-class objects by matching their DXF type against retained `CLASS` definitions.
- Updated the warning to state that the DWG bridge may drop entity types or metadata, while DXF retains the raw groups.
- Added an engine regression test for the additional metadata-loss categories.
- Extended the interactive pre-save warning to describe both object and metadata loss.
- Added a UI regression test proving metadata-only DWG loss holds QSAVE before writing, and Cancel leaves the original file bytes unchanged.

### Still missing

- The underlying `acadrust` DWG bridge still does not preserve all those categories. This increment improves loss detection/reporting; it does not make the DWG writer lossless.
- No independent AutoCAD/ODA round-trip corpus has been executed.
- Need a tested preservation strategy for unsupported DWG classes, table/block XDATA, reactors and colour-book metadata, or an explicit safe-save refusal where preservation cannot be guaranteed.
- Need tests against real DWG samples, multiple DWG versions, proxy/custom classes, large drawings and malformed inputs.
- The existing backup mechanism remains a safety net, not a substitute for lossless round-tripping.

**Status: IMPROVED LOSS DETECTION; LOSSLESS DWG COMPATIBILITY STILL OPEN / CRITICAL.**

## Workstream B — 3D B-rep foundation (major)

### Implemented in this increment

Added `cadcraft-geom::brep` with:

- Explicit vertices, edges, half-edges, reciprocal twins, loops, planar faces, shells and solids.
- Topology validation for finite vertices, valid references, reciprocal half-edge adjacency, closed loops, edge manifold pairing and face/shell/solid ownership.
- Polygon-prism extrusion from a simple planar polygon, supporting clockwise/counter-clockwise profiles and positive/negative height.
- Scale-aware profile validation and rejection of degenerate/self-intersecting profiles.
- Volume and planar surface-area properties.
- Regression cases for a rectangular prism, negative extrusion, microscopic geometry, concave profiles at large world coordinates, self-intersecting profiles and zero height.

### Still missing

- Rust compilation and execution of the new tests.
- Persistent editable B-rep integration into the document entity model, renderer, selection, grips and undo/redo. The current command emits faceted 3DFACE shell entities, not a persistent B-rep solid.
- Interactive Android point-picking workflow and true 3D viewport/navigation.
- General planar-face holes, arbitrary profile planes/UCS, curved surfaces, analytic/NURBS surface representation and curve/surface intersections.
- REVOLVE, SWEEP, LOFT and robust Boolean union/subtract/intersect.
- Shell, 3D fillet/chamfer, healing and topology repair.
- Centre of mass, centroid, inertia tensor and material-density mass properties.
- STEP/IGES and robust 3D DWG/ACIS exchange.
- Geometry fuzzing, real-world model corpus and performance tests.

**Status: B-REP FOUNDATION PLUS FIRST DOCUMENT-INTEGRATED EXTRUDE COMMAND; OUTPUT IS A FACETED 3DFACE SHELL, NOT A PRODUCTION 3D MODELLER.**

## Validation performed

- `tools/test-critical-cad-gaps.py`: 17/17 source-contract assertions passed.
- Full `tools/test-ar-cad.sh`: 185/185 source-level assertions passed; shell syntax and Python compile checks passed.
- Rust tests: not run; Cargo/rustc are unavailable.
- Android build/APK: not attempted in this increment; previous preflight remains blocked by missing Rust/Cargo and Android toolchain.

The source-level assertions verify that expected code and regression tests are present. They do not prove Rust compilation, runtime geometry correctness, DWG interoperability or production readiness.

## Updated release status

- Point 3 — DWG/DXF reliability: **PENDING / CRITICAL**. Loss detection and interactive pre-save protection are improved; actual lossless preservation remains unfinished.
- 3D modelling: **PARTIAL FOUNDATION**. Basic polygon extrusion and topology structures are added; all production solid modelling operations remain unfinished.
- Production release: **BLOCKED** until Rust tests, real-file interoperability and Android device acceptance pass.


## Follow-up increment — metadata-only save protection

The UI save-loss dialog now explicitly states that the format may lose objects or metadata. A UI regression test constructs a drawing with foreign entity reactors, colour-book metadata, table XDATA and a retained custom-class object, then verifies that QSAVE is held before the writer runs and Cancel leaves the original DWG bytes unchanged. This closes a safety gap in the warning path; it does not repair the DWG bridge itself.


## Follow-up increment — document-integrated EXTRUDE

Added `engine::cmd::draw2::solid3d` and registered the `EXTRUDE` command (alias `EXT`). It accepts either a closed straight-segment lightweight polyline handle or explicit simple polygon points with a height. It builds and validates the B-rep prism, triangulates concave cap faces with ear clipping, and emits native `3DFACE` entities into the current document. It reports B-rep volume and surface area. Regression tests cover direct command dispatch, rectangle extrusion, concave caps, a closed-polyline handle and invalid height.

Important limitation: the resulting entities are a faceted surface shell. The B-rep is not persisted as an editable solid object; Boolean operations, arbitrary profile planes, curved profiles, true 3D viewport interaction and solid editing remain open.

Latest validation: 17/17 critical-gap source checks and 185/185 total source assertions pass. Rust compilation and runtime tests remain blocked because Cargo/Rust are not installed.


## Follow-up increment: persistent faceted-solid groups

The EXTRUDE command now writes a shared `ARCAD_BREP` XDATA marker to every generated 3DFACE: group identity, marker version, face index and expected face count. `SOLIDINFO` finds all faces in the group and validates the closed shell. `SOLIDEDITDELETE` deletes the complete group only when its member count matches the stored count; it refuses a partial delete when a member is missing. This is group-level persistence and lifecycle management for faceted geometry, not a true editable B-rep. Confirm XDATA survival through actual DXF and DWG save/reopen tests before relying on persistence in external files.
