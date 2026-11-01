#!/usr/bin/env python3
"""Source-level acceptance checks for AR CAD Android UX step 2."""
from pathlib import Path
import tomllib
ROOT = Path(__file__).resolve().parents[1]
CAD = ROOT / "source" / "cadcraft"
checks = []
def check(name, condition):
    checks.append((name, bool(condition)))
chrome = (CAD / "crates/ui-egui/src/chrome.rs").read_text()
ui_lib = (CAD / "crates/ui-egui/src/lib.rs").read_text()
main = (CAD / "apps/cadcraft/src/main.rs").read_text()
dialogs = (CAD / "crates/ui-egui/src/dialogs.rs").read_text()
autosave = (CAD / "crates/ui-egui/src/autosave.rs").read_text()
menus = (CAD / "crates/ui-egui/src/menus.rs").read_text()
app_cargo = tomllib.loads((CAD / "apps/cadcraft/Cargo.toml").read_text())
check("AR CAD brand constant", 'const APP_BRAND: &str = "AR CAD";' in chrome)
check("Responsive start page breakpoint", "MOBILE_START_BREAKPOINT: f32 = 760.0" in chrome and "mobile_start_page(app, ui)" in chrome)
check("Large touch start actions", all(label in chrome for label in ["New drawing", "Open drawing", "New metric drawing", "Open sample"]))
check("Recent drawings list", "Recent drawings" in chrome and "ScrollArea::vertical()" in chrome)
check("Touch toolbar target", "ANDROID_TOOLBAR_BUTTON: f32 = 42.0" in chrome and "if android { 52.0 } else { 34.0 }" in chrome)
check("Android canvas-first defaults", 'show_toolsets: !cfg!(target_os = "android")' in ui_lib and 'show_palettes: !cfg!(target_os = "android")' in ui_lib)
check("Android app ID", 'const APP_ID: &str = "com.arcad.app";' in main)
check("Native app title branding", '.with_title("AR CAD")' in main and '"AR CAD",\n        options' in main)
check("Android phone viewport sizing", 'if android { [320.0, 480.0] } else { [900.0, 560.0] }' in main)
check("Upstream self-updater disabled on Android", '#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]\n    if let Some(done) = craft_update::self_update::finish' in main)
check("Upstream update prompt disabled on Android", '#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]\n        updates_ui::show(self, ui.ctx());' in ui_lib)
check("Android app icon installed", (CAD / "assets/app-icon/cadcraft-256.png").is_file())
check("Android default unit test present", "android_starts_with_a_canvas_first_workspace" in ui_lib)
check("Responsive layout unit test present", "start_page_uses_touch_layout_only_at_narrow_widths" in chrome)
check("About screen uses AR CAD identity", "About AR CAD" in dialogs and 'crate::tl!("AR CAD")' in dialogs)
check("Menu labels use AR CAD identity", "About AR CAD" in menus and "Quit AR CAD" in menus)
check("Recovery notices use AR CAD identity", "AR CAD did not close normally." in autosave)
check("App package manifest parses", app_cargo.get("package", {}).get("name") == "cadcraft")
for name, ok in checks:
    print(("PASS" if ok else "FAIL") + " - " + name)
print(f"\n{sum(ok for _, ok in checks)}/{len(checks)} source-level checks passed.")
raise SystemExit(0 if all(ok for _, ok in checks) else 1)
