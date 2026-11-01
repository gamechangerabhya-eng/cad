package com.arcad.app

fun main() {
    val gate = NativePreviewGate<List<Int>>()
    val original = listOf(1, 2, 3)
    val first = gate.begin(original)
    check(gate.isCurrent(first, original))
    check(!gate.isCurrent(first, listOf(1, 2, 4)))

    val second = gate.begin(original)
    check(second > first)
    check(!gate.isCurrent(first, original)) // superseded request
    check(gate.isCurrent(second, original))

    gate.invalidate()
    check(!gate.isCurrent(second, original))
    val third = gate.begin(original)
    check(gate.isCurrent(third, original))
    println("Native preview request gate tests passed")
}
