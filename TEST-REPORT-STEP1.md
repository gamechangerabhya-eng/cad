# AR CAD Step 1 — Test Report

Date: 2026-10-31

## Result
Source integration checks are partially verified. **APK build and Android runtime verification are blocked** because this execution environment does not contain the required native Android toolchain.

## Passed checks
- 32 shell scripts passed `bash -n` syntax validation.
- ArtCraft GitHub Actions workflow parsed as YAML.
- CADCraft workspace manifest reports version 0.15.1.
- 896 tracked CADCraft source paths were checked; no tracked source files were missing.
- Debug and release wrapper scripts contain the AR CAD app name, application ID `com.arcad.app`, and CADCraft build target.
- ArtCraft builder source contains its CADCraft target adapter and native CADCraft I/O registration.
- Custom launcher icon is a valid 512x512 PNG.
- Source ZIP passed `zipfile.testzip()` integrity validation (3,747 entries).

## Build attempt
Command: `./build-ar-cad-debug.sh`

Result: stopped at build preflight with:
`ERROR: Rust/Cargo is required.`

No APK was generated.

## Required but unavailable
- Rust compiler (`rustc`) and Cargo
- `cargo-ndk`
- Android SDK and platform API 35
- Android NDK
- Gradle
- Kotlin compiler (`kotlinc`)
- Android Debug Bridge (`adb`) / Android device or emulator

The AR CAD Kotlin test runner was attempted but could not start because `kotlinc` is unavailable. A precompiled test JAR is present but has no executable main manifest, so it was not treated as a passing test run.

## Conclusion
Step 1 source layout and static integration checks pass. Step 1 is **not fully build-tested** and must not be marked complete for APK delivery until the required toolchain is available and a real APK is built, inspected, installed, and smoke-tested on Android.
