package com.arcad.app

import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Assert.fail
import org.junit.Test

class CadDxfCodecTest {
    @Test fun asciiEntitiesAndLayersRoundTrip() {
        val source = listOf(
            DxfEntity.Line(1.0, 2.0, 10.0, 20.0, "WALLS"),
            DxfEntity.Circle(5.0, 6.0, 2.0),
            DxfEntity.Arc(10.0, 10.0, 3.0, 0.0, 90.0, "CURVES"),
            DxfEntity.Polyline(listOf(0.0 to 0.0, 10.0 to 0.0, 10.0 to 5.0), true, "BOUNDARY")
        )
        val dxf = CadDxfCodec.writeAscii(source, listOf("ARCHITECTURE"))
        val parsed = CadDxfCodec.parseAscii(dxf)
        assertEquals(source, parsed.entities)
        assertTrue("ARCHITECTURE" in parsed.layers)
        assertTrue("WALLS" in parsed.layers)
        assertTrue(parsed.skippedTypes.isEmpty())
        assertEquals(4, parsed.unitsCode)
    }

    @Test fun imperialDrawingUnitsAreConvertedToMillimetres() {
        val inches = "0\nSECTION\n2\nHEADER\n9\n\$INSUNITS\n70\n1\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nLINE\n10\n1\n20\n2\n11\n3\n21\n4\n0\nENDSEC\n0\nEOF\n"
        val parsed = CadDxfCodec.parseAscii(inches)
        val line = parsed.entities.single() as DxfEntity.Line
        assertEquals(25.4, line.x1, 1e-8)
        assertEquals(50.8, line.y1, 1e-8)
        assertEquals(1, parsed.unitsCode)
    }

    @Test fun unsupportedAndBlockContentAreReported() {
        val source = "0\nSECTION\n2\nENTITIES\n0\nSPLINE\n8\n0\n0\nENDSEC\n0\nEOF\n"
        assertEquals(1, CadDxfCodec.parseAscii(source).skippedTypes["SPLINE"])
        val blocks = "0\nSECTION\n2\nBLOCKS\n0\nBLOCK\n2\nB1\n0\nLINE\n10\n0\n20\n0\n11\n1\n21\n1\n0\nENDBLK\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n0\nLINE\n10\n0\n20\n0\n11\n2\n21\n2\n0\nENDSEC\n0\nEOF\n"
        assertEquals(1, CadDxfCodec.parseAscii(blocks).skippedTypes["BLOCKS section objects"])
    }

    @Test fun arcAngleConversionHandlesWrapAndMirroring() {
        val crossing = CadAngleMath.dxfToCanvas(350.0, 10.0)
        assertEquals(-350.0, crossing.startDegrees, 1e-8)
        assertEquals(-370.0, crossing.endDegrees, 1e-8)
        val restored = CadAngleMath.canvasToDxf(crossing.startDegrees, crossing.endDegrees)
        assertEquals(350.0, restored.startDegrees, 1e-8)
        assertEquals(10.0, restored.endDegrees, 1e-8)
        val mirrored = CadAngleMath.canvasToDxf(0.0, 90.0)
        assertEquals(270.0, mirrored.startDegrees, 1e-8)
        assertEquals(0.0, mirrored.endDegrees, 1e-8)
    }

    @Test fun malformedAndUnsafeFilesAreRejected() {
        val invalid = listOf(
            "0\nSECTION\n2\nHEADER\n0\nENDSEC\n0\nEOF\n",
            "0\nSECTION\n2\nENTITIES\n0\nCIRCLE\n10\nNaN\n20\n0\n40\n1\n0\nENDSEC\n0\nEOF\n",
            "0\nSECTION\n2\nENTITIES\n0\nLWPOLYLINE\n90\n4\n10\n0\n20\n0\n10\n1\n20\n1\n0\nENDSEC\n0\nEOF\n"
        )
        invalid.forEach { input ->
            try { CadDxfCodec.parseAscii(input); fail("Expected malformed DXF to be rejected") }
            catch (_: IllegalArgumentException) { }
        }
        try { CadDxfCodec.writeAscii(listOf(DxfEntity.Circle(0.0, 0.0, -1.0))); fail("Expected invalid radius rejection") }
        catch (_: IllegalArgumentException) { }
    }
}
