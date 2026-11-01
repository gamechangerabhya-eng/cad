package com.arcad.app

import kotlin.random.Random

/** Deterministic generated corpus to catch serialization regressions across mixed drawings. */
fun runDxfCodecPropertyTests() {
    val random = Random(0xA4CAD)
    fun coordinate(): Double = random.nextInt(-500_000, 500_001) / 100.0
    val entities = buildList {
        repeat(500) { index ->
            val layer = listOf("0", "WALLS", "MEP", "STRUCTURE")[index % 4]
            when (index % 4) {
                0 -> add(DxfEntity.Line(coordinate(), coordinate(), coordinate(), coordinate(), layer))
                1 -> add(DxfEntity.Circle(coordinate(), coordinate(), random.nextInt(1, 50_000) / 100.0, layer))
                2 -> add(DxfEntity.Arc(coordinate(), coordinate(), random.nextInt(1, 50_000) / 100.0, random.nextInt(0, 360).toDouble(), random.nextInt(0, 360).toDouble(), layer))
                else -> add(DxfEntity.Polyline(List(random.nextInt(2, 9)) { coordinate() to coordinate() }, random.nextBoolean(), layer))
            }
        }
    }
    val encoded = CadDxfCodec.writeAscii(entities, listOf("AUX", "NOTES"))
    val decoded = CadDxfCodec.parseAscii(encoded)
    check(decoded.entities == entities) { "Generated mixed-entity DXF round-trip changed geometry" }
    check(setOf("0", "WALLS", "MEP", "STRUCTURE", "AUX", "NOTES").all { it in decoded.layers }) {
        "Generated mixed-entity DXF round-trip lost layer names"
    }
    println("DXF property tests passed: 500 deterministic mixed entities and layer round-trip")
}

fun main() = runDxfCodecPropertyTests()
