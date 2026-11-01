package com.arcad.app

import kotlin.math.abs

private fun close(actual: Double, expected: Double, tolerance: Double = 1e-7) {
    check(abs(actual - expected) <= tolerance) { "Expected $expected, got $actual" }
}

fun main() {
    // Open horizontal polyline: positive offset is to its left (+Y).
    val open = CadPolylineOps.offset(
        listOf(CadPoint(0.0, 0.0), CadPoint(10.0, 0.0), CadPoint(10.0, 10.0)),
        closed = false,
        distance = 2.0
    ) ?: error("Open offset failed")
    check(open.size == 3)
    close(open[0].x, 0.0); close(open[0].y, 2.0)
    close(open[1].x, 8.0); close(open[1].y, 2.0)
    close(open[2].x, 8.0); close(open[2].y, 10.0)

    // Negative offset goes to the right side.
    val negative = CadPolylineOps.offset(
        listOf(CadPoint(0.0, 0.0), CadPoint(10.0, 0.0)), false, -3.0
    ) ?: error("Negative offset failed")
    close(negative[0].y, -3.0); close(negative[1].y, -3.0)

    // Closed square retains closure semantics and produces an inset square
    // for a counter-clockwise input with negative (right-hand) offset.
    val square = listOf(CadPoint(0.0, 0.0), CadPoint(10.0, 0.0), CadPoint(10.0, 10.0), CadPoint(0.0, 10.0))
    val inset = CadPolylineOps.offset(square, true, 1.0) ?: error("Closed offset failed")
    check(inset.size == 4)
    close(inset.minOf { it.x }, 1.0); close(inset.maxOf { it.x }, 9.0)
    close(inset.minOf { it.y }, 1.0); close(inset.maxOf { it.y }, 9.0)

    // Acute corner is bevelled instead of generating an excessive miter.
    val acute = CadPolylineOps.offset(
        listOf(CadPoint(0.0, 0.0), CadPoint(10.0, 0.0), CadPoint(0.1, 0.1)),
        false, 2.0, miterLimit = 2.0
    ) ?: error("Acute offset should bevel")
    check(acute.size >= 4) { "Expected a bevel join, got ${acute.size} points" }

    check(CadPolylineOps.offset(listOf(CadPoint(0.0, 0.0), CadPoint(0.0, 0.0)), false, 2.0) == null)
    check(CadPolylineOps.offset(listOf(CadPoint(0.0, 0.0), CadPoint(1.0, 0.0)), false, 0.0)!!.size == 2)
    check(runCatching { CadPolylineOps.offset(listOf(CadPoint(0.0, 0.0), CadPoint(1.0, 0.0)), false, Double.NaN) }.isFailure)
    check(runCatching { CadPolylineOps.offset(listOf(CadPoint(0.0, 0.0), CadPoint(1.0, 0.0)), false, 1.0, 0.5) }.isFailure)
    println("CadPolylineOffsetTest: PASS")
}
