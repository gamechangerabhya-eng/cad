# AR CAD — DWG/DXF and 3D next increment

Date: 2026-11-01

## Workstream 1 — DWG/DXF integrity

- Confirmed the active DWG bridge is isolated behind `cadcraft-dwg` and converts through DXF using the unmodified `acadrust` dependency.
- The current wrapper does not expose a general raw-object passthrough or a lossless custom-class metadata channel. Replacing or patching the bridge safely requires changes in the upstream codec and real DWG corpus tests; this increment does not pretend to solve that.
- Existing user-facing QSAVE protection remains in place: the UI holds a lossy save, enumerates unsupported entity/metadata categories, and offers Save Anyway / Save As / Cancel. Cancel preserves the source file. The warning now includes metadata categories.
- Remaining critical gaps: save-as warning coverage for all user workflows, actual raw custom-class/object/XDATA/reactor/colour-book preservation, and independent AutoCAD/ODA round-trip verification.

## Workstream 2 — first document-integrated 3D operation

- Added `EXTRUDE` to the Draw command registry.
- Inputs: `{handle, height}` for a closed straight-segment lightweight polyline, or `{points, height}` for an explicit simple polygon.
- Builds and validates a B-rep prism using the geometry crate.
- Emits native document `3DFACE` entities for the outer shell; concave caps are ear-clipped into triangles.
- Reports volume and surface area from the validated B-rep.
- The resulting document representation is a faceted 3DFACE shell, not a persistent editable B-rep solid. It is not yet a parametric or boolean-capable solid modeler.

## Validation status

- 17/17 critical-gap source-contract checks pass for command registration, profile validation, B-rep validation, native 3DFACE emission, concave cap triangulation and closed-polyline handle integration.
- Rust compilation and tests remain unverified in this environment because Cargo/Rust are unavailable. Source-contract checks are not runtime proof.
- No APK or device verification was performed.

## Point 3 — DWG/DXF reliability

**PENDING — CRITICAL.** The current increment improves loss awareness and save safety, not lossless bridge compatibility.
