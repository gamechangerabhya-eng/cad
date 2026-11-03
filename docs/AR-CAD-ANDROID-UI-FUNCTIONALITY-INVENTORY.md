# AR CAD Android UI Functionality Inventory

Status: architecture inventory completed before Android UI restructuring; builder-owned
Android workspace implemented in `templates/android/cadcraft_mobile_ui.rs`  
Application baseline: CADCraft 0.15.1  
Scope: builder-owned Android UI and integration only; the CAD engine, document model,
geometry, and file formats remain unchanged. The UI template is installed by a guarded builder
patch during Android builds; no upstream checkout changes are committed.

## Architecture and preservation boundaries

| Area | Existing implementation | Mobile UI boundary |
|---|---|---|
| CAD engine | `cadcraft_engine::Session`; command implementations under `crates/engine/src/cmd/` | UI actions call the existing `CadApp::start`, `CadApp::run`, or `CadApp::cmdline` paths. |
| Document model | `cadcraft_doc` state and entities owned by `Session` | Read through `CadApp::session`; no document or geometry state is maintained in controls. |
| Rendering | `cadcraft-render` and the egui UI crate's canvas, including its GPU canvas path | Reuse the existing canvas and docking renderer. |
| Command system | `cadcraft_engine::cmd::command_specs()` and `Session` command execution/prompt state | Command actions dispatch by existing command ID; interactive commands retain their existing prompt sequence. |
| Tool registration | Engine `CommandSpec` metadata plus `crates/ui-egui/src/palettes.rs` Tool Sets | Keep existing command metadata and make commands discoverable in mobile categories and command search. |
| Menu registration | `crates/ui-egui/src/menus.rs`, including UI commands and menu entries | Existing menu commands remain dispatchable; mobile panels become additional entry points. |
| Undo / redo | Engine undo history, `u` / `undo` and `redo` commands | Top-bar buttons call those commands; no separate history implementation. |
| Selection | `Session::selection()`, selection tools, canvas hit testing, selection context menu | Existing selection and command eligibility remain authoritative. |
| Snapping | Canvas input and engine snap state/settings | Reuse canvas snap calculations and expose existing drafting controls. |
| Constraints | Existing engine constraint commands and document constraint model | Surface existing commands via command search/categories; do not calculate constraints in UI code. |
| Drawing / modify | Existing registered engine commands and interactive prompts | Mobile buttons start the registered commands; touch points continue through existing canvas prompt handling. |
| Open / save | UI `Services`, file commands and Android launcher file service | Use existing `open`, `qsave`, and `saveas` command flows. |
| Import / export | Existing exchange/file command registrations | Expose existing commands in Document and command search. |
| Keyboard shortcuts | Engine `CommandSpec.shortcut`, menu shortcuts, command-line aliases | Keep keyboard events and shortcuts intact; show metadata where space permits. |
| Mouse / canvas input | Existing canvas pointer handling, view navigation, hit tests and command prompts | Keep renderer/input path; Android touch-up and two-finger pan fixes remain in the builder patch. |
| Android touch | `lib/scrolling.sh` guarded CADCraft patches plus egui/winit Android events | Preserve tap, drag, point placement, pan, pinch/zoom and canvas focus/IME behavior. |
| Android launcher | Shared generator in `lib/build-common.sh`; CADCraft currently uses the generated eframe launcher | Keep the launcher, app services, SAF integration, and eframe/egui wiring. |

## Feature-to-entry-point inventory

The command IDs and parameter/enablement metadata are registered by the engine, not by the
mobile controls. An empty `CommandSpec.params` means the command uses its normal interactive
prompt or has no parameter object; non-empty metadata is the engine's declared parameter
contract. Selection requirements and disabled reasons are determined by each command's existing
`enabled` function.

| Feature | Current UI entry point | Existing command/API | Shortcut | Selection requirement | Parameters | Current Android accessibility | New mobile UI entry point | Preserved / validation |
|---|---|---|---|---|---|---|---|---|
| Select entities, window/crossing selection, selection tools | Canvas, Tool Sets, Select menu, command line | Registered selection commands; `Session::selection()` | Per command metadata | Varies by selection mode | Per command metadata | Canvas touch and desktop-style Tool Sets | Select category, contextual actions, command search | Must remain engine-dispatched; test selection and clear |
| Clear selection / select all | Edit menu, command line | `ai_deselect`, `selectall`, selection commands | `Delete` / `Cmd+A` where registered | Clear requires selection; select-all needs a drawing | `{}` or command defaults | Accessible through existing menus/command line and some toolbar buttons | Select category and canvas context actions | Verify selected entities and selection reset |
| Draw primitives and curves | Tool Sets, Draw menus, Android drafting toolbar, command line | Registered draw commands (line, polyline, circle, rectangle, arc, ellipse, spline, polygon, etc.) | Per command metadata/aliases | Usually none; active prompt owns point collection | Per command metadata | Touch toolbar plus canvas prompt | Draw category sheet and searchable command palette | Verify commands start the same engine prompts |
| Text, hatch, fill and other drawing entities | Tool Sets, menus, dialogs, command line | Registered text, hatch, fill and entity commands | Per command metadata | Varies | Per command metadata | Touch toolbar has common text/hatch actions; advanced options remain menu/command-line based | Draw category and command search | Preserve dialog and prompt paths |
| Move, copy, rotate, scale, delete | Tool Sets, Modify/Edit menus, Android drafting toolbar, context menu | Registered edit/modify commands | Per command metadata; delete shortcut where registered | Commonly selection-dependent, enforced by engine | Per command metadata | Common actions on Android toolbar; advanced parameters via prompts | Modify category and selection context toolbar | Verify commands use existing dispatcher and selection |
| Trim, extend, offset, fillet, array and advanced modify | Tool Sets, Modify menus, command line | Registered modify/array commands | Per command metadata/aliases | Varies by command | Per command metadata | Existing Tool Sets/menus/command line; not all are on the phone toolbar | Expandable Modify groups and command search | Verify all registered commands remain discoverable |
| Grip and control-point editing | Canvas grips, Tool Sets, command line | Existing grip/control-point commands and canvas interaction | Per command metadata | Selected entity or grip, enforced by existing tools | Per command metadata | Touch canvas; grip hit targets and Android pointer handling | Canvas contextual actions; command search for advanced edits | Preserve grip editing; device-level hit-target validation pending |
| Dimensions, leaders, annotations and text edit | Annotate/Dimension menus, Tool Sets, command line | Registered dimension, leader, annotative and text-edit commands | Per command metadata | Varies | Per command metadata | Common dimension action on Android toolbar; advanced actions through existing UI | Measure/Annotate category and command search | Verify interactive prompts and edit dialogs |
| Measurement and inquiry | Inquiry menu, Tool Sets, command line | `dist`, `area`, `list`, audit and inquiry commands | Per command metadata | Varies | Per command metadata | A few inspection actions on Android toolbar | Measure/Annotate category | Preserve existing result reporting |
| Layers and properties | Layers/Properties palettes, menus, command line | UI dialog/toggle commands and layer/property commands | Menu/command metadata | Property edits may require selection | Per command metadata | Palettes can be toggled; Android uses a compact toolset rail | Expandable Layers/Properties sheet and contextual Properties action | Verify existing palette and property editors open |
| Snaps, grid, drafting modes and display toggles | Status bar, Options, command line | Existing UI toggles and engine settings | Menu/command metadata | None | Toggle/default parameters | Existing enlarged Android status controls | View category and command guidance area | Preserve setting effects; no new snapping math |
| Zoom, pan, fit, orbit and view navigation | Canvas gestures, status bar, View menu, toolbar | Existing view commands and `Session` view operations | Per command metadata | None | Command parameters/defaults | Pinch/two-finger pan and enlarged zoom controls | Floating view controls and View category | Verify touch gestures and view commands |
| New / open / recent / save / save as | File menu, toolbar, Start page, command line | `new`, `open`, `qsave`, `saveas`, recent-file UI | `Cmd+N`, `Cmd+O`, `Cmd+S` where registered | None | Existing file-service prompts | Existing Android toolbar and SAF-backed launcher services | Top bar and Document sheet | Verify SAF picker, open, save and save-as |
| Import, export, exchange and share | File/Insert menus, toolbar, command line | Existing exchange/import/export commands | Per command metadata | Varies | Per command metadata | Some actions on existing toolbar; other actions via menus/command line | Document sheet and command search | Verify actual command availability; no placeholder controls |
| Blocks, references, layouts, plotting and other advanced features | Menus, palettes, dialogs, Tool Sets, command line | Existing UI and engine command registrations | Per command metadata | Varies | Per command metadata | Most available through desktop-oriented menus/palettes/command line | Searchable command palette and expandable category groups | Verify all engine command specs can be found |
| Command line, aliases, prompt keywords and history | Command line overlay/panel | `CadApp::cmdline`, `CadApp::start`, `Session` prompt state | Keyboard Enter/Tab and registered shortcuts | Depends on current command | Existing engine prompt contract | Touch command line and Android IME bridge | Persistent compact command guidance/input affordance | Verify Enter, keyword choices, cancel/back and IME |
| Undo / redo | Toolbar, Edit menu, command line | `u` / `undo`, `redo` | `Cmd+Z`, `Cmd+Shift+Z` | None; history decides whether an operation exists | Existing defaults | Existing toolbar buttons | Top app bar | Verify history remains engine-owned |
| Contextual entity actions | Canvas context menu and menus | Existing context/menu commands | Per command metadata | Selection-dependent where registered | Per command metadata | Long-press/context menu support depends on canvas input | Touch-sized contextual action strip/sheet | Verify every action dispatches through existing command paths |
| Document recovery, close/save-loss prompts and dialogs | Existing modal/dialog flows | `CadApp` lifecycle, autosave, recovery and dialogs | N/A | N/A | Existing dialog-specific inputs | Existing Android dialogs | Preserve as-is above the mobile workspace | Verify dialogs remain reachable and dismiss correctly |

## Android design system and responsive layout

The mobile workspace reuses the existing centralized `theme::Tokens` palette (including the
dark-first canvas/chrome, light theme, accent, selection, snap, warning, text, and field colors)
instead of introducing per-control color literals. Shared sizing constants define the 56-point
app bar, 64-point phone navigation row, 48-point minimum touch targets, 360-point expandable
command sheet cap, and 760-point wide-layout breakpoint.

- Phone-sized layouts use a compact app/document bar, scrollable category navigation at the
  bottom, a dismissible command sheet, and floating zoom/fit controls.
- Wide layouts use a persistent category rail and preserve the same command sheet and canvas.
- A selected-entity action strip uses live command enablement metadata and remains horizontally
  scrollable on narrow screens.
- Command search queries every engine and UI command, including aliases, menu paths, and
  shortcuts. Commands with engine-disabled states show the engine's reason. Commands requiring
  non-interactive arguments remain discoverable but direct users to the existing command-line
  JSON entry rather than issuing an invalid request.
- Document selection, close requests, undo/redo, save, drafting toggles, view changes, dialogs,
  panel rendering, and canvas drawing use existing app/engine APIs.
- Existing egui canvas touch-up, point-prompt, pan, pinch/zoom, snap, grip, command-line, and IME
  pathways remain in place.

## Inventory coverage and validation checklist

- [x] Engine, document, rendering, command, menu, selection, snapping, constraint, file,
  shortcut, input, Android launcher, and egui integration boundaries identified.
- [x] Every feature family has a current entry point, existing API/command, shortcut/parameter
  source, mobile entry-point mapping, and validation status.
- [x] The mobile command browser reads the live engine/UI command registry instead of replacing it
  with a hard-coded list.
- [ ] Compile/test the Android UI template and generated source with the project Rust toolchain.
- [ ] Validate file operations and Android touch interactions on a device/emulator.
- [ ] Run the narrowest available static/build checks for changed builder-owned files.

The inventory intentionally records command metadata as the authoritative source instead of
duplicating the engine's expanding command list into Android UI code. A mobile command browser
must query that registry so advanced commands are not silently omitted.
