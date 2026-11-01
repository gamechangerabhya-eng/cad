# AR CAD — Autodesk, ArtCraft Android and CADCraft functionality review

**Review date:** 2026-11-01  
**Active project:** `AR-CAD`  
**Target:** AutoCAD 2027 desktop reference plus AutoCAD mobile/web workflows  
**Source trees reviewed:** ArtCraft Android builder/runtime, CADCraft application/engine embedded under `source/cadcraft`, AR CAD Android integration and local geometry/file-format changes.

## Executive result

**AR CAD is not yet functionally equivalent to AutoCAD Mobile or AutoCAD desktop.** It has a substantial CADCraft engine and an ArtCraft Android host layer, but this integration does not itself close CADCraft's known feature gaps. The active editor is CADCraft; ArtCraft is the Android runtime/build integration. The earlier Compose/JNI AR CAD implementation under `legacy/AR-CAD-0.6.1` is not the active document/editor path.

The local builder version is `0.4.1`, matching the public ArtCraft Android v0.4.1 release label. The embedded CADCraft source declares `0.15.1` and the import manifest has a local `v0.15.1` label. This checkout has no `.git` metadata, so neither the exact imported CADCraft commit nor exact source parity with upstream `main` can be proven from this archive. Do not automatically replace the CADCraft source with a newer upstream snapshot until the ArtCraft generator compatibility and all local AR CAD patches have been diffed and reconciled.

This review also found an immediate mobile discoverability gap: the Android quick toolbar exposed file, basic draw and edit actions, but not direct annotation, measurement, layer, property or block access. Those shortcuts are now added in source (details below).

## Repository responsibilities and comparison

| Reference | Responsibility | What it contributes to AR CAD | Finding |
|---|---|---|---|
| ArtCraft Android (`MostafaXS1/artcraft-android`) | Android host/runtime and APK build pipeline | GameActivity/egui/wgpu/winit host, SAF/open-with/save-back, touch/mouse/trackpad/keyboard/IME/clipboard, Android preferences, app wrapper and build scripts | Correct platform layer; it is not a CAD geometry engine. Source wiring is present, but APK/device operation is not validated in this environment. |
| CADCraft (`storytold/cadcraft`) | CAD document model, rendering, command engine, geometry, file formats and desktop UI | Active drawing/editor functionality inherited by AR CAD, plus local AR CAD-specific source changes | Broad 2D feature coverage; documented gaps remain in DWG fidelity, precision, blocks/xrefs, plotting, annotation and 3D. |
| AR CAD integration | Product identity, Android defaults/toolbar, local enhancements and release gate | `com.arcad.app`, AR CAD branding, Android-first layout, toolbar, local geometry/B-rep groundwork, conservative DWG export refusal | A meaningful custom product layer, but not a separate complete CAD engine yet. |

### Autodesk baseline used (AutoCAD 2027)

- **AutoCAD mobile:** 2D drawing creation/editing, offline work with later sync, layers, blocks, annotation/markup, measurements, coordinates/properties and access to drawings in local/cloud storage. The mobile app is not the same feature surface as desktop.
- **AutoCAD web:** command line, Trace, cloud open/share, layers, dimensions, xrefs, object grips, editable blocks, MTEXT and sheet set workflows.
- **AutoCAD desktop 2027:** broader 2D/3D modelling, solid/surface/mesh workflows, DWG Compare, Markup Import/Assist, Smart Blocks, AutoLISP and Autodesk Assistant. AutoCAD 2027 additionally highlights Forma Data Management check-out/collaboration, guided Geometry Cleanup, Connected References and Connected Support Files. Those are desktop/connected-project capabilities; they should not be represented as existing mobile features.

The full desktop feature set is a longer-term reference; mobile parity must be assessed against mobile workflows rather than expecting every desktop-only tool to be on a phone toolbar. Autodesk's 2027 Geometry Cleanup is not equivalent to CADCraft's AUDIT: AUDIT repairs drawing database/reference integrity, whereas Geometry Cleanup detects geometric conditions such as gaps, overshoots, undershoots and misaligned angles.

## Complete feature/functionality status table

**Status meanings:** `SOURCE PRESENT` = code exists in the active project path, runtime still needs verification; `PARTIAL` = a meaningful subset works or is implemented but important gaps remain; `MISSING` = no equivalent is implemented; `BLOCKED VALIDATION` = implementation exists but cannot be compiled or exercised in this environment.

| # | Feature area / operations | Autodesk reference capability | ArtCraft Android contribution | CADCraft / AR CAD implementation found | Status and remaining gap |
|---:|---|---|---|---|---|
| 1 | Android runtime and package | Native mobile app, touch-first interaction, Android file/system integration | Android launcher generation, GameActivity, graphics/input bridges, SAF, IME, system clipboard, preferences and update plumbing | AR CAD package `com.arcad.app`; active CADCraft launcher path; Android-specific chrome and defaults | **SOURCE PRESENT / BLOCKED VALIDATION** — no Cargo/Gradle/ADB toolchain or device test here. |
| 2 | Start, new/open/save | Start page, local/cloud DWG open, save/share | SAF file selection and document-provider access | Touch start page, recent files, New/Open/Save/Save As and save backup/recovery paths | **PARTIAL** — source actions exist; provider/device workflows and lifecycle must be tested on device. |
| 3 | Offline editing and sync | Mobile offline drawing and later sync | ArtCraft v0.4.1 describes two-way SAF auto-sync, autosave/recovery and session restoration | CADCraft autosave/recovery and document persistence are present in source | **PARTIAL** — no cloud conflict resolution/version history or real provider sync test is evidenced; no device test. |
| 4 | 2D draw | LINE, PLINE, CIRCLE, ARC, RECTANG, ELLIPSE, POLYGON, SPLINE, text and other primitives | Host input/rendering only | Common primitives plus TEXT, MTEXT, HATCH and dimensions in engine; mobile toolbar now exposes core primitives and added annotation shortcuts | **PARTIAL** — advanced prompt options and geometry edge cases still require completion and runtime regression. |
| 5 | Modify/edit | MOVE, COPY, ROTATE, SCALE, MIRROR, OFFSET, TRIM, EXTEND, FILLET, CHAMFER, STRETCH, BREAK, JOIN, arrays | Touch/pointer and keyboard input | Broad modify command set with local precision/tolerance work | **PARTIAL** — mixed curve topology, complex OFFSET arrangements, complete FILLET combinations and advanced command options remain open. |
| 6 | Precision drafting | Object snaps, tracking, coordinate/dynamic input, ortho/polar, grips | Touch/mouse/keyboard event handling | Endpoint/midpoint/center/intersection/tangent/perpendicular snaps, polar/ortho/grid, Dynamic Input and hot grips | **PARTIAL** — extension/parallel tracking, some snap workflows, multifunctional/multi-grips and difficult curve intersections are not full AutoCAD parity. |
| 7 | Measurements and inquiry | Distance, angle, area, radius, coordinates, entity properties | Touch host | DIST, AREA, LIST and property inspector exist; mobile quick access includes Distance, Area and List | **SOURCE PRESENT / PARTIAL** — direct mobile shortcuts added; all measurement modes/selection edge cases and device UX still need testing. |
| 8 | Layers and properties | Create/rename/lock/visibility/plot state, inspect and edit object properties | Screen/input integration | Layer palette and Layer Properties Manager, layer commands, Properties palette; mobile shortcut now opens Layers and Properties | **SOURCE PRESENT / PARTIAL** — property coverage and mobile panel usability need device validation; not every advanced layer-state/VP workflow matches AutoCAD. |
| 9 | Dimensions and annotation | Linear/aligned/angular/radial dimensions, leaders, markup | Touch host | DIM command family, associative dimensions, MLEADER and TABLE; mobile toolbar now exposes DIMLINEAR | **PARTIAL** — no full DIMSTYLE manager, smart dimensions and some annotative/leader/table editing details remain. |
| 10 | Text and fonts | TEXT/MTEXT, font fidelity, SHX support, fields | Android system-font bridge | TTF/OTF/TTC support and MTEXT editor/formatting | **PARTIAL** — SHX/big fonts not parsed; in-place edit and fields/spell-check/complex Indic shaping have gaps. |
| 11 | Hatches and gradients | Hatch patterns, islands, boundary editing, gradients | Host integration | HATCH, associative boundary support, pattern fills, gradients and HATCHEDIT | **PARTIAL** — pattern breadth and advanced hatch editing/associativity limitations remain. |
| 12 | Blocks and attributes | Insert/edit blocks, attributes, dynamic blocks, block libraries | SAF/file loading; no CAD block engine | Static block editor, insert/attributes/nested blocks; some dynamic-block data and reference grips/visibility editing | **PARTIAL** — dynamic-block authoring/parameters/actions and some editing/sync workflows remain incomplete. |
| 13 | Xrefs | Attach, reload, unload, bind, clip, edit/reference compare | File access and path resolution host | XATTACH, XREF palette and common path/load operations; REFEDIT for block refs | **PARTIAL** — XCLIP and xref editing in place are missing; some unloaded state/overrides only last for the session. |
| 14 | Layouts, viewports and plotting | Paper space, viewports, plot styles, plotters, multi-sheet publishing | Android print/platform services in supported workflows | Layouts, viewports, page setup, Plot dialog/preview, PDF export, CTB/STB-like internal configs | **PARTIAL** — binary CTB/STB/PC3/PMP interchange and system-printer coverage remain incomplete. |
| 15 | DWG/DXF file integrity | DWG open/save and reliable exchange with AutoCAD | SAF open/save route | Native DXF reader/writer; raw unknown DXF content preservation; DWG via `acadrust` bridge; I/O preflight now refuses known lossy DWG exports and incomplete source reads | **CRITICAL PARTIAL** — safer fail-closed behaviour, not a lossless DWG codec. No independent real-file corpus or AutoCAD/ODA round-trip evidence. Point 3 remains open. |
| 16 | 3D modelling and navigation | Solid, surface and mesh creation/editing, booleans, 3D orbit and visual styles | Android host input/rendering | B-rep topology foundation, prism extrusion operation, faceted 3DFACE output and grouped-face inspection/deletion | **MAJOR GAP** — no persistent editable analytic solid in the document model, no booleans, shell, 3D fillet/chamfer, healing, mass/centroid/inertia, STEP/IGES or full interactive 3D viewport. |
| 17 | Drawing validation and geometry cleanup | AutoCAD 2027 guided Geometry Cleanup for gaps, overshoots, undershoots and misaligned angles; AUDIT/RECOVER | File/open/recovery host only | AUDIT/RECOVER and drawing integrity checks exist; mobile toolbar now exposes Audit | **PARTIAL** — the new mobile Audit shortcut is implemented, but no guided geometry-cleanup detector/repair workflow yet; AUDIT is not a substitute for Geometry Cleanup. |
| 18 | Compare, revision and audit history | DWG Compare, activity insights, cloud checkout/version history | Platform storage only | Compare commands and local drawing recovery source paths exist | **PARTIAL** — no Autodesk-style collaborative activity feed, Forma project checkout or verified external DWG compare workflow. |
| 19 | Trace, markup and collaboration | Trace overlays, mobile sketch/markup, shared views, multi-user review | File sharing/open-with only | No complete Trace/Markup Assist or multi-user review subsystem identified | **MISSING / PARTIAL** — local notes/markup workflow and controlled collaboration need implementation. |
| 20 | Automation and extensions | AutoLISP/Visual LISP, APIs, plug-ins, macros | Android runtime only | AutoLISP core, SCRIPT-like execution through CLI/MCP/control channel, CLI and MCP are present | **PARTIAL / DIFFERENTIATOR** — MCP/agent control is valuable; Visual LISP ActiveX, DCL, reactors and broad native plugin compatibility are absent. |
| 21 | AI assistance and Smart Blocks | AutoCAD 2027 Autodesk Assistant with drawing-context/MCP workflows, Smart Blocks, Markup Assist | Android host only | CADCraft MCP can be driven by external agents; no complete in-app drawing-aware assistant, Smart Block inference or markup recognition identified | **MISSING / DIFFERENTIATOR** — external MCP control is not an in-app assistant or Smart Blocks equivalent. |
| 22 | Languages and accessibility | Mobile UI/text input accessibility and multilingual support | Android IME/system fonts/accessibility plumbing | Multiple UI language catalogs, larger touch targets and recorded widget labels | **PARTIAL** — Devanagari/Indic drawing-text shaping and independent screen-reader/mobile accessibility testing remain open. |
| 23 | Reliability, performance and release quality | Large DWG responsiveness, stable open/save, verified supported workflows | ArtCraft build/update/recovery framework | Source QA, shell/Python checks, engine tests present in repository | **BLOCKED VALIDATION** — no Rust compile/tests, Android build, device test, performance corpus or independent format validator executed in this environment. |

## AR CAD source updates started in this review

### Mobile drafting toolbar enhancement

Updated `source/cadcraft/crates/ui-egui/src/chrome.rs`:

- Expanded the Android drawing toolbar from three to four independently horizontally scrollable rows.
- Added `Text`, `MText`, `Hatch` and `Dimension` to the mobile Draw row.
- Added a Tools row with `Distance`, `Area`, `List`, `Audit`, `Layers`, `Properties` and `Blocks`.
- Defined the inspection action array explicitly after source review found a dangling `ANDROID_INSPECT_TOOLS` reference; this removes a real unresolved-name compile risk that the earlier text-search QA missed.
- Raised the mobile drawing-toolbar reserved height from 134 to 176 points so all four rows have usable touch targets.
- Commands route through the existing command/UI dispatcher; this is not a second, disconnected tool implementation.

Updated `docs/test-step3.py` to verify the inspection array declaration itself, command registration (including `AUDIT`), UI action routing, four toolbar rows and core touch targets.

**Current source-level check:** the updated Step 3 test reports 37/37 checks. The Rust unit tests in `chrome.rs` were updated as well, but cannot be executed here without Cargo.

## Prioritised gap/failure register

| Priority | Gap / failure | Current mitigation | Work required to close | Status |
|---|---|---|---|---|
| P0 | DWG bridge may discard unmodelled content | I/O-layer blocker detection and fail-closed refusal | Real DWG corpus, AutoCAD/ODA black-box open/save/reopen, fix bridge or keep affected saves blocked | **OPEN — CRITICAL** |
| P0 | No Rust/Android executable validation in current environment | Source acceptance tests and static syntax checks | Rust 1.95+, Cargo, Android SDK/NDK/Gradle, CI and ARM64 device tests | **BLOCKED** |
| P1 | No full B-rep solid workflow | Topology foundation and faceted EXTRUDE grouping | Persistent solid entity, robust topology, Boolean ops, editing tools and 3D view | **OPEN** |
| P1 | Xrefs/dynamic blocks incomplete | Core xref/static block and partial dynamic block support | XCLIP, xref in-place edit, authoring, references/grips/sync | **OPEN** |
| P1 | Complex 2D geometry operations not complete | Prior exact-predicate/tolerance/curve-offset increments | Mixed arc/line arrangement topology, comprehensive FILLET and spline/ellipse guarantees | **OPEN** |
| P1 | Android tool discoverability for measurement/annotation was weak | Four-row mobile toolbar with measurement, annotate and palette shortcuts | Test on phone/tablet; refine layout and add gesture-specific quick properties | **SOURCE UPDATE DONE; DEVICE TEST OPEN** |
| P2 | Trace/Markup/AI/collaboration missing | No equivalent subsystem | Implement original AR CAD trace overlays, markup import, local assistant/Smart Blocks | **OPEN** |
| P2 | Binary plotter config/exchange and specialist formats missing | PDF output and internal plotter/style formats | Import/convert official plotter/plot-style formats where documented/licensed | **OPEN** |

## Validation and limitations

- `./tools/test-ar-cad.sh`: rerun after the mobile toolbar change; report exact totals from the current output.
- Rust compilation/runtime tests: not executed because `cargo`, `rustc` and `rustfmt` are not installed.
- Android APK: not built, by instruction.
- Physical Android device tests: not performed.
- AutoCAD/ODA interoperability: not performed; no licensed sample corpus is present.
- Repository provenance: the archive has no `.git` directory. The imported CADCraft version is recorded as local tag `v0.15.1`, not a proven upstream commit SHA.

## Public references

- Autodesk AutoCAD 2027 product details and highlights: https://www.autodesk.com/products/autocad/product-details
- Autodesk AutoCAD 2027 What's New: https://help.autodesk.com/cloudhelp/2027/ENU/AutoCAD-WhatsNew/files/GUID-D52A51CA-BDA1-4A78-9E67-87B340C55490.htm
- Autodesk AutoCAD 2026 features (broader product context): https://www.autodesk.com/in/products/autocad/features
- Autodesk AutoCAD Web features: https://www.autodesk.com/in/products/autocad-web/features
- Autodesk AutoCAD mobile app listing: https://apps.apple.com/in/app/autocad/id393149734
- ArtCraft Android repository: https://github.com/MostafaXS1/artcraft-android
- ArtCraft Android architecture: https://github.com/MostafaXS1/artcraft-android/blob/main/architecture.md
- ArtCraft Android v0.4.1 release: https://github.com/MostafaXS1/artcraft-android/releases/tag/v0.4.1
- CADCraft repository: https://github.com/storytold/cadcraft
- CADCraft roadmap: https://github.com/storytold/cadcraft/blob/main/ROADMAP.md
- CADCraft documented gaps (AutoCAD 2027 target): https://github.com/storytold/cadcraft/blob/main/docs/gaps.md
- CADCraft file-format parity: https://github.com/storytold/cadcraft/blob/main/docs/file-format-parity.md


## Review follow-up — 2026-11-01: Android action-array defect and Autodesk 2027 QA gap

A second source inspection found that `android_drafting_toolbar` referenced `ANDROID_INSPECT_TOOLS` but the constant had not been declared in the Rust file. The previous source-presence checks searched for the reference and therefore failed to detect the unresolved name. Added the actual constant with Distance, Area, List, Audit, Layers, Properties and Blocks actions. Added a source check requiring the constant declaration and each exact tuple, and included `audit` in the registered engine-command check. Step 3 source checks now report 37/37. This still needs Rust compilation.

The Autodesk 2027 comparison now separately records guided Geometry Cleanup and connected project features. CADCraft's AUDIT command is not represented as a Geometry Cleanup equivalent. AR CAD exposes Audit as a mobile shortcut, while geometric detection/repair for gaps, overshoots, undershoots and misaligned angles remains open. Autodesk Assistant/MCP drawing-context workflows, Forma Data Management Checkout, Connected References and Connected Support Files are also added to the comparison.
