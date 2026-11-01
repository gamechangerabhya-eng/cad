#!/usr/bin/env python3
"""Static contract check: public C header declarations must have Rust exports."""
from pathlib import Path
import re

root = Path(__file__).resolve().parent
header = (root / "include/arcad_ffi.h").read_text()
rust = (root / "arcad-ffi/src/lib.rs").read_text()
# Match exported C function names declared in the header.
declared = set(re.findall(r"\b(arcad_[a-zA-Z0-9_]+)\s*\(", header))
exported = set(re.findall(r"fn\s+(arcad_[a-zA-Z0-9_]+)\s*\(", rust))
missing = sorted(declared - exported)
extra = sorted(exported - declared)
if missing or extra:
    raise SystemExit(f"FFI contract mismatch; missing={missing}, undeclared={extra}")
upstream = root.parent / "third_party/cadcraft-main/cadcraft-main"
for crate in ("doc", "engine", "io"):
    manifest = upstream / "crates" / crate / "Cargo.toml"
    if not manifest.is_file():
        raise SystemExit(f"missing vendored CADCraft crate: {manifest}")
print(f"FFI contract passed: {len(declared)} C functions have matching Rust exports")
print("Vendored CADCraft doc/engine/io crate paths verified")
