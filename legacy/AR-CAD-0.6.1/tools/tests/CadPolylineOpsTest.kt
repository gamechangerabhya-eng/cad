package com.arcad.app

import kotlin.math.abs
import kotlin.math.hypot

private fun nearPolyline(actual: Double, expected: Double, eps: Double = 1e-7) {
    check(abs(actual - expected) <= eps) { "Expected $expected, got $actual" }
}

fun main() {
    val rightAngle = listOf(CadPoint(-10.0, 0.0), CadPoint(0.0, 0.0), CadPoint(0.0, 10.0))
    val chamfer = CadPolylineOps.chamfer(rightAngle, false, 1, 2.0)!!
    check(chamfer.size == 4)
    nearPolyline(chamfer[1].x, -2.0); nearPolyline(chamfer[1].y, 0.0)
    nearPolyline(chamfer[2].x, 0.0); nearPolyline(chamfer[2].y, 2.0)
    check(chamfer.first() == rightAngle.first() && chamfer.last() == rightAngle.last())

    val fillet = CadPolylineOps.fillet(rightAngle, false, 1, 2.0, 8)!!
    check(fillet.size == 11) { "Expected 2 original endpoints plus 9 arc points, got ${fillet.size}" }
    nearPolyline(fillet[1].x, -2.0); nearPolyline(fillet[1].y, 0.0)
    nearPolyline(fillet.lastIndex.let { fillet[it - 1].x }, 0.0)
    nearPolyline(fillet[fillet.lastIndex - 1].y, 2.0)
    val center = CadPoint(-2.0, 2.0)
    fillet.subList(1, fillet.lastIndex).forEach { point -> nearPolyline(hypot(point.x-center.x, point.y-center.y), 2.0, 1e-6) }

    val closed = listOf(CadPoint(0.0,0.0), CadPoint(10.0,0.0), CadPoint(10.0,10.0), CadPoint(0.0,10.0))
    check(CadPolylineOps.chamfer(closed, true, 0, 1.0)!!.size == 5)
    check(CadPolylineOps.fillet(closed, true, 0, 1.0)!!.size == 12)
    check(CadPolylineOps.chamfer(rightAngle, false, 0, 1.0) == null)
    check(CadPolylineOps.fillet(rightAngle, false, 0, 1.0) == null)
    check(CadPolylineOps.chamfer(rightAngle, false, 1, 10.0) == null)
    check(CadPolylineOps.fillet(rightAngle, false, 1, 10.0) == null)
    check(runCatching { CadPolylineOps.fillet(rightAngle, false, 1, Double.NaN) }.isFailure)
    check(runCatching { CadPolylineOps.chamfer(rightAngle, false, 1, -1.0) }.isFailure)

    // Clockwise corner and acute/obtuse geometry must keep every fillet sample on the requested radius.
    val clockwise = listOf(CadPoint(-10.0, 0.0), CadPoint(0.0, 0.0), CadPoint(0.0, -10.0))
    val clockwiseFillet = CadPolylineOps.fillet(clockwise, false, 1, 1.5, 12)!!
    val clockwiseCenter = CadPoint(-1.5, -1.5)
    clockwiseFillet.subList(1, clockwiseFillet.lastIndex).forEach { point ->
        nearPolyline(hypot(point.x-clockwiseCenter.x, point.y-clockwiseCenter.y), 1.5, 1e-6)
    }
    val obtuse = listOf(CadPoint(-10.0, 0.0), CadPoint(0.0, 0.0), CadPoint(5.0, 8.6602540378))
    check(CadPolylineOps.fillet(obtuse, false, 1, 1.0) != null)
    val collinear = listOf(CadPoint(-10.0, 0.0), CadPoint(0.0, 0.0), CadPoint(10.0, 0.0))
    check(CadPolylineOps.fillet(collinear, false, 1, 1.0) == null)
    println("CAD polyline fillet/chamfer tests passed: open/closed, clockwise, obtuse, invalid radius and endpoint guards")
}
