#!/usr/bin/env python3
"""Static regression guard: release builds may never select the JNI stub."""
from pathlib import Path

root = Path(__file__).resolve().parents[1]
cmake = (root / "app/src/main/cpp/CMakeLists.txt").read_text()
gradle = (root / "app/build.gradle.kts").read_text()
proguard = (root / "app/proguard-rules.pro").read_text()
elf_verifier = (root / "tools/verify-android-elf.py").read_text()

assert 'option(ARCAD_ALLOW_NATIVE_STUB "Allow the UI-only JNI stub for debug builds" OFF)' in cmake
assert "elseif(ARCAD_ALLOW_NATIVE_STUB)" in cmake
assert "message(FATAL_ERROR" in cmake
assert 'arguments += listOf("-DARCAD_ALLOW_NATIVE_STUB=ON")' in gradle
assert 'arguments += listOf("-DARCAD_ALLOW_NATIVE_STUB=OFF")' in gradle
assert 'environment("ARCAD_REQUIRE_LOCKFILE", "1")' in gradle
assert "dependsOn(buildCadCraftAndroidNative)" in gradle
assert "dependsOn(verifyCadCraftReleaseNative)" in gradle
assert 'it.name.startsWith("configureCMake")' in gradle
assert 'it.name.startsWith("externalNativeBuild")' in gradle
assert 'listOf("arm64-v8a", "x86_64")' in gradle
assert "-keep class com.arcad.app.NativeCadBridge { *; }" in proguard
assert '"arm64-v8a": 183' in elf_verifier and '"x86_64": 62' in elf_verifier
assert 'tools/verify-android-elf.py' in (root / "native/build-android.sh").read_text()

print("Release gate passed: release requires real Rust libraries; debug stub is explicit; JNI names are kept by R8")
