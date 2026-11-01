#!/usr/bin/env python3
"""Apply guarded Android-builder compatibility fixes to the CADCraft checkout."""

from pathlib import Path
import sys


def replace_exact(path: Path, old: str, new: str, expected_count: int = 1) -> bool:
    text = path.read_text(encoding="utf-8")
    old_count = text.count(old)
    new_count = text.count(new)
    new_count_outside_old = text.replace(old, "").count(new)
    if old_count == expected_count and new_count_outside_old == 0:
        if old in new and new_count == expected_count:
            return False
        path.write_text(text.replace(old, new), encoding="utf-8")
        return True
    if old_count == 0 and new_count_outside_old == expected_count:
        return False
    raise RuntimeError(
        f"Unexpected CADCraft source in {path}: expected {expected_count} old occurrence(s), "
        f"found {old_count} old and {new_count_outside_old} patched occurrence(s); "
        "refusing an unverified patch."
    )


def remove_exact(path: Path, old: str, marker: str) -> bool:
    text = path.read_text(encoding="utf-8")
    old_count = text.count(old)
    if old_count == 1:
        updated = text.replace(old, "", 1)
        if marker in updated:
            raise RuntimeError(f"Removing the verified block from {path} left its marker behind.")
        path.write_text(updated, encoding="utf-8")
        return True
    if old_count == 0 and marker not in text:
        return False
    raise RuntimeError(
        f"Unexpected CADCraft source in {path}: expected one verified block, "
        f"found {old_count}; refusing an unverified removal."
    )


def remove_duplicate_test(path: Path, name: str, duplicate: str) -> bool:
    text = path.read_text(encoding="utf-8")
    declaration = f"    fn {name}() {{"
    declaration_count = text.count(declaration)
    if declaration_count == 2 and text.count(duplicate) == 1:
        updated = text.replace(duplicate, "", 1)
        if updated.count(declaration) != 1:
            raise RuntimeError(f"Removing duplicate test {name} did not leave exactly one definition.")
        path.write_text(updated, encoding="utf-8")
        return True
    if declaration_count == 1 and duplicate not in text:
        return False
    raise RuntimeError(
        f"Unexpected CADCraft source in {path}: expected two copies of {name} "
        f"with the known duplicate body, found {declaration_count} definitions."
    )


def main() -> None:
    if len(sys.argv) != 2:
        raise SystemExit(f"usage: {Path(sys.argv[0]).name} CADCRAFT_SOURCE")

    src = Path(sys.argv[1])
    patches = [
        (
            "crates/geom/src/intersect.rs",
            "use crate::{Arc, Bounds2, Circle, Ellipse, Line, Segment, Spline, ToleranceContext, Vec2, angle_in_sweep};",
            "use crate::{Bounds2, Circle, Ellipse, Line, Segment, Spline, ToleranceContext, Vec2, angle_in_sweep};",
            1,
        ),
        (
            "crates/engine/src/cmd/modify.rs",
            "ccw_sweep, intersect_ext, intersect_segments_with_tolerance, line_line_infinite, "
            "line_line_infinite_with_tolerance, norm_angle,",
            "ccw_sweep, intersect_segments_with_tolerance, line_line_infinite_with_tolerance, norm_angle,",
            1,
        ),
        (
            "crates/engine/src/cmd/modify2.rs",
            "ccw_sweep, intersect_ext, line_circle, line_line_infinite, norm_angle,",
            "ccw_sweep, intersect_ext, norm_angle,",
            1,
        ),
        (
            "crates/engine/src/cmd/modify2.rs",
            "use super::curves::{self, Chain, MAX_GEN, TanObj};",
            "use super::curves::{self, Chain, MAX_GEN};",
            1,
        ),
        (
            "crates/engine/src/cmd/modify2/fillet.rs",
            "curves, line, other, push_entity, set_kind,",
            "curves, line, other, set_kind,",
            1,
        ),
        (
            "crates/engine/src/snap.rs",
            "use cadcraft_geom::{Bounds2, Circle, EPS, Ellipse,",
            "use cadcraft_geom::{Bounds2, Circle, Ellipse,",
            1,
        ),
        (
            "crates/engine/src/cmd/draw2/solid3d.rs",
            '.enabled(|_| true),',
            '.enabled(|_| Ok(())),',
            3,
        ),
        (
            "crates/engine/src/cmd/draw2/solid3d.rs",
            'format!("cannot create solid: {e}")',
            'format!("cannot create solid: {e:?}")',
            1,
        ),
        (
            "crates/engine/src/cmd/draw2/solid3d.rs",
            'format!("invalid solid topology: {e}")',
            'format!("invalid solid topology: {e:?}")',
            1,
        ),
        (
            "crates/engine/src/cmd/modify/break_join.rs",
            "EntityKind::Circle(c) => c.bounds(),",
            "EntityKind::Circle(c) => Bounds2::new(\n"
            "            c.center.xy() - Vec2::new(c.radius, c.radius),\n"
            "            c.center.xy() + Vec2::new(c.radius, c.radius),\n"
            "        ),",
            1,
        ),
        (
            "crates/engine/src/cmd/overkill.rs",
            "let wrapped: Vec<&LineRec> = wrapped_records.iter().collect();\n"
            "            same_direction(&wrapped, cands, o, gone, changed, rep);",
            "same_direction(&wrapped_records, cands, o, gone, changed, rep);",
            1,
        ),
        (
            "crates/ui-egui/tests/save_loss.rs",
            "*app.session.doc_mut().unwrap().foreign = std::sync::Arc::new(foreign);",
            "app.session.doc_mut().unwrap().foreign = std::sync::Arc::new(foreign);",
            1,
        ),
        (
            "crates/ui-egui/tests/open_bytes.rs",
            "larger than the 256 MB CADCraft opens in the browser",
            "larger than the 256 MB AR CAD opens in the browser",
            2,
        ),
        (
            "crates/ui-egui/src/i18n/tests.rs",
            '("about", "About CADCraft"),',
            '("about", "About AR CAD"),',
            1,
        ),
        (
            "crates/ui-egui/src/menus.rs",
            "crate::updates_ui::check_now(app);",
            '#[cfg(not(target_os = "android"))] crate::updates_ui::check_now(app);',
            1,
        ),
        (
            "crates/ui-egui/src/lib.rs",
            "pub updates_rt: updates_ui::Runtime,",
            '#[cfg(not(target_os = "android"))] pub updates_rt: updates_ui::Runtime,',
            1,
        ),
        (
            "crates/ui-egui/src/lib.rs",
            "updates_rt: updates_ui::Runtime::default(),",
            '#[cfg(not(target_os = "android"))] updates_rt: updates_ui::Runtime::default(),',
            1,
        ),
        (
            "crates/ui-egui/src/lib.rs",
            "updates_ui::tick(self, ctx);",
            '#[cfg(not(target_os = "android"))] updates_ui::tick(self, ctx);',
            1,
        ),
    ]

    changed = []
    for relative_path, old, new, count in patches:
        path = src / relative_path
        if replace_exact(path, old, new, count):
            changed.append(relative_path)

    # These four fork patches are serialized as unused in Cargo.lock and are not dependencies
    # of any CADCraft workspace manifest. Remove only those exact stale entries.
    cargo_manifest = src / "Cargo.toml"
    cargo_lock = src / "Cargo.lock"
    egui_revision = "6d518fc799681571963842f95b55a6b68431b15a"
    egui_source = f"git+https://github.com/storytold/egui.git?rev={egui_revision}#{egui_revision}"
    for crate in ("egui_demo_lib", "egui_extras", "egui_inspection", "egui_system_font_provider"):
        manifest_entry = (
            f'{crate} = {{ git = "https://github.com/storytold/egui.git", rev = "{egui_revision}" }}\n'
        )
        if remove_exact(cargo_manifest, manifest_entry, f"{crate} = "):
            changed.append("Cargo.toml ([patch.crates-io])")

        lock_entry = (
            "[[patch.unused]]\n"
            f'name = "{crate}"\n'
            'version = "0.37.0"\n'
            f'source = "{egui_source}"\n'
        )
        if remove_exact(cargo_lock, lock_entry, f'name = "{crate}"'):
            changed.append("Cargo.lock ([[patch.unused]])")

    duplicate_test_path = src / "crates/engine/src/cmd/draw2/solid3d.rs"
    duplicate_test = (
        "    #[test]\n"
        "    fn grouped_extrusion_info_rejects_duplicate_face_indices() {\n"
        "        let mut s = Session::new();\n"
        "        let result = s.execute(\"extrude\", &json!({\"points\": [[0,0],[2,0],[2,2],[0,2]], \"height\": 3.0})).unwrap();\n"
        "        let handles = result[\"handles\"].as_array().unwrap();\n"
        "        let first = Handle::parse_hex(handles[0].as_str().unwrap()).unwrap();\n"
        "        let second = Handle::parse_hex(handles[1].as_str().unwrap()).unwrap();\n"
        "        let first_index = s.doc().unwrap().entity(first).unwrap().common.xdata.iter()\n"
        "            .filter(|tag| tag.code == 1071).next().unwrap().value.clone();\n"
        "        s.doc_mut().unwrap().modify_entity(second, move |entity| {\n"
        "            let mut seen = false;\n"
        "            for tag in &mut entity.common.xdata {\n"
        "                if tag.code == 1071 {\n"
        "                    if !seen { tag.value = first_index.clone(); break; }\n"
        "                    seen = true;\n"
        "                }\n"
        "            }\n"
        "        }).unwrap();\n"
        "        let info = s.execute(\"solidinfo\", &json!({\"handle\": first.hex()})).unwrap();\n"
        "        assert_eq!(info[\"metadataValid\"], false);\n"
        "        assert_eq!(info[\"valid\"], false);\n"
        "        assert!(s.execute(\"solideditdelete\", &json!({\"handle\": first.hex()})).is_err());\n"
        "    }\n\n"
    )
    if remove_duplicate_test(duplicate_test_path, "grouped_extrusion_info_rejects_duplicate_face_indices", duplicate_test):
        changed.append("crates/engine/src/cmd/draw2/solid3d.rs (duplicate test)")

    if changed:
        print("Applied CADCraft compile compatibility fixes:")
        for path in changed:
            print(f"  {path}")
    else:
        print("CADCraft compile compatibility fixes already present.")


if __name__ == "__main__":
    main()
