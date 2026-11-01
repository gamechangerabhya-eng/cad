# AR CAD — Step 2: Android application UX and branding

## Implemented in source
1. AR CAD in-app/window identity (`AR CAD`), menus/About/recovery messages and Android application ID `com.arcad.app`.
2. Custom AR CAD native app icon is installed in the CADCraft app asset path; original icon is preserved in `branding/cadcraft-upstream-icon-256.png`.
3. Touch-first start page when the available drawing area is under 760 logical pixels:
   - Large New Drawing and Open Drawing actions.
   - New Metric Drawing and Open Sample actions.
   - Full-width, vertically scrolling recent drawing rows.
   - Empty-state guidance for first launch.
4. Android toolbar is 52 logical pixels tall with 42-pixel command targets; desktop sizing is unchanged.
5. Android first-launch defaults hide left/right side panels so the canvas has more space. Existing panel commands/menu actions can reopen them.
6. CADCraft upstream self-update helper and update prompt are disabled on Android; AR CAD must own its own release channel.
7. Unit tests added for mobile breakpoint and Android canvas-first defaults.

## Validation
- Source-level checks are run in `tools/test-step2.py`.
- Shell syntax and source identity checks are also rerun.
- Rust compilation and unit tests require Rust/Cargo, which are not installed in this execution environment.
- Android build and device tests remain blocked by missing Rust/Cargo, cargo-ndk, Android SDK/NDK, Gradle, Kotlin compiler and ADB.

## Acceptance status
Source implementation: complete for this step's scoped branding and responsive start-page work.
Rust compile/unit tests: pending toolchain.
Android APK/device verification: pending toolchain and device.
