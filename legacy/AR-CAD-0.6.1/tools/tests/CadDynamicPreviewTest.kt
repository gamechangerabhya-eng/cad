package com.arcad.app

import kotlin.math.abs

fun main() {
    fun near(actual: Double, expected: Double, epsilon: Double = 1e-8) {
        check(abs(actual - expected) <= epsilon) { "Expected $expected, got $actual" }
    }
    val horizontal = CadDynamicPreview.measure(CadPoint(10.0, 20.0), CadPoint(40.0, 20.0))
    near(horizontal.dx, 30.0); near(horizontal.dy, 0.0); near(horizontal.length, 30.0); near(horizontal.angleDegrees, 0.0)
    val vertical = CadDynamicPreview.measure(CadPoint(0.0, 0.0), CadPoint(0.0, -12.0))
    near(vertical.length, 12.0); near(vertical.angleDegrees, -90.0)
    val diagonal = CadDynamicPreview.measure(CadPoint(0.0, 0.0), CadPoint(3.0, 4.0))
    near(diagonal.length, 5.0); near(diagonal.angleDegrees, Math.toDegrees(kotlin.math.atan2(4.0, 3.0)))
    val zero = CadDynamicPreview.measure(CadPoint(1.0, 1.0), CadPoint(1.0, 1.0))
    near(zero.length, 0.0); near(zero.angleDegrees, 0.0)
    check(runCatching { CadDynamicPreview.measure(CadPoint(Double.NaN, 0.0), CadPoint(0.0, 0.0)) }.isFailure)
    println("CadDynamicPreviewTest: PASS")
}
