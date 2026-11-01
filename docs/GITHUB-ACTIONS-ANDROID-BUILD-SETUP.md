# AR CAD — GitHub Actions Android build setup

## Workflow

The workflow is `.github/workflows/build.yml`.

- Push to any branch: builds a **debug APK**.
- Actions → AR CAD Android APK → Run workflow: choose `debug` or `release`.
- Push a version tag matching `v*.*.*` (for example `v1.0.0`): selects a **release APK**.
- The job runs the source acceptance checks, CADCraft workspace check and Rust regression tests before building the Android APK.
- The final step validates the expected APK filename, package ID, app label, archive integrity and APK signature, then uploads the APK, logs and SHA-256 checksum as the `ar-cad-android-apk` workflow artifact.

## First test: debug build (no secrets required)

1. Upload/commit the project with the `.github/workflows/build.yml` file retained at that exact path.
2. Open the repository's **Actions** tab.
3. Select **AR CAD Android APK**.
4. Select **Run workflow**, choose `debug`, and start the run. Alternatively, push a commit to a branch.
5. Open the run and wait for the `Build AR CAD (CADCraft 0.15.1)` job to finish.
6. If successful, open **Artifacts** at the bottom of the run and download `ar-cad-android-apk`.

Expected debug artifact inside the ZIP: `cadcraft-0.15.1-debug.apk`.

## Release signing secrets (only needed for release builds)

Create or use a persistent JKS keystore. The key alias must be exactly `artcraftbuilder`, and the password must be the same store/key password because that is what the Android builder configures. Do not commit the keystore or password to the repository.

Add these two repository Actions secrets under **Settings → Secrets and variables → Actions**:

- `AR_CAD_KEYSTORE_BASE64`: the complete keystore file encoded as one Base64 value.
- `AR_CAD_KEYSTORE_PASSWORD`: the keystore password.

Base64 examples:

Linux/macOS:

```bash
base64 -w 0 artcraft-builder.jks
```

PowerShell:

```powershell
[Convert]::ToBase64String([IO.File]::ReadAllBytes('artcraft-builder.jks'))
```

Paste the resulting Base64 text into the first secret. Never paste either secret into chat, source files, workflow YAML, issues or logs. Keep a secure offline backup of the JKS file: losing the signing key can prevent future updates from being accepted as updates to the same installed app.

Expected release artifact inside the ZIP: `cadcraft-0.15.1.apk`.

## What a green run proves — and what it does not

A successful GitHub run proves that the hosted runner completed the configured Rust checks, Android packaging, APK metadata checks and signature verification for that build. It does not by itself prove CAD correctness, DWG/DXF interoperability, device compatibility, performance, or production readiness. Install and exercise the APK on representative Android devices before distributing it.

If a run fails, open the failed step and download the workflow artifact for its build logs. Preserve the full error output when reporting the failure; do not share signing secrets.
