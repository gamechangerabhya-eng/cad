#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"

resolve_ndk() {
  local candidate="${ANDROID_NDK_HOME:-${ANDROID_NDK_ROOT:-}}"
  if [[ -n "$candidate" && -d "$candidate" ]]; then
    printf '%s\n' "$candidate"
    return 0
  fi

  local sdk="${ANDROID_SDK_ROOT:-${ANDROID_HOME:-}}"
  if [[ -z "$sdk" && -f "$ROOT/local.properties" ]]; then
    sdk="$(sed -n 's/^sdk\.dir=//p' "$ROOT/local.properties" | tail -n 1 | sed 's/\\\\:/\:/g')"
  fi
  if [[ -n "$sdk" && -d "$sdk/ndk" ]]; then
    find "$sdk/ndk" -mindepth 1 -maxdepth 1 -type d -print0 \
      | xargs -0 -r -n1 printf '%s\n' \
      | sort -V \
      | tail -n 1
    return 0
  fi
  return 1
}

NDK="$(resolve_ndk || true)"
if [[ -z "$NDK" || ! -d "$NDK" ]]; then
  echo "Android NDK not found. Set ANDROID_NDK_HOME, ANDROID_NDK_ROOT, ANDROID_SDK_ROOT, or sdk.dir in local.properties." >&2
  exit 2
fi
if ! command -v cargo >/dev/null 2>&1; then
  echo "Rust/Cargo is required (vendored CADCraft declares Rust 1.95+)." >&2
  exit 2
fi
if ! cargo ndk --version >/dev/null 2>&1; then
  echo "Install cargo-ndk first: cargo install cargo-ndk" >&2
  exit 2
fi

export ANDROID_NDK_HOME="$NDK"
export ANDROID_NDK_ROOT="$NDK"
OUT="$ROOT/app/src/main/jniLibs"
mkdir -p "$OUT"
MANIFEST="$ROOT/native/arcad-ffi/Cargo.toml"
LOCK_FILE="$ROOT/native/arcad-ffi/Cargo.lock"

# Resolve once, then compile in locked mode. Commit this generated lockfile to
# make subsequent release builds reproducible.
if [[ ! -f "$LOCK_FILE" ]]; then
  if [[ "${ARCAD_REQUIRE_LOCKFILE:-0}" == "1" ]]; then
    echo "Release build requires the committed native/arcad-ffi/Cargo.lock." >&2
    echo "Run 'bash native/build-android.sh' once to resolve dependencies, review and commit the generated lockfile, then retry the release build." >&2
    exit 5
  fi
  echo "Generating native/arcad-ffi/Cargo.lock for the first development build..."
  cargo generate-lockfile --manifest-path "$MANIFEST"
fi

test -s "$LOCK_FILE" || { echo "Cargo.lock generation failed." >&2; exit 3; }
cargo ndk -t arm64-v8a -t x86_64 -o "$OUT" build \
  --manifest-path "$MANIFEST" --release --locked

for abi in arm64-v8a x86_64; do
  lib="$OUT/$abi/libarcad_ffi.so"
  test -s "$lib" || {
    echo "Rust build did not produce $lib" >&2
    exit 3
  }
  python3 "$ROOT/tools/verify-android-elf.py" "$lib" "$abi"
done

echo "CADCraft Rust Android libraries built and checked for arm64-v8a and x86_64."
