package com.arcad.app

private fun spatialCheck(value: Boolean, message: String) { if (!value) error(message) }

fun main() {
    val geometries = buildList<CadObjectSnap.Geometry> {
        repeat(10_000) { index ->
            val y = 10_000.0 + index * 10.0
            add(CadObjectSnap.Geometry.Segment(CadPoint(-5.0, y), CadPoint(5.0, y)))
        }
        add(CadObjectSnap.Geometry.Segment(CadPoint(-10.0, 0.0), CadPoint(10.0, 0.0)))
        add(CadObjectSnap.Geometry.Circle(CadPoint(100.0, 100.0), 8.0))
        add(CadObjectSnap.Geometry.Arc(CadPoint(200.0, 200.0), 12.0, 0.0, 90.0))
        add(CadObjectSnap.Geometry.Polyline(listOf(CadPoint(300.0, 300.0), CadPoint(310.0, 300.0), CadPoint(310.0, 310.0)), false))
    }
    val index = CadSpatialIndex.build(geometries)
    spatialCheck(index.indexedGeometryCount == geometries.size, "index dropped valid geometry")
    val local = index.query(CadSpatialIndex.Bounds(-1.0, -1.0, 1.0, 1.0))
    spatialCheck(local.size == 1 && local.single() == geometries[10_000], "local query returned incorrect candidates")
    spatialCheck(index.queryCount(CadSpatialIndex.Bounds(91.0, 91.0, 109.0, 109.0)) == 1, "circle bounding-box query failed")
    spatialCheck(index.queryCount(CadSpatialIndex.Bounds(188.0, 188.0, 212.0, 212.0)) == 1, "arc conservative bounds query failed")
    spatialCheck(index.queryCount(CadSpatialIndex.Bounds(304.0, 304.0, 306.0, 306.0)) == 1, "polyline bounds query failed")
    spatialCheck(index.queryCount(CadSpatialIndex.Bounds(50_000.0, 50_000.0, 50_001.0, 50_001.0)) == 0, "empty region should return no candidates")

    // Indexed candidate resolution must agree with the full resolver for a representative
    // mix of endpoints, intersections, circles and arcs.
    val mixed = listOf(
        CadObjectSnap.Geometry.Segment(CadPoint(0.0, 0.0), CadPoint(20.0, 0.0)),
        CadObjectSnap.Geometry.Segment(CadPoint(10.0, -10.0), CadPoint(10.0, 10.0)),
        CadObjectSnap.Geometry.Circle(CadPoint(40.0, 40.0), 5.0),
        CadObjectSnap.Geometry.Arc(CadPoint(60.0, 60.0), 5.0, 0.0, 90.0)
    )
    val mixedIndex = CadSpatialIndex.build(mixed)
    val probes = listOf(CadPoint(10.0, 0.0), CadPoint(0.1, 0.1), CadPoint(45.0, 40.0), CadPoint(60.0, 65.0))
    probes.forEach { cursor ->
        val radius = 0.5
        val window = CadSpatialIndex.Bounds(cursor.x - radius, cursor.y - radius, cursor.x + radius, cursor.y + radius)
        val full = CadObjectSnap.resolve(cursor, mixed, radius)
        val indexed = CadObjectSnap.resolve(cursor, mixedIndex.query(window), radius)
        spatialCheck(full?.kind == indexed?.kind && (full == null || indexed != null &&
            kotlin.math.abs(full.point.x - indexed.point.x) < 1e-8 && kotlin.math.abs(full.point.y - indexed.point.y) < 1e-8),
            "indexed snap differs from full scan at $cursor")
    }
    spatialCheck(runCatching { CadSpatialIndex.Bounds(2.0, 0.0, 1.0, 1.0) }.isFailure, "inverted bounds accepted")
    spatialCheck(runCatching { CadSpatialIndex.build(listOf(CadObjectSnap.Geometry.Circle(CadPoint(0.0, 0.0), Double.NaN))) }.isSuccess,
        "index should safely skip malformed geometry rather than crash construction")
    println("CAD spatial index tests passed: 10,004 geometry build, local query pruning, circle/arc/polyline bounds, empty regions and indexed-vs-full snap equivalence")
}
