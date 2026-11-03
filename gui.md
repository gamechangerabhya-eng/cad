Act as a senior Android UX engineer, Rust/egui developer, CAD product designer, touch-input specialist, QA engineer, Git specialist, and GitHub Actions build engineer.



==================================================
PRIMARY OBJECTIVE
==================================================

Transform the current application into a polished, responsive, touch-first Android CAD application.

The app has already been ported and its GUI has been changed, but it still has serious usability and functional problems:

- The interface still contains Windows/desktop-style elements.
- A permanent desktop-style command line is still present.
- Touch input is unreliable.
- Drawing a basic line using touch does not work correctly.
- Some buttons and panels extend outside the device screen.
- The layout does not adapt correctly to different Android devices.
- Portrait and landscape layouts need improvement.
- Phone and tablet layouts need separate responsive behavior.
- Some controls are too small for fingers.
- Some commands still depend on keyboard input.
- Startup and splash-screen quality need improvement.
- The application needs a completely fresh and professional Android CAD appearance.
- Existing CAD functions may not be properly connected to the redesigned GUI.

This is primarily an Android GUI, interaction, input-routing, and usability improvement task.

Preserve the existing CAD engine and functionality.

The required result is:

FULL EXISTING CAD FUNCTIONALITY
+
FRESH PROFESSIONAL ANDROID GUI
+
RELIABLE TOUCH INPUT
+
RESPONSIVE PHONE AND TABLET LAYOUTS
+
ANDROID-NATIVE COMMAND EXPERIENCE
+
POLISHED SPLASH AND STARTUP
+
KEYBOARD, MOUSE, TRACKPAD, AND STYLUS SUPPORT
+
SUCCESSFUL APK BUILD
+
GITHUB ACTIONS SUCCESS

Do not consider the task complete merely because the APK compiles.

The application must be functionally usable.

==================================================
1. NON-NEGOTIABLE FUNCTIONALITY PRESERVATION
==================================================

Do not:

- Rewrite the CAD engine
- Create a separate simplified mobile CAD engine
- Replace geometry algorithms
- Change calculation results
- Change document formats
- Change import or export behavior without a verified defect
- Change snapping calculations
- Change constraint behavior
- Remove advanced tools
- Delete commands to simplify the GUI
- Comment out functionality to make Android compile
- Disable tests
- Hide errors
- Downgrade CADCraft
- Replace existing command logic with direct UI mutations
- Remove keyboard or mouse support
- Treat build success as functional success

Preserve:

- Geometry engine
- Document model
- Rendering system
- Existing CAD commands
- Drawing tools
- Modify tools
- Selection
- Snapping
- Constraints
- Layers and properties where available
- Offset
- Fillet
- Trim and extend
- Move, copy, rotate, scale, and delete
- Import and export
- File open and save
- Autosave and recovery where supported
- Undo and redo
- Existing keyboard shortcuts
- Mouse and trackpad support

The correct architecture is:

Touch / Stylus / Mouse / Keyboard
                ↓
       Unified Android Input Layer
                ↓
       Existing CAD Command System
                ↓
         Existing Document Model
                ↓
       Existing Geometry and Renderer
                ↓
         Existing Undo/Redo System

UI controls and gesture handlers must call the existing CAD commands.

Do not directly mutate CAD geometry from touch or UI code.

==================================================
2. FIRST INSPECT THE CURRENT IMPLEMENTATION
==================================================

Before changing code, inspect:

- Repository instructions
- Current branch and Git status
- Recent commits
- Existing Android adapter
- Existing GUI redesign
- CAD command registry
- Menu and tool registration
- Canvas rendering
- Pointer and touch input
- winit event handling
- egui input handling
- Screen-to-world coordinate conversion
- Android activity and Java bridge
- Splash implementation
- File bridge
- Cargo manifests
- Cargo.lock
- Vendored dependencies
- Build scripts
- GitHub Actions workflows

Read applicable files such as:

- AGENTS.md
- README.md
- architecture documentation
- Cargo.toml files
- Cargo.lock
- AndroidManifest.xml templates
- Android activity sources
- Builder scripts
- .github/workflows files

Also inspect the reference Android implementation and record:

- Reference branch or tag
- Reference commit SHA
- Relevant files and patches
- Applicable mobile input and Android integration patterns

Do not copy the reference repository blindly.

Adapt its applicable Android implementation to the current CADCraft 0.15.1 APIs.

Do not downgrade CADCraft merely because an older reference build compiles.

==================================================
3. CREATE A FUNCTIONALITY AND UI INVENTORY
==================================================

Before removing or replacing UI components, create an inventory containing:

[ ] Feature or command
[ ] Existing desktop UI entry point
[ ] Existing keyboard shortcut
[ ] Existing command/API
[ ] Parameters required
[ ] Selection requirements
[ ] Current Android UI entry point
[ ] Current touch behavior
[ ] New Android UI entry point
[ ] New touch workflow
[ ] Exact-value input workflow
[ ] Confirm behavior
[ ] Cancel behavior
[ ] Undo/redo behavior
[ ] Validation result
[ ] Final status

Audit at minimum:

DRAW:
- Line
- Polyline
- Rectangle
- Circle
- Arc
- Ellipse
- Spline
- Text
- Dimensions where supported

MODIFY:
- Move
- Copy
- Rotate
- Scale
- Delete
- Trim
- Extend
- Offset
- Fillet
- Control-point editing

SELECTION:
- Tap selection
- Clear selection
- Multi-selection
- Selection window
- Selection at different zoom levels

VIEW:
- Pan
- Zoom
- Fit drawing
- Recenter
- Grid
- Snap controls

DOCUMENT:
- New
- Open
- Save
- Save As
- Import
- Export
- Autosave
- Recovery
- Undo
- Redo

Do not remove the old visible route to a command until the new Android route has been implemented and validated.

==================================================
4. BUILD A FRESH ORIGINAL ANDROID CAD DESIGN
==================================================

Create a fresh, original, professional AR CAD visual system.

The application may be inspired by the efficiency of professional CAD applications, but do not copy proprietary branding, icons, layouts, assets, or visual identity from AutoCAD or another commercial product.

The new design should feel:

- Professional
- Technical
- Precise
- Modern
- Clean
- Touch-friendly
- Efficient
- Suitable for long working sessions
- Consistent across screen sizes
- Familiar to CAD users
- Native to Android

Avoid:

- Windows-style menu bars
- Permanent desktop command fields
- Tiny buttons
- Hover-only menus
- Crowded horizontal toolbars
- Oversized cards
- Excessive gradients
- Excessive animations
- Multiple competing accent colors
- Large decorative headers
- Panels that permanently cover the canvas
- Desktop windows displayed inside the application

The canvas must remain the central focus.

Create centralized design tokens for:

- Canvas background
- Panel background
- App background
- Primary accent
- Selection color
- Snap color
- Warning color
- Error color
- Disabled state
- Grid color
- Measurement text
- Typography
- Spacing
- Corner radius
- Elevation
- Button sizing
- Touch-target sizing
- Icon sizing
- Compact, medium, and expanded layouts

Do not scatter hard-coded visual values throughout the project.

==================================================
5. RESPONSIVE ANDROID WORKSPACE
==================================================

Replace the remaining desktop layout with a responsive Android CAD workspace.

Recommended structure:

TOP APP BAR:
- AR CAD identity
- Current document name
- Unsaved-change indicator
- Undo
- Redo
- Save
- Overflow menu

CENTRAL CANVAS:
- Maximum available space
- Grid
- Selection states
- Snap indicators
- Drawing previews
- Measurement overlays
- Compact command guidance
- Navigation controls

PRIMARY TOOL NAVIGATION:
- Select
- Draw
- Modify
- Measure or Annotate
- View
- Document

CONTEXTUAL ACTION BAR:
- Show only actions applicable to the current selection
- Move
- Copy
- Rotate
- Scale
- Delete
- Offset
- Fillet
- Properties

TOOL SHEET:
- Opens tools from the selected category
- Scrollable
- Collapsible
- Dismissible
- Must not permanently consume canvas space

PARAMETER SHEET:
- Coordinates
- Length
- Width
- Height
- Radius
- Diameter
- Angle
- Offset distance
- Fillet radius
- Rotation
- Scale
- Other existing command parameters

VIEW CONTROLS:
- Zoom in
- Zoom out
- Fit drawing
- Recenter where applicable

Do not display every command simultaneously.

Keep all commands accessible through logical categories or command search.

==================================================
6. FIX SCREEN OVERFLOW
==================================================

Some controls currently extend beyond the visible screen.

Fix this before adding optional enhancements.

Inspect all components for:

- Hard-coded widths
- Hard-coded heights
- Fixed button counts
- Fixed coordinates
- Horizontal rows without scrolling
- Panels without maximum dimensions
- Dialogs larger than the viewport
- Long text without truncation
- Missing safe-area handling
- Incorrect Android density scaling
- Incorrect egui pixels-per-point
- Controls under the status bar
- Controls under the navigation bar
- Controls hidden by the software keyboard
- Landscape layouts using portrait assumptions
- Phone layouts using tablet dimensions

No essential control may leave the usable viewport.

If controls do not fit:

- Collapse secondary controls
- Use a scrollable tool row
- Use a tool-category sheet
- Use a More menu
- Use responsive wrapping where appropriate
- Use a searchable command palette
- Move properties into a bottom sheet

Do not shrink touch controls until they are unusable.

==================================================
7. CENTRAL RESPONSIVE LAYOUT SYSTEM
==================================================

Create one centralized responsive layout model.

Determine layout primarily using available viewport dimensions, orientation, density, safe-area insets, and keyboard state.

Support:

COMPACT:
- Small phone portrait
- Bottom tool navigation
- Full-width bottom sheets
- Minimal permanent controls
- No permanent side panels

MEDIUM:
- Large phone or small tablet
- Compact bottom or side tool navigation
- Optional collapsible properties
- Expanded command information when space allows

EXPANDED:
- Tablet landscape
- Left tool palette
- Central canvas
- Optional right properties panel
- Persistent compact command information
- Keyboard and mouse productivity support

Every major component must use this centralized layout information.

Do not scatter unrelated width checks throughout the code.

==================================================
8. SAFE AREA AND SOFTWARE KEYBOARD
==================================================

Account for:

- Android status bar
- Android navigation bar
- Gesture-navigation area
- Display cutouts
- Rounded screen corners
- Edge-to-edge mode
- Software keyboard
- Orientation changes
- Foldable display boundaries where information is available

When the software keyboard opens:

- Keep the active field visible.
- Keep Apply and Cancel accessible.
- Resize or reposition bottom sheets.
- Update available canvas dimensions.
- Update coordinate conversions if canvas placement changes.
- Do not let keyboard visibility corrupt the active command.

No essential control may be hidden behind Android system UI.

==================================================
9. REMOVE THE PERMANENT DESKTOP COMMAND LINE
==================================================

The desktop-style command line must not remain in the normal Android workspace.

Do not delete the underlying command parser or command system.

Replace only the visible interaction.

Use this architecture:

Android Tool UI
       ↓
Mobile Command Controller
       ↓
Existing Command Parser/System
       ↓
Existing Command Execution
       ↓
Existing Document and Undo/Redo

Before hiding the command line, map everything it provides:

- Command activation
- Aliases
- Coordinates
- Distances
- Angles
- Parameters
- Confirmation
- Cancellation
- Command history
- Suggestions
- Error messages
- Multi-step state

Replace it with:

A. TOOL PALETTE
- Touch-accessible command categories

B. CONTEXTUAL ACTION BAR
- Selection-specific actions

C. MOBILE COMMAND CARD
- Active command
- Current step
- Short instruction
- Current measurement
- Back
- Cancel
- Confirm or Finish

D. PARAMETER BOTTOM SHEET
- Exact numeric and text values

E. SEARCHABLE COMMAND PALETTE
- Existing registered commands
- Recent commands
- Favorite commands
- Keyboard shortcuts when available

F. OPTIONAL ADVANCED COMMAND CONSOLE
- Hidden by default
- Available from the overflow menu
- Displayed as a dismissible sheet or dialog
- Intended for advanced physical-keyboard users
- Must not reserve permanent canvas space
- Must not be required for normal operation

Every essential command must work without opening the advanced console.

==================================================
10. MOBILE COMMAND STATE
==================================================

Create a mobile command controller that maps onto the existing CAD command state.

Possible interface states:

- Idle
- ToolSelected
- WaitingForSelection
- WaitingForFirstPoint
- WaitingForNextPoint
- WaitingForParameter
- DraggingPreview
- PreviewReady
- Confirming
- Completed
- Cancelled
- Error

Every state must define:

- Current command name
- Current step
- Required next action
- Canvas preview
- Available parameters
- Enabled controls
- Back behavior
- Cancel behavior
- Confirm behavior
- Orientation-change behavior

Do not create a second CAD execution engine.

The mobile state must control and reflect the existing command system.

No command may become stuck without a visible Cancel or exit path.

==================================================
11. FIX TOUCH INPUT AT THE EVENT PIPELINE
==================================================

Touch reliability is a blocking requirement.

Trace:

Android MotionEvent
       ↓
Android Activity/native glue
       ↓
winit Touch/WindowEvent
       ↓
egui raw input
       ↓
Canvas hit testing
       ↓
Gesture classification
       ↓
Screen-to-world transformation
       ↓
Existing CAD command
       ↓
Document transaction
       ↓
Repaint/render

Investigate:

- UI panels intercepting touches
- Transparent overlays consuming input
- Incorrect pointer ID tracking
- Missing touch-release events
- Touch-cancel behavior
- Tap/drag confusion
- Pan/draw conflict
- Pinch/draw conflict
- Long-press conflict
- Incorrect scale factor
- Incorrect toolbar offset
- Incorrect canvas bounds
- Broken screen-to-world conversion
- Tool activation reset
- Document mutation failure
- Missing repaint request
- Orientation change state corruption
- Software-keyboard viewport changes
- Multiple egui/winit versions affecting event handling

Use temporary focused diagnostic logging where necessary.

Remove excessive diagnostics after validation.

==================================================
12. GESTURE STATE MACHINE
==================================================

Implement a deliberate gesture state machine.

Possible states:

- Idle
- PotentialTap
- PointPlacement
- EntityDrag
- HandleDrag
- CanvasPan
- PinchZoom
- LongPressPending
- ContextActions
- Cancelled

Requirements:

- Small movement remains a tap.
- Movement over the threshold becomes a drag.
- One-finger input follows the active tool.
- Two-finger movement pans the canvas.
- Pinch changes zoom.
- Pinch must not create geometry.
- Two-finger pan must not select or move entities.
- Long press must not accidentally place a point.
- Touch cancellation must clear transient interaction safely.
- A completed drag must create one undo transaction.
- A second pointer must not corrupt the current command.
- Touch release must complete the correct action.

==================================================
13. LINE DRAWING IS THE FIRST ACCEPTANCE FLOW
==================================================

Fix and validate touch-based line drawing before optional enhancements.

Required workflow:

1. Tap Draw.
2. Tap Line.
3. Line tool visibly activates.
4. Display “Select start point.”
5. Tap the canvas.
6. Show a start-point indicator.
7. Display “Select endpoint.”
8. Show a live line preview.
9. Display current length and angle.
10. Tap the endpoint.
11. Existing Line command creates the line.
12. Allow continued line drawing or Finish.
13. Cancel exits cleanly.
14. Undo removes the line.
15. Redo restores the line.

Trace any failure through:

- Tool selection
- Canvas focus
- Touch event
- First point
- Command state
- Second point
- Coordinate conversion
- Snapping
- Document transaction
- Rendering/repaint

Do not implement another Line algorithm.

Repair the connection to the existing Line command.

==================================================
14. DIRECT MANIPULATION
==================================================

Use direct manipulation where appropriate.

Examples:

MOVE:
- Select entity
- Tap Move or drag selected entity
- Show preview
- Show snap feedback
- Confirm
- Existing Move command commits the result

OFFSET:
- Select entity
- Tap Offset
- Enter distance or drag preview
- Select side
- Confirm
- Existing Offset command commits the result

FILLET:
- Tap Fillet
- Enter radius
- Select first entity
- Select second entity
- Preview
- Confirm
- Existing Fillet command commits the result

CIRCLE:
- Tap Circle
- Tap center
- Drag or tap radius
- Allow exact radius input
- Confirm

Do not directly modify geometry from gesture handlers.

==================================================
15. EXACT NUMERIC INPUT
==================================================

Professional CAD requires precision.

Provide a mobile-friendly exact-value sheet for existing applicable parameters:

- X coordinate
- Y coordinate
- Length
- Distance
- Width
- Height
- Radius
- Diameter
- Angle
- Offset
- Fillet radius
- Rotation
- Scale
- Copy count
- Text

Support:

- Correct Android numeric keyboard
- Decimal values
- Negative values where applicable
- Unit display
- Current value
- Validation
- Clear error messages
- Apply
- Cancel
- Physical keyboard
- Orientation changes
- One logical undo transaction

Support both approximate and precise interaction:

1. User drags visually.
2. Live measurement appears.
3. User taps the value.
4. Exact-value sheet opens.
5. Existing command receives the validated value.
6. Apply commits it.
7. Undo restores the prior state.

Do not change existing CAD unit or precision calculations.

==================================================
16. CONTEXTUAL PROPERTIES
==================================================

Create a responsive contextual properties interface.

When nothing is selected:

- Show document or view options only when requested.

When one entity is selected:

- Show existing editable properties.

When multiple entities are selected:

- Show shared valid actions and properties.

Applicable properties may include:

- Entity type
- Position
- Dimensions
- Layer
- Color
- Line style
- Radius
- Angle
- Constraint state

Do not expose unsafe internal values.

Property changes must use the existing document command and undo systems.

==================================================
17. TOUCH TARGETS AND VISUAL FEEDBACK
==================================================

Make all controls finger-friendly.

Provide:

- Consistent minimum touch targets
- Clear spacing
- Larger invisible hit areas around precise handles
- Selected state
- Pressed state
- Disabled state
- Clear destructive-action styling
- Short toolbar labels
- Full labels in expanded sheets
- No clipped text
- No overlapping icons
- No controls under system UI

Improve canvas feedback:

- Active tool
- Selected entity
- Touch-down point
- Start point
- Endpoint
- Control handles
- Snap candidate
- Active snap
- Drawing preview
- Modification preview
- Invalid operation
- Command success
- Command cancellation
- Unsaved document
- Save status

Feedback must remain compact and must not cover important drawing content.

==================================================
18. NEW PRACTICAL ENHANCEMENTS
==================================================

Only after touch, line drawing, responsive layout, and commands are reliable, add applicable enhancements.

A. RECENT TOOLS
- Show recently used commands.
- Store command identifiers only.

B. FAVORITE TOOLS
- Allow common tools to be pinned.
- Keep all tools available through normal categories.

C. SEARCHABLE COMMAND PALETTE
- Search existing commands.
- Do not create fake commands.

D. QUICK ACTIONS
- New
- Open
- Save
- Undo
- Redo
- Fit drawing
- Snap toggle

E. CUSTOMIZABLE TOOL SHORTCUTS
- Allow safe reordering.
- Include Reset to Default.
- Never hide the only route to a command.

F. SMART CONTEXTUAL ACTIONS
- Show commands valid for selected entity types.

G. SNAP INDICATOR
- Show the current snap type.
- Use the existing snap engine.

H. TOOL PRESETS
- Reuse values such as the previous offset or fillet radius.
- Always show the active value.

I. COMMAND HISTORY
- Show recent operations in a dismissible sheet.
- Do not restore a permanent desktop log.

J. FIRST-USE TOUCH GUIDE
- Select
- Draw
- Drag
- Pinch zoom
- Two-finger pan
- Exact input
- Cancel active command
- Show only initially or from Help.

K. OPTIONAL HAPTIC FEEDBACK
- Use only when safely supported.
- Use subtle feedback for snapping or confirmed placement.
- Provide a setting to disable it.
- Do not vibrate continuously.

Do not prioritize decorative enhancements ahead of usability.

==================================================
19. POLISHED SPLASH AND STARTUP
==================================================

Create a premium Android startup flow:

Android launch theme
       ↓
AR CAD branded splash
       ↓
Subtle technical animation
       ↓
Real initialization state
       ↓
First renderable CAD frame
       ↓
Smooth transition
       ↓
Touch-ready workspace

Splash requirements:

- Existing AR CAD icon/logo
- AR CAD brand name
- Professional technical background
- Lightweight fade and scale animation
- Optional subtle grid or line animation
- Real loading indication only while initialization occurs
- Phone and tablet support
- Portrait and landscape support

Do not use:

- Fixed artificial delay
- Sleep calls
- Heavy video
- Large animation framework without need
- Duplicate Activity
- Duplicate Rust initialization
- Duplicate document initialization
- UI-thread blocking

Prevent:

- White flash
- Unwanted black flash
- Blank frame after splash
- Splash dismissal before first usable frame
- Startup ANR
- Surface initialization error
- Session restoration failure
- Application stuck indefinitely on splash

Track real readiness through:

- Activity created
- Native library loaded
- Rust initialized
- Settings loaded
- Session restored
- Document initialized
- WGPU initialized
- First frame ready
- Workspace interactive

If startup fails, show a clear error instead of leaving the splash indefinitely.

==================================================
20. REUSABLE COMPONENTS
==================================================

Use maintainable shared components instead of one large UI function.

Possible components:

- DesignTokens
- ResponsiveLayout
- AndroidCadAppBar
- MobileToolNavigation
- CompactToolButton
- ToolCategorySheet
- ContextualActionBar
- MobileCommandCard
- NumericInputSheet
- ResponsivePropertiesPanel
- CanvasNavigationControls
- TouchSelectionHandles
- SnapIndicator
- StatusMessage
- SearchableCommandPalette
- AdvancedCommandConsole
- StartupSplash
- FirstUseGuide

Adapt names to current repository conventions.

Separate:

- Styling
- Layout
- Interaction interpretation
- Command dispatch
- Document mutation
- Rendering
- Android platform integration

==================================================
21. WARNINGS MUST NOT CAUSE FEATURE REMOVAL
==================================================

Review all build warnings.

Do not hide warnings using:

- #![allow(warnings)]
- Broad #[allow(dead_code)]
- Broad #[allow(unused)]
- Broad #[allow(deprecated)]
- RUSTFLAGS=-Awarnings
- Log filtering

Do not delete functions such as:

- nearest_on_polyline
- offset_kind
- offset_polyline
- split_segment_at_points
- Snap helpers
- Fillet helpers
- Geometry helpers

merely because they are reported as unused.

First inspect:

- All call sites
- Command registration
- Desktop UI registration
- Feature flags
- Git history
- Android tool availability
- Whether the mobile GUI disconnected the caller
- Whether the function should support selection, snapping, Offset, or another existing feature

Reconnect supported functionality where appropriate.

Remove code only if it is proven obsolete across supported platforms.

Investigate and classify:

- Project-owned actionable warnings
- Generated-code warnings
- Vendored-code warnings
- External dependency warnings
- Toolchain warnings
- Intentional policy warnings

Do not violate font licensing or repository instructions while handling font warnings.

==================================================
22. DEPENDENCY AND INPUT VERSION REVIEW
==================================================

Inspect the resolved dependency graph, especially when multiple versions appear.

Run applicable commands such as:

cargo metadata --format-version 1
cargo tree -d
cargo tree -i egui
cargo tree -i egui-winit
cargo tree -i egui-wgpu
cargo tree -i eframe
cargo tree -i winit
cargo tree -i android-activity
cargo tree -i jni
cargo tree -i raw-window-handle

Determine:

- Which version receives Android touch events
- Which version is used by the launcher
- Which vendored fixes are active
- Whether multiple versions affect runtime input
- Whether patched packages are actually used
- Whether Cargo.lock resolves an unintended source
- Whether the builder regenerates dependency configuration

Do not align versions blindly.

Do not run a broad cargo update.

Make only evidence-based dependency changes.

==================================================
23. TEST MATRIX
==================================================

Test representative viewport classes.

SMALL PHONE PORTRAIT:
[ ] No overflow
[ ] No clipped controls
[ ] Bottom navigation fits
[ ] Sheets scroll
[ ] Line drawing works
[ ] Confirm and Cancel visible

SMALL PHONE LANDSCAPE:
[ ] Canvas remains usable
[ ] Tools fit
[ ] Panels do not consume the entire screen

LARGE PHONE:
[ ] Layout uses available space appropriately
[ ] Gestures work
[ ] No unnecessary empty space

TABLET PORTRAIT:
[ ] Tool palette usable
[ ] Properties panel usable
[ ] Canvas remains dominant

TABLET LANDSCAPE:
[ ] Expanded layout works
[ ] Touch works
[ ] Keyboard works
[ ] Mouse works
[ ] Panels fit correctly

Validate:

STARTUP:
[ ] Cold launch
[ ] Warm launch
[ ] Splash animation
[ ] Workspace transition
[ ] No ANR
[ ] No startup crash
[ ] Session restore

TOUCH:
[ ] Tap
[ ] Drag
[ ] Long press
[ ] Touch release
[ ] Touch cancellation
[ ] Multi-touch
[ ] Pinch zoom
[ ] Two-finger pan

DRAW:
[ ] Line
[ ] Polyline
[ ] Rectangle
[ ] Circle
[ ] Arc
[ ] Ellipse
[ ] Spline
[ ] Text where supported

MODIFY:
[ ] Move
[ ] Copy
[ ] Rotate
[ ] Scale
[ ] Delete
[ ] Trim
[ ] Extend
[ ] Offset
[ ] Fillet
[ ] Handle editing

PRECISION:
[ ] Exact coordinate
[ ] Exact length
[ ] Exact radius
[ ] Exact angle
[ ] Exact offset
[ ] Validation
[ ] Apply
[ ] Cancel

DOCUMENT:
[ ] New
[ ] Open
[ ] Save
[ ] Save As
[ ] Import
[ ] Export
[ ] Autosave and recovery where supported

OTHER INPUT:
[ ] Physical keyboard
[ ] Existing shortcuts
[ ] Mouse
[ ] Trackpad
[ ] Stylus where available

Do not mark tests as passed unless they were executed.

Mark unavailable device-only tests as NOT EXECUTED and state why.

==================================================
24. IMPLEMENT IN CONTROLLED PHASES
==================================================

PHASE 1:
- Inspect repository
- Inventory functions
- Audit current GUI
- Audit permanent command line
- Identify overflow and touch failures

PHASE 2:
- Build centralized design system
- Build responsive layout system
- Fix system insets
- Fix toolbar and panel overflow
- Fix keyboard resize behavior

PHASE 3:
- Trace Android touch pipeline
- Implement gesture state machine
- Fix line drawing
- Fix selection
- Fix pan and zoom

PHASE 4:
- Replace permanent command line
- Add mobile command card
- Add parameter sheet
- Add Confirm, Cancel, and Back
- Add optional hidden advanced console

PHASE 5:
- Connect all existing commands to the new GUI
- Add contextual actions
- Add properties
- Add exact input
- Validate undo/redo

PHASE 6:
- Apply fresh visual design
- Improve spacing, colors, icons, and feedback
- Remove remaining desktop patterns

PHASE 7:
- Improve splash and startup
- Validate lifecycle
- Validate session restoration

PHASE 8:
- Add safe enhancements
- Recent tools
- Favorites
- Command search
- Tool presets
- Onboarding
- Optional haptics

PHASE 9:
- Run complete regression testing
- Build APK
- Review warnings
- Commit
- Push
- Monitor GitHub Actions
- Fix failures
- Repeat until success

Do not make one massive unreviewable change.

Use focused commits after validated logical phases.

==================================================
25. LOCAL VALIDATION
==================================================

After every logical phase:

1. Format modified code.
2. Run static analysis.
3. Run targeted crate checks.
4. Run relevant tests.
5. Run host checks.
6. Run Android-target checks.
7. Run cargo-ndk.
8. Run the actual APK packaging process.
9. Capture the full build log.
10. Review errors.
11. Review warnings.
12. Install and test on an emulator/device when available.
13. Review the Git diff.

Use the same build path used by GitHub Actions.

Do not invent a simpler alternate build process.

Do not declare success from compilation alone.

==================================================
26. GIT REVIEW AND COMMITS
==================================================

Before committing, run:

git status
git diff --check
git diff
git diff --stat

Verify:

- CAD engine behavior was not unintentionally changed.
- Existing commands were not removed.
- File-format behavior remains unchanged.
- No temporary logging remains.
- No secrets are included.
- No signing keys are included.
- No APKs are included unless repository policy requires them.
- No build directories are included.
- No Android SDK/NDK files are included.
- No unrelated files changed.
- Generated files were not edited without updating their generator/template.
- Cargo.lock changes are intentional and understood.

Stage only reviewed files.

Avoid blindly using:

git add .

Create focused commits with meaningful messages, for example:

feat(android-ui): add responsive CAD workspace

fix(android-ui): prevent controls from overflowing small screens

fix(android-input): stabilize touch gesture recognition

fix(canvas): restore touch line drawing

feat(commands): replace desktop command line with mobile controls

feat(commands): add exact parameter input

feat(android-ui): add contextual CAD properties

feat(android-ui): refresh AR CAD visual design

feat(startup): improve branded splash and startup transition

feat(android-ui): add command search and recent tools

test(android): validate responsive touch workflows

Do not use vague commit messages.

Do not force-push.

==================================================
27. AUTOMATIC PUSH AND GITHUB ACTIONS LOOP
==================================================

After all locally testable changes are validated:

1. Confirm the current branch.
2. Confirm the Git remote.
3. Confirm the intended commits are on the branch.
4. Push the current branch.
5. If upstream is not configured, set it safely.
6. Record:

git rev-parse HEAD

7. Find the GitHub Actions run for that exact commit SHA.
8. Confirm:

LOCAL HEAD SHA
=
PUSHED SHA
=
GITHUB ACTIONS HEAD SHA

9. Monitor the workflow through completion.
10. Inspect the complete log.
11. Confirm that the APK artifact was generated.

If GitHub CLI is installed and authenticated, use actual returned values with commands such as:

gh run list --branch "$(git branch --show-current)" --limit 10

gh run view <actual-run-id>

gh run watch <actual-run-id> --exit-status

gh run view <actual-run-id> --log-failed

Do not invent a run ID.

If GitHub Actions fails:

1. Retrieve the complete failed job log.
2. Confirm it belongs to the latest pushed SHA.
3. Identify the first meaningful root error.
4. Separate errors from warnings.
5. Trace the affected source, adapter, generator, template, or dependency.
6. Apply the smallest safe fix.
7. Format and test locally.
8. Review the diff.
9. Create a new focused commit.
10. Push it.
11. Locate the new workflow run.
12. Verify the new SHA.
13. Monitor it through completion.
14. Repeat until successful or a genuine external blocker is proven.

Do not stop after pushing.

Do not treat a successful push as a successful build.

Do not stop after the first failed workflow.

==================================================
28. APK VERIFICATION
==================================================

After GitHub Actions succeeds:

Verify:

[ ] Workflow used the latest pushed SHA
[ ] APK artifact exists
[ ] APK is non-empty
[ ] Correct app identity is packaged
[ ] Correct activity is packaged
[ ] Expected ABI is present
[ ] arm64-v8a native library is present where expected
[ ] AndroidManifest.xml is present
[ ] New GUI is included
[ ] Splash resources are included
[ ] Application launches
[ ] No immediate crash
[ ] No ANR
[ ] No blank startup frame
[ ] Phone layout fits
[ ] Tablet layout fits
[ ] Portrait works
[ ] Landscape works
[ ] Line drawing works through touch
[ ] Selection works
[ ] Dragging works
[ ] Pan and zoom work
[ ] Exact input works
[ ] Permanent desktop command line is absent
[ ] Advanced command console remains optionally available
[ ] Existing CAD commands remain available
[ ] Keyboard and mouse support remain operational

Do not commit the APK unless repository policy explicitly requires it.

==================================================
29. STOP CONDITIONS
==================================================

Continue until:

SUCCESS:

- Responsive Android GUI works.
- Touch input works reliably.
- Line drawing works.
- Essential commands are touch-accessible.
- No essential controls leave the screen.
- Permanent desktop command line is replaced.
- Splash and startup are polished.
- Existing CAD behavior remains available.
- Local build succeeds.
- Changes are committed.
- Changes are pushed.
- GitHub Actions succeeds for the latest SHA.
- APK artifact is generated and verified.

OR GENUINE EXTERNAL BLOCKER:

- GitHub authentication unavailable
- Push permission unavailable
- Required private dependency unavailable
- Required signing credential unavailable
- GitHub Actions logs inaccessible
- Required external tool unavailable and cannot be installed

Compilation errors, warnings, failed tests, layout defects, and touch failures are not external blockers.

Investigate and fix them.

==================================================
30. FINAL REPORT
==================================================

Provide:

FINAL STATUS:
SUCCESS, PARTIAL, or BLOCKED

FUNCTIONALITY PRESERVATION:
- CAD engine unchanged
- Existing commands preserved
- Document/file behavior preserved
- Any verified functional defects fixed
- Any unavoidable behavior changes with evidence

RESPONSIVE GUI:
- Small phone portrait
- Small phone landscape
- Large phone
- Tablet portrait
- Tablet landscape
- System-inset handling
- Software-keyboard handling
- Overflow fixes

ANDROID COMMAND EXPERIENCE:
- Desktop command line replacement
- Tool categories
- Contextual actions
- Command guidance
- Numeric parameter entry
- Searchable commands
- Optional advanced console

TOUCH:
- Root cause of touch issues
- Event-path changes
- Gesture state machine
- Line drawing
- Selection
- Dragging
- Pinch zoom
- Two-finger pan
- Long press

VISUAL DESIGN:
- Design system
- Navigation
- Canvas treatment
- Selection and snap feedback
- Properties
- Status messages

STARTUP:
- Splash design
- Animation
- Initialization readiness
- Workspace transition
- Startup validation

ENHANCEMENTS:
- Recent tools
- Favorite tools
- Command search
- Tool presets
- First-use guide
- Haptic feedback
- Other implemented improvements

WARNINGS:
- Resolved warnings
- Remaining classified warnings
- Runtime/build impact

TESTING:
- Automated tests executed
- Emulator tests executed
- Physical-device tests executed
- Tests not executed

FILES CHANGED:
List every changed file and why.

COMMITS:
List each commit SHA and message.

GITHUB ACTIONS:
- Workflow name
- Run ID
- Head SHA
- Final result

APK:
- Artifact name
- Output path
- File size
- ABI
- Verification result

FINAL CONFIRMATION:
Confirm that the final application is a responsive, touch-first Android CAD application; that the permanent desktop command line was replaced with Android-native controls; and that the existing CAD engine, commands, calculations, document behavior, and file formats were not rewritten, simplified, or removed.