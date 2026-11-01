# AR CAD — Step 3: Android touch-first drafting controls

## Scope

Step 3 adds a dedicated mobile drafting toolbar to the Android CAD workspace. It uses CADCraft's existing command dispatcher and interactive command machines; it does not create a parallel drawing engine or a visual-only mock toolbar.

## Implemented

- Replaced the compact desktop-style Android toolbar with two labelled, horizontally scrollable rows while a drawing is open.
- **Draw row:** Line, Polyline, Circle, Rectangle, Arc, Ellipse and Polygon.
- **Edit row:** Move, Copy, Rotate, Scale, Offset, Trim, Extend, Erase, Undo and Redo.
- Android build-time patches enlarge drafting controls to at least 48 logical points high; the independently scrollable command rows remain usable on narrow screens.
- The start page retains its smaller 52-point brand strip; drafting controls appear only after leaving the start page.
- Every button dispatches the corresponding existing CADCraft command through `CadApp::start` and records its accessible control label.
- CADCraft's Android builder patch reads the final canvas position from egui's `Touch::End` event and uses it to complete point prompts even when the pinned egui-winit 0.37 event loop clears pointer hover with `PointerGone`; explicit Zoom In/Out touch controls are also available.
- Android pinch input uses a shorter recognition deadzone and moves the egui pointer to the live two-finger midpoint, so canvas zoom stays anchored beneath the fingers. A pan-to-pinch transition is only allowed when finger spread changes more than midpoint movement.
- Android starts with Tool Sets collapsed to preserve canvas width, but keeps a 48-point left rail that reopens the panel; the duplicate in-window desktop menu stays hidden.
- Desktop toolbar behavior is unchanged.

## Validation

- `python docs/test-step3.py`: 21/21 source-level acceptance checks passed.
- `python docs/test-step2.py`: 18/18 source-level regression checks passed.
- `python docs/test-steps4-6.py`: source-level assertions cover Android touch-up and point-prompt wiring.
- Engine source inspection confirms every listed draw/edit command exists in the imported CADCraft command registry.

## Not verified

Source-level checks do not prove runtime behavior. Rust compilation, Cargo unit tests, APK generation, APK inspection, emulator/device installation and touch interaction tests require the Android build environment and a device; those were not performed during this investigation.

## Acceptance status

- Step 3 source implementation: complete.
- Step 3 source-level checks: passed.
- Compile/build/device acceptance: pending a compatible Android build environment.

A successful source-level check is not evidence that the Android toolbar has been rendered or tested on a physical phone.
