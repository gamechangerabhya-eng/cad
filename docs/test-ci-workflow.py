#!/usr/bin/env python3
from pathlib import Path

root = Path(__file__).resolve().parents[1]
workflow = (root / ".github/workflows/build.yml").read_text()
checks = {
    "version tags select release builds": "startsWith(github.ref, 'refs/tags/v') && 'release'" in workflow,
    "manual workflow mode remains selectable": "options: [debug, release]" in workflow,
    "release mode requires a persistent keystore": 'Release builds require AR_CAD_KEYSTORE_BASE64 and AR_CAD_KEYSTORE_PASSWORD' in workflow,
    "release APK is signature-verified": 'apksigner" verify --verbose --print-certs' in workflow,
    "APK package identity is checked": "package: name='com.arcad.app'" in workflow,
    "Rust tests are explicitly enabled in CI": 'RUN_RUST_TESTS: "1"' in workflow,
    "Android licence pipeline handles SIGPIPE safely": "set +o pipefail" in workflow and "set -o pipefail" in workflow,
    "cargo-ndk installation is version-pinned": "cargo install cargo-ndk --version 4.1.2 --locked" in workflow,
    "debug and release APK names are checked explicitly": 'expected="output/cadcraft-$version.apk"' in workflow and 'expected="output/cadcraft-$version-debug.apk"' in workflow,
    "all APK variants receive signature verification": 'apksigner" verify --verbose --print-certs "$apk"' in workflow,
    "release private-key entry and expected alias are validated": 'keytool -list -v -keystore keys/artcraft-builder.jks' in workflow and '-alias artcraftbuilder' in workflow and 'Entry type: PrivateKeyEntry' in workflow,
    "Rust build cache is keyed to the lockfile": "actions/cache@v6" in workflow and "hashFiles('source/cadcraft/Cargo.lock')" in workflow,
    "Rust check and APK build share one target directory": 'CARGO_TARGET_DIR: ${{ github.workspace }}/build/cadcraft/target' in workflow,
}
for label, ok in checks.items():
    print(("PASS" if ok else "FAIL") + " - " + label)
if not all(checks.values()):
    raise SystemExit(1)
print(f"{len(checks)}/{len(checks)} CI workflow checks passed.")
