#!/usr/bin/env python3
"""Source acceptance checks for AR CAD Steps 4–6 mobile drafting improvements."""
from pathlib import Path
import sys
root=Path(__file__).resolve().parents[1]
chrome=(root/'source/cadcraft/crates/ui-egui/src/chrome.rs').read_text()
palettes=(root/'source/cadcraft/crates/ui-egui/src/palettes.rs').read_text()
cmdline=(root/'source/cadcraft/crates/ui-egui/src/cmdline.rs').read_text()
canvas=(root/'source/cadcraft/crates/ui-egui/src/canvas.rs').read_text()
scrolling=(root/'lib/scrolling.sh').read_text()
checks=[]
def check(label, ok): checks.append((label,bool(ok)))
# Step 4 — touch drafting/navigation
check('Android build provides a 56-point status strip for 48-point controls', 'exact_size(56.0)' in scrolling and '[72.0, 48.0]' in scrolling)
check('Grid, Snap, Ortho, Polar, OSnap and Dynamic Input toggles exist', all(f'("{x}"' in chrome for x in ['Grid','Snap','Ortho','Polar','OSnap','Input']))
check('Android drafting toggles dispatch existing transparent commands', 'run_transparent(app, command)' in chrome)
check('Zoom Fit control invokes engine Zoom Extents', 'app.session.zoom_extents()' in chrome and '"Zoom Fit"' in chrome)
check('Pinch zoom remains anchored to the gesture focal point', 'crate::animation::zoom_delta' in canvas and 'zoom_about(f64::from(pinch), nav.xf.to_world(at))' in canvas)
android_winit=(root/'vendor/winit/src/platform_impl/android/mod.rs').read_text()
check('Android pinch zoom tracks the live two-finger midpoint', 'GestureStep::Pinch { factor, center }' in android_winit and 'position: center' in android_winit)
check('Android pinch recognition has a short deadzone and avoids pan-to-pinch misclassification', 'const PINCH_DEADZONE_PX: f64 = 12.0;' in android_winit and 'spread_change > PINCH_SPREAD_RATIO * moved' in android_winit)
check('Canvas pan remains supported', 'handle_button_navigation(app, nav, inside)' in canvas)
check('Android two-finger pixel scrolls are patched to pan, not mouse-wheel zoom', 'cfg!(any(target_os = \"macos\", target_os = \"android\"))' in scrolling and 'Android two-finger slides pan instead of zoom' in scrolling)
check('CADCraft reads Android touch-up position from egui Touch::End events', 'egui::Event::Touch { phase: egui::TouchPhase::End, pos, .. } => Some(*pos)' in scrolling and 'hover_pos: i.pointer.hover_pos().or(touch_end_pos)' in scrolling)
check('CADCraft canvas hit-testing recognizes a released touch location', 'nav.response.hovered() || touch_end_inside' in scrolling)
check('CADCraft point prompts accept a paired Android touch-up release', 'touch_end_primary_release: touch_end_pos.is_some()' in scrolling and 'point_prompt && context.touch_end_primary_release' in scrolling)
check('Android keeps the desktop menu hidden but lets the left Tool Sets rail reopen the panel', 'hide the desktop menu bar on Android' in scrolling and 'allow the left Tool Sets rail to reopen the panel' in scrolling and 'app.ui.show_toolsets)?;' in scrolling)
check('Android Tool Sets is available in both the touch toolbar and left rail', '("Tool Sets", "ui.toggle.toolsets")' in scrolling and '48.0 } else { 22.0' in scrolling and 'app.run("ui.toggle.toolsets", json!({"on": true}))' in palettes)
check('Tool Sets open failures are echoed to command history', 'app.session.echo(error);' in scrolling and 'report Tool Sets open errors' in scrolling)
check('CADCraft Android provides explicit touch zoom in/out controls', '(\"Zoom -\", 0.8), (\"Zoom +\", 1.25)' in scrolling and 'add touch zoom controls' in scrolling)
check('Android drafting toolbar buttons are patched to 48-point touch targets', 'ANDROID_DRAFTING_BUTTON: [f32; 2] = [84.0, 48.0]' in scrolling and 'ANDROID_DRAFTING_BAR_HEIGHT: f32 = 212.0' in scrolling)
check('Android status and zoom buttons use 48-point touch targets', 'exact_size(56.0)' in scrolling and '[72.0, 48.0]' in scrolling and '[84.0, 48.0]' in scrolling)
# Step 5 — mobile layer/property controls
check('Android palettes use a narrower 260-point width', 'if cfg!(target_os = "android") { 260.0 } else { 300.0 }' in palettes)
check('Android palette interactive controls have 40-point minimum size', 'interact_size = vec2(40.0, 40.0)' in palettes)
check('Layer and Properties sections remain in the mobile palette', 'layers_section(app, ui)' in palettes and 'properties_section(app, ui)' in palettes)
check('Properties editing continues through CAD command dispatcher', 'app.run(c, p)' in palettes)
# Step 6 — file and command workflows
check('Android toolbar exposes New, Open, Save and Save As', all(f'("{x}"' in chrome for x in ['New','Open','Save','Save As']))
check('Android command bar uses almost the full canvas width', '(canvas.width() - 16.0).max(0.0)' in cmdline)
check('Android command bar has a larger 38-point height', 'let h = if android { 38.0 } else { 24.0 };' in cmdline)
check('Android command prompt keywords have expanded touch hit areas', 'let keyword_height = if android { 30.0 }' in cmdline)
for label,ok in checks: print(('PASS' if ok else 'FAIL')+' - '+label)
print(f'\n{sum(ok for _,ok in checks)}/{len(checks)} Step 4–6 source checks passed.')
sys.exit(0 if all(ok for _,ok in checks) else 1)
