package com.arcad.app

import kotlin.math.abs

private fun near(actual: Double, expected: Double, epsilon: Double = 1e-8) {
    check(abs(actual - expected) <= epsilon) { "Expected $expected, got $actual" }
}

fun main() {
    val absolute = CadDynamicInput.parsePoint("125.5,-20")
    near(absolute.x, 125.5); near(absolute.y, -20.0)

    val base = CadPoint(100.0, 200.0)
    val relative = CadDynamicInput.parsePoint("@10,-5", base)
    near(relative.x, 110.0); near(relative.y, 195.0)

    val polar = CadDynamicInput.parsePoint("@10<90", base)
    near(polar.x, 100.0); near(polar.y, 210.0)
    val polarNegative = CadDynamicInput.parsePoint("@5<-90", base)
    near(polarNegative.x, 100.0); near(polarNegative.y, 195.0)

    val zero = CadDynamicInput.parsePoint("@0<0", base)
    near(zero.x, base.x); near(zero.y, base.y)

    listOf("", "1", "1,2,3", "@10", "@-1<45", "@1<", "NaN,1", "Infinity,0", "1e99,0", "1<45").forEach { bad ->
        check(runCatching { CadDynamicInput.parsePoint(bad, base) }.isFailure) { "Expected rejection for '$bad'" }
    }
    println("CAD dynamic coordinate input tests passed")
}
