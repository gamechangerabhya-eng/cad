package com.arcad.app

import kotlin.math.cos
import kotlin.math.sin
import kotlin.math.PI
import kotlin.math.abs

/** Parses CAD-style coordinate entry independent of UI and locale. */
object CadDynamicInput {
    private const val MAX_COORDINATE = 1.0e9

    /**
     * Supported forms:
     *  - `x,y` absolute Cartesian coordinate
     *  - `@dx,dy` relative Cartesian displacement from [base]
     *  - `@distance<angle` relative polar displacement; angle is degrees
     */
    fun parsePoint(input: String, base: CadPoint = CadPoint(0.0, 0.0)): CadPoint {
        require(base.x.isFinite() && base.y.isFinite()) { "Base coordinate must be finite" }
        val text = input.trim().replace(" ", "")
        require(text.isNotEmpty()) { "Enter a coordinate" }
        val relative = text.startsWith("@")
        val body = if (relative) text.drop(1) else text
        require(body.isNotEmpty()) { "Coordinate is incomplete" }

        val result = if ('<' in body) {
            require(relative) { "Polar input must be relative, for example @25<45" }
            require(body.count { it == '<' } == 1 && ',' !in body) { "Invalid polar coordinate" }
            val parts = body.split('<')
            val distance = parts[0].toDoubleOrNull() ?: throw IllegalArgumentException("Invalid polar distance")
            val angle = parts[1].toDoubleOrNull() ?: throw IllegalArgumentException("Invalid polar angle")
            require(distance >= 0.0 && distance.isFinite() && angle.isFinite()) { "Polar values must be finite and distance non-negative" }
            val radians = angle * PI / 180.0
            CadPoint(base.x + distance * cos(radians), base.y + distance * sin(radians))
        } else {
            require(body.count { it == ',' } == 1) { "Use x,y or @dx,dy" }
            val parts = body.split(',')
            val x = parts[0].toDoubleOrNull() ?: throw IllegalArgumentException("Invalid X coordinate")
            val y = parts[1].toDoubleOrNull() ?: throw IllegalArgumentException("Invalid Y coordinate")
            require(x.isFinite() && y.isFinite()) { "Coordinates must be finite" }
            if (relative) CadPoint(base.x + x, base.y + y) else CadPoint(x, y)
        }
        require(result.x.isFinite() && result.y.isFinite()) { "Calculated coordinate is not finite" }
        require(abs(result.x) <= MAX_COORDINATE && abs(result.y) <= MAX_COORDINATE) { "Coordinate exceeds supported drawing range" }
        return result
    }
}
