package com.arcad.app

private fun assertAngle(actual: Double, expected: Double) {
    check(kotlin.math.abs(actual - expected) < 1e-8) { "Expected $expected°, got $actual°" }
}

fun main() {
    val ordinary = CadAngleMath.dxfToCanvas(0.0, 90.0)
    assertAngle(ordinary.startDegrees, 0.0)
    assertAngle(ordinary.endDegrees, -90.0)
    val ordinaryBack = CadAngleMath.canvasToDxf(ordinary.startDegrees, ordinary.endDegrees)
    assertAngle(ordinaryBack.startDegrees, 0.0)
    assertAngle(ordinaryBack.endDegrees, 90.0)

    val crossing = CadAngleMath.dxfToCanvas(350.0, 10.0)
    assertAngle(crossing.startDegrees, -350.0)
    assertAngle(crossing.endDegrees, -370.0)
    val crossingBack = CadAngleMath.canvasToDxf(crossing.startDegrees, crossing.endDegrees)
    assertAngle(crossingBack.startDegrees, 350.0)
    assertAngle(crossingBack.endDegrees, 10.0)

    val mirrored = CadAngleMath.canvasToDxf(0.0, 90.0)
    assertAngle(mirrored.startDegrees, 270.0)
    assertAngle(mirrored.endDegrees, 0.0)
    println("Arc angle tests passed: Y-axis inversion, CCW orientation, 0/360 crossing and mirrored arc export")
}
