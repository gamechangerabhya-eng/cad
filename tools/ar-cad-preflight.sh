#!/usr/bin/env bash
set -u
ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
missing=0
check_cmd() {
  local name="$1"
  if command -v "$name" >/dev/null 2>&1; then
    printf 'OK      %-16s %s\n' "$name" "$(command -v "$name")"
  else
    printf 'MISSING %-16s\n' "$name"
    missing=1
  fi
}
for cmd in git cargo rustc rustup javac java; do check_cmd "$cmd"; done
if command -v cargo >/dev/null 2>&1; then
  cargo ndk --version >/dev/null 2>&1 && echo 'OK      cargo-ndk' || { echo 'MISSING cargo-ndk'; missing=1; }
fi
sdk="${ANDROID_HOME:-${ANDROID_SDK_ROOT:-}}"
if [[ -n "$sdk" && -d "$sdk" ]]; then
  echo "OK      Android SDK     $sdk"
  [[ -d "$sdk/platforms/android-35" ]] && echo 'OK      SDK platform    android-35' || { echo 'MISSING SDK platform android-35'; missing=1; }
  [[ -d "$sdk/build-tools" ]] && echo 'OK      SDK build-tools' || { echo 'MISSING SDK build-tools'; missing=1; }
else
  echo 'MISSING Android SDK (ANDROID_HOME / ANDROID_SDK_ROOT)'
  missing=1
fi
ndk="${ANDROID_NDK_HOME:-${ANDROID_NDK_ROOT:-}}"
if [[ -n "$ndk" && -d "$ndk" ]]; then echo "OK      Android NDK     $ndk"; else echo 'MISSING Android NDK (ANDROID_NDK_HOME / ANDROID_NDK_ROOT)'; missing=1; fi
if [[ -f "$ROOT/source/cadcraft/Cargo.toml" ]]; then
  grep -q 'version = "0.15.1"' "$ROOT/source/cadcraft/Cargo.toml" && echo 'OK      CADCraft        0.15.1' || { echo 'FAIL    CADCraft version mismatch'; missing=1; }
else
  echo 'FAIL    CADCraft source missing'
  missing=1
fi
if [[ "$missing" -ne 0 ]]; then
  echo
  echo 'Build preflight failed. Install the missing toolchain components before invoking the APK build.'
  exit 1
fi
echo 'Build preflight passed.'
