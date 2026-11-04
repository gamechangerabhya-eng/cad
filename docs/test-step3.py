#!/usr/bin/env python3
"""Source-level acceptance checks for AR CAD Step 3 Android drafting controls."""
from pathlib import Path
import sys

root = Path(__file__).resolve().parents[1]
chrome = (root / "source/cadcraft/crates/ui-egui/src/chrome.rs").read_text()
scrolling = (root / "lib/scrolling.sh").read_text()
menus = (root / "source/cadcraft/crates/ui-egui/src/menus.rs").read_text()
engine = root / "source/cadcraft/crates/engine/src/cmd"
mobile = (root / "templates/android/cadcraft_mobile_ui.rs").read_text(encoding="utf-8")
checks = []

def check(name, condition):
    checks.append((name, bool(condition)))

check("Android drawing toolbar reserves space for four touch-first rows", "ANDROID_DRAFTING_BAR_HEIGHT: f32 = 176.0" in chrome and '("TOOLS", ANDROID_INSPECT_TOOLS)' in chrome)
check("Android inspection action array is declared with valid tool actions", "const ANDROID_INSPECT_TOOLS: &[(&str, &str)]" in chrome and all(f'("{label}", "{command}")' in chrome for label, command in [("Distance", "dist"), ("Area", "area"), ("List", "list"), ("Audit", "audit"), ("Layers", "ui.dialog.layers"), ("Properties", "ui.toggle.palettes"), ("Blocks", "ui.dialog.blocks")]))
check("Upstream toolbar declares base dimensions before the Android builder enlarges them", "const ANDROID_DRAFTING_BUTTON: [f32; 2] = [72.0, 36.0]" in chrome)
check("File, Draw, Edit and Tools rows are horizontally scrollable", chrome.count("egui::ScrollArea::horizontal()") >= 2 and all(x in chrome for x in ["ANDROID_FILE_TOOLS", "ANDROID_DRAW_TOOLS", "ANDROID_EDIT_TOOLS", "ANDROID_INSPECT_TOOLS"]))
check("Android toolbar is only shown while a drawing is open", "if !app.ui.start_tab" in chrome and "android_drafting_toolbar(ui, row)" in chrome)
check("Toolbar commands dispatch through CadApp command router", "app.start(command)" in chrome)
check("Widget accessibility labels are recorded", 'record_widget(ui, &response, label, "button", None)' in chrome)
check("Android build adds a Tool Sets button to the touch tools row", '("Tool Sets", "ui.toggle.toolsets")' in scrolling and "add Tool Sets to the Android touch toolbar" in scrolling)
check("Compact app bar uses fixed action slots and moves Save out of the row", "APP_BAR_COMPACT_BREAKPOINT: f32 = 420.0" in mobile and "fn title_width(self, remaining_width: f32)" in mobile and "if metrics.show_save" in mobile)
check("Document title truncates instead of pushing app-bar actions", "Label::new(RichText::new(&document.title)" in mobile and ".truncate()" in mobile and "metrics.title_width(ui.available_width())" in mobile)
check("Selection actions use fixed-height buttons inside a taller strip", "SELECTION_BAR_HEIGHT: f32 = 60.0" in mobile and "SELECTION_ACTION_HEIGHT: f32 = 40.0" in mobile and "selection_command_button(ui, app, item, state, t)" in mobile)
check("More sheet shortcuts use width-aware paired tiles", "fn more_action_tile_width(available_width: f32)" in mobile and "ui.horizontal_wrapped(|ui|" in mobile and "more_action_button(ui, \"Command history\"" in mobile)
check("More sheet shows the all-command list only after search", "(state.category != Category::More || searching)" in mobile and "Search every CAD command" in mobile)

for command in ["new", "open", "qsave", "saveas", "line", "pline", "circle", "rectang", "arc", "ellipse", "polygon", "text", "mtext", "hatch", "dimlinear", "dist", "area", "list", "audit"]:
    found = any(f'CommandSpec::new("{command}"' in p.read_text(errors="ignore") for p in engine.rglob("*.rs"))
    check(f"Mobile action is present in CADCraft engine: {command}", found)
for command in ["move", "copy", "rotate", "scale", "offset", "trim", "extend", "erase"]:
    found = any(f'CommandSpec::new("{command}"' in p.read_text(errors="ignore") for p in engine.rglob("*.rs"))
    check(f"Modify command is present in CADCraft engine: {command}", found)
for command in ["ui.dialog.layers", "ui.dialog.blocks", "ui.toggle.palettes"]:
    check(f"Android inspect action is backed by a UI command: {command}", command in menus and command in chrome)

for name, ok in checks:
    print(("PASS" if ok else "FAIL") + " - " + name)
print(f"\n{sum(ok for _, ok in checks)}/{len(checks)} Step 3 source checks passed.")
sys.exit(0 if all(ok for _, ok in checks) else 1)
