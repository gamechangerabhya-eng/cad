#!/usr/bin/env bash
# Install the builder-owned CADCraft Android workspace into the upstream checkout at build time.
# The checked-out source is never committed or edited manually; this guarded patch is reapplied
# from templates/android/cadcraft_mobile_ui.rs for each Android build.
set -euo pipefail

src=${1:?usage: cadcraft-mobile-ui.sh <source-dir> <app>}
app=${2:?usage: cadcraft-mobile-ui.sh <source-dir> <app>}
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

[[ "$app" == "cadcraft" ]] || exit 0

python3 - "$root" "$src" <<'PY'
from pathlib import Path
import sys

root, src = Path(sys.argv[1]), Path(sys.argv[2])
ui_src = src / "crates" / "ui-egui" / "src"
lib = ui_src / "lib.rs"
template = root / "templates" / "android" / "cadcraft_mobile_ui.rs"
mobile = ui_src / "android_mobile.rs"

if not lib.is_file():
    raise SystemExit(f"CADCraft UI crate is missing: {lib}")
if not template.is_file():
    raise SystemExit(f"Builder-owned Android UI template is missing: {template}")

lib_text = lib.read_text(encoding="utf-8")
module_decl = "pub mod android_mobile;\n"
if module_decl not in lib_text:
    anchor = "pub mod aliasdlg;\n"
    if lib_text.count(anchor) != 1:
        raise SystemExit("Unexpected CADCraft UI module declarations; refusing to install the Android workspace.")
    lib_text = lib_text.replace(anchor, module_decl + anchor, 1)

mobile_branch = '        if cfg!(target_os = "android") {\n            android_mobile::draw(self, ui);\n            return;\n        }\n'
draw_anchor = "    fn draw(&mut self, ui: &mut egui::Ui) {\n        let t0 = now_ms();\n"
if mobile_branch not in lib_text:
    if lib_text.count(draw_anchor) != 1:
        raise SystemExit("Unexpected CADCraft draw entry point; refusing to install the Android workspace.")
    lib_text = lib_text.replace(
        draw_anchor,
        "    fn draw(&mut self, ui: &mut egui::Ui) {\n" + mobile_branch + "        let t0 = now_ms();\n",
        1,
    )

mobile.write_text(template.read_text(encoding="utf-8"), encoding="utf-8")
lib.write_text(lib_text, encoding="utf-8")
print("  Installed builder-owned touch-first CADCraft Android workspace.")
PY
