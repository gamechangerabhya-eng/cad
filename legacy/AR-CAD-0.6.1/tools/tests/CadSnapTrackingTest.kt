package com.arcad.app

import kotlin.math.abs
import kotlin.math.hypot

private fun trackingCheck(value: Boolean, message: String) { if (!value) error(message) }
private fun trackingNear(a: Double, b: Double, e: Double = 1e-7) = abs(a - b) <= e

fun main() {
    val horizontal = CadObjectSnap.Geometry.Segment(CadPoint(0.0, 0.0), CadPoint(10.0, 0.0))

    val extension = CadSnapTracking.resolve(
        cursor = CadPoint(13.0, 0.2), anchor = null, geometries = listOf(horizontal), tolerance = 0.5,
        enabledKinds = setOf(CadSnapTracking.Kind.EXTENSION)
    )
    trackingCheck(extension?.kind == CadSnapTracking.Kind.EXTENSION, "extension tracking candidate not found")
    trackingCheck(trackingNear(extension!!.point.x, 13.0) && trackingNear(extension.point.y, 0.0), "extension projection incorrect")
    trackingCheck(extension.guides.isNotEmpty(), "extension guide lines missing")

    val noBackProjection = CadSnapTracking.resolve(
        cursor = CadPoint(7.0, 0.1), anchor = null, geometries = listOf(horizontal), tolerance = 0.5,
        enabledKinds = setOf(CadSnapTracking.Kind.EXTENSION)
    )
    trackingCheck(noBackProjection == null, "extension tracking incorrectly projected onto finite segment")

    val vertical = CadObjectSnap.Geometry.Segment(CadPoint(5.0, -5.0), CadPoint(5.0, 5.0))
    val perpendicular = CadSnapTracking.resolve(
        cursor = CadPoint(5.1, 2.0), anchor = CadPoint(1.0, 2.0), geometries = listOf(vertical), tolerance = 0.5,
        enabledKinds = setOf(CadSnapTracking.Kind.PERPENDICULAR)
    )
    trackingCheck(perpendicular?.kind == CadSnapTracking.Kind.PERPENDICULAR, "perpendicular tracking not found")
    trackingCheck(trackingNear(perpendicular!!.point.x, 5.0) && trackingNear(perpendicular.point.y, 2.0), "perpendicular foot incorrect")
    trackingCheck(perpendicular.guides.any { trackingNear(it.start.x, 1.0) && trackingNear(it.end.x, 5.0) }, "perpendicular construction guide missing")

    val circle = CadObjectSnap.Geometry.Circle(CadPoint(0.0, 0.0), 5.0)
    val tangent = CadSnapTracking.resolve(
        cursor = CadPoint(2.5, 4.330127018922193), anchor = CadPoint(10.0, 0.0), geometries = listOf(circle), tolerance = 0.1,
        enabledKinds = setOf(CadSnapTracking.Kind.TANGENT)
    )
    trackingCheck(tangent?.kind == CadSnapTracking.Kind.TANGENT, "tangent tracking candidate not found")
    trackingCheck(trackingNear(hypot(tangent!!.point.x, tangent.point.y), 5.0), "tangent point is not on circle")
    val radiusDotTangent = tangent.point.x * (tangent.point.x - 10.0) + tangent.point.y * tangent.point.y
    trackingCheck(abs(radiusDotTangent) < 1e-6, "tangent is not perpendicular to radius")

    val insideCircle = CadSnapTracking.resolve(
        cursor = CadPoint(4.0, 0.0), anchor = CadPoint(1.0, 0.0), geometries = listOf(circle), tolerance = 1.0,
        enabledKinds = setOf(CadSnapTracking.Kind.TANGENT)
    )
    trackingCheck(insideCircle == null, "tangent incorrectly generated from inside circle")

    val arc = CadObjectSnap.Geometry.Arc(CadPoint(0.0, 0.0), 5.0, 0.0, 90.0)
    val outsideSweep = CadSnapTracking.resolve(
        cursor = CadPoint(2.5, -4.330127018922193), anchor = CadPoint(10.0, 0.0), geometries = listOf(arc), tolerance = 0.2,
        enabledKinds = setOf(CadSnapTracking.Kind.TANGENT)
    )
    trackingCheck(outsideSweep == null, "tangent snapped to point outside arc sweep")

    trackingCheck(runCatching {
        CadSnapTracking.resolve(CadPoint(Double.NaN, 0.0), null, listOf(horizontal), 1.0)
    }.isFailure, "non-finite cursor accepted")
    trackingCheck(runCatching {
        CadSnapTracking.resolve(CadPoint(0.0, 0.0), null, listOf(horizontal), -1.0)
    }.isFailure, "negative tolerance accepted")

    println("CAD snap tracking tests passed: extension rays, perpendicular projection, circle tangents, arc sweep filtering and input guards")
}
