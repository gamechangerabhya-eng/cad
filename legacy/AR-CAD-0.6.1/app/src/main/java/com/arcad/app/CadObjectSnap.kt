package com.arcad.app

import kotlin.math.abs
import kotlin.math.atan2
import kotlin.math.cos
import kotlin.math.hypot
import kotlin.math.max
import kotlin.math.min
import kotlin.math.sin
import kotlin.math.sqrt

/**
 * Model-space object snap resolver. Coordinates use the drawing's model coordinate
 * system (the Android canvas currently uses Y-down coordinates).
 *
 * Supported modes: endpoints, intersections, midpoints, centers, circle/arc
 * quadrants and nearest points. The caller supplies a model-space tolerance
 * derived from a fixed screen-pixel tolerance divided by the current zoom.
 */
object CadObjectSnap {
    private const val EPS = 1e-9

    sealed interface Geometry {
        data class Segment(val a: CadPoint, val b: CadPoint) : Geometry
        data class Circle(val center: CadPoint, val radius: Double) : Geometry
        data class Arc(val center: CadPoint, val radius: Double, val startDegrees: Double, val endDegrees: Double) : Geometry
        data class Polyline(val points: List<CadPoint>, val closed: Boolean) : Geometry
    }

    enum class Kind { ENDPOINT, INTERSECTION, MIDPOINT, CENTER, QUADRANT, NEAREST }
    data class Result(val point: CadPoint, val kind: Kind, val distance: Double)

    private data class Candidate(val point: CadPoint, val kind: Kind, val priority: Int)
    private data class SegmentRef(val a: CadPoint, val b: CadPoint)

    fun resolve(
        point: CadPoint,
        geometries: List<Geometry>,
        tolerance: Double,
        enabledKinds: Set<Kind> = Kind.values().toSet()
    ): Result? {
        require(point.x.isFinite() && point.y.isFinite()) { "Snap cursor must be finite" }
        require(tolerance.isFinite() && tolerance >= 0.0) { "Snap tolerance must be finite and non-negative" }
        if (geometries.isEmpty() || tolerance <= 0.0 || enabledKinds.isEmpty()) return null

        val segments = ArrayList<SegmentRef>()
        geometries.forEach { geometry -> when (geometry) {
            is Geometry.Segment -> segments += SegmentRef(geometry.a, geometry.b)
            is Geometry.Polyline -> {
                geometry.points.zipWithNext().forEach { (a, b) -> segments += SegmentRef(a, b) }
                if (geometry.closed && geometry.points.size > 2) segments += SegmentRef(geometry.points.last(), geometry.points.first())
            }
            else -> Unit
        } }

        val candidates = ArrayList<Candidate>()
        fun add(p: CadPoint, kind: Kind) {
            if (kind in enabledKinds && p.x.isFinite() && p.y.isFinite()) {
                candidates += Candidate(p, kind, priority(kind))
            }
        }
        geometries.forEach { geometry -> when (geometry) {
            is Geometry.Segment -> {
                add(geometry.a, Kind.ENDPOINT); add(geometry.b, Kind.ENDPOINT)
                add(midpoint(geometry.a, geometry.b), Kind.MIDPOINT)
                closestOnSegment(point, geometry.a, geometry.b)?.let { add(it, Kind.NEAREST) }
            }
            is Geometry.Polyline -> {
                geometry.points.forEach { add(it, Kind.ENDPOINT) }
                geometry.points.zipWithNext().forEach { (a, b) ->
                    add(midpoint(a, b), Kind.MIDPOINT)
                    closestOnSegment(point, a, b)?.let { add(it, Kind.NEAREST) }
                }
                if (geometry.closed && geometry.points.size > 2) {
                    val a = geometry.points.last(); val b = geometry.points.first()
                    add(midpoint(a, b), Kind.MIDPOINT)
                    closestOnSegment(point, a, b)?.let { add(it, Kind.NEAREST) }
                }
            }
            is Geometry.Circle -> {
                add(geometry.center, Kind.CENTER)
                quadrantPoints(geometry.center, geometry.radius).forEach { add(it, Kind.QUADRANT) }
                nearestOnCircle(point, geometry.center, geometry.radius)?.let { add(it, Kind.NEAREST) }
            }
            is Geometry.Arc -> {
                arcEndpoints(geometry).forEach { add(it, Kind.ENDPOINT) }
                add(geometry.center, Kind.CENTER)
                quadrantPoints(geometry.center, geometry.radius).filter { angleOnArc(angleOf(geometry.center, it), geometry.startDegrees, geometry.endDegrees) }
                    .forEach { add(it, Kind.QUADRANT) }
                nearestOnArc(point, geometry)?.let { add(it, Kind.NEAREST) }
            }
        } }

        // Intersection work is restricted to geometry whose bounding box is close
        // to the cursor. This avoids a global O(n²) scan on large drawings.
        fun boundsNear(a: CadPoint, b: CadPoint): Boolean {
            val dx = max(0.0, max(min(a.x, b.x) - point.x, point.x - max(a.x, b.x)))
            val dy = max(0.0, max(min(a.y, b.y) - point.y, point.y - max(a.y, b.y)))
            return hypot(dx, dy) <= tolerance + EPS
        }
        val nearbySegments = segments.filter { boundsNear(it.a, it.b) }
        for (i in nearbySegments.indices) for (j in i + 1 until nearbySegments.size) {
            segmentIntersection(nearbySegments[i], nearbySegments[j])?.let { add(it, Kind.INTERSECTION) }
        }

        // Segment/circle and segment/arc intersections.
        val round = geometries.filter { it is Geometry.Circle || it is Geometry.Arc }.filter { geometry ->
            val pair = centerRadius(geometry) ?: return@filter false
            val radius = pair.second
            boundsNear(CadPoint(pair.first.x - radius, pair.first.y - radius), CadPoint(pair.first.x + radius, pair.first.y + radius))
        }
        round.forEach { geometry ->
            when (geometry) {
                is Geometry.Circle -> nearbySegments.forEach { segment -> segmentCircleIntersections(segment, geometry.center, geometry.radius).forEach { add(it, Kind.INTERSECTION) } }
                is Geometry.Arc -> nearbySegments.forEach { segment ->
                    segmentCircleIntersections(segment, geometry.center, geometry.radius)
                        .filter { angleOnArc(angleOf(geometry.center, it), geometry.startDegrees, geometry.endDegrees) }
                        .forEach { add(it, Kind.INTERSECTION) }
                }
                else -> Unit
            }
        }

        // Circle/circle, circle/arc and arc/arc intersections near the cursor.
        for (i in round.indices) for (j in i + 1 until round.size) {
            val first = round[i]; val second = round[j]
            val c1 = centerRadius(first) ?: continue
            val c2 = centerRadius(second) ?: continue
            circleIntersections(c1.first, c1.second, c2.first, c2.second).forEach { p ->
                if (onRoundGeometry(first, p) && onRoundGeometry(second, p)) add(p, Kind.INTERSECTION)
            }
        }

        return candidates.asSequence()
            .map { candidate -> candidate to distance(point, candidate.point) }
            .filter { it.second <= tolerance + EPS }
            .minWithOrNull(compareBy<Pair<Candidate, Double>> { it.first.priority }.thenBy { it.second })
            ?.let { (candidate, distance) -> Result(candidate.point, candidate.kind, distance) }
    }

    private fun priority(kind: Kind): Int = when (kind) {
        Kind.INTERSECTION -> 0
        Kind.ENDPOINT -> 1
        Kind.MIDPOINT -> 2
        Kind.CENTER -> 3
        Kind.QUADRANT -> 4
        Kind.NEAREST -> 5
    }

    private fun distance(a: CadPoint, b: CadPoint) = hypot(a.x - b.x, a.y - b.y)
    private fun midpoint(a: CadPoint, b: CadPoint) = CadPoint((a.x + b.x) / 2.0, (a.y + b.y) / 2.0)
    private fun angleOf(center: CadPoint, point: CadPoint) = Math.toDegrees(atan2(point.y - center.y, point.x - center.x))
    private fun quadrantPoints(center: CadPoint, radius: Double) = listOf(
        CadPoint(center.x + radius, center.y), CadPoint(center.x, center.y + radius),
        CadPoint(center.x - radius, center.y), CadPoint(center.x, center.y - radius)
    )
    private fun arcEndpoints(arc: Geometry.Arc) = listOf(arc.startDegrees, arc.endDegrees).map { angle ->
        val radians = Math.toRadians(angle)
        CadPoint(arc.center.x + arc.radius * cos(radians), arc.center.y + arc.radius * sin(radians))
    }
    private fun normalize(degrees: Double) = ((degrees % 360.0) + 360.0) % 360.0
    private fun angleOnArc(angle: Double, start: Double, end: Double): Boolean {
        val sweep = end - start
        if (abs(sweep) >= 359.999999) return true
        return if (sweep >= 0.0) normalize(angle - start) <= sweep + EPS
        else normalize(start - angle) <= -sweep + EPS
    }
    private fun closestOnSegment(p: CadPoint, a: CadPoint, b: CadPoint): CadPoint? {
        val dx = b.x - a.x; val dy = b.y - a.y; val length2 = dx * dx + dy * dy
        if (length2 <= EPS) return a
        val t = (((p.x - a.x) * dx + (p.y - a.y) * dy) / length2).coerceIn(0.0, 1.0)
        return CadPoint(a.x + t * dx, a.y + t * dy)
    }
    private fun nearestOnCircle(p: CadPoint, center: CadPoint, radius: Double): CadPoint? {
        if (!radius.isFinite() || radius <= 0.0) return null
        val dx = p.x - center.x; val dy = p.y - center.y; val length = hypot(dx, dy)
        if (length <= EPS) return null
        return CadPoint(center.x + dx / length * radius, center.y + dy / length * radius)
    }
    private fun nearestOnArc(p: CadPoint, arc: Geometry.Arc): CadPoint? {
        val candidate = nearestOnCircle(p, arc.center, arc.radius) ?: return null
        return if (angleOnArc(angleOf(arc.center, candidate), arc.startDegrees, arc.endDegrees)) candidate
        else arcEndpoints(arc).minByOrNull { distance(p, it) }
    }
    private fun segmentIntersection(first: SegmentRef, second: SegmentRef): CadPoint? {
        val rx = first.b.x - first.a.x; val ry = first.b.y - first.a.y
        val sx = second.b.x - second.a.x; val sy = second.b.y - second.a.y
        val denominator = rx * sy - ry * sx
        if (abs(denominator) <= EPS) return null
        val qx = second.a.x - first.a.x; val qy = second.a.y - first.a.y
        val t = (qx * sy - qy * sx) / denominator
        val u = (qx * ry - qy * rx) / denominator
        return if (t in -EPS..(1.0 + EPS) && u in -EPS..(1.0 + EPS)) CadPoint(first.a.x + t * rx, first.a.y + t * ry) else null
    }
    private fun segmentCircleIntersections(segment: SegmentRef, center: CadPoint, radius: Double): List<CadPoint> {
        val dx = segment.b.x - segment.a.x; val dy = segment.b.y - segment.a.y
        val fx = segment.a.x - center.x; val fy = segment.a.y - center.y
        val a = dx * dx + dy * dy
        if (a <= EPS || radius <= 0.0) return emptyList()
        val b = 2.0 * (fx * dx + fy * dy)
        val c = fx * fx + fy * fy - radius * radius
        val discriminant = b * b - 4.0 * a * c
        if (discriminant < -EPS) return emptyList()
        val root = sqrt(max(0.0, discriminant))
        val t1 = (-b - root) / (2.0 * a); val t2 = (-b + root) / (2.0 * a)
        return listOf(t1, t2).filter { it in -EPS..(1.0 + EPS) }.distinctBy { (it * 1e9).toLong() }
            .map { CadPoint(segment.a.x + it * dx, segment.a.y + it * dy) }
    }
    private fun centerRadius(geometry: Geometry): Pair<CadPoint, Double>? = when (geometry) {
        is Geometry.Circle -> geometry.center to geometry.radius
        is Geometry.Arc -> geometry.center to geometry.radius
        else -> null
    }
    private fun onRoundGeometry(geometry: Geometry, point: CadPoint): Boolean = when (geometry) {
        is Geometry.Circle -> true
        is Geometry.Arc -> angleOnArc(angleOf(geometry.center, point), geometry.startDegrees, geometry.endDegrees)
        else -> false
    }
    private fun circleIntersections(c1: CadPoint, r1: Double, c2: CadPoint, r2: Double): List<CadPoint> {
        val dx = c2.x - c1.x; val dy = c2.y - c1.y; val d = hypot(dx, dy)
        if (d <= EPS || d > r1 + r2 + EPS || d < abs(r1 - r2) - EPS || r1 <= 0.0 || r2 <= 0.0) return emptyList()
        val a = (r1 * r1 - r2 * r2 + d * d) / (2.0 * d)
        val h2 = r1 * r1 - a * a
        if (h2 < -EPS) return emptyList()
        val h = sqrt(max(0.0, h2)); val xm = c1.x + a * dx / d; val ym = c1.y + a * dy / d
        val rx = -dy * h / d; val ry = dx * h / d
        val first = CadPoint(xm + rx, ym + ry)
        val second = CadPoint(xm - rx, ym - ry)
        return if (distance(first, second) <= EPS) listOf(first) else listOf(first, second)
    }
}
