#!/usr/bin/env python3
"""Source acceptance checks for AR CAD Steps 7–10 release hardening."""
from pathlib import Path
import re
import sys

root = Path(__file__).resolve().parents[1]
checks = []
def check(label, condition):
    checks.append((label, bool(condition)))

# Step 7: save, DXF and recovery coverage must be present in the CADCraft source.
engine_tests = root / "source/cadcraft/crates/engine/tests"
ui_tests = root / "source/cadcraft/crates/ui-egui/tests"
check("Engine autosave and crash-recovery tests are included", (engine_tests / "autosave.rs").is_file() and (engine_tests / "drawing_recovery_command.rs").is_file())
check("Engine save-backup and Save As regression tests are included", (engine_tests / "save_backup.rs").is_file() and (engine_tests / "save_as_export.rs").is_file())
check("UI recovery and recent-file persistence tests are included", (ui_tests / "drawing_recovery.rs").is_file() and (ui_tests / "recent_files.rs").is_file())
check("DXF parser/writer test suite is included", (root / "source/cadcraft/crates/dxf/tests/roundtrip.rs").is_file())

# Step 8: one repeatable gate must execute all source acceptance tests and optional Rust tests.
runner = (root / "tools/test-ar-cad.sh").read_text()
check("Unified QA runner includes previous source regression suites", all(x in runner for x in ["test-step2.py", "test-step3.py", "test-steps4-6.py"]))
check("Unified QA runner includes Steps 7–10 acceptance suite", "test-steps7-10.py" in runner)
check("Unified QA runner supports full Rust workspace component tests", "RUN_RUST_TESTS" in runner and "cargo test --locked" in runner)
check("CI workflow invokes the unified QA runner", "tools/test-ar-cad.sh" in (root / ".github/workflows/build.yml").read_text())

# Step 9: verify architecture and release targets remain explicitly constrained.
workflow = (root / ".github/workflows/build.yml").read_text()
build = (root / "lib/build-common.sh").read_text()
check("Android Rust build remains ARM64-only and aligned with min SDK", 'cargo ndk -t arm64-v8a -P "$MIN_SDK"' in build)
check("Android workflow installs the ARM64 Rust target", "aarch64-linux-android" in workflow)
check("Release profile enables thin LTO", "lto = \"thin\"" in (root / "source/cadcraft/Cargo.toml").read_text())
check("APK package identity remains com.arcad.app", "com.arcad.app" in workflow and "com.arcad.app" in (root / "build-ar-cad-release.sh").read_text())

# Step 10: releases must not silently rotate their signing key on every cloud build.
check("Release workflow requires persistent signing secrets", "AR_CAD_KEYSTORE_BASE64" in workflow and "AR_CAD_KEYSTORE_PASSWORD" in workflow)
check("Workflow defaults to a debug build until release signing is configured", re.search(r"default:\s*debug", workflow) is not None)
check("Release verification checks APK signature", "apksigner\" verify" in workflow)
check("Release artifact checksum is generated", "sha256sum" in workflow)
check("Release checklist clearly blocks public release until device tests pass", "physical-device" in (root / "docs/AR-CAD-RELEASE-CHECKLIST.md").read_text().lower())

for label, ok in checks:
    print(("PASS" if ok else "FAIL") + " - " + label)
print(f"\n{sum(ok for _, ok in checks)}/{len(checks)} Steps 7–10 source checks passed.")
sys.exit(0 if all(ok for _, ok in checks) else 1)
