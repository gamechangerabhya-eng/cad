#!/usr/bin/env bash
set -Eeuo pipefail
ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT"
exec bash "$ROOT/build.sh" -cadcraft \
  -appname="AR CAD" \
  -packagename="com.arcad.app" \
  -appicon="$ROOT/branding/ar-cad-icon.png" "$@"
