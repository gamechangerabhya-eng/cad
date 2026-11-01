#!/usr/bin/env python3
"""Exercise the Android Rust build script using a fake cargo/cargo-ndk tool."""
from pathlib import Path
import os
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "native" / "build-android.sh"
subprocess.run(["bash", "-n", str(SCRIPT)], check=True)

with tempfile.TemporaryDirectory(prefix="arcad-native-build-test-") as temp:
    root = Path(temp)
    (root / "native/arcad-ffi").mkdir(parents=True)
    (root / "app/src/main").mkdir(parents=True)
    (root / "tools").mkdir(parents=True)
    (root / "tools/verify-android-elf.py").write_bytes((ROOT / "tools/verify-android-elf.py").read_bytes())
    (root / "native/build-android.sh").write_bytes(SCRIPT.read_bytes())
    ndk = root / "fake-ndk"
    ndk.mkdir()
    sdk = root / "fake-sdk"
    (sdk / "ndk/27.2.12479018").mkdir(parents=True)
    bindir = root / "bin"
    bindir.mkdir()
    log = root / "cargo-args.log"
    cargo = bindir / "cargo"
    cargo.write_text("""#!/usr/bin/env bash
set -euo pipefail
if [[ "${1:-}" == ndk && "${2:-}" == --version ]]; then echo 'cargo-ndk fake'; exit 0; fi
printf '%s\\n' "$*" >> "$CARGO_TEST_LOG"
if [[ "${1:-}" == generate-lockfile ]]; then
  while (($#)); do if [[ "$1" == --manifest-path ]]; then manifest="$2"; shift 2; else shift; fi; done
  mkdir -p "$(dirname "$manifest")"
  printf '# generated test lockfile\\n' > "$(dirname "$manifest")/Cargo.lock"
  exit 0
fi
out=''
while (($#)); do
  if [[ "$1" == -o ]]; then out="$2"; shift 2; else shift; fi
done
[[ -n "$out" ]]
mkdir -p "$out/arm64-v8a" "$out/x86_64"
python3 - "$out" <<'PY'
import pathlib, struct, sys
root = pathlib.Path(sys.argv[1])
for abi, machine in (("arm64-v8a", 183), ("x86_64", 62)):
    header = bytearray(64)
    header[:4] = bytes((127, 69, 76, 70))
    header[4:7] = bytes((2, 1, 1))
    struct.pack_into("<HH", header, 16, 3, machine)
    (root / abi / "libarcad_ffi.so").write_bytes(header)
PY
""")
    cargo.chmod(0o755)

    env = os.environ.copy()
    env.pop("ANDROID_NDK_HOME", None)
    env.pop("ANDROID_NDK_ROOT", None)
    env.update({"ANDROID_SDK_ROOT": str(sdk), "PATH": f"{bindir}:{env['PATH']}", "CARGO_TEST_LOG": str(log)})
    first = subprocess.run(["bash", str(root / "native/build-android.sh")], env=env, text=True, capture_output=True)
    assert first.returncode == 0, (first.stdout, first.stderr)
    assert "Generating native/arcad-ffi/Cargo.lock" in first.stdout
    assert (root / "native/arcad-ffi/Cargo.lock").is_file()
    first_args = log.read_text().splitlines()[-1]
    assert "--locked" in first_args, first_args
    assert "arm64-v8a" in first_args and "x86_64" in first_args

    log.write_text("")
    second = subprocess.run(["bash", str(root / "native/build-android.sh")], env=env, text=True, capture_output=True)
    assert second.returncode == 0, (second.stdout, second.stderr)
    assert "generate-lockfile" not in log.read_text()
    assert "--locked" in log.read_text()

    # Release mode must refuse to resolve an unpinned dependency graph.
    lock = root / "native/arcad-ffi/Cargo.lock"
    lock.unlink()
    env["ARCAD_REQUIRE_LOCKFILE"] = "1"
    third = subprocess.run(["bash", str(root / "native/build-android.sh")], env=env, text=True, capture_output=True)
    assert third.returncode == 5, (third.stdout, third.stderr)
    assert "requires the committed" in third.stderr

print("Native Android build-script tests passed: SDK NDK discovery, bootstrap lockfile, locked ABI builds and fail-closed release lock gate")
