package com.arcad.app

private fun assertNear(a: Double, b: Double) { check(kotlin.math.abs(a - b) < 1e-7) { "$a != $b" } }

fun main() {
    val sample = """0
SECTION
2
ENTITIES
0
LINE
8
WALLS
10
1.25
20
2.5
11
10
21
20
0
CIRCLE
8
0
10
5
20
6
40
2
0
ARC
8
CURVES
10
10
20
10
40
3
50
0
51
90
0
LWPOLYLINE
8
BOUNDARY
90
4
70
1
10
0
20
0
10
10
20
0
10
10
20
5
10
0
20
5
0
ENDSEC
0
EOF
"""
    val parsed = CadDxfCodec.parseAscii(sample)
    check(parsed.entities.size == 4) { "Expected 4 entities, got ${parsed.entities.size}" }
    check(parsed.skippedTypes.isEmpty())
    val line = parsed.entities[0] as DxfEntity.Line
    assertNear(line.x2, 10.0); check(line.layer == "WALLS")
    val circle = parsed.entities[1] as DxfEntity.Circle
    assertNear(circle.radius, 2.0)
    val arc = parsed.entities[2] as DxfEntity.Arc
    assertNear(arc.endDegrees, 90.0)
    val poly = parsed.entities[3] as DxfEntity.Polyline
    check(poly.closed && poly.points.size == 4)

    val encoded = CadDxfCodec.writeAscii(parsed.entities, listOf("ARCHITECTURE", "0"))
    check(encoded.contains("ARCHITECTURE")) { "Layer table was not written" }
    val roundTrip = CadDxfCodec.parseAscii(encoded)
    check(roundTrip.entities == parsed.entities) { "DXF round trip changed entities: ${roundTrip.entities}" }
    check("ARCHITECTURE" in roundTrip.layers && "BOUNDARY" in roundTrip.layers) { "Layer table/entity layers not recovered: ${roundTrip.layers}" }
    check(roundTrip.unitsCode == 4) { "Exported DXF should declare millimetres" }

    val inches = "0\nSECTION\n2\nHEADER\n9\n\$INSUNITS\n70\n1\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nLINE\n10\n1\n20\n2\n11\n3\n21\n4\n0\nENDSEC\n0\nEOF\n"
    val inchLine = CadDxfCodec.parseAscii(inches).entities.single() as DxfEntity.Line
    assertNear(inchLine.x1, 25.4); assertNear(inchLine.y1, 50.8)

    val unknown = sample.replace("0\nARC\n", "0\nSPLINE\n")
    val partial = CadDxfCodec.parseAscii(unknown)
    check(partial.skippedTypes["SPLINE"] == 1)
    val blockDrawing = "0\nSECTION\n2\nBLOCKS\n0\nBLOCK\n2\nB1\n0\nLINE\n10\n0\n20\n0\n11\n1\n21\n1\n0\nENDBLK\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nLINE\n10\n0\n20\n0\n11\n2\n21\n2\n0\nENDSEC\n0\nEOF\n"
    val blockResult = CadDxfCodec.parseAscii(blockDrawing)
    check(blockResult.skippedTypes["BLOCKS section objects"] == 1) { "BLOCKS loss was not reported" }
    val styles = "0\nSECTION\n2\nTABLES\n0\nTABLE\n2\nDIMSTYLE\n70\n1\n0\nDIMSTYLE\n2\nSTANDARD\n0\nENDTAB\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nLINE\n10\n0\n20\n0\n11\n1\n21\n1\n0\nENDSEC\n0\nEOF\n"
    check(CadDxfCodec.parseAscii(styles).skippedTypes["TABLES DIMSTYLE table"] == 1) { "Style table loss was not reported" }

    // Never flatten unsupported 3D or bulged geometry into misleading 2D chords.
    val elevated = "0\nSECTION\n2\nENTITIES\n0\nLINE\n10\n0\n20\n0\n30\n5\n11\n10\n21\n0\n31\n5\n0\nENDSEC\n0\nEOF\n"
    val elevatedResult = CadDxfCodec.parseAscii(elevated)
    check(elevatedResult.entities.isEmpty() && elevatedResult.skippedTypes["3D/elevated entities"] == 1)
    val bulged = "0\nSECTION\n2\nENTITIES\n0\nLWPOLYLINE\n90\n2\n10\n0\n20\n0\n42\n1\n10\n10\n20\n0\n0\nENDSEC\n0\nEOF\n"
    val bulgeResult = CadDxfCodec.parseAscii(bulged)
    check(bulgeResult.entities.isEmpty() && bulgeResult.skippedTypes["bulged or width-bearing polylines"] == 1)
    val layerStyle = "0\nSECTION\n2\nTABLES\n0\nTABLE\n2\nLAYER\n70\n1\n0\nLAYER\n2\nRED\n70\n0\n62\n1\n6\nDASHED\n0\nENDTAB\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nLINE\n10\n0\n20\n0\n11\n1\n21\n1\n0\nENDSEC\n0\nEOF\n"
    check(CadDxfCodec.parseAscii(layerStyle).skippedTypes["Layer style/plot metadata"] == 1) { "Layer style loss was not reported" }
    val defaultExtrusion = "0\nSECTION\n2\nENTITIES\n0\nLINE\n10\n0\n20\n0\n11\n1\n21\n1\n210\n0\n220\n0\n230\n1\n0\nENDSEC\n0\nEOF\n"
    check(CadDxfCodec.parseAscii(defaultExtrusion).entities.size == 1) { "Default extrusion vector should remain valid" }
    check(runCatching { CadDxfCodec.parseAscii("é".repeat(CadDxfCodec.MAX_FILE_BYTES / 2 + 1)) }.isFailure) { "UTF-8 byte limit was not enforced" }

    check(runCatching { CadDxfCodec.parseAscii("0\nSECTION\n2\nENTITIES\n0\nLINE\n10\n0\n20\n0\n11\n1\n21\n1\n") }.isFailure) { "Truncated DXF without ENDSEC/EOF should be rejected" }
    check(runCatching { CadDxfCodec.parseAscii("0\nSECTION\n2\nENTITIES\n0\nLINE\n10\n0\n20\n0\n11\n1\n21\n1\n0\nEOF\n0\nENDSEC\n") }.isFailure) { "EOF before ENTITIES ENDSEC should be rejected" }
    check(runCatching { CadDxfCodec.parseAscii("0\nSECTION\n2\nENTITIES\n0\nLINE\n10\n0\n20\n0\n11\n1\n21\n1\n0\nENDSEC\n0\nEOF\n0\nSECTION\n") }.isFailure) { "Data after EOF should be rejected" }
    check(runCatching { CadDxfCodec.parseAscii("0\nSECTION\n2\nENTITIES\n0\nCIRCLE\n10\nNaN\n20\n0\n40\n1\n0\nENDSEC\n0\nEOF\n") }.isFailure)
    check(runCatching { CadDxfCodec.writeAscii(listOf(DxfEntity.Circle(0.0, 0.0, -1.0))) }.isFailure)
    check(runCatching { CadDxfCodec.parseAscii("0\nSECTION\n2\nENTITIES\n0\nLINE\n10\n0\n20\n0\n11\n1\n") }.isFailure)
    check(runCatching { CadDxfCodec.parseAscii("0\nSECTION\n2\nHEADER\n0\nENDSEC\n0\nEOF\n") }.isFailure)
    check(runCatching { CadDxfCodec.parseAscii("0\nSECTION\n2\nENTITIES\n0\nLINE\n10\n0\n20\n0\n11\n1\n21\n1\n0\nENDSEC\n0\nEOF\n" + " ".repeat(CadDxfCodec.MAX_FILE_BYTES)) }.isFailure)
    check(runCatching { CadDxfCodec.writeAscii(listOf(DxfEntity.Polyline(listOf(0.0 to 0.0), false))) }.isFailure)
    println("DXF codec tests passed: ASCII parse, entity mapping, layers, round-trip, loss reporting, 2D safety, UTF-8 limits, invalid numeric/radius and malformed input")
}
