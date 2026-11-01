package com.arcad.app

import kotlin.math.PI
import kotlin.math.abs
import kotlin.math.acos
import kotlin.math.atan2
import kotlin.math.cos
import kotlin.math.hypot
import kotlin.math.sin
import kotlin.math.sqrt

/** Corner editing for open/closed straight-segment polylines. Coordinates are model-space mm. */
object CadPolylineOps {
    private const val EPS = 1e-9
    private fun distance(a: CadPoint, b: CadPoint) = hypot(b.x - a.x, b.y - a.y)
    private fun add(a: CadPoint, b: CadPoint) = CadPoint(a.x + b.x, a.y + b.y)
    private fun sub(a: CadPoint, b: CadPoint) = CadPoint(a.x - b.x, a.y - b.y)
    private fun mul(a: CadPoint, k: Double) = CadPoint(a.x * k, a.y * k)
    private fun dot(a: CadPoint, b: CadPoint) = a.x * b.x + a.y * b.y
    private fun cross(a: CadPoint, b: CadPoint) = a.x * b.y - a.y * b.x
    private fun unit(a: CadPoint): CadPoint? {
        val length = hypot(a.x, a.y)
        return if (length <= EPS) null else CadPoint(a.x / length, a.y / length)
    }

    private data class Corner(val points: List<CadPoint>)

    private fun corner(points: List<CadPoint>, closed: Boolean, index: Int): Triple<CadPoint, CadPoint, CadPoint>? {
        if (points.size < 3 || index !in points.indices) return null
        if (!closed && (index == 0 || index == points.lastIndex)) return null
        val previous = points[(index - 1 + points.size) % points.size]
        val vertex = points[index]
        val next = points[(index + 1) % points.size]
        if (distance(previous, vertex) <= EPS || distance(vertex, next) <= EPS) return null
        return Triple(previous, vertex, next)
    }

    /** Fillet one polyline vertex with a circular arc approximated by [arcSegments] chords. */
    fun fillet(points: List<CadPoint>, closed: Boolean, cornerIndex: Int, radius: Double, arcSegments: Int = 8): List<CadPoint>? {
        require(radius.isFinite() && radius > 0.0) { "Fillet radius must be positive and finite" }
        require(arcSegments in 2..128) { "Arc segment count must be between 2 and 128" }
        val (a, b, c) = corner(points, closed, cornerIndex) ?: return null
        val u = unit(sub(a, b)) ?: return null
        val v = unit(sub(c, b)) ?: return null
        val theta = acos(dot(u, v).coerceIn(-1.0, 1.0))
        if (theta <= EPS || PI - theta <= EPS) return null
        val tangentDistance = radius / kotlin.math.tan(theta / 2.0)
        if (!tangentDistance.isFinite() || tangentDistance >= distance(a, b) - EPS || tangentDistance >= distance(c, b) - EPS) return null
        val tangentIn = add(b, mul(u, tangentDistance))
        val tangentOut = add(b, mul(v, tangentDistance))
        val bisector = unit(add(u, v)) ?: return null
        val centerDistance = radius / sin(theta / 2.0)
        val center = add(b, mul(bisector, centerDistance))
        val start = atan2(tangentIn.y - center.y, tangentIn.x - center.x)
        val end = atan2(tangentOut.y - center.y, tangentOut.x - center.x)
        var sweep = end - start
        val orientation = cross(sub(tangentIn, center), sub(tangentOut, center))
        if (orientation >= 0.0) while (sweep < 0.0) sweep += 2.0 * PI
        else while (sweep > 0.0) sweep -= 2.0 * PI
        if (abs(sweep) > PI + 1e-7) sweep -= kotlin.math.sign(sweep) * 2.0 * PI
        val replacement = (0..arcSegments).map { i ->
            val angle = start + sweep * i / arcSegments
            CadPoint(center.x + radius * cos(angle), center.y + radius * sin(angle))
        }
        return replaceCorner(points, cornerIndex, replacement)
    }

    /** Chamfer one polyline vertex using equal distances measured along both adjacent edges. */
    fun chamfer(points: List<CadPoint>, closed: Boolean, cornerIndex: Int, chamferDistance: Double): List<CadPoint>? {
        require(chamferDistance.isFinite() && chamferDistance > 0.0) { "Chamfer distance must be positive and finite" }
        val (a, b, c) = corner(points, closed, cornerIndex) ?: return null
        val lenA = distance(a, b)
        val lenC = distance(b, c)
        if (chamferDistance >= lenA - EPS || chamferDistance >= lenC - EPS) return null
        val towardA = unit(sub(a, b)) ?: return null
        val towardC = unit(sub(c, b)) ?: return null
        val first = add(b, mul(towardA, chamferDistance))
        val second = add(b, mul(towardC, chamferDistance))
        return replaceCorner(points, cornerIndex, listOf(first, second))
    }

    /**
     * Offset an open or closed straight-segment polyline. Positive distance offsets
     * to the left of the direction of travel. Acute joins whose miter exceeds
     * [miterLimit] times the offset distance are bevelled to avoid spikes.
     * Self-intersection cleanup is intentionally not implied; callers should warn
     * users that complex concave offsets may require further trimming.
     */
    fun offset(points: List<CadPoint>, closed: Boolean, distance: Double, miterLimit: Double = 8.0): List<CadPoint>? {
        require(distance.isFinite()) { "Offset distance must be finite" }
        require(miterLimit.isFinite() && miterLimit >= 1.0) { "Miter limit must be at least 1" }
        require(points.size >= if (closed) 3 else 2) { "Not enough vertices to offset" }
        require(points.all { it.x.isFinite() && it.y.isFinite() }) { "Polyline coordinates must be finite" }
        if (abs(distance) <= EPS) return points.toList()

        val segmentCount = if (closed) points.size else points.size - 1
        val directions = ArrayList<CadPoint>(segmentCount)
        val normals = ArrayList<CadPoint>(segmentCount)
        repeat(segmentCount) { i ->
            val a = points[i]
            val b = points[(i + 1) % points.size]
            val direction = unit(sub(b, a)) ?: return null
            directions += direction
            normals += CadPoint(-direction.y, direction.x)
        }

        fun offsetPoint(point: CadPoint, normal: CadPoint) = add(point, mul(normal, distance))
        fun lineIntersection(p: CadPoint, r: CadPoint, q: CadPoint, s: CadPoint): CadPoint? {
            val denominator = cross(r, s)
            if (abs(denominator) <= EPS) return null
            val t = cross(sub(q, p), s) / denominator
            return add(p, mul(r, t))
        }

        val result = ArrayList<CadPoint>(points.size + 4)
        if (!closed) result += offsetPoint(points.first(), normals.first())
        val firstVertex = if (closed) 0 else 1
        val lastVertexExclusive = if (closed) points.size else points.lastIndex
        for (i in firstVertex until lastVertexExclusive) {
            val previousSegment = (i - 1 + segmentCount) % segmentCount
            val nextSegment = i % segmentCount
            val before = offsetPoint(points[i], normals[previousSegment])
            val after = offsetPoint(points[i], normals[nextSegment])
            val intersection = lineIntersection(before, directions[previousSegment], after, directions[nextSegment])
            when {
                intersection == null -> {
                    // Collinear same-direction segments share one offset point.
                    if (dot(normals[previousSegment], normals[nextSegment]) > 0.999999) {
                        result += offsetPoint(points[i], normals[nextSegment])
                    } else {
                        // A reversal has no stable offset join.
                        return null
                    }
                }
                hypot(intersection.x - points[i].x, intersection.y - points[i].y) <= abs(distance) * miterLimit + EPS -> result += intersection
                else -> {
                    // Bevel an excessive miter rather than creating an unstable spike.
                    result += before
                    result += after
                }
            }
        }
        if (!closed) result += offsetPoint(points.last(), normals.last())
        if (result.size < if (closed) 3 else 2) return null
        if (result.any { !it.x.isFinite() || !it.y.isFinite() }) return null
        return result
    }

    private fun replaceCorner(points: List<CadPoint>, index: Int, replacement: List<CadPoint>): List<CadPoint> {
        val output = ArrayList<CadPoint>(points.size + replacement.size)
        points.forEachIndexed { i, point -> if (i == index) output.addAll(replacement) else output.add(point) }
        return output
    }
}
