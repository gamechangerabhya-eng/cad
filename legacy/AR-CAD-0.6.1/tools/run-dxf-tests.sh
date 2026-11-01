#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="$ROOT/tools/dxf-tests.jar"
kotlinc "$ROOT/app/src/main/java/com/arcad/app/CadDxfCodec.kt" "$ROOT/app/src/main/java/com/arcad/app/CadAngleMath.kt" "$ROOT/tools/tests/DxfCodecTest.kt" "$ROOT/tools/tests/CadAngleMathTest.kt" "$ROOT/tools/tests/DxfCodecPropertyTest.kt" "$ROOT/app/src/main/java/com/arcad/app/CadRecoveryStore.kt" "$ROOT/tools/tests/CadRecoveryStoreTest.kt" "$ROOT/app/src/main/java/com/arcad/app/CadMeasurements.kt" "$ROOT/tools/tests/CadMeasurementsTest.kt" "$ROOT/app/src/main/java/com/arcad/app/CadModifyOps.kt" "$ROOT/tools/tests/CadModifyOpsTest.kt" "$ROOT/app/src/main/java/com/arcad/app/CadPolylineOps.kt" "$ROOT/app/src/main/java/com/arcad/app/CadObjectSnap.kt" "$ROOT/app/src/main/java/com/arcad/app/CadDynamicInput.kt" "$ROOT/app/src/main/java/com/arcad/app/CadDynamicPreview.kt" "$ROOT/app/src/main/java/com/arcad/app/CadSnapTracking.kt" "$ROOT/app/src/main/java/com/arcad/app/CadSpatialIndex.kt" "$ROOT/tools/tests/CadSnapTrackingTest.kt" "$ROOT/tools/tests/CadSpatialIndexTest.kt" "$ROOT/tools/tests/CadObjectSnapTest.kt" "$ROOT/tools/tests/CadDynamicInputTest.kt" "$ROOT/tools/tests/CadDynamicPreviewTest.kt" "$ROOT/tools/tests/CadPolylineOpsTest.kt" "$ROOT/tools/tests/CadPolylineOffsetTest.kt" "$ROOT/app/src/main/java/com/arcad/app/NativeCadBridge.kt" "$ROOT/tools/tests/CadNativeDocumentTest.kt" "$ROOT/tools/tests/CadNativeSaveSafetyTest.kt" "$ROOT/tools/tests/CadNativeRenderTest.kt" "$ROOT/app/src/main/java/com/arcad/app/NativePreviewGate.kt" "$ROOT/tools/tests/NativePreviewGateTest.kt" -include-runtime -d "$OUT"
java -cp "$OUT" com.arcad.app.DxfCodecTestKt
java -cp "$OUT" com.arcad.app.CadAngleMathTestKt
java -cp "$OUT" com.arcad.app.DxfCodecPropertyTestKt
java -cp "$OUT" com.arcad.app.CadRecoveryStoreTestKt
java -cp "$OUT" com.arcad.app.CadMeasurementsTestKt
java -cp "$OUT" com.arcad.app.CadModifyOpsTestKt
java -cp "$OUT" com.arcad.app.CadPolylineOpsTestKt
java -cp "$OUT" com.arcad.app.CadPolylineOffsetTestKt
java -cp "$OUT" com.arcad.app.CadObjectSnapTestKt
java -cp "$OUT" com.arcad.app.CadDynamicInputTestKt
java -cp "$OUT" com.arcad.app.CadDynamicPreviewTestKt
java -cp "$OUT" com.arcad.app.CadSnapTrackingTestKt
java -cp "$OUT" com.arcad.app.CadSpatialIndexTestKt
java -cp "$OUT" com.arcad.app.CadNativeDocumentTestKt
java -cp "$OUT" com.arcad.app.CadNativeSaveSafetyTestKt
java -cp "$OUT" com.arcad.app.CadNativeRenderTestKt
java -cp "$OUT" com.arcad.app.NativePreviewGateTestKt
python3 "$ROOT/native/check-ffi-contract.py"
python3 "$ROOT/native/check-jni-contract.py"

python3 "$ROOT/tools/check-native-build-script.py"
python3 "$ROOT/tools/check-release-gate.py"
