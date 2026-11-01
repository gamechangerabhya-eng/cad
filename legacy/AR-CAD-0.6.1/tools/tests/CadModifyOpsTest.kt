package com.arcad.app

import kotlin.math.abs

private fun near(actual: Double, expected: Double, eps: Double = 1e-8) {
    check(abs(actual - expected) <= eps) { "Expected $expected, got $actual" }
}

fun main() {
    val horizontal = CadLine(CadPoint(0.0, 0.0), CadPoint(10.0, 0.0))
    val vertical = CadLine(CadPoint(5.0, -5.0), CadPoint(5.0, 5.0))
    val trimmedLeft = CadModifyOps.trim(horizontal, listOf(vertical), CadPoint(0.0, 0.0))!!
    near(trimmedLeft.start.x, 5.0); near(trimmedLeft.end.x, 10.0)
    val trimmedRight = CadModifyOps.trim(horizontal, listOf(vertical), CadPoint(10.0, 0.0))!!
    near(trimmedRight.start.x, 0.0); near(trimmedRight.end.x, 5.0)

    val leftBoundary = CadLine(CadPoint(-4.0, -2.0), CadPoint(-4.0, 2.0))
    val extendedLeft = CadModifyOps.extend(horizontal, listOf(leftBoundary), CadPoint(0.0, 0.0))!!
    near(extendedLeft.start.x, -4.0); near(extendedLeft.end.x, 10.0)
    val rightBoundary = CadLine(CadPoint(14.0, -2.0), CadPoint(14.0, 2.0))
    val extendedRight = CadModifyOps.extend(horizontal, listOf(rightBoundary), CadPoint(10.0, 0.0))!!
    near(extendedRight.start.x, 0.0); near(extendedRight.end.x, 14.0)

    val offset = CadModifyOps.offset(horizontal, 3.0)
    near(offset.start.x, 0.0); near(offset.start.y, 3.0)
    near(offset.end.x, 10.0); near(offset.end.y, 3.0)
    val diagonal = CadModifyOps.offset(CadLine(CadPoint(0.0,0.0), CadPoint(3.0,4.0)), 5.0)
    near(diagonal.start.x, -4.0); near(diagonal.start.y, 3.0)

    check(CadModifyOps.trim(horizontal, emptyList(), CadPoint(0.0,0.0)) == null)
    check(CadModifyOps.extend(horizontal, emptyList(), CadPoint(0.0,0.0)) == null)
    check(runCatching { CadModifyOps.offset(horizontal, Double.NaN) }.isFailure)
    check(runCatching { CadModifyOps.offset(CadLine(CadPoint(1.0,1.0), CadPoint(1.0,1.0)), 2.0) }.isFailure)
    println("CAD modify operation tests passed")
}
