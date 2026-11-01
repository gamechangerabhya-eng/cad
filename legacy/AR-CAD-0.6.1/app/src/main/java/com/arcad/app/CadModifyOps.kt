package com.arcad.app

import kotlin.math.abs
import kotlin.math.hypot

/** Pure model-space operations for straight-line entities. Coordinates are millimetres. */
data class CadPoint(val x: Double, val y: Double)
data class CadLine(val start: CadPoint, val end: CadPoint)

object CadModifyOps {
    private const val EPS = 1e-9

    private fun cross(ax: Double, ay: Double, bx: Double, by: Double) = ax * by - ay * bx
    private fun length(a: CadPoint, b: CadPoint) = hypot(b.x - a.x, b.y - a.y)

    /** Intersection of the infinite first line and finite second segment. */
    private fun intersection(first: CadLine, second: CadLine): Pair<Double, Double>? {
        val rx = first.end.x - first.start.x
        val ry = first.end.y - first.start.y
        val sx = second.end.x - second.start.x
        val sy = second.end.y - second.start.y
        val denominator = cross(rx, ry, sx, sy)
        if (abs(denominator) <= EPS) return null
        val qpx = second.start.x - first.start.x
        val qpy = second.start.y - first.start.y
        val t = cross(qpx, qpy, sx, sy) / denominator
        val u = cross(qpx, qpy, rx, ry) / denominator
        return if (u >= -EPS && u <= 1.0 + EPS) t to u else null
    }

    /** Trim the endpoint nearest pickPoint to its nearest intersection with a cutter segment. */
    fun trim(target: CadLine, cutters: List<CadLine>, pickPoint: CadPoint): CadLine? {
        require(length(target.start, target.end) > EPS) { "Cannot trim a zero-length line" }
        val fromStart = length(target.start, pickPoint) <= length(target.end, pickPoint)
        val hits = cutters.asSequence().filter { it != target }.mapNotNull { intersection(target, it) }
            .map { it.first }.filter { it > EPS && it < 1.0 - EPS }.distinctBy { (it * 1e9).toLong() }
        val t = (if (fromStart) hits.minOrNull() else hits.maxOrNull()) ?: return null
        return if (fromStart) target.copy(start = CadPoint(target.start.x + (target.end.x-target.start.x)*t, target.start.y + (target.end.y-target.start.y)*t))
        else target.copy(end = CadPoint(target.start.x + (target.end.x-target.start.x)*t, target.start.y + (target.end.y-target.start.y)*t))
    }

    /** Extend the endpoint nearest pickPoint to the closest valid intersection with another segment. */
    fun extend(target: CadLine, boundaries: List<CadLine>, pickPoint: CadPoint): CadLine? {
        require(length(target.start, target.end) > EPS) { "Cannot extend a zero-length line" }
        val fromStart = length(target.start, pickPoint) <= length(target.end, pickPoint)
        val hits = boundaries.asSequence().filter { it != target }.mapNotNull { intersection(target, it) }
            .map { it.first }.filter { if (fromStart) it < -EPS else it > 1.0 + EPS }
        val t = if (fromStart) hits.maxOrNull() ?: return null else hits.minOrNull() ?: return null
        val point = CadPoint(target.start.x + (target.end.x-target.start.x)*t, target.start.y + (target.end.y-target.start.y)*t)
        return if (fromStart) target.copy(start = point) else target.copy(end = point)
    }

    /** Create a parallel offset of a line. Positive distance is its left-hand normal. */
    fun offset(line: CadLine, distance: Double): CadLine {
        require(distance.isFinite()) { "Offset distance must be finite" }
        val dx = line.end.x - line.start.x
        val dy = line.end.y - line.start.y
        val size = hypot(dx, dy)
        require(size > EPS) { "Cannot offset a zero-length line" }
        val ox = -dy / size * distance
        val oy = dx / size * distance
        return CadLine(CadPoint(line.start.x + ox, line.start.y + oy), CadPoint(line.end.x + ox, line.end.y + oy))
    }
}
