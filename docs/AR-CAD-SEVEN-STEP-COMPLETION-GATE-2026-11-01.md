# AR CAD — Seven-step completion and release gate

Revision: 2026-11-01 (R4 follow-up)

This is an evidence-based tracker, not a declaration that the product is complete. The seven stages below correspond to the agreed engineering order: compile/test the Rust engine, close 2D geometry, complete 3D solids, finish references/blocks, validate Android runtime, certify file interoperability, and build/sign/deliver the APK.

| Step | Acceptance criteria | Current evidence | Status |
|---|---|---|---|
| 1. Rust workspace compile and tests | `cargo check --locked --workspace --all-targets`; targeted Rust tests; no compiler/test failures | CI now runs both commands with Rust 1.95.0. Local environment has no `cargo` or `rustc`; CI has not been triggered here | **Configured; execution pending** |
| 2. 2D geometry completion | Complete OFFSET topology for open/closed line/arc/polyline paths; complete line/arc/polyline FILLET; certified spline/ellipse intersections; Geometry Cleanup detections and safe preview/undo | Robust orientation predicates, shared tolerance migration, partial multi-loop OFFSET, spline/ellipse solver improvements, FILLET work, OVERKILL seam/arc improvements. No complete arrangement engine or full repair UI | **Partial** |
| 3. Persistent editable 3D solids | Native document entity, persistent save/load, grips/properties/transforms, arbitrary profile planes/holes/curved surfaces, booleans, fillet/chamfer, healing and properties | B-rep topology foundation and validated polygon extrusion; emitted geometry is grouped 3DFACE shell, with metadata checks and SOLIDINFO/delete | **Partial; not native editable solid** |
| 4. Xrefs and dynamic blocks | XCLIP, in-place Xref editing, durable overrides, dynamic block authoring and parameter/action evaluation | Attach/load/bind/unload/reload and dynamic-block read/inspection/grip/value capabilities exist; BEDIT explicitly does not author/edit dynamic parameters/actions | **Partial** |
| 5. Android runtime validation | Build/install on supported Android versions; verify touch, keyboard/IME, SAF, lifecycle, large drawings, orientation and memory | Android host/build scripts and toolbar source are present. No local Android SDK/Gradle/ADB; no device execution evidence | **Configured; execution pending** |
| 6. DWG/DXF interoperability | Versioned fixture corpus, read/write/round-trip checks, XDATA/reactors/custom classes/ACIS/layouts, independent AutoCAD and ODA comparison | Fail-closed export checks for known loss and partial reads; one sample DXF. No comprehensive corpus or independent AutoCAD/ODA validation | **Partial; not certified** |
| 7. APK build, signing and delivery | Reproducible debug and release APK; correct package/label; signature verification; SHA-256; install/launch smoke test; deliver artifact | GitHub Actions workflow builds APK, verifies package/zip, signs release when repository secrets exist, and uploads artifact. It now pins Rust 1.95.0 and checks the full workspace first. Workflow not run from this environment; no APK artifact exists | **Configured; execution pending** |

## Latest locally executed checks

`bash tools/test-ar-cad.sh` completed with exit code 0:

- Step 2 source checks: 18/18
- Step 3 source checks: 37/37
- Steps 4–6 source checks: 14/14
- Steps 7–10 source checks: 17/17
- Geometry priority: 14/14
- Tolerance migration: 20/20
- Precision safety: 11/11
- P1 checks: 49/49
- Critical-gap checks: 40/40
- **Total: 220/220 source assertions**
- Shell syntax and Python compileall: passed
- Rust tests: skipped by default; explicit run fails because Cargo is absent

These checks inspect source and scripts. They do not establish runtime correctness.

## Environment blockers

This execution environment has no `cargo`, `rustc`, `rustfmt`, `gradle`, `adb`, Android SDK, network DNS access, or external AutoCAD/ODA installation. The project cannot be compiled, APK-built, installed, signed with a user's release key, or interoperability-certified from this environment. CI is prepared to do the compilation and APK stages on GitHub Actions, but it has not been dispatched and no CI result can be asserted.

## Release decision

Do not label AR CAD production-ready, 95% complete, AutoCAD-equivalent, or DWG/DXF lossless-certified until all seven stages meet their acceptance criteria and evidence is attached. In particular, successful source assertions are not substitutes for compiler output, device tests, file round trips, or a signed APK artifact.
