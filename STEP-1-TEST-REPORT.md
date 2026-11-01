# Step 1 verification report

## Passed checks

- `bash -n` passed for ArtCraft build scripts, AR CAD debug/release wrappers, and the AR CAD preflight script.
- Both AR CAD build wrappers returned the builder's help output successfully.
- ArtCraft Python build modules passed `python -m compileall -q lib`.
- All 20 Cargo manifests across the CADCraft workspace and ArtCraft adapters parsed successfully with Python `tomllib`.
- CADCraft workspace manifest reports version 0.15.1.
- CADCraft source import has the expected builder origin URL and a local import tag; 896 tracked `crates/` and `apps/` paths are present with no missing files.
- AR CAD package/display-name overrides are present in the build wrappers: `com.arcad.app` / `AR CAD`.
- Custom launcher icon exists.
- Original AR CAD 0.6.1 source is preserved under `legacy/AR-CAD-0.6.1/`.
- ArtCraft builder CADCraft target and `cadcraft-io` file-format registration hooks were verified in source.
- Existing AR CAD FFI contract check passed (21 C functions matched Rust exports).
- Existing AR CAD JNI contract check passed (17 Kotlin native methods matched C++ exports).
- Existing AR CAD native build-script checks passed (SDK/NDK discovery, lockfile bootstrap, locked ABI builds, fail-closed release gate).
- Existing AR CAD release-gate check passed (real Rust libraries required for release; debug stub explicit; JNI names preserved by R8).

## Blocked checks

- The full Kotlin/JVM DXF test runner could not start because `kotlinc` is not installed in this environment.
- Rust/Cargo compilation and the ArtCraft Android launcher build could not run because `cargo`, `rustc`, and `rustup` are not installed.
- APK packaging/device validation could not run because Android SDK, Android NDK, Gradle, and ADB are not installed.
- The new GitHub Actions workflow has been authored but has not run on GitHub; no CI result is claimed.

## Result

The source integration and static checks are complete. A verified APK is **not** available yet. Do not mark the Android build/runtime acceptance gate complete until the CI or a local Android build compiles the Rust Android target, packages the APK, verifies `com.arcad.app` and `AR CAD`, and is tested on a real Android device.
- AR CAD GitHub Actions workflow parsed as valid YAML and contains one `build-ar-cad` job.
- Build preflight was executed and correctly failed closed, identifying missing Cargo/Rust, Android SDK, and Android NDK rather than attempting an unverifiable build.
