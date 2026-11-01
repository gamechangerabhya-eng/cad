package com.arcad.app

import kotlin.math.max
import kotlin.math.min

/**
 * Immutable packed AABB/BVH index for model-space CAD snap geometry.
 * Build once when the visible drawing changes; query a small cursor window per pointer event.
 * Geometry bounds are conservative (arcs use their full circle bounds), so queries may
 * return harmless false positives but must never omit a possible snap candidate.
 */
class CadSpatialIndex private constructor(
    private val geometries: List<CadObjectSnap.Geometry>,
    private val root: Node?,
    val indexedGeometryCount: Int
) {
    data class Bounds(val minX: Double, val minY: Double, val maxX: Double, val maxY: Double) {
        init {
            require(minX.isFinite() && minY.isFinite() && maxX.isFinite() && maxY.isFinite())
            require(minX <= maxX && minY <= maxY)
        }
        fun intersects(other: Bounds): Boolean =
            minX <= other.maxX && maxX >= other.minX && minY <= other.maxY && maxY >= other.minY
        fun union(other: Bounds) = Bounds(
            min(minX, other.minX), min(minY, other.minY),
            max(maxX, other.maxX), max(maxY, other.maxY)
        )
        val centerX get() = (minX + maxX) * 0.5
        val centerY get() = (minY + maxY) * 0.5
    }

    private sealed interface Node { val bounds: Bounds
        data class Branch(override val bounds: Bounds, val left: Node, val right: Node) : Node
        data class Leaf(override val bounds: Bounds, val indices: IntArray) : Node
    }

    /** Returns geometry whose conservative bounds intersect [window]. */
    fun query(window: Bounds): List<CadObjectSnap.Geometry> {
        val tree = root ?: return emptyList()
        val found = ArrayList<Int>()
        val stack = ArrayDeque<Node>()
        stack.add(tree)
        while (stack.isNotEmpty()) {
            when (val node = stack.removeLast()) {
                is Node.Branch -> if (node.bounds.intersects(window)) {
                    stack.add(node.left)
                    stack.add(node.right)
                }
                is Node.Leaf -> if (node.bounds.intersects(window)) {
                    node.indices.forEach { index -> if (boundsOf(geometries[index]).intersects(window)) found += index }
                }
            }
        }
        return found.map { geometries[it] }
    }

    /** Number of leaf geometry candidates in a query, useful for deterministic performance tests. */
    fun queryCount(window: Bounds): Int = query(window).size

    companion object {
        private const val LEAF_SIZE = 8

        fun build(geometries: List<CadObjectSnap.Geometry>): CadSpatialIndex {
            val safe = geometries.toList()
            val entries = safe.mapIndexedNotNull { index, geometry ->
                runCatching { boundsOf(geometry) }.getOrNull()?.let { Entry(index, it) }
            }
            fun buildNode(items: List<Entry>): Node? {
                if (items.isEmpty()) return null
                val bounds = items.drop(1).fold(items.first().bounds) { acc, entry -> acc.union(entry.bounds) }
                if (items.size <= LEAF_SIZE) return Node.Leaf(bounds, items.map { it.index }.toIntArray())
                val splitX = (bounds.maxX - bounds.minX) >= (bounds.maxY - bounds.minY)
                val sorted = if (splitX) items.sortedBy { it.bounds.centerX } else items.sortedBy { it.bounds.centerY }
                val middle = sorted.size / 2
                val left = buildNode(sorted.subList(0, middle)) ?: return null
                val right = buildNode(sorted.subList(middle, sorted.size)) ?: return null
                return Node.Branch(bounds, left, right)
            }
            return CadSpatialIndex(safe, buildNode(entries), entries.size)
        }

        private data class Entry(val index: Int, val bounds: Bounds)

        fun boundsOf(geometry: CadObjectSnap.Geometry): Bounds = when (geometry) {
            is CadObjectSnap.Geometry.Segment -> fromPoints(listOf(geometry.a, geometry.b))
            is CadObjectSnap.Geometry.Polyline -> fromPoints(geometry.points)
            is CadObjectSnap.Geometry.Circle -> circleBounds(geometry.center, geometry.radius)
            is CadObjectSnap.Geometry.Arc -> circleBounds(geometry.center, geometry.radius)
        }

        private fun circleBounds(center: CadPoint, radius: Double): Bounds {
            require(radius.isFinite() && radius >= 0.0)
            return Bounds(center.x - radius, center.y - radius, center.x + radius, center.y + radius)
        }

        private fun fromPoints(points: List<CadPoint>): Bounds {
            require(points.isNotEmpty())
            require(points.all { it.x.isFinite() && it.y.isFinite() })
            return Bounds(
                points.minOf { it.x }, points.minOf { it.y },
                points.maxOf { it.x }, points.maxOf { it.y }
            )
        }
    }
}
