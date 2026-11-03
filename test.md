Act as a senior Android product designer, professional CAD UX architect, Rust/egui developer, touch-input engineer, Android security engineer, QA engineer, Git specialist, and GitHub Actions build engineer.

 

==================================================
PRIMARY MISSION
==================================================

Upgrade the existing AR CAD application into a premium, attractive, responsive, touch-first Android CAD application.

The app currently builds and has already received some GUI changes, but it still has significant usability and quality problems:

- The GUI is still not polished or attractive enough.
- Some Windows/desktop-oriented patterns remain.
- The permanent command-line experience is unsuitable for Android.
- Touch input is unreliable.
- Drawing a Line through touch is not consistently working.
- Canvas Zoom In and Zoom Out controls are not user-friendly.
- Pinch zoom and two-finger navigation need improvement.
- Canvas navigation controls cover too much drawing space.
- When Line or another tool is selected, top-side buttons become misaligned.
- Active-command controls may shift, overlap, wrap incorrectly, or leave the screen.
- Some buttons and panels extend outside the visible device area.
- The layout does not adapt correctly to phones, tablets, portrait, and landscape.
- Startup and splash behavior need a premium redesign.
- Existing CAD functions may not be properly connected to the redesigned interface.
- The application needs practical mobile enhancements.
- First installation must require a one-time activation password.

The final result must provide:

PREMIUM ORIGINAL ANDROID CAD GUI
+
RELIABLE TOUCH INPUT
+
STABLE TOOLBAR ALIGNMENT
+
PROFESSIONAL CANVAS NAVIGATION
+
RESPONSIVE PHONE AND TABLET LAYOUTS
+
ANDROID-NATIVE COMMAND EXPERIENCE
+
FULL EXISTING CAD FUNCTIONALITY
+
POLISHED SPLASH AND STARTUP
+
ONE-TIME INSTALLATION ACTIVATION
+
KEYBOARD, MOUSE, TRACKPAD, AND STYLUS SUPPORT
+
SUCCESSFUL APK AND GITHUB ACTIONS BUILD

Do not consider the task successful merely because Gradle produces an APK.

The installed application must be functionally usable.

==================================================
1. PRESERVE EXISTING CAD FUNCTIONALITY
==================================================

This is an Android GUI, interaction, input-routing, startup, and activation-security improvement.

Do not:

- Rewrite the CAD engine
- Create a separate simplified mobile drawing engine
- Replace geometry algorithms
- Change CAD calculation results
- Change document formats
- Change import/export behavior without a verified defect
- Change snapping calculations
- Change constraint logic
- Remove advanced tools
- Remove commands to simplify the GUI
- Comment out functionality to pass the build
- Disable tests
- Hide warnings globally
- Downgrade CADCraft
- Directly mutate geometry inside UI or gesture handlers
- Remove keyboard or mouse support
- Treat build success as functional success

Preserve:

- Geometry engine
- Document model
- Rendering engine
- Existing command execution
- Drawing tools
- Selection
- Snapping
- Constraints
- Layers and properties where supported
- Move
- Copy
- Rotate
- Scale
- Delete
- Trim
- Extend
- Offset
- Fillet
- Import and export
- Open, Save, and Save As
- Autosave and recovery where supported
- Undo and redo
- Existing keyboard shortcuts
- Mouse and trackpad support

Use this architecture:

Touch / Stylus / Mouse / Keyboard
                ↓
       Unified Mobile Input Layer
                ↓
       Existing CAD Command System
                ↓
         Existing Document Model
                ↓
      Existing Geometry and Renderer
                ↓
         Existing Undo/Redo System

Every UI control and gesture must invoke an existing CAD command.

Do not place CAD business logic directly inside buttons, panels, bottom sheets, or gesture handlers.

==================================================
2. INSPECT BEFORE MODIFYING
==================================================

Before changing code:

1. Inspect the complete repository structure.
2. Read AGENTS.md and all repository instructions.
3. Inspect the current branch and Git status.
4. Inspect recent commits related to Android UI and input.
5. Inspect the current Android adapter.
6. Inspect the current GUI redesign.
7. Inspect the CAD command registry.
8. Inspect menu and tool registration.
9. Inspect canvas rendering.
10. Inspect touch and pointer events.
11. Inspect winit event handling.
12. Inspect egui raw-input handling.
13. Inspect screen-to-world coordinate conversion.
14. Inspect the visible desktop command-line implementation.
15. Inspect the Android Activity and Java bridge.
16. Inspect splash and launch-theme implementation.
17. Inspect file services and Android Storage Access Framework integration.
18. Inspect Cargo manifests and Cargo.lock.
19. Inspect vendored dependencies.
20. Inspect build scripts and GitHub Actions workflows.
21. Inspect the reference Android repository.

Record:

- Reference branch/tag
- Reference commit SHA
- Relevant adapter files
- Relevant input patches
- Relevant launcher/template files
- Applicable Android integration patterns

Do not copy the reference repository blindly.

Port applicable Android integration forward to CADCraft 0.15.1.

Do not downgrade CADCraft merely because an older reference version compiles.

==================================================
3. CREATE A COMPLETE AUDIT
==================================================

Create an implementation audit containing:

[ ] Component or feature
[ ] Current behavior
[ ] Current visual problem
[ ] Current touch problem
[ ] Phone impact
[ ] Tablet impact
[ ] Existing command/API
[ ] Required improvement
[ ] File affected
[ ] Validation completed
[ ] Final status

Audit:

- Splash screen
- Activation screen
- Top app bar
- Top active-command controls
- Bottom toolbar
- Side toolbar
- Tool sheets
- Contextual actions
- Canvas
- Zoom controls
- Pan controls
- Command guidance
- Numeric input
- Properties panel
- Dialogs
- Menus
- Status messages
- Selection feedback
- Snap feedback
- Software keyboard behavior
- Phone portrait
- Phone landscape
- Tablet portrait
- Tablet landscape

Also inventory all commands:

[ ] Command name
[ ] Existing desktop entry point
[ ] Existing keyboard shortcut
[ ] Existing command/API
[ ] Required parameters
[ ] Selection requirements
[ ] Current Android route
[ ] New touch route
[ ] Exact-value input route
[ ] Confirm behavior
[ ] Cancel behavior
[ ] Undo/redo behavior
[ ] Test result

Do not begin with random color or spacing changes.

Identify structural and functional root causes first.

==================================================
4. PREMIUM ORIGINAL AR CAD DESIGN SYSTEM
==================================================

Create an original professional Android CAD visual system.

The interface may be inspired by professional technical drawing applications, but do not copy AutoCAD branding, icons, colors, assets, or proprietary layouts.

The visual system must feel:

- Professional
- Technical
- Precise
- Modern
- Clean
- Premium
- Touch-friendly
- Efficient
- Suitable for long working sessions
- Native to Android
- Familiar to CAD users

Recommended direction:

- Deep graphite or blue-black canvas
- Charcoal/slate application surfaces
- Technical blue or cyan primary accent
- Restrained teal secondary accent
- Bright high-contrast selection color
- Distinct snap color
- Controlled warning and error colors
- Clean line-style icons
- Compact technical typography
- Minimal shadows
- Subtle elevation
- Consistent corner radius
- Clear selected-tool state
- High readability
- Reduced visual clutter

Create centralized design tokens for:

- Canvas color
- App background
- Panel surfaces
- Primary accent
- Secondary accent
- Selection color
- Snap indicator
- Warning color
- Error color
- Primary text
- Secondary text
- Disabled state
- Pressed state
- Focus state
- Typography
- Spacing
- Corner radius
- Elevation
- Button dimensions
- Touch-target dimensions
- Icon dimensions
- Animation timing
- Compact/medium/expanded layouts

Do not scatter hard-coded colors and dimensions through unrelated files.

Validate contrast and readability.

Do not use color as the only indication of an error, selection, tool state, or command state.

Avoid:

- Large decorative cards
- Excessive gradients
- Excessive animation
- Competing accent colors
- Huge headers
- Windows menu bars
- Tiny nested dropdowns
- Crowded horizontal rows
- Panels permanently covering the canvas

The canvas must remain the main visual focus.

==================================================
5. RESPONSIVE ANDROID LAYOUT SYSTEM
==================================================

Create one centralized responsive-layout model.

Determine layout using:

- Available width
- Available height
- Device orientation
- Display density
- Safe-area insets
- Software keyboard visibility
- Current panel visibility
- Current command state

Support:

COMPACT:
- Small phones
- Bottom primary navigation
- Full-width scrollable bottom sheets
- Compact app bar
- No permanent side panels
- Maximum canvas area

MEDIUM:
- Large phones and small tablets
- Compact bottom or side navigation
- Collapsible properties
- More command information where space permits

EXPANDED:
- Tablets in landscape
- Left tool rail
- Central canvas
- Optional right properties panel
- Stable compact command guidance
- Keyboard, mouse, stylus, and touch productivity

Every major UI component must use this shared layout model.

Do not scatter unrelated width checks throughout the source.

==================================================
6. FIX SCREEN OVERFLOW
==================================================

No essential component may extend outside the usable viewport.

Inspect:

- Hard-coded dimensions
- Fixed button counts
- Fixed coordinates
- Unbounded horizontal rows
- Missing scrolling
- Oversized labels
- Dialogs larger than the viewport
- Panels without maximum dimensions
- Controls under status/navigation bars
- Controls hidden by the software keyboard
- Landscape layouts using portrait assumptions
- Phone layouts using tablet dimensions

If controls do not fit:

- Collapse secondary actions
- Use tool-category sheets
- Use horizontal scrolling where clear
- Use a More action
- Use responsive wrapping where readable
- Use the searchable command palette
- Move properties into scrollable sheets

Do not shrink buttons until they become difficult to touch.

During active commands, Confirm and Cancel must always remain accessible.

==================================================
7. SAFE AREA AND SOFTWARE KEYBOARD
==================================================

Account for:

- Android status bar
- Android navigation bar
- Gesture navigation
- Display cutouts
- Rounded corners
- Edge-to-edge mode
- Software keyboard
- Orientation changes
- Foldable boundaries when available

When the keyboard opens:

- Keep the active input visible.
- Keep Apply and Cancel visible.
- Resize or reposition bottom sheets.
- Update available canvas dimensions.
- Update canvas coordinate transformation when layout changes.
- Preserve active command state.

No essential control may be hidden behind Android system UI.

==================================================
8. STABLE TOP APP BAR
==================================================

The current top controls become misaligned when Line or another tool is selected.

Fix this architecturally.

The top app bar must have a stable structure and height.

It should contain only persistent application-level actions:

LEFT AREA:
- Navigation or compact AR CAD identity

CENTER AREA:
- Document name
- Optional compact save state

RIGHT AREA:
- Undo
- Redo
- Save
- Overflow menu

Do not insert command-specific buttons directly into the top app bar.

Selecting Line, Circle, Move, Offset, Fillet, or another command must not:

- Change top-bar height
- Shift persistent actions
- Push buttons outside the screen
- Wrap persistent controls unpredictably
- Overlap document text
- Cause canvas movement
- Rebuild the entire app bar
- Create a second toolbar row without responsive planning

Use fixed logical regions and consistent alignment.

Use truncation for long document names.

On compact screens, move secondary actions into the overflow menu instead of shrinking primary actions excessively.

==================================================
9. MOBILE COMMAND CARD
==================================================

Move active-command actions out of the top app bar.

Create a dedicated responsive Mobile Command Card.

It should contain:

- Active command name
- Current step
- Short instruction
- Current measurement
- Back
- Cancel
- Confirm or Finish

Example:

LINE
Select endpoint
Length: 25.00 mm · Angle: 90°
[Back] [Cancel] [Finish]

Placement:

- On compact phones, dock above the bottom toolbar or display as a compact bottom card.
- On landscape phones, use a compact side or bottom placement.
- On tablets, use a small command panel near the canvas edge.
- Keep it separate from the persistent top app bar.

Requirements:

- Stable alignment
- Responsive width
- Maximum width
- No horizontal overflow
- No overlap with navigation controls
- No overlap with Android system UI
- No canvas obstruction around the current input point
- Long instructions truncate or wrap safely
- Confirm and Cancel remain visible
- Height remains predictable

The Line tool must not cause top-side buttons to become misaligned.

==================================================
10. REMOVE THE PERMANENT DESKTOP COMMAND LINE
==================================================

Remove the permanent desktop command line from the normal Android workspace.

Do not delete the underlying parser or command execution system.

Replace the visible command line with:

A. TOOL PALETTE
- Select
- Draw
- Modify
- Measure/Annotate
- View
- Document

B. CONTEXTUAL ACTION BAR
- Move
- Copy
- Rotate
- Scale
- Delete
- Offset
- Fillet
- Properties

C. MOBILE COMMAND CARD
- Active command and current step

D. PARAMETER BOTTOM SHEET
- Coordinates and exact values

E. SEARCHABLE COMMAND PALETTE
- Existing commands only
- Recent commands
- Favorites
- Keyboard shortcuts where available

F. OPTIONAL ADVANCED COMMAND CONSOLE
- Available from the overflow menu
- Hidden by default
- Dismissible sheet/dialog
- No permanent canvas space
- Physical-keyboard friendly
- Not required for normal use

Every essential command must be touch-accessible without opening the advanced console.

==================================================
11. TOUCH INPUT PIPELINE
==================================================

Touch reliability is a blocking requirement.

Trace:

Android MotionEvent
       ↓
Activity/native glue
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

- Transparent overlays consuming touch
- UI panels intercepting canvas input
- Incorrect pointer IDs
- Lost release events
- Missing touch-cancel handling
- Tap/drag confusion
- Pan/drawing conflicts
- Pinch/point-placement conflicts
- Long-press conflicts
- Incorrect display scale
- Incorrect egui pixels-per-point
- Incorrect canvas bounds
- Incorrect app-bar or toolbar offsets
- Broken screen-to-world conversion
- Tool state resetting
- Missing repaint
- Orientation state loss
- Keyboard-induced viewport changes
- Duplicate egui/winit versions affecting input

Use focused temporary diagnostics.

Remove noisy diagnostic logging after validation.

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
- Movement beyond threshold becomes drag.
- One finger follows the active CAD tool.
- Two fingers pan the canvas.
- Pinch zooms.
- Pinch must not create geometry.
- Two-finger pan must not move selected geometry.
- Long press must not place an unwanted point.
- Touch cancellation clears transient state.
- One drag produces one undo transaction.
- Adding/removing a second pointer must not corrupt the command.
- Touch release completes the intended action.

==================================================
13. PROFESSIONAL CANVAS NAVIGATION
==================================================

Redesign Zoom In, Zoom Out, Fit, Center, and Pan controls.

Do not keep large controls permanently floating over important canvas content.

Gestures must be the primary phone navigation:

- Pinch outward → Zoom In
- Pinch inward → Zoom Out
- Two-finger drag → Pan
- Double tap → Fit or context-appropriate zoom when safe

Keep compact controls as an accessibility and fallback mechanism.

Recommended compact control:

[Navigate]
     ↓
[+] [-] [Fit] [Center]

Or use a compact vertical control placed in a safe edge region:

[Fit]
[ + ]
[ - ]
[Center]

Requirements:

- Compact appearance
- Minimum finger-safe hit area
- Does not cover selected geometry unnecessarily
- Does not overlap bottom navigation
- Respects navigation insets
- Moves when bottom sheets open
- Adapts to phones and tablets
- Does not overlap the software keyboard
- Does not block command guidance
- Can collapse when not needed

Do not place oversized Zoom buttons in the center or primary work area.

==================================================
14. PINCH ZOOM QUALITY
==================================================

Pinch zoom must be smooth, stable, and predictable.

Track:

- Pointer IDs
- Initial finger distance
- Current distance
- Gesture center
- Initial zoom
- Current zoom
- Minimum zoom
- Maximum zoom
- Viewport translation
- Screen-to-world transformation
- World-to-screen transformation

The world point under the pinch center should remain visually stable where supported by the current viewport architecture.

Prevent:

- Jump when second finger touches
- Jump when one finger lifts
- Sudden scale changes
- Geometry placement during pinch
- Entity selection during pinch
- Entity movement during pinch
- Infinite zoom
- Negative zoom
- NaN viewport state
- Excessively sensitive scaling
- Excessively slow scaling

Use bounded zoom values.

Do not change document geometry.

Only change the camera/viewport.

==================================================
15. BUTTON-BASED ZOOM
==================================================

Button zoom must use predictable increments and smooth visual transitions.

Tap Zoom In:

Current viewport
       ↓
Small bounded zoom increment
       ↓
Smooth viewport update

Tap Zoom Out:

Current viewport
       ↓
Small bounded zoom decrement
       ↓
Smooth viewport update

Do not animate document geometry.

Do not let animation interfere with:

- Point placement
- Snapping
- Selection
- Drawing preview
- Exact coordinate input

Optional long-press continuous zoom may be added only if it remains controllable and does not conflict with gestures.

==================================================
16. FIT DRAWING AND ZOOM TO SELECTION
==================================================

Fit Drawing must:

- Use existing document or geometry bounds.
- Preserve aspect ratio.
- Add visual padding.
- Account for visible app bars.
- Account for bottom sheets.
- Account for properties panels.
- Avoid cropping geometry.
- Work after orientation changes.
- Adjust only the viewport.

When entities are selected, provide Zoom to Selection through contextual actions when supported.

Use existing selected-entity bounds.

Do not duplicate geometry calculations if the engine already exposes them.

==================================================
17. FIRST ACCEPTANCE TEST: LINE DRAWING
==================================================

Touch-based Line drawing is the first mandatory functional acceptance flow.

Required behavior:

1. Tap Draw.
2. Tap Line.
3. Line becomes visibly active.
4. Top app bar remains aligned and unchanged.
5. Mobile Command Card displays “Select start point.”
6. Tap the canvas.
7. Start-point indicator appears.
8. Mobile Command Card displays “Select endpoint.”
9. Moving the pointer displays a live preview.
10. Length and angle appear in the command card.
11. Tap the endpoint.
12. Existing Line command commits the entity.
13. Continue drawing or tap Finish.
14. Cancel exits cleanly.
15. Undo removes the line.
16. Redo restores the line.

Do not implement a second Line algorithm.

Repair only the touch, state, coordinate, preview, or command wiring needed to use the existing Line command.

==================================================
18. DIRECT MANIPULATION
==================================================

Implement touch-friendly editing through existing commands:

- Tap entity to select
- Tap empty canvas to clear selection
- Long press for contextual actions
- Drag selected entity through Move
- Drag line endpoints through existing edit commands
- Drag circle radius handles when supported
- Drag arc control points when supported
- Drag polyline vertices
- Show live preview
- Show current measurement
- Show snapping
- Confirm or cancel
- Preserve constraints
- Create one undo transaction

Use larger invisible touch hit regions around visually precise handles.

Do not make the visual handles unnecessarily large.

==================================================
19. EXACT NUMERIC INPUT
==================================================

Provide precise input for:

- X
- Y
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

Requirements:

- Correct Android keyboard
- Decimal input
- Negative values where valid
- Unit display
- Current value
- Input validation
- Clear errors
- Apply
- Cancel
- Physical-keyboard support
- Orientation preservation
- One undo transaction

Allow:

Visual drag
→ live value
→ tap value
→ exact input
→ Apply through existing command

Do not change CAD precision or units.

==================================================
20. CONTEXTUAL PROPERTIES
==================================================

Create a responsive properties interface.

For one selected entity:

- Show its existing editable properties.

For multiple selected entities:

- Show shared valid actions and properties.

Applicable values may include:

- Entity type
- Position
- Dimensions
- Layer
- Color
- Line style
- Radius
- Angle
- Constraint state

Use existing document commands for edits.

Do not expose unsafe internal values.

==================================================
21. ONE-TIME INSTALLATION ACTIVATION
==================================================

Add a one-time first-install activation screen.

The required activation password is:

abhya1998

Do not store the password as plain text in committed source code, resources, build scripts, logs, workflows, preferences, assets, or commit messages.

Use the supplied password only to generate a secure verifier.

Preferred design:

1. Generate a strong random salt.
2. Derive the verifier using Argon2id where compatible.
3. If Argon2id is not practical in the current Android/Rust build, use PBKDF2-HMAC-SHA-256 with a strong reviewed configuration.
4. Embed only:
   - Algorithm identifier
   - Salt
   - KDF parameters
   - Derived verifier
5. Derive a candidate verifier from entered input.
6. Compare using constant-time comparison where supported.
7. Never log the entered password.
8. Clear password input state after validation.
9. Never display the real password in UI errors.
10. Do not use plain SHA-256 alone as the preferred verifier.

Do not place the password in Git history.

Before every commit, search staged and tracked source for accidental plain-text exposure.

==================================================
22. ACTIVATION EXPERIENCE
==================================================

First-install flow:

App launch
       ↓
Branded splash
       ↓
Check activation state
       ↓
If not activated:
Show activation screen
       ↓
Enter password
       ↓
Validate securely
       ↓
Store protected activation state
       ↓
Open CAD workspace

If already activated:

App launch
       ↓
Branded splash
       ↓
Read protected activation state
       ↓
Open CAD workspace

Activation screen:

- AR CAD identity
- Short explanation
- Masked password field
- Show/hide control
- Activate button
- Clear invalid-password error
- Loading state during verification
- Keyboard-safe layout
- Phone/tablet support
- Portrait/landscape support
- Empty-input handling
- No password hint
- No display of the real value
- No password analytics or logging

Do not retain entered password after activation.

==================================================
23. PROTECTED ACTIVATION STATE
==================================================

Store only a protected activation flag or signed activation record.

Use Android Keystore-backed protection where practical.

Activation should survive:

- Normal restart
- Device reboot
- Same-signature application update

Activation should be requested again after:

- Uninstall/reinstall
- App data clearing
- Activation-state corruption
- Explicit protected reset, if implemented

Review:

- Android backup behavior
- Keystore alias handling
- Preference protection
- Activity export status
- Intent handling
- Deep links
- File-open intents
- State corruption recovery
- Update behavior

The CAD workspace must not be reachable before activation through:

- Alternative activities
- Deep links
- File intents
- Notification intents
- Advanced command console
- Restored state
- Exported components

Do not add online tracking, backend activation, analytics, or device collection.

==================================================
24. ACTIVATION ATTEMPT PROTECTION
==================================================

Add reasonable local guessing protection:

- Permit a small number of immediate attempts.
- Add increasing temporary delay after repeated failure.
- Do not permanently lock out the legitimate user.
- Do not reveal password length.
- Do not reveal which character is wrong.
- Do not block the Android UI thread.
- Reset failure state after successful activation.

Document accurately that a local verifier prevents casual access but cannot provide unbreakable licensing against APK reverse engineering or modification.

==================================================
25. PREMIUM SPLASH AND STARTUP
==================================================

Create a continuous startup flow:

Android launch theme
       ↓
AR CAD branded splash
       ↓
Subtle technical animation
       ↓
Real initialization
       ↓
Activation-state check
       ↓
Activation screen or first CAD frame
       ↓
Smooth transition
       ↓
Touch-ready workspace

Splash requirements:

- Existing AR CAD logo/icon
- AR CAD brand name
- Professional technical background
- Lightweight fade/scale animation
- Optional subtle grid/line effect
- Real readiness-based transition
- Phone and tablet support
- Portrait and landscape support

Do not use:

- Artificial fixed delays
- Sleep calls
- Heavy video
- Large animation dependencies without need
- Duplicate Activity
- Duplicate Rust initialization
- Duplicate document initialization
- UI-thread blocking

Prevent:

- White flash
- Unwanted black flash
- Blank intermediate frame
- Splash dismissal before first usable frame
- Startup ANR
- Surface initialization failure
- Session restoration failure
- Splash stuck indefinitely

Track readiness through:

- Activity creation
- Native library loading
- Rust initialization
- Settings loading
- Activation-state check
- Session restoration
- Document initialization
- WGPU initialization
- First frame ready
- Workspace interactive

If startup fails, show a clear startup error rather than remaining permanently on the splash.

==================================================
26. PRACTICAL ENHANCEMENTS
==================================================

Only after touch, layout, navigation, and commands work reliably, add applicable enhancements:

A. RECENT TOOLS
- Store command identifiers.
- Show recently used tools.

B. FAVORITE TOOLS
- Let users pin common commands.
- Keep every command available normally.

C. SEARCHABLE COMMAND PALETTE
- Search existing commands only.

D. QUICK ACTIONS
- New
- Open
- Save
- Undo
- Redo
- Fit Drawing
- Snap toggle

E. CUSTOMIZABLE TOOL SHORTCUTS
- Safe reordering
- Reset to Default
- Never hide the only command route

F. SMART CONTEXTUAL ACTIONS
- Show actions valid for selected entity types.

G. SNAP INDICATOR
- Show active snap mode.
- Use the existing snap engine.

H. TOOL PRESETS
- Reuse safe previous values such as offset or fillet radius.
- Always display the active value.

I. COMMAND HISTORY
- Dismissible sheet
- No permanent desktop log

J. FIRST-USE TOUCH GUIDE
- Draw
- Select
- Drag
- Pinch zoom
- Two-finger pan
- Exact input
- Cancel command
- Show once or from Help

K. OPTIONAL HAPTICS
- Subtle snap or placement confirmation
- User setting to disable
- No continuous vibration

Do not prioritize decorative features ahead of core functionality.

==================================================
27. REUSABLE COMPONENTS
==================================================

Create reusable components such as:

- DesignTokens
- ResponsiveLayout
- AndroidCadAppBar
- MobileToolNavigation
- ToolCategorySheet
- ContextualActionBar
- MobileCommandCard
- NumericInputSheet
- ResponsivePropertiesPanel
- CanvasNavigationControls
- TouchSelectionHandles
- SnapIndicator
- SearchableCommandPalette
- AdvancedCommandConsole
- StartupSplash
- ActivationScreen
- SecureActivationStore
- FirstUseGuide
- StatusMessage

Adapt names to repository conventions.

Separate:

- Styling
- Layout
- Gesture interpretation
- Command dispatch
- Viewport navigation
- Activation verification
- Activation storage
- Document mutation
- Rendering
- Android platform integration

Do not implement the entire mobile interface in one large function.

==================================================
28. WARNINGS AND DEPENDENCY REVIEW
==================================================

Review every build warning.

Do not hide warnings using:

- #![allow(warnings)]
- Broad #[allow(dead_code)]
- Broad #[allow(unused)]
- Broad #[allow(deprecated)]
- RUSTFLAGS=-Awarnings
- Log filtering

Do not delete:

- nearest_on_polyline
- offset_kind
- offset_polyline
- split_segment_at_points
- Snap helpers
- Fillet helpers
- Geometry helpers

merely because they appear unused.

First inspect:

- Call sites
- Command registration
- Desktop tool registration
- Feature flags
- Git history
- Android tool availability
- Whether the redesigned UI disconnected the caller

Reconnect supported functionality where appropriate.

Review dependency versions for:

- egui
- egui-winit
- egui-wgpu
- eframe
- winit
- android-activity
- jni
- raw-window-handle
- Any cryptographic dependency added for activation

Do not run a broad cargo update.

Make targeted evidence-based changes.

Review APK size impact from new dependencies.

==================================================
29. IMPLEMENTATION PHASES
==================================================

PHASE 1:
- Audit repository
- Audit current GUI
- Audit commands
- Audit touch
- Audit canvas navigation
- Audit top-toolbar alignment
- Audit startup
- Audit activation requirements

PHASE 2:
- Create centralized design system
- Create responsive layout framework
- Fix safe areas
- Fix overflow
- Fix keyboard resize
- Stabilize the top app bar

PHASE 3:
- Move command-specific controls into Mobile Command Card
- Remove toolbar shifts
- Fix Line active-state alignment
- Validate all active-command layouts

PHASE 4:
- Trace touch input
- Implement gesture state machine
- Fix Line drawing
- Fix selection
- Fix drag editing

PHASE 5:
- Replace canvas navigation controls
- Improve pinch zoom
- Improve two-finger pan
- Add Fit Drawing
- Add Zoom to Selection where supported

PHASE 6:
- Replace permanent desktop command line
- Add tool categories
- Add parameter sheets
- Add command search
- Keep optional advanced console

PHASE 7:
- Connect all existing commands
- Improve properties
- Improve exact input
- Validate undo/redo

PHASE 8:
- Implement secure one-time activation
- Protect activation state
- Test bypass routes
- Verify no plain-text password exposure

PHASE 9:
- Improve splash and startup
- Integrate activation-state check
- Validate lifecycle and session restore

PHASE 10:
- Apply premium colors, typography, spacing, icons, and feedback
- Remove remaining desktop patterns

PHASE 11:
- Add safe practical enhancements
- Recent tools
- Favorites
- Tool presets
- First-use guide
- Optional haptics

PHASE 12:
- Run complete testing
- Build APK
- Review warnings
- Commit
- Push
- Monitor GitHub Actions
- Fix failures
- Repeat until success

Do not create one massive unreviewable commit.

==================================================
30. TESTING REQUIREMENTS
==================================================

SECURITY:

[ ] First launch requests activation
[ ] Correct password activates
[ ] Incorrect password is rejected
[ ] Empty input is rejected
[ ] Password input is masked
[ ] Show/hide works
[ ] Password is not logged
[ ] Plain password is absent from committed source
[ ] Plain password is absent from resources
[ ] Plain password is absent from build logs
[ ] Activation survives restart
[ ] Activation survives same-signature update
[ ] App-data reset requires activation again
[ ] Reinstall requires activation again
[ ] Corrupted activation state fails securely
[ ] Attempt delay does not block UI thread
[ ] No alternate route bypasses activation

STARTUP:

[ ] Cold launch
[ ] Warm launch
[ ] Branded splash
[ ] Activation transition
[ ] Workspace transition
[ ] No white flash
[ ] No blank frame
[ ] No crash
[ ] No ANR

RESPONSIVE UI:

[ ] Small phone portrait
[ ] Small phone landscape
[ ] Large phone
[ ] Tablet portrait
[ ] Tablet landscape
[ ] System insets
[ ] Software keyboard
[ ] No overflow
[ ] Stable top app bar
[ ] Active Line controls aligned
[ ] Other active-tool controls aligned

CANVAS NAVIGATION:

[ ] Pinch zoom
[ ] Two-finger pan
[ ] No zoom jump
[ ] No geometry creation during pinch
[ ] No selection during pinch
[ ] Compact Zoom controls
[ ] Fit Drawing
[ ] Zoom to Selection where supported
[ ] Minimum/maximum zoom
[ ] Orientation-change behavior
[ ] No overlap with toolbars/sheets

TOUCH:

[ ] Tap
[ ] Drag
[ ] Long press
[ ] Touch release
[ ] Touch cancellation
[ ] Multi-touch
[ ] Handle dragging

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

[ ] Coordinates
[ ] Length
[ ] Radius
[ ] Angle
[ ] Offset
[ ] Apply
[ ] Cancel
[ ] Undo
[ ] Redo

DOCUMENT:

[ ] New
[ ] Open
[ ] Save
[ ] Save As
[ ] Import
[ ] Export
[ ] Autosave/recovery

OTHER INPUT:

[ ] Physical keyboard
[ ] Existing shortcuts
[ ] Mouse
[ ] Trackpad
[ ] Stylus where available

Do not mark tests as passed unless executed.

Mark unavailable device-only tests as NOT EXECUTED with the reason.

==================================================
31. LOCAL VALIDATION
==================================================

After every logical phase:

1. Format modified code.
2. Run static analysis.
3. Run relevant tests.
4. Run Cargo checks.
5. Run Android-target checks.
6. Run cargo-ndk.
7. Run the real APK packaging process.
8. Capture the full build log.
9. Review errors.
10. Review warnings.
11. Scan for plain-text password exposure.
12. Install on emulator/device when available.
13. Validate touch, layout, navigation, startup, and activation.
14. Review Git diff.

Use the same build path used by GitHub Actions.

Do not declare success from compilation alone.

==================================================
32. GIT REVIEW AND COMMITS
==================================================

Before committing:

git status
git diff --check
git diff
git diff --stat

Search staged and tracked source for accidental exposure of the activation password.

The plain-text value must not be committed.

Verify:

- CAD behavior was not unintentionally changed.
- Existing commands were not removed.
- No temporary diagnostics remain.
- No plain-text password is staged.
- No credentials are staged.
- No signing keys are staged.
- No APK/build directories are staged unless policy requires them.
- No SDK/NDK files are staged.
- Generated files are updated through templates/generators.
- Cargo.lock changes are intentional.
- No unrelated files changed.

Stage only reviewed files.

Avoid blindly using:

git add .

Use focused commits such as:

feat(android-ui): add premium AR CAD design system

fix(android-ui): add responsive screen fitting

fix(android-ui): stabilize top app bar alignment

fix(commands): align active Line controls

fix(android-input): stabilize touch gesture handling

fix(canvas): restore reliable touch line drawing

feat(canvas): redesign zoom and navigation controls

fix(canvas): improve pinch zoom and two-finger pan

feat(commands): replace desktop command line

feat(commands): add exact parameter input

feat(security): add secure first-install activation

feat(startup): improve branded launch experience

feat(android-ui): add contextual tools and properties

test(android): validate activation and touch workflows

Do not include the password in commit messages.

Do not force-push.

==================================================
33. AUTOMATIC PUSH AND GITHUB ACTIONS LOOP
==================================================

After local validation:

1. Confirm the current branch.
2. Confirm the Git remote.
3. Commit reviewed changes.
4. Push the branch.
5. If upstream is missing, set it safely.
6. Record:

git rev-parse HEAD

7. Find the GitHub Actions run for that exact SHA.
8. Confirm:

LOCAL HEAD SHA
=
PUSHED SHA
=
GITHUB ACTIONS HEAD SHA

9. Monitor the workflow through completion.
10. Inspect the complete log.
11. Confirm APK generation.
12. Scan workflow logs for accidental password exposure.

If GitHub CLI is authenticated, use actual returned values with commands such as:

gh run list --branch "$(git branch --show-current)" --limit 10

gh run view <actual-run-id>

gh run watch <actual-run-id> --exit-status

gh run view <actual-run-id> --log-failed

Do not invent a run ID.

If the workflow fails:

1. Retrieve the complete failed log.
2. Confirm it belongs to the latest SHA.
3. Identify the first meaningful root error.
4. Separate errors from warnings.
5. Fix the root cause.
6. Run local validation.
7. Review for password/secret exposure.
8. Create a new focused commit.
9. Push.
10. Find the new workflow run.
11. Verify the new SHA.
12. Monitor the run.
13. Repeat until successful or genuinely blocked.

Do not stop after pushing.

Do not treat push success as build success.

==================================================
34. FINAL APK VERIFICATION
==================================================

After GitHub Actions succeeds, verify:

[ ] Latest pushed SHA was built
[ ] APK artifact exists
[ ] APK is non-empty
[ ] Correct application identity
[ ] Correct Activity
[ ] Expected ABI
[ ] AndroidManifest.xml included
[ ] Premium design included
[ ] Splash assets included
[ ] Activation screen included
[ ] Plain password is not present in committed resources/logs
[ ] First installation asks for activation
[ ] Correct password activates
[ ] Restart does not request activation again
[ ] App-data reset requests activation again
[ ] Workspace cannot be bypassed before activation
[ ] Application launches after activation
[ ] No crash
[ ] No ANR
[ ] Phone layout fits
[ ] Tablet layout fits
[ ] Top app bar stays aligned
[ ] Selecting Line does not misalign buttons
[ ] Command card fits
[ ] Zoom controls are compact
[ ] Pinch zoom works smoothly
[ ] Two-finger pan works
[ ] Fit Drawing works
[ ] Touch Line drawing works
[ ] Selection works
[ ] Drag editing works
[ ] Exact input works
[ ] Existing commands remain available
[ ] Keyboard and mouse remain usable

Do not commit the APK unless repository policy explicitly requires it.

==================================================
35. STOP CONDITIONS
==================================================

Continue until:

SUCCESS:

- Premium Android GUI works.
- Controls fit phones and tablets.
- Top app bar remains aligned.
- Selecting Line does not shift or overlap buttons.
- Mobile Command Card works.
- Touch is reliable.
- Line drawing works.
- Canvas zoom is user-friendly.
- Pinch zoom and two-finger pan work.
- Permanent desktop command line is replaced.
- Secure first-install activation works.
- Splash/startup is polished.
- Existing CAD functionality remains available.
- Local build succeeds.
- Changes are committed.
- Changes are pushed.
- GitHub Actions succeeds for the latest SHA.
- APK is generated and verified.

OR GENUINE EXTERNAL BLOCKER:

- GitHub authentication unavailable
- Push permission unavailable
- Private dependency unavailable
- Required signing credential unavailable
- Build logs inaccessible
- Required external tool unavailable and cannot be installed

Compilation errors, warnings, failed tests, layout defects, activation defects, touch failures, and zoom problems are not external blockers.

Investigate and fix them.

==================================================
36. FINAL REPORT
==================================================

Provide:

FINAL STATUS:
SUCCESS, PARTIAL, or BLOCKED

DESIGN:
- Color system
- Typography
- Layout
- App bar
- Tool navigation
- Mobile Command Card
- Properties
- Canvas feedback

RESPONSIVENESS:
- Small phone
- Large phone
- Tablet
- Portrait
- Landscape
- Insets
- Keyboard handling
- Overflow fixes

TOP TOOLBAR:
- Root cause of misalignment
- Line-tool behavior
- Stable app-bar structure
- Command-card placement
- Validation results

CANVAS NAVIGATION:
- Previous zoom problems
- New compact controls
- Pinch zoom
- Two-finger pan
- Fit Drawing
- Zoom to Selection
- Zoom bounds
- Validation results

TOUCH:
- Root cause
- Event-path changes
- Gesture state machine
- Line drawing
- Selection
- Drag
- Long press

SECURITY:
- Password-verifier algorithm
- Salt and KDF parameters
- Constant-time comparison
- Activation-state protection
- Android Keystore usage
- Attempt protection
- Backup behavior
- Bypass testing
- Confirmation that plain-text password was not committed or logged
- Security limitations

STARTUP:
- Splash design
- Animation
- Activation transition
- Workspace transition
- Startup validation

FUNCTIONALITY:
- Existing commands preserved
- Verified functional defects fixed
- Tests not executed

WARNINGS:
- Resolved warnings
- Remaining classified warnings
- Build/runtime impact

FILES CHANGED:
List every changed file and reason.

COMMITS:
List each SHA and commit message.

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
Confirm that the application now has a premium Android GUI, stable top-toolbar alignment, improved canvas navigation, reliable touch interaction, secure first-install activation, polished startup, and preserved CAD functionality without committing or logging the plain-text activation password.