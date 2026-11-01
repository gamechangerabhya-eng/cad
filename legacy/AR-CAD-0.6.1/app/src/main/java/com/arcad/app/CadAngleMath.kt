package com.arcad.app

/** Converts between DXF's counter-clockwise Y-up arc angles and the canvas model's Y-down angles. */
data class ArcAnglePair(val startDegrees: Double, val endDegrees: Double)

object CadAngleMath {
    fun normalize(degrees: Double): Double = ((degrees % 360.0) + 360.0) % 360.0

    fun dxfToCanvas(startDegrees: Double, endDegrees: Double): ArcAnglePair {
        val start = -normalize(startDegrees)
        val sweep = normalize(endDegrees - startDegrees)
        return ArcAnglePair(start, start - sweep)
    }

    fun canvasToDxf(startDegrees: Double, endDegrees: Double): ArcAnglePair {
        val sweep = endDegrees - startDegrees
        return if (sweep <= 0.0) {
            ArcAnglePair(normalize(-startDegrees), normalize(-endDegrees))
        } else {
            // A reflected canvas arc reverses direction; swap its endpoints for DXF's CCW convention.
            ArcAnglePair(normalize(-endDegrees), normalize(-startDegrees))
        }
    }
}
