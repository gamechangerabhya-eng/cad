package com.arcad.app

import kotlin.math.abs
import kotlin.math.atan2
import kotlin.math.hypot
import kotlin.math.max
import kotlin.math.min
import kotlin.math.sqrt

/**
 * Construction tracking candidates used by precision drafting.
 * All coordinates and tolerances are in model units. This helper is deliberately
 * independent of Compose so the geometry can be tested on the host JVM.
 */
object CadSnapTracking {
    private const val EPS = 1e-9

    enum class Kind { EXTENSION, PERPENDICULAR, TANGENT }
    data class Guide(val start: CadPoint, val end: CadPoint)
    data class Result(val point: CadPoint, val kind: Kind, val distance: Double, val guides: List<Guide>)

    /**
     * Resolve construction-tracking candidates near [cursor]. [anchor] is the
     * previous committed point in a command (for perpendicular/tangent tracking).
     * Extension tracking projects beyond segment endpoints, never back onto the
     * finite segment. Tangencies are calculated from [anchor] to circles/arcs.
     */
    fun resolve(
        cursor: CadPoint,
        anchor: CadPoint?,
        geometries: List<CadObjectSnap.Geometry>,
        tolerance: Double,
        enabledKinds: Set<Kind> = Kind.values().toSet()
    ): Result? {
        require(cursor.x.isFinite() && cursor.y.isFinite()) { "Tracking cursor must be finite" }
        require(tolerance.isFinite() && tolerance >= 0.0) { "Tracking tolerance must be finite and non-negative" }
        if (tolerance <= 0.0 || geometries.isEmpty() || enabledKinds.isEmpty()) return null
        if (anchor != null) require(anchor.x.isFinite() && anchor.y.isFinite()) { "Tracking anchor must be finite" }

        val candidates = ArrayList<Result>()
        fun add(point: CadPoint, kind: Kind, guides: List<Guide>) {
            if (!point.x.isFinite() || !point.y.isFinite()) return
            val distance = hypot(cursor.x - point.x, cursor.y - point.y)
            if (distance <= tolerance + EPS) candidates += Result(point, kind, distance, guides)
        }

        if (Kind.EXTENSION in enabledKinds) {
            geometries.forEach { geometry ->
                val segments = when (geometry) {
                    is CadObjectSnap.Geometry.Segment -> listOf(geometry.a to geometry.b)
                    is CadObjectSnap.Geometry.Polyline -> buildList {
                        geometry.points.zipWithNext().forEach { add(it.first to it.second) }
                        if (geometry.closed && geometry.points.size > 2) add(geometry.points.last() to geometry.points.first())
                    }
                    else -> emptyList()
                }
                segments.forEach segmentLoop@ { (a, b) ->
                    val dx = b.x - a.x; val dy = b.y - a.y
                    val length = hypot(dx, dy)
                    if (length <= EPS) return@segmentLoop
                    val ux = dx / length; val uy = dy / length
                    // Ray extending beyond b in the segment's forward direction.
                    projectToRay(cursor, b, ux, uy)?.takeIf { it.t > EPS }?.let { projection ->
                        add(projection.point, Kind.EXTENSION, listOf(Guide(a, b), Guide(b, projection.point)))
                    }
                    // Ray extending beyond a in the reverse direction.
                    projectToRay(cursor, a, -ux, -uy)?.takeIf { it.t > EPS }?.let { projection ->
                        add(projection.point, Kind.EXTENSION, listOf(Guide(a, b), Guide(a, projection.point)))
                    }
                }
            }
        }

        if (anchor != null && Kind.PERPENDICULAR in enabledKinds) {
            geometries.forEach { geometry ->
                val segments = when (geometry) {
                    is CadObjectSnap.Geometry.Segment -> listOf(geometry.a to geometry.b)
                    is CadObjectSnap.Geometry.Polyline -> buildList {
                        geometry.points.zipWithNext().forEach { add(it.first to it.second) }
                        if (geometry.closed && geometry.points.size > 2) add(geometry.points.last() to geometry.points.first())
                    }
                    else -> emptyList()
                }
                segments.forEach segmentLoop@ { (a, b) ->
                    val foot = projectionOnLine(anchor, a, b) ?: return@segmentLoop
                    // Perpendicular object snap is meaningful only when the foot lies on the segment.
                    if (foot.t in -EPS..(1.0 + EPS)) {
                        add(foot.point, Kind.PERPENDICULAR, listOf(Guide(anchor, foot.point), Guide(a, b)))
                    }
                }
            }
        }

        if (anchor != null && Kind.TANGENT in enabledKinds) {
            geometries.forEach { geometry ->
                val circle = when (geometry) {
                    is CadObjectSnap.Geometry.Circle -> geometry.center to geometry.radius
                    is CadObjectSnap.Geometry.Arc -> geometry.center to geometry.radius
                    else -> null
                } ?: return@forEach
                val tangentPoints = tangentPoints(anchor, circle.first, circle.second)
                tangentPoints.forEach tangentLoop@ { tangent ->
                    if (geometry is CadObjectSnap.Geometry.Arc && !angleOnArc(angleOf(circle.first, tangent), geometry.startDegrees, geometry.endDegrees)) return@tangentLoop
                    add(tangent, Kind.TANGENT, listOf(Guide(anchor, tangent)))
                }
            }
        }

        return candidates.minWithOrNull(compareBy<Result> { priority(it.kind) }.thenBy { it.distance })
    }

    private data class Projection(val point: CadPoint, val t: Double)
    private fun projectToRay(p: CadPoint, origin: CadPoint, ux: Double, uy: Double): Projection? {
        val t = (p.x - origin.x) * ux + (p.y - origin.y) * uy
        if (!t.isFinite()) return null
        return Projection(CadPoint(origin.x + t * ux, origin.y + t * uy), t)
    }
    private fun projectionOnLine(p: CadPoint, a: CadPoint, b: CadPoint): Projection? {
        val dx = b.x - a.x; val dy = b.y - a.y
        val length2 = dx * dx + dy * dy
        if (!length2.isFinite() || length2 <= EPS) return null
        val t = ((p.x - a.x) * dx + (p.y - a.y) * dy) / length2
        return Projection(CadPoint(a.x + t * dx, a.y + t * dy), t)
    }
    private fun tangentPoints(from: CadPoint, center: CadPoint, radius: Double): List<CadPoint> {
        if (!radius.isFinite() || radius <= 0.0) return emptyList()
        val dx = from.x - center.x; val dy = from.y - center.y
        val d2 = dx * dx + dy * dy
        val r2 = radius * radius
        if (!d2.isFinite() || d2 <= r2 + EPS) return emptyList()
        val baseScale = r2 / d2
        val sideScale = radius * sqrt(max(0.0, d2 - r2)) / d2
        val first = CadPoint(center.x + baseScale * dx - sideScale * dy, center.y + baseScale * dy + sideScale * dx)
        val second = CadPoint(center.x + baseScale * dx + sideScale * dy, center.y + baseScale * dy - sideScale * dx)
        return if (hypot(first.x - second.x, first.y - second.y) <= EPS) listOf(first) else listOf(first, second)
    }
    private fun angleOf(center: CadPoint, point: CadPoint) = Math.toDegrees(atan2(point.y - center.y, point.x - center.x))
    private fun normalize(degrees: Double) = ((degrees % 360.0) + 360.0) % 360.0
    private fun angleOnArc(angle: Double, start: Double, end: Double): Boolean {
        val sweep = end - start
        if (abs(sweep) >= 359.999999) return true
        return if (sweep >= 0.0) normalize(angle - start) <= sweep + EPS else normalize(start - angle) <= -sweep + EPS
    }
    private fun priority(kind: Kind) = when (kind) {
        Kind.PERPENDICULAR -> 0
        Kind.TANGENT -> 1
        Kind.EXTENSION -> 2
    }
}
