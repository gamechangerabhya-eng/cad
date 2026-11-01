#!/usr/bin/env python3
"""Check Kotlin external declarations are implemented by the JNI C++ adapter."""
from pathlib import Path
import re

root = Path(__file__).resolve().parents[1]
kotlin = (root / "app/src/main/java/com/arcad/app/NativeCadBridge.kt").read_text()
cpp = (root / "app/src/main/cpp/arcad_jni.cpp").read_text()
methods = set(re.findall(r"external fun\s+(native[A-Za-z0-9_]+)\s*\(", kotlin))
exports = set(re.findall(r"Java_com_arcad_app_NativeCadBridge_(native[A-Za-z0-9_]+)\s*\(", cpp))
if methods != exports:
    raise SystemExit(f"JNI contract mismatch; missing={sorted(methods-exports)}, extra={sorted(exports-methods)}")
cmake = (root / "app/src/main/cpp/CMakeLists.txt").read_text()
for required in ("arcad_jni.cpp", "arcad_jni_stub.cpp", "libarcad_ffi.so", "arcad_ffi", "log"):
    if required not in cmake:
        raise SystemExit(f"CMake configuration missing {required}")
print(f"JNI contract passed: {len(methods)} Kotlin native methods have matching C++ exports")
print("CMake Rust shared-library import and Android log linkage declarations verified")
