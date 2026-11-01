#!/usr/bin/env python3
"""Apply guarded Android-builder compatibility fixes to the CADCraft checkout."""

from pathlib import Path
import sys


def replace_exact(path: Path, old: str, new: str, expected_count: int = 1) -> bool:
    text = path.read_text(encoding="utf-8")
    old_count = text.count(old)
    new_count_outside_old = text.replace(old, "").count(new)
    if old_count == expected_count and new_count_outside_old == 0:
        path.write_text(text.replace(old, new), encoding="utf-8")
        return True
    if old_count == 0 and new_count_outside_old == expected_count:
        return False
    raise RuntimeError(
        f"Unexpected CADCraft source in {path}: expected {expected_count} old occurrence(s), "
        f"found {old_count} old and {new_count_outside_old} patched occurrence(s); "
        "refusing an unverified patch."
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
    ]

    changed = []
    for relative_path, old, new, count in patches:
        path = src / relative_path
        if replace_exact(path, old, new, count):
            changed.append(relative_path)

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
