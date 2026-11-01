#!/usr/bin/env python3
"""Source-level acceptance checks for AR CAD Step 3 Android drafting controls."""
from pathlib import Path
import sys

root = Path(__file__).resolve().parents[1]
chrome = (root / "source/cadcraft/crates/ui-egui/src/chrome.rs").read_text()
menus = (root / "source/cadcraft/crates/ui-egui/src/menus.rs").read_text()
engine = root / "source/cadcraft/crates/engine/src/cmd"
checks = []

def check(name, condition):
    checks.append((name, bool(condition)))

check("Android drawing toolbar reserves space for four touch-first rows", "ANDROID_DRAFTING_BAR_HEIGHT: f32 = 176.0" in chrome and '("TOOLS", ANDROID_INSPECT_TOOLS)' in chrome)
check("Android inspection action array is declared with valid tool actions", "const ANDROID_INSPECT_TOOLS: &[(&str, &str)]" in chrome and all(f'("{label}", "{command}")' in chrome for label, command in [("Distance", "dist"), ("Area", "area"), ("List", "list"), ("Audit", "audit"), ("Layers", "ui.dialog.layers"), ("Properties", "ui.toggle.palettes"), ("Blocks", "ui.dialog.blocks")]))
check("Touch controls use 72x36 point targets", "const ANDROID_DRAFTING_BUTTON: [f32; 2] = [72.0, 36.0]" in chrome)
check("File, Draw, Edit and Tools rows are horizontally scrollable", chrome.count("egui::ScrollArea::horizontal()") >= 2 and all(x in chrome for x in ["ANDROID_FILE_TOOLS", "ANDROID_DRAW_TOOLS", "ANDROID_EDIT_TOOLS", "ANDROID_INSPECT_TOOLS"]))
check("Android toolbar is only shown while a drawing is open", "if !app.ui.start_tab" in chrome and "android_drafting_toolbar(ui, row)" in chrome)
check("Toolbar commands dispatch through CadApp command router", "app.start(command)" in chrome)
check("Widget accessibility labels are recorded", 'record_widget(ui, &response, label, "button", None)' in chrome)

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
