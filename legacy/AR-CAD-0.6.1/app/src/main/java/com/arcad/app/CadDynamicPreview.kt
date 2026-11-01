package com.arcad.app

import kotlin.math.atan2
import kotlin.math.hypot

/** Live drafting metrics between a fixed first point and the current pointer. */
object CadDynamicPreview {
    data class Metrics(val dx: Double, val dy: Double, val length: Double, val angleDegrees: Double)

    fun measure(base: CadPoint, cursor: CadPoint): Metrics {
        require(base.x.isFinite() && base.y.isFinite() && cursor.x.isFinite() && cursor.y.isFinite()) {
            "Preview coordinates must be finite"
        }
        val dx = cursor.x - base.x
        val dy = cursor.y - base.y
        val length = hypot(dx, dy)
        val angle = if (length <= 1e-12) 0.0 else Math.toDegrees(atan2(dy, dx))
        require(dx.isFinite() && dy.isFinite() && length.isFinite() && angle.isFinite()) {
            "Preview metrics exceed supported numeric range"
        }
        return Metrics(dx, dy, length, angle)
    }
}
