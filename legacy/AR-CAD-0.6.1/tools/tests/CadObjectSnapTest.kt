package com.arcad.app

private fun snapCheck(value: Boolean, message: String) { if (!value) error(message) }
private fun near(actual: Double, expected: Double, epsilon: Double = 1e-7) = kotlin.math.abs(actual - expected) <= epsilon

fun main() {
    val line = CadObjectSnap.Geometry.Segment(CadPoint(0.0, 0.0), CadPoint(10.0, 0.0))
    val endpoint = CadObjectSnap.resolve(CadPoint(0.2, 0.1), listOf(line), 0.5)
    snapCheck(endpoint?.kind == CadObjectSnap.Kind.ENDPOINT && near(endpoint.point.x, 0.0), "endpoint snap failed")

    val midpoint = CadObjectSnap.resolve(CadPoint(5.0, 0.2), listOf(line), 0.5)
    snapCheck(midpoint?.kind == CadObjectSnap.Kind.MIDPOINT && near(midpoint.point.x, 5.0), "midpoint snap failed")

    val crossing = listOf(
        line,
        CadObjectSnap.Geometry.Segment(CadPoint(5.0, -5.0), CadPoint(5.0, 5.0))
    )
    val intersection = CadObjectSnap.resolve(CadPoint(5.0, 0.0), crossing, 0.1)
    snapCheck(intersection?.kind == CadObjectSnap.Kind.INTERSECTION && near(intersection.point.y, 0.0), "segment intersection snap failed")

    val circle = CadObjectSnap.Geometry.Circle(CadPoint(20.0, 20.0), 5.0)
    val center = CadObjectSnap.resolve(CadPoint(20.1, 20.0), listOf(circle), 0.5)
    snapCheck(center?.kind == CadObjectSnap.Kind.CENTER && near(center.point.x, 20.0), "circle center snap failed")
    val quadrant = CadObjectSnap.resolve(CadPoint(25.0, 20.1), listOf(circle), 0.5)
    snapCheck(quadrant?.kind == CadObjectSnap.Kind.QUADRANT && near(quadrant.point.x, 25.0), "circle quadrant snap failed")
    val nearest = CadObjectSnap.resolve(CadPoint(20.0, 24.0), listOf(circle), 1.1)
    snapCheck(nearest != null && near(nearest.point.y, 25.0), "circle nearest snap failed")

    val lineCircle = listOf(
        CadObjectSnap.Geometry.Segment(CadPoint(10.0, 20.0), CadPoint(30.0, 20.0)), circle
    )
    val hit = CadObjectSnap.resolve(CadPoint(25.0, 20.0), lineCircle, 0.1)
    snapCheck(hit?.kind == CadObjectSnap.Kind.INTERSECTION && near(hit.point.x, 25.0), "segment-circle intersection failed")

    val twoCircles = listOf(
        CadObjectSnap.Geometry.Circle(CadPoint(0.0, 0.0), 5.0),
        CadObjectSnap.Geometry.Circle(CadPoint(6.0, 0.0), 5.0)
    )
    val cc = CadObjectSnap.resolve(CadPoint(3.0, 4.0), twoCircles, 0.1)
    snapCheck(cc?.kind == CadObjectSnap.Kind.INTERSECTION && near(cc.point.x, 3.0) && near(cc.point.y, 4.0), "circle-circle intersection failed")

    val arc = CadObjectSnap.Geometry.Arc(CadPoint(0.0, 0.0), 5.0, 0.0, 90.0)
    val validQuadrant = CadObjectSnap.resolve(CadPoint(0.0, 5.0), listOf(arc), 0.1)
    snapCheck(validQuadrant?.kind == CadObjectSnap.Kind.ENDPOINT || validQuadrant?.kind == CadObjectSnap.Kind.QUADRANT, "arc in-sweep snap failed")
    val invalidQuadrant = CadObjectSnap.resolve(CadPoint(-5.0, 0.0), listOf(arc), 0.5)
    snapCheck(invalidQuadrant == null, "arc snapped outside its sweep")

    val polyline = CadObjectSnap.Geometry.Polyline(listOf(CadPoint(0.0, 0.0), CadPoint(10.0, 0.0), CadPoint(10.0, 10.0)), false)
    val polyMid = CadObjectSnap.resolve(CadPoint(10.0, 5.0), listOf(polyline), 0.1)
    snapCheck(polyMid?.kind == CadObjectSnap.Kind.MIDPOINT && near(polyMid.point.y, 5.0), "polyline segment midpoint failed")


    // Individual mode toggles: disabled candidates must not win, while enabled
    // candidates continue to resolve normally.
    val endpointOnly = CadObjectSnap.resolve(
        CadPoint(5.0, 0.2), listOf(line), 0.5, setOf(CadObjectSnap.Kind.ENDPOINT)
    )
    snapCheck(endpointOnly == null, "disabled midpoint/nearest modes still attracted cursor")
    val midpointOnly = CadObjectSnap.resolve(
        CadPoint(5.0, 0.2), listOf(line), 0.5, setOf(CadObjectSnap.Kind.MIDPOINT)
    )
    snapCheck(midpointOnly?.kind == CadObjectSnap.Kind.MIDPOINT, "enabled midpoint mode did not resolve")
    val intersectionDisabled = CadObjectSnap.resolve(
        CadPoint(5.0, 0.0), crossing, 0.1, setOf(CadObjectSnap.Kind.ENDPOINT, CadObjectSnap.Kind.NEAREST)
    )
    snapCheck(intersectionDisabled?.kind != CadObjectSnap.Kind.INTERSECTION, "disabled intersection mode still resolved")
    snapCheck(CadObjectSnap.resolve(CadPoint(0.0, 0.0), listOf(line), 1.0, emptySet()) == null, "empty mode set should disable snapping")

    snapCheck(CadObjectSnap.resolve(CadPoint(100.0, 100.0), listOf(line), 1.0) == null, "out-of-tolerance point snapped")
    snapCheck(runCatching { CadObjectSnap.resolve(CadPoint(Double.NaN, 0.0), listOf(line), 1.0) }.isFailure, "non-finite cursor was accepted")
    snapCheck(runCatching { CadObjectSnap.resolve(CadPoint(0.0, 0.0), listOf(line), -1.0) }.isFailure, "negative tolerance was accepted")

    val largeDrawing = buildList {
        repeat(2_000) { index ->
            val y = 1_000.0 + index * 3.0
            add(CadObjectSnap.Geometry.Segment(CadPoint(-1_000.0, y), CadPoint(1_000.0, y)))
        }
        add(CadObjectSnap.Geometry.Segment(CadPoint(-2.0, 0.0), CadPoint(2.0, 0.0)))
        add(CadObjectSnap.Geometry.Segment(CadPoint(0.0, -2.0), CadPoint(0.0, 2.0)))
    }
    val localHit = CadObjectSnap.resolve(CadPoint(0.0, 0.0), largeDrawing, 0.1)
    snapCheck(localHit?.kind == CadObjectSnap.Kind.INTERSECTION && near(localHit.point.x, 0.0), "near-cursor intersection search failed in a large drawing")

    println("CAD object snap tests passed: endpoint, midpoint, center, quadrant, nearest, segment/segment, segment/circle, circle/circle, arc sweep, tolerance guards, per-mode enable/disable and 2,002-entity local-intersection filtering")
}
