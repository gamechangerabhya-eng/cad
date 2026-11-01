package com.arcad.app

import kotlin.math.PI
import kotlin.math.cos
import kotlin.math.hypot
import kotlin.math.sin

/** Pure geometry helpers shared by the mobile UI and host-side tests. */
object CadMeasurements {
    data class Metrics(val length: Double? = null, val area: Double? = null)

    fun regularPolygon(cx: Double, cy: Double, radius: Double, sides: Int): List<Pair<Double, Double>> {
        require(cx.isFinite() && cy.isFinite() && radius.isFinite()) { "Polygon values must be finite" }
        require(radius > 0.0) { "Polygon radius must be positive" }
        require(sides in 3..128) { "Polygon must have between 3 and 128 sides" }
        return (0 until sides).map { index ->
            val angle = (2.0 * PI * index / sides) - PI / 2.0
            (cx + radius * cos(angle)) to (cy + radius * sin(angle))
        }
    }

    fun polylineLength(points: List<Pair<Double, Double>>, closed: Boolean = false): Double {
        require(points.size >= 2) { "At least two points are required" }
        require(points.all { it.first.isFinite() && it.second.isFinite() }) { "Coordinates must be finite" }
        var total = 0.0
        for (index in 0 until points.lastIndex) {
            val a = points[index]; val b = points[index + 1]
            total += hypot(b.first - a.first, b.second - a.second)
        }
        if (closed && points.size > 2) {
            val a = points.last(); val b = points.first()
            total += hypot(b.first - a.first, b.second - a.second)
        }
        return total
    }

    /** Shoelace area; absolute area is returned regardless of winding direction. */
    fun polygonArea(points: List<Pair<Double, Double>>): Double {
        require(points.size >= 3) { "At least three points are required" }
        require(points.all { it.first.isFinite() && it.second.isFinite() }) { "Coordinates must be finite" }
        var twiceArea = 0.0
        points.indices.forEach { index ->
            val current = points[index]
            val next = points[(index + 1) % points.size]
            twiceArea += current.first * next.second - next.first * current.second
        }
        return kotlin.math.abs(twiceArea) / 2.0
    }

    fun circleMetrics(radius: Double): Metrics {
        require(radius.isFinite() && radius > 0.0) { "Circle radius must be positive and finite" }
        return Metrics(length = 2.0 * PI * radius, area = PI * radius * radius)
    }

    fun rectangleMetrics(width: Double, height: Double): Metrics {
        require(width.isFinite() && height.isFinite()) { "Rectangle dimensions must be finite" }
        val w = kotlin.math.abs(width); val h = kotlin.math.abs(height)
        return Metrics(length = 2.0 * (w + h), area = w * h)
    }
}
