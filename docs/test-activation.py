#!/usr/bin/env python3
"""Source acceptance checks for the CADCraft Android first-install activation gate."""
from pathlib import Path
import sys

root = Path(__file__).resolve().parents[1]
activity = (root / "templates/android/SafActivity.java").read_text(encoding="utf-8")
builder = (root / "lib/build-common.sh").read_text(encoding="utf-8")
checks = []


def check(label, ok):
    checks.append((label, bool(ok)))


check(
    "Only CADCraft generated Activities require activation",
    '.replace("__ACTIVATION_REQUIRED__", "true" if app == "cadcraft" else "false")' in builder,
)
check(
    "CADCraft startup splash is enabled only for its generated Activity",
    '.replace("__STARTUP_SPLASH_REQUIRED__", "true" if app == "cadcraft" else "false")' in builder,
)
check(
    "Activation uses PBKDF2-HMAC-SHA-256 with 600,000 iterations",
    'ACTIVATION_ALGORITHM = "PBKDF2WithHmacSHA256"' in activity
    and '.replace("__ACTIVATION_ITERATIONS__", "600000" if app == "cadcraft" else "0")' in builder,
)
check(
    "Activation verifier comparison uses constant-time equality",
    "MessageDigest.isEqual(expected, actual)" in activity,
)
check(
    "Protected activation state uses Android Keystore AES-GCM",
    '"AndroidKeyStore"' in activity and '"AES/GCM/NoPadding"' in activity,
)
check(
    "Password validation runs off the UI thread and clears input state",
    'new Thread(() -> {' in activity
    and '}, "ar-cad-activation").start();' in activity
    and 'activationInput.setText("");' in activity
    and "Arrays.fill(candidate, '\\0');" in activity,
)
check(
    "Activation state is checked before file intents and on subsequent launches",
    activity.index("ensureActivationGate();") < activity.index("deferOrHandleIncomingIntent(getIntent());")
    and "protected void onResume()" in activity
    and "protected void onNewIntent(Intent intent)" in activity,
)
check(
    "Incoming file intents wait until activation succeeds",
    "deferredIncomingIntents.add(intent);" in activity
    and "releaseDeferredIncomingIntents();" in activity
    and "handleIncomingFiles(intent);" in activity,
)
check(
    "Branded splash fades only after the native CAD UI reports its first frame",
    "ObjectAnimator.ofFloat(icon, View.ALPHA" in activity
    and "public void onNativeFirstFrameReady()" in activity
    and "self.notify_first_frame()" in builder
    and "dismissStartupSplash();" in activity,
)
check(
    "Startup failure leaves an explicit close action instead of a permanent splash",
    "45_000L" in activity
    and "CAD workspace could not finish starting." in activity
    and 'startupExitButton.setVisibility(View.VISIBLE);' in activity,
)
check(
    "Retries use temporary increasing delays without permanent lockout",
    "ACTIVATION_MAX_FAILURES = 20" in activity
    and "ACTIVATION_MAX_DELAY_MS = 30_000L" in activity
    and "1_000L << Math.min(failures - 4, 5)" in activity,
)
check(
    "Generated launcher version invalidates stale Activity templates",
    'GENERATOR_VERSION = "65"' in builder,
)

for label, ok in checks:
    print(("PASS" if ok else "FAIL") + " - " + label)
print(f"\n{sum(ok for _, ok in checks)}/{len(checks)} activation checks passed.")
sys.exit(0 if all(ok for _, ok in checks) else 1)
