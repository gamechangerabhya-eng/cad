package com.arcad.app

/** Minimal, defensive ASCII-DXF interchange for AR CAD. Coordinates are model-space units. */
sealed interface DxfEntity {
    data class Line(val x1: Double, val y1: Double, val x2: Double, val y2: Double, val layer: String = "0") : DxfEntity
    data class Circle(val x: Double, val y: Double, val radius: Double, val layer: String = "0") : DxfEntity
    data class Arc(val x: Double, val y: Double, val radius: Double, val startDegrees: Double, val endDegrees: Double, val layer: String = "0") : DxfEntity
    data class Polyline(val points: List<Pair<Double, Double>>, val closed: Boolean, val layer: String = "0") : DxfEntity
}

data class DxfImportResult(val entities: List<DxfEntity>, val skippedTypes: Map<String, Int>, val layers: List<String>, val unitsCode: Int)

object CadDxfCodec {
    const val MAX_FILE_BYTES = 25 * 1024 * 1024
    const val MAX_ENTITIES = 100_000
    const val MAX_VERTICES = 100_000
    private const val MAX_GROUPS_PER_ENTITY = 250_000
    private const val MAX_COORD = 1.0e9

    fun parseAscii(text: String): DxfImportResult {
        require(text.toByteArray(Charsets.UTF_8).size <= MAX_FILE_BYTES) { "DXF exceeds 25 MB limit" }
        return parseAscii(text.reader().buffered())
    }

    fun parseAscii(reader: java.io.BufferedReader): DxfImportResult {
        val lines = reader.lineSequence().iterator()
        val output = mutableListOf<DxfEntity>()
        val skipped = linkedMapOf<String, Int>()
        val layers = linkedSetOf("0")
        var inEntities = false
        var sawEntitiesSection = false
        var sawEntitiesEnd = false
        var sawEof = false
        var consumedBytes = 0L
        var currentSection = ""
        var currentTable = ""
        var headerVariable: String? = null
        var unitsCode = 0
        var currentType: String? = null
        var fields = mutableListOf<Pair<Int, String>>()
        fun consume() {
            val type = currentType ?: return
            if (type in setOf("SECTION", "SEQEND", "ENDSEC", "EOF", "")) return
            if (currentSection == "TABLES" && currentTable == "LAYER" && type == "LAYER") {
                fields.firstOrNull { it.first == 2 }?.second?.let { name -> layers += safeLayer(name) }
                // The mobile document currently stores layer names/visibility only. Do not
                // silently imply that CAD layer colour, linetype, lineweight or plot flags survive.
                val color = fields.firstOrNull { it.first == 62 }?.second?.toIntOrNull() ?: 7
                val lineType = fields.firstOrNull { it.first == 6 }?.second?.uppercase() ?: "CONTINUOUS"
                val lineWeight = fields.firstOrNull { it.first == 370 }?.second?.toIntOrNull() ?: -3
                val flags = fields.firstOrNull { it.first == 70 }?.second?.toIntOrNull() ?: 0
                if (color != 7 || lineType != "CONTINUOUS" || lineWeight != -3 || flags != 0) {
                    skipped["Layer style/plot metadata"] = (skipped["Layer style/plot metadata"] ?: 0) + 1
                }
                return
            }
            if (!inEntities) {
                if (currentSection == "TABLES" && type == "TABLE" && currentTable.isNotBlank() && currentTable != "LAYER") {
                    val key = "TABLES $currentTable table"
                    skipped[key] = (skipped[key] ?: 0) + 1
                }
                if (currentSection == "CLASSES" && type == "CLASS") skipped["CLASSES section records"] = (skipped["CLASSES section records"] ?: 0) + 1
                if (currentSection == "BLOCKS" && type !in setOf("BLOCK", "ENDBLK")) skipped["BLOCKS section objects"] = (skipped["BLOCKS section objects"] ?: 0) + 1
                if (currentSection == "OBJECTS") skipped["OBJECTS section records"] = (skipped["OBJECTS section records"] ?: 0) + 1
                return
            }
            if (output.size >= MAX_ENTITIES) throw IllegalArgumentException("DXF entity limit exceeded")
            val layer = fields.firstOrNull { it.first == 8 }?.second?.ifBlank { "0" } ?: "0"
            safeLayer(layer)
            layers += layer
            fun number(code: Int, default: Double? = null): Double {
                val raw = fields.firstOrNull { it.first == code }?.second
                val value = raw?.toDoubleOrNull() ?: default ?: throw IllegalArgumentException("$type is missing group $code")
                require(value.isFinite() && kotlin.math.abs(value) <= MAX_COORD) { "$type has invalid coordinate/value" }
                return value
            }
            fun nonZero(code: Int): Boolean = fields.filter { it.first == code }.any {
                val value = it.second.toDoubleOrNull() ?: throw IllegalArgumentException("$type has invalid group $code")
                require(value.isFinite()) { "$type has non-finite group $code" }
                kotlin.math.abs(value) > 1.0e-10
            }
            // This editor is strictly 2D for now. Reject rather than flatten 3D geometry,
            // thickness, non-default extrusion vectors, or bulged/width-bearing polylines.
            val hasZ = when (type) {
                "LINE" -> nonZero(30) || nonZero(31)
                "CIRCLE", "ARC" -> nonZero(30)
                "LWPOLYLINE" -> nonZero(38)
                else -> false
            }
            val hasThicknessOrExtrusion = nonZero(39) || nonZero(210) || nonZero(220) || (fields.any { it.first == 230 } && kotlin.math.abs((fields.first { it.first == 230 }.second.toDoubleOrNull() ?: 1.0) - 1.0) > 1.0e-10)
            val hasUnsupportedPolylineShape = type == "LWPOLYLINE" && (nonZero(42) || nonZero(40) || nonZero(41))
            if (hasZ || hasThicknessOrExtrusion || hasUnsupportedPolylineShape) {
                val reason = when {
                    hasZ -> "3D/elevated entities"
                    hasThicknessOrExtrusion -> "thickness or extrusion geometry"
                    else -> "bulged or width-bearing polylines"
                }
                skipped[reason] = (skipped[reason] ?: 0) + 1
                return
            }
            when (type) {
                "LINE" -> output += DxfEntity.Line(number(10), number(20), number(11), number(21), layer)
                "CIRCLE" -> output += DxfEntity.Circle(number(10), number(20), number(40).also { require(it > 0.0) { "Circle radius must be positive" } }, layer)
                "ARC" -> output += DxfEntity.Arc(number(10), number(20), number(40).also { require(it > 0.0) { "Arc radius must be positive" } }, number(50), number(51), layer)
                "LWPOLYLINE" -> {
                    val xs = fields.filter { it.first == 10 }.map { it.second.toDoubleOrNull() ?: throw IllegalArgumentException("Invalid polyline X") }
                    val ys = fields.filter { it.first == 20 }.map { it.second.toDoubleOrNull() ?: throw IllegalArgumentException("Invalid polyline Y") }
                    require(xs.size >= 2 && xs.size == ys.size) { "Polyline requires at least two XY vertices" }
                    require(xs.size <= MAX_VERTICES) { "Polyline vertex limit exceeded" }
                    val declared = fields.firstOrNull { it.first == 90 }?.second?.toIntOrNull()
                    require(declared == null || declared == xs.size) { "Polyline vertex count does not match group 90" }
                    require((xs + ys).all { it.isFinite() && kotlin.math.abs(it) <= MAX_COORD }) { "Polyline has invalid coordinates" }
                    val flags = fields.firstOrNull { it.first == 70 }?.second?.toIntOrNull() ?: 0
                    output += DxfEntity.Polyline(xs.indices.map { xs[it] to ys[it] }, flags and 1 != 0, layer)
                }
                else -> skipped[type] = (skipped[type] ?: 0) + 1
            }
        }

        var lineNumber = 0
        while (lines.hasNext()) {
            require(!sawEof) { "DXF contains data after its EOF marker" }
            val codeLine = lines.next().trim(); lineNumber++
            require(lines.hasNext()) { "DXF has an unmatched group-code/value pair at line $lineNumber" }
            val value = lines.next().trim(); lineNumber++
            consumedBytes += codeLine.toByteArray(Charsets.UTF_8).size + value.toByteArray(Charsets.UTF_8).size + 2L
            require(consumedBytes <= MAX_FILE_BYTES) { "DXF exceeds 25 MB limit" }
            if (codeLine == "0" && value.equals("ENDSEC", ignoreCase = true) && currentSection == "ENTITIES") sawEntitiesEnd = true
            if (codeLine == "0" && value.equals("EOF", ignoreCase = true)) {
                require(sawEntitiesEnd) { "DXF EOF marker appears before the ENTITIES section is closed" }
                sawEof = true
            }
            val code = codeLine.toIntOrNull() ?: throw IllegalArgumentException("Invalid DXF group code at line ${lineNumber - 1}")
            require(code in 0..1071) { "DXF group code out of range at line ${lineNumber - 1}" }
            if (code == 0) {
                consume()
                currentType = value.uppercase()
                fields = mutableListOf()
                if (currentType == "SECTION") { inEntities = false; currentSection = ""; currentTable = ""; headerVariable = null }
            } else {
                if (currentType == "SECTION" && code == 2) {
                    currentSection = value.uppercase()
                    inEntities = currentSection == "ENTITIES"
                    if (inEntities) sawEntitiesSection = true
                } else if (currentType == "TABLE" && code == 2) {
                    currentTable = value.uppercase()
                } else if (currentSection == "HEADER" && code == 9) {
                    headerVariable = value.uppercase()
                } else if (currentSection == "HEADER" && headerVariable == "\$INSUNITS" && code == 70) {
                    unitsCode = value.toIntOrNull() ?: throw IllegalArgumentException("Invalid \$INSUNITS value")
                    require(unitsCode in 0..24) { "Unsupported \$INSUNITS code $unitsCode" }
                    headerVariable = null
                } else if (inEntities || currentSection == "TABLES" && currentTable == "LAYER") {
                    require(fields.size < MAX_GROUPS_PER_ENTITY) { "DXF entity has too many group values" }
                    fields += code to value
                }
            }
            if (currentType == "ENDSEC") inEntities = false
        }
        consume()
        require(sawEntitiesSection) { "DXF has no ENTITIES section" }
        require(sawEntitiesEnd) { "DXF ENTITIES section is not terminated by ENDSEC" }
        require(sawEof) { "DXF is missing its EOF marker" }
        require(output.isNotEmpty() || skipped.isNotEmpty()) { "No entities found in DXF ENTITIES section" }
        val factor = unitToMmFactor(unitsCode)
        val scaled = output.map { entity -> when (entity) {
            is DxfEntity.Line -> entity.copy(x1 = entity.x1 * factor, y1 = entity.y1 * factor, x2 = entity.x2 * factor, y2 = entity.y2 * factor)
            is DxfEntity.Circle -> entity.copy(x = entity.x * factor, y = entity.y * factor, radius = entity.radius * factor)
            is DxfEntity.Arc -> entity.copy(x = entity.x * factor, y = entity.y * factor, radius = entity.radius * factor)
            is DxfEntity.Polyline -> entity.copy(points = entity.points.map { it.first * factor to it.second * factor })
        } }
        require(scaled.all { entity -> when (entity) {
            is DxfEntity.Line -> listOf(entity.x1, entity.y1, entity.x2, entity.y2).all { it.isFinite() && kotlin.math.abs(it) <= MAX_COORD }
            is DxfEntity.Circle -> listOf(entity.x, entity.y, entity.radius).all { it.isFinite() && kotlin.math.abs(it) <= MAX_COORD }
            is DxfEntity.Arc -> listOf(entity.x, entity.y, entity.radius).all { it.isFinite() && kotlin.math.abs(it) <= MAX_COORD }
            is DxfEntity.Polyline -> entity.points.all { it.first.isFinite() && it.second.isFinite() && kotlin.math.abs(it.first) <= MAX_COORD && kotlin.math.abs(it.second) <= MAX_COORD }
        } }) { "Converted drawing exceeds supported coordinate range" }
        return DxfImportResult(scaled, skipped, layers.toList(), unitsCode)
    }

    fun writeAscii(entities: List<DxfEntity>, extraLayers: List<String> = emptyList()): String {
        require(entities.size <= MAX_ENTITIES) { "Entity limit exceeded" }
        val out = StringBuilder(256 + entities.size * 100)
        fun pair(code: Int, value: Any) {
            out.append(code).append('\n').append(value).append('\n')
            require(out.length <= MAX_FILE_BYTES) { "DXF export exceeds 25 MB limit" }
        }
        fun num(v: Double): String {
            require(v.isFinite() && kotlin.math.abs(v) <= MAX_COORD) { "Cannot export invalid coordinate/value" }
            return java.lang.String.format(java.util.Locale.US, "%.8f", v).trimEnd('0').trimEnd('.').ifEmpty { "0" }
        }
        pair(0, "SECTION"); pair(2, "HEADER"); pair(9, "\$ACADVER"); pair(1, "AC1015"); pair(9, "\$INSUNITS"); pair(70, 4); pair(0, "ENDSEC")
        val layers = (extraLayers + entities.map { when (it) {
            is DxfEntity.Line -> it.layer
            is DxfEntity.Circle -> it.layer
            is DxfEntity.Arc -> it.layer
            is DxfEntity.Polyline -> it.layer
        } } + "0").map(::safeLayer).distinct().sorted()
        pair(0, "SECTION"); pair(2, "TABLES"); pair(0, "TABLE"); pair(2, "LAYER"); pair(70, layers.size)
        layers.forEach { layer -> pair(0, "LAYER"); pair(2, layer); pair(70, 0); pair(62, 7); pair(6, "CONTINUOUS") }
        pair(0, "ENDTAB"); pair(0, "ENDSEC")
        pair(0, "SECTION"); pair(2, "ENTITIES")
        entities.forEach { entity ->
            when (entity) {
                is DxfEntity.Line -> {
                    pair(0, "LINE"); pair(8, safeLayer(entity.layer)); pair(10, num(entity.x1)); pair(20, num(entity.y1)); pair(30, "0"); pair(11, num(entity.x2)); pair(21, num(entity.y2)); pair(31, "0")
                }
                is DxfEntity.Circle -> {
                    require(entity.radius > 0); pair(0, "CIRCLE"); pair(8, safeLayer(entity.layer)); pair(10, num(entity.x)); pair(20, num(entity.y)); pair(30, "0"); pair(40, num(entity.radius))
                }
                is DxfEntity.Arc -> {
                    require(entity.radius > 0); pair(0, "ARC"); pair(8, safeLayer(entity.layer)); pair(10, num(entity.x)); pair(20, num(entity.y)); pair(30, "0"); pair(40, num(entity.radius)); pair(50, num(entity.startDegrees)); pair(51, num(entity.endDegrees))
                }
                is DxfEntity.Polyline -> {
                    require(entity.points.size >= 2 && entity.points.size <= MAX_VERTICES)
                    pair(0, "LWPOLYLINE"); pair(8, safeLayer(entity.layer)); pair(90, entity.points.size); pair(70, if (entity.closed) 1 else 0)
                    entity.points.forEach { (x, y) -> pair(10, num(x)); pair(20, num(y)) }
                }
            }
        }
        pair(0, "ENDSEC"); pair(0, "EOF")
        return out.toString()
    }

    fun unitsLabel(code: Int): String = when (code) {
        0 -> "unitless (assumed mm)"; 1 -> "inches"; 2 -> "feet"; 3 -> "miles"; 4 -> "millimetres"; 5 -> "centimetres"; 6 -> "metres"; 7 -> "kilometres"
        8 -> "microinches"; 9 -> "mils"; 10 -> "yards"; 11 -> "angstroms"; 12 -> "nanometres"; 13 -> "microns"; 14 -> "decimetres"; 15 -> "decametres"
        16 -> "hectometres"; 17 -> "gigametres"; 18 -> "astronomical units"; 19 -> "light years"; 20 -> "parsecs"; 21 -> "US survey feet"; 22 -> "US survey inches"; 23 -> "US survey yards"; 24 -> "US survey miles"
        else -> "unknown units"
    }

    private fun unitToMmFactor(code: Int): Double = when (code) {
        1 -> 25.4; 2 -> 304.8; 3 -> 1_609_344.0; 4 -> 1.0; 5 -> 10.0; 6 -> 1_000.0; 7 -> 1_000_000.0
        8 -> 0.0000254; 9 -> 0.0254; 10 -> 914.4; 11 -> 1.0e-7; 12 -> 1.0e-6; 13 -> 0.001; 14 -> 100.0; 15 -> 10_000.0
        16 -> 100_000.0; 17 -> 1.0e12; 18 -> 149_597_870_700_000.0; 19 -> 9.4607304725808e18; 20 -> 3.0856775814913673e19
        21 -> 1200.0 / 3937.0 * 1000.0; 22 -> 100.0 / 3937.0 * 1000.0; 23 -> 3600.0 / 3937.0 * 1000.0; 24 -> 6336000.0 / 3937.0 * 1000.0
        else -> 1.0
    }

    private fun safeLayer(layer: String): String {
        val value = layer.trim().ifEmpty { "0" }
        require(value.length <= 255 && value.none { it == '\n' || it == '\r' || it in "<>/\\\":;?*|,=" }) { "Invalid DXF layer name" }
        require(value.all { it.code in 32..126 }) { "DXF layer names must be ASCII in this export version" }
        return value
    }
}
