package com.arcad.app

import kotlin.math.PI
import kotlin.math.abs

private fun near(actual: Double, expected: Double, epsilon: Double = 1e-8) {
    check(abs(actual - expected) <= epsilon) { "Expected $expected, got $actual" }
}

fun main() {
    val square = CadMeasurements.regularPolygon(0.0, 0.0, 1.0, 4)
    check(square.size == 4)
    near(CadMeasurements.polylineLength(square, closed = true), 4.0 * kotlin.math.sqrt(2.0))
    near(CadMeasurements.polygonArea(square), 2.0)

    val triangle = CadMeasurements.regularPolygon(10.0, -5.0, 2.0, 3)
    check(triangle.size == 3)
    near(CadMeasurements.polygonArea(triangle), 3.0 * kotlin.math.sqrt(3.0))

    val rectangle = listOf(0.0 to 0.0, 4.0 to 0.0, 4.0 to 3.0, 0.0 to 3.0)
    near(CadMeasurements.polylineLength(rectangle, closed = true), 14.0)
    near(CadMeasurements.polygonArea(rectangle), 12.0)
    near(CadMeasurements.polygonArea(rectangle.reversed()), 12.0)
    near(CadMeasurements.polylineLength(rectangle, closed = false), 11.0)

    val circle = CadMeasurements.circleMetrics(2.0)
    near(circle.length!!, 4.0 * PI)
    near(circle.area!!, 4.0 * PI)
    val rect = CadMeasurements.rectangleMetrics(-4.0, 3.0)
    near(rect.length!!, 14.0)
    near(rect.area!!, 12.0)

    check(runCatching { CadMeasurements.regularPolygon(0.0, 0.0, 1.0, 2) }.isFailure)
    check(runCatching { CadMeasurements.regularPolygon(0.0, 0.0, 1.0, 129) }.isFailure)
    check(runCatching { CadMeasurements.regularPolygon(0.0, 0.0, Double.NaN, 6) }.isFailure)
    check(runCatching { CadMeasurements.polygonArea(listOf(0.0 to 0.0, 1.0 to 0.0)) }.isFailure)
    println("CAD measurement tests passed")
}
