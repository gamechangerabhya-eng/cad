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
    ]

    changed = []
    for relative_path, old, new, count in patches:
        path = src / relative_path
        if replace_exact(path, old, new, count):
            changed.append(relative_path)

    if changed:
        print("Applied CADCraft compile compatibility fixes:")
        for path in changed:
            print(f"  {path}")
    else:
        print("CADCraft compile compatibility fixes already present.")


if __name__ == "__main__":
    main()
