# AR CAD release checklist

A release is blocked until every mandatory item below is evidenced by a build log, APK inspection, device test or signed-off record.

## Automated checks
- [ ] `tools/test-ar-cad.sh` passes with `RUN_RUST_TESTS=1`.
- [ ] Android ARM64 native compilation succeeds with the pinned SDK/NDK toolchain.
- [ ] Gradle creates a non-unsigned APK.
- [ ] APK package ID is `com.arcad.app` and label is `AR CAD`.
- [ ] APK archive integrity and `apksigner verify` pass.
- [ ] SHA-256 checksum is retained with the artifact.

## Physical-device checks
- [ ] Install on an ARM64 Android phone at the declared minimum and target Android API range.
- [ ] Launch, background, resume and rotate/recreate activity without losing the document.
- [ ] Create line, circle, rectangle and polyline; test selection and edit commands.
- [ ] Verify Grid, Snap, Ortho, Polar, Object Snap, Dynamic Input, pan and pinch zoom.
- [ ] Save, close, reopen, export and re-import a DXF; compare geometry and units.
- [ ] Force-stop during edits; verify recovery offers only recoverable AR CAD drawings.
- [ ] Test Open/Save/Save As with Android Storage Access Framework locations.
- [ ] Check command entry, keyboard/IME, accessibility labels, text scaling and narrow screens.
- [ ] Run a sustained drawing session and record startup time, memory and thermal behavior.

## Signing and distribution
- [ ] Configure persistent `AR_CAD_KEYSTORE_BASE64` and `AR_CAD_KEYSTORE_PASSWORD` secrets.
- [ ] Confirm the keystore contains alias `artcraftbuilder` and the key password matches the store password.
- [ ] Store an offline backup of the keystore and password in a secure place.
- [ ] Verify release APK signature and checksum.
- [ ] Publish version notes with known CADCraft format/feature limitations.

A checkbox must not be marked complete based only on source inspection. Physical-device checks require actual installation and interaction on hardware.
