# AR CAD — Current implementation priorities (2026-11-01)

## Priority 1 — DWG/DXF data integrity (critical, safe refusal implemented; certification open)

### Implemented in the latest increment
- Added an I/O-boundary DWG preflight so every caller is protected, not just the UI.
- Refuse DWG output for unknown entities (except ACIS modeler entities with a matching modern data-store record), known entity/table/block metadata losses, custom-class objects and incomplete source reads.
- Refuse DXF export when the source parser recorded skipped records; a partial source is not labelled a complete export.
- Removed the UI Save Anyway option; Save As and Cancel are the only responses to known DWG loss.
- Added direct I/O regression cases for unknown entities, metadata, incomplete reads, and clean modelled DWG output.
- Existing DXF tests exercise unknown payload/reference round-trips across ASCII/binary versions.

### Still required for full Point 3 certification
- Run Rust tests with Cargo/rustc in CI.
- Validate against a licensed real-file corpus and independent AutoCAD/ODA open-save-reopen cycles across DWG 2000–2018.
- Expand the blocker inventory for any additional unsupported object families found in that corpus; do not allow export until tested.
- Verify proxy/custom-class, xdata, reactors, colour-book, ACIS solids, large files, damaged files, layouts, blocks, xrefs, dimensions and hatch interoperability.

## Priority 2 — 3D solid kernel (major, foundation only)

### Implemented in the latest increment
- Added `cadcraft-geom::brep` with vertices, edges, half-edges, twins, loops, planar faces, shells and solids.
- Added topology validation for references, twin reciprocity, closed loops, edge pairing and ownership.
- Added simple polygon-prism extrusion for either winding and positive/negative height.
- Added volume and planar surface-area calculations.
- Added source regression cases for rectangular/concave prisms, negative height, microscopic profiles, large coordinate offsets, broken twins, bow-tie profiles and zero height.
- Integrated an `EXTRUDE` engine command that creates native faceted `3DFACE` geometry from closed lightweight polylines or explicit XY points.
- Added a mesh manifold check before document mutation: each undirected edge must be used exactly twice with opposite orientation; non-finite vertices and zero-length edges are rejected.
- Added regression tests for missing-face rejection and concave cap triangulation seam closure.

### Still required
- Compile and run the new Rust tests.
- Integrate persistent editable B-rep entities into the document model, renderer, selection, grips and undo/redo; current `EXTRUDE` output is faceted `3DFACE`, not editable B-rep.
- Add arbitrary-plane profiles, holes, curved surfaces and NURBS surfaces.
- Implement REVOLVE, SWEEP, LOFT, robust Boolean union/subtract/intersect, shell, 3D fillet/chamfer and healing.
- Add centre of mass, inertia tensor, materials/mass properties, STEP/IGES and 3D DWG/ACIS validation.

## Previous 2D geometry work remains open

The curved OFFSET primitive-splitting foundation remains partial. The half-edge planar arrangement, curved loop reconstruction, open-path crossing resolution and complete line/arc/polyline FILLET matrix are still required. Do not mark those features complete based on primitive splitting alone.

## Revalidation evidence

- `tools/test-critical-cad-gaps.py`: 29/29 source-contract assertions passed.
- `tools/test-ar-cad.sh`: 197/197 source-level assertions passed; shell syntax and Python compile checks passed.
- Rust tests: not executed because Cargo/rustc are unavailable.
- Android APK build and device tests: not executed.

These source-level checks do not establish compilation, geometry runtime correctness, DWG interoperability or production readiness.

## Release decision

**Production release remains blocked.** Keep APK release deferred until Rust-enabled CI executes all new and existing tests, the DWG preservation strategy is verified against real files, the 3D kernel is integrated and tested, and Android device acceptance passes.
