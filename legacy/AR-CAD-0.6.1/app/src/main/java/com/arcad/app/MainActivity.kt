package com.arcad.app

import android.net.Uri
import android.graphics.BitmapFactory
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.lifecycle.lifecycleScope
import androidx.activity.compose.setContent
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.gestures.detectDragGestures
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.gestures.awaitEachGesture
import androidx.compose.foundation.gestures.awaitFirstDown
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.material3.*
import androidx.compose.runtime.*
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.withContext
import kotlinx.coroutines.launch
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.onSizeChanged
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import org.json.JSONArray
import org.json.JSONObject
import kotlin.math.abs
import kotlin.math.hypot
import kotlin.math.max
import kotlin.math.min
import kotlin.math.atan2

private val Navy = Color(0xFF101A2B)
private val Panel = Color(0xFF19263A)
private val Accent = Color(0xFF27B6A5)
private val Paper = Color(0xFF0B1220)
private val LayerPalette = listOf(Accent, Color(0xFF68A8FF), Color(0xFFE5A94D), Color(0xFFB58CFF), Color(0xFF70C98A), Color(0xFFFF8A80))

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent { ARCadApp() }
    }
}

private enum class Tool(val label: String) {
    SELECT("Select"), LINE("Line"), CIRCLE("Circle"), RECT("Rectangle"), POLYGON("Polygon"), ARC("Arc"), POLYLINE("Polyline"), MEASURE("Measure"), MOVE("Move"), PAN("Pan")
}

private sealed interface CadEntity {
    data class Segment(val a: Offset, val b: Offset, val layer: String = "0") : CadEntity
    data class Circle(val center: Offset, val radius: Float, val layer: String = "0") : CadEntity
    data class Rect(val a: Offset, val b: Offset, val layer: String = "0") : CadEntity
    data class Polyline(val points: List<Offset>, val closed: Boolean, val layer: String = "0") : CadEntity
    data class Arc(val center: Offset, val radius: Float, val startDegrees: Float, val endDegrees: Float, val layer: String = "0") : CadEntity
}

private fun distance(a: Offset, b: Offset): Float = hypot((a.x - b.x).toDouble(), (a.y - b.y).toDouble()).toFloat()

private fun distanceToSegment(p: Offset, a: Offset, b: Offset): Float {
    val dx = b.x - a.x
    val dy = b.y - a.y
    val length2 = dx * dx + dy * dy
    if (length2 <= 0.0001f) return distance(p, a)
    val t = (((p.x - a.x) * dx + (p.y - a.y) * dy) / length2).coerceIn(0f, 1f)
    return distance(p, Offset(a.x + t * dx, a.y + t * dy))
}

private fun hitTest(entity: CadEntity, p: Offset, tolerance: Float): Boolean = when (entity) {
    is CadEntity.Segment -> distanceToSegment(p, entity.a, entity.b) <= tolerance
    is CadEntity.Circle -> abs(distance(p, entity.center) - entity.radius) <= tolerance
    is CadEntity.Rect -> {
        val l = min(entity.a.x, entity.b.x); val r = max(entity.a.x, entity.b.x)
        val t = min(entity.a.y, entity.b.y); val b = max(entity.a.y, entity.b.y)
        val inside = p.x in l..r && p.y in t..b
        val edge = minOf(abs(p.x-l), abs(p.x-r), abs(p.y-t), abs(p.y-b))
        inside || edge <= tolerance && p.x in (l-tolerance)..(r+tolerance) && p.y in (t-tolerance)..(b+tolerance)
    }
    is CadEntity.Polyline -> {
        val segments = entity.points.zipWithNext().map { distanceToSegment(p, it.first, it.second) }.toMutableList()
        if (entity.closed && entity.points.size > 2) segments += distanceToSegment(p, entity.points.last(), entity.points.first())
        segments.minOrNull()?.let { it <= tolerance } ?: false
    }
    is CadEntity.Arc -> {
        val radial = distance(p, entity.center)
        var angle = Math.toDegrees(atan2((p.y - entity.center.y).toDouble(), (p.x - entity.center.x).toDouble())).toFloat()
        if (angle < 0f) angle += 360f
        val sweep = entity.endDegrees - entity.startDegrees
        val normalize = { value: Float -> ((value % 360f) + 360f) % 360f }
        val start = normalize(entity.startDegrees)
        val inSweep = when {
            abs(sweep) >= 359.999f -> true
            sweep >= 0f -> normalize(angle - start) <= sweep
            else -> normalize(start - angle) <= -sweep
        }
        abs(radial - entity.radius) <= tolerance && inSweep
    }
}

private fun translated(entity: CadEntity, delta: Offset): CadEntity = when (entity) {
    is CadEntity.Segment -> entity.copy(a = entity.a + delta, b = entity.b + delta)
    is CadEntity.Circle -> entity.copy(center = entity.center + delta)
    is CadEntity.Rect -> entity.copy(a = entity.a + delta, b = entity.b + delta)
    is CadEntity.Polyline -> entity.copy(points = entity.points.map { it + delta })
    is CadEntity.Arc -> entity.copy(center = entity.center + delta)
}


private class BoundedInputStream(input: java.io.InputStream, private val limit: Int) : java.io.FilterInputStream(input) {
    private var total = 0
    override fun read(): Int {
        val value = super.read()
        if (value >= 0) { total++; require(total <= limit) { "File exceeds ${limit / (1024 * 1024)} MB limit" } }
        return value
    }
    override fun read(buffer: ByteArray, offset: Int, length: Int): Int {
        val count = super.read(buffer, offset, length)
        if (count > 0) { total += count; require(total <= limit) { "File exceeds ${limit / (1024 * 1024)} MB limit" } }
        return count
    }
}

private fun readLimitedText(input: java.io.InputStream, limit: Int): String {
    input.use { stream ->
        val output = java.io.ByteArrayOutputStream()
        val buffer = ByteArray(8192)
        var total = 0
        while (true) {
            val count = stream.read(buffer)
            if (count < 0) break
            total += count
            require(total <= limit) { "File exceeds ${limit / (1024 * 1024)} MB limit" }
            output.write(buffer, 0, count)
        }
        return output.toByteArray().toString(Charsets.UTF_8)
    }
}

private data class DecodedCadDrawing(val entities: List<CadEntity>, val layers: List<String>)

private fun validateCadEntity(entity: CadEntity) {
    fun coord(value: Float) { require(value.isFinite() && kotlin.math.abs(value) <= 1.0e9f) { "Drawing contains an invalid coordinate" } }
    require(entity.layer.isNotBlank() && entity.layer.length <= 255 && entity.layer.none { it == '\n' || it == '\r' }) { "Drawing contains an invalid layer name" }
    when (entity) {
        is CadEntity.Segment -> { coord(entity.a.x); coord(entity.a.y); coord(entity.b.x); coord(entity.b.y) }
        is CadEntity.Rect -> { coord(entity.a.x); coord(entity.a.y); coord(entity.b.x); coord(entity.b.y) }
        is CadEntity.Circle -> { coord(entity.center.x); coord(entity.center.y); coord(entity.radius); require(entity.radius > 0f) }
        is CadEntity.Arc -> { coord(entity.center.x); coord(entity.center.y); coord(entity.radius); coord(entity.startDegrees); coord(entity.endDegrees); require(entity.radius > 0f) }
        is CadEntity.Polyline -> { require(entity.points.size in 2..CadDxfCodec.MAX_VERTICES); entity.points.forEach { coord(it.x); coord(it.y) } }
    }
}

private fun encodeDrawing(entities: List<CadEntity>, layers: List<String>): String {
    require(entities.size <= CadDxfCodec.MAX_ENTITIES) { "Drawing entity limit exceeded" }
    entities.forEach(::validateCadEntity)
    require(layers.size <= 10_000) { "Drawing layer limit exceeded" }
    val layerArray = JSONArray(); (layers + entities.map { it.layer } + "0").distinctBy { it.lowercase() }.forEach { layerArray.put(it) }
    val root = JSONObject().put("format", "ar-cad").put("version", 2).put("layers", layerArray)
    val array = JSONArray()
    entities.forEach { entity ->
        val item = JSONObject()
        when (entity) {
            is CadEntity.Segment -> item.put("type", "line").put("layer", entity.layer).put("x1", entity.a.x).put("y1", entity.a.y).put("x2", entity.b.x).put("y2", entity.b.y)
            is CadEntity.Circle -> item.put("type", "circle").put("layer", entity.layer).put("x", entity.center.x).put("y", entity.center.y).put("r", entity.radius)
            is CadEntity.Rect -> item.put("type", "rect").put("layer", entity.layer).put("x1", entity.a.x).put("y1", entity.a.y).put("x2", entity.b.x).put("y2", entity.b.y)
            is CadEntity.Polyline -> {
                val points = JSONArray(); entity.points.forEach { points.put(JSONArray().put(it.x).put(it.y)) }
                item.put("type", "polyline").put("layer", entity.layer).put("closed", entity.closed).put("points", points)
            }
            is CadEntity.Arc -> item.put("type", "arc").put("layer", entity.layer).put("x", entity.center.x).put("y", entity.center.y).put("r", entity.radius).put("start", entity.startDegrees).put("end", entity.endDegrees)
        }
        array.put(item)
    }
    val json = root.put("entities", array).toString(2)
    require(json.length <= CadDxfCodec.MAX_FILE_BYTES) { "Drawing exceeds 25 MB limit" }
    return json
}

private fun decodeDrawing(json: String): DecodedCadDrawing {
    require(json.toByteArray(Charsets.UTF_8).size <= CadDxfCodec.MAX_FILE_BYTES) { "Drawing exceeds 25 MB limit" }
    val root = JSONObject(json)
    require(root.optString("format") == "ar-cad") { "This is not an AR CAD drawing." }
    require(root.optInt("version", 1) in 1..2) { "Unsupported AR CAD document version" }
    val array = root.getJSONArray("entities")
    require(array.length() <= CadDxfCodec.MAX_ENTITIES) { "Drawing entity limit exceeded" }
    val decodedEntities = buildList<CadEntity> {
        for (i in 0 until array.length()) {
            val item = array.getJSONObject(i)
            val layerName = item.optString("layer", "0")
            require(layerName.length <= 255 && layerName.none { it == '\n' || it == '\r' }) { "Invalid layer name" }
            fun finiteFloat(key: String): Float {
                val value = item.getDouble(key)
                require(value.isFinite() && kotlin.math.abs(value) <= 1.0e9) { "Invalid $key value" }
                return value.toFloat()
            }
            when (item.getString("type")) {
                "line" -> add(CadEntity.Segment(Offset(finiteFloat("x1"), finiteFloat("y1")), Offset(finiteFloat("x2"), finiteFloat("y2")), layerName))
                "circle" -> add(CadEntity.Circle(Offset(finiteFloat("x"), finiteFloat("y")), finiteFloat("r").also { require(it > 0f) { "Circle radius must be positive" } }, layerName))
                "rect" -> add(CadEntity.Rect(Offset(finiteFloat("x1"), finiteFloat("y1")), Offset(finiteFloat("x2"), finiteFloat("y2")), layerName))
                "polyline" -> {
                    val points = item.getJSONArray("points")
                    require(points.length() in 2..CadDxfCodec.MAX_VERTICES) { "Invalid polyline vertex count" }
                    val vertices = (0 until points.length()).map { index ->
                        val pair = points.getJSONArray(index)
                        require(pair.length() == 2) { "Invalid polyline point" }
                        val x = pair.getDouble(0); val y = pair.getDouble(1)
                        require(x.isFinite() && y.isFinite() && kotlin.math.abs(x) <= 1.0e9 && kotlin.math.abs(y) <= 1.0e9) { "Invalid polyline coordinate" }
                        Offset(x.toFloat(), y.toFloat())
                    }
                    add(CadEntity.Polyline(vertices, item.optBoolean("closed", false), layerName))
                }
                "arc" -> add(CadEntity.Arc(Offset(finiteFloat("x"), finiteFloat("y")), finiteFloat("r").also { require(it > 0f) { "Arc radius must be positive" } }, finiteFloat("start"), finiteFloat("end"), layerName))
                else -> throw IllegalArgumentException("Unsupported entity type: ${item.optString("type")}")
            }
        }
    }
    val storedLayers = root.optJSONArray("layers")
    val layers = buildList<String> {
        add("0")
        if (storedLayers != null) for (index in 0 until storedLayers.length()) {
            val name = storedLayers.getString(index)
            require(name.isNotBlank() && name.length <= 255 && name.none { it == '\n' || it == '\r' }) { "Invalid layer name" }
            if (none { it.equals(name, true) }) add(name)
        }
        decodedEntities.forEach { if (none { name -> name.equals(it.layer, true) }) add(it.layer) }
    }
    return DecodedCadDrawing(decodedEntities, layers)
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun ARCadApp() {
    val context = LocalContext.current
    var tool by remember { mutableStateOf(Tool.LINE) }
    val entities = remember { mutableStateListOf<CadEntity>() }
    val undo = remember { mutableStateListOf<List<CadEntity>>() }
    val redo = remember { mutableStateListOf<List<CadEntity>>() }
    var pending by remember { mutableStateOf<Offset?>(null) }
    var arcStart by remember { mutableStateOf<Offset?>(null) }
    val polylinePoints = remember { mutableStateListOf<Offset>() }
    var cursor by remember { mutableStateOf(Offset.Zero) }
    var snapResult by remember { mutableStateOf<CadObjectSnap.Result?>(null) }
    var trackingResult by remember { mutableStateOf<CadSnapTracking.Result?>(null) }
    var dynamicInputText by remember { mutableStateOf("") }
    var zoom by remember { mutableFloatStateOf(1f) }
    var pan by remember { mutableStateOf(Offset.Zero) }
    var message by remember { mutableStateOf("Ready · Choose a drawing tool") }
    var selectedIndex by remember { mutableIntStateOf(-1) }
    var gridOn by remember { mutableStateOf(true) }
    var orthoOn by remember { mutableStateOf(false) }
    var snapOn by remember { mutableStateOf(true) }
    var trackingOn by remember { mutableStateOf(true) }
    var showSnapSettings by remember { mutableStateOf(false) }
    var snapModes by remember { mutableStateOf(CadObjectSnap.Kind.values().toSet()) }
    var activeLayer by remember { mutableStateOf("0") }
    val layerNames = remember { mutableStateListOf("0") }
    val hiddenLayers = remember { mutableStateListOf<String>() }
    var showLayerManager by remember { mutableStateOf(false) }
    var showNewDrawingDialog by remember { mutableStateOf(false) }
    var pendingDxfImport by remember { mutableStateOf<DxfImportResult?>(null) }
    var newLayerName by remember { mutableStateOf("") }
    var polygonSidesText by remember { mutableStateOf("6") }
    var polygonSides by remember { mutableIntStateOf(6) }
    var showPolygonSettings by remember { mutableStateOf(false) }
    var canvasWidth by remember { mutableFloatStateOf(0f) }
    var canvasHeight by remember { mutableFloatStateOf(0f) }
    var recoveryReady by remember { mutableStateOf(false) }
    var nativePreview by remember { mutableStateOf<androidx.compose.ui.graphics.ImageBitmap?>(null) }
    var nativePreviewTitle by remember { mutableStateOf("CADCraft native preview") }
    val nativePreviewGate = remember { NativePreviewGate<Pair<List<CadEntity>, List<String>>>() }
    val recoveryStore = remember(context) { CadRecoveryStore(context.filesDir) }

    // Recover the last valid document before enabling autosave, so an empty initial
    // Compose state can never overwrite a drawing left by the previous process.
    LaunchedEffect(Unit) {
        val recovered = withContext(Dispatchers.IO) {
            recoveryStore.load { contents -> runCatching { decodeDrawing(contents) }.isSuccess }
        }
        if (recovered != null) {
            runCatching { decodeDrawing(recovered) }.onSuccess { loaded ->
                entities.clear(); entities.addAll(loaded.entities)
                layerNames.clear(); layerNames.addAll(loaded.layers)
                message = "Recovered previous drawing · ${loaded.entities.size} entities"
            }
        }
        recoveryReady = true
    }

    LaunchedEffect(recoveryReady, entities.toList(), layerNames.toList()) {
        if (!recoveryReady) return@LaunchedEffect
        delay(350)
        val snapshot = runCatching { encodeDrawing(entities.toList(), layerNames.toList()) }.getOrNull() ?: return@LaunchedEffect
        val saveResult = withContext(Dispatchers.IO) { runCatching { recoveryStore.save(snapshot) } }
        saveResult.exceptionOrNull()?.let { message = "Autosave failed: ${it.message ?: "storage error"}" }
    }

    fun syncLayers(items: List<CadEntity>, extraLayers: List<String> = emptyList()) {
        val names = (listOf("0") + extraLayers + items.map { it.layer }).distinctBy { it.lowercase() }.sorted()
        layerNames.clear(); layerNames.addAll(names)
        hiddenLayers.retainAll(names)
        if (activeLayer !in names) activeLayer = names.firstOrNull() ?: "0"
    }
    fun createLayer() {
        val name = newLayerName.trim()
        if (name.isNotEmpty() && name.length <= 255 && name.none { it == '\n' || it == '\r' } && layerNames.none { it.equals(name, true) }) {
            layerNames.add(name); activeLayer = name; newLayerName = ""; message = "Layer $name created and set current"
        } else message = "Layer name is empty, invalid or already exists"
    }
    fun startNewDrawing() {
        entities.clear(); undo.clear(); redo.clear(); pending = null; arcStart = null; polylinePoints.clear(); selectedIndex = -1
        layerNames.clear(); layerNames.add("0"); hiddenLayers.clear(); activeLayer = "0"
        zoom = 1f; pan = Offset(canvasWidth / 2f, canvasHeight / 2f); message = "New drawing created"
    }
    fun checkpoint() {
        undo.add(entities.toList())
        if (undo.size > 100) undo.removeAt(0)
        redo.clear()
    }
    fun undoOnce() {
        if (undo.isNotEmpty()) {
            redo.add(entities.toList())
            val previous = undo.removeAt(undo.lastIndex)
            entities.clear(); entities.addAll(previous)
            selectedIndex = -1; pending = null; message = "Undo completed"
        } else message = "Nothing to undo"
    }
    fun redoOnce() {
        if (redo.isNotEmpty()) {
            undo.add(entities.toList())
            val next = redo.removeAt(redo.lastIndex)
            entities.clear(); entities.addAll(next)
            selectedIndex = -1; pending = null; message = "Redo completed"
        } else message = "Nothing to redo"
    }
    fun entityCenter(entity: CadEntity): Offset = when (entity) {
        is CadEntity.Segment -> (entity.a + entity.b) / 2f
        is CadEntity.Rect -> (entity.a + entity.b) / 2f
        is CadEntity.Circle -> entity.center
        is CadEntity.Arc -> entity.center
        is CadEntity.Polyline -> Offset(entity.points.map { it.x }.average().toFloat(), entity.points.map { it.y }.average().toFloat())
    }
    fun transformEntity(entity: CadEntity, scale: Float = 1f, rotationDegrees: Float = 0f, mirrorY: Boolean = false): CadEntity {
        val center = entityCenter(entity)
        val radians = Math.toRadians(rotationDegrees.toDouble())
        fun point(p: Offset): Offset {
            var x = (p.x - center.x) * scale
            var y = (p.y - center.y) * scale
            if (mirrorY) y = -y
            val rx = x * kotlin.math.cos(radians).toFloat() - y * kotlin.math.sin(radians).toFloat()
            val ry = x * kotlin.math.sin(radians).toFloat() + y * kotlin.math.cos(radians).toFloat()
            return Offset(center.x + rx, center.y + ry)
        }
        return when (entity) {
            is CadEntity.Segment -> entity.copy(a = point(entity.a), b = point(entity.b))
            is CadEntity.Rect -> CadEntity.Polyline(listOf(point(entity.a), point(Offset(entity.b.x, entity.a.y)), point(entity.b), point(Offset(entity.a.x, entity.b.y))), true, entity.layer)
            is CadEntity.Circle -> entity.copy(radius = entity.radius * scale)
            is CadEntity.Arc -> entity.copy(radius = entity.radius * scale, startDegrees = (if (mirrorY) -entity.startDegrees else entity.startDegrees) + rotationDegrees, endDegrees = (if (mirrorY) -entity.endDegrees else entity.endDegrees) + rotationDegrees)
            is CadEntity.Polyline -> entity.copy(points = entity.points.map(::point))
        }
    }
    fun selectedMeasurementSummary(index: Int): String {
        if (index !in entities.indices) return ""
        val entity = entities[index]
        val number = { value: Double -> String.format(java.util.Locale.US, "%.3f", value) }
        return when (entity) {
            is CadEntity.Segment -> "Length ${number(distance(entity.a, entity.b).toDouble())} mm"
            is CadEntity.Circle -> {
                val metrics = CadMeasurements.circleMetrics(entity.radius.toDouble())
                "Circumference ${number(metrics.length!!)} mm · Area ${number(metrics.area!!)} mm²"
            }
            is CadEntity.Rect -> {
                val metrics = CadMeasurements.rectangleMetrics((entity.b.x - entity.a.x).toDouble(), (entity.b.y - entity.a.y).toDouble())
                "Perimeter ${number(metrics.length!!)} mm · Area ${number(metrics.area!!)} mm²"
            }
            is CadEntity.Polyline -> {
                val points = entity.points.map { it.x.toDouble() to it.y.toDouble() }
                val length = CadMeasurements.polylineLength(points, entity.closed)
                if (entity.closed && points.size >= 3) "Perimeter ${number(length)} mm · Area ${number(CadMeasurements.polygonArea(points))} mm²"
                else "Length ${number(length)} mm · ${points.size} vertices"
            }
            is CadEntity.Arc -> {
                val sweep = kotlin.math.abs(entity.endDegrees - entity.startDegrees).coerceAtMost(360f).toDouble()
                "Arc length ${number(entity.radius * Math.toRadians(sweep))} mm · Radius ${number(entity.radius.toDouble())} mm"
            }
        }
    }

    fun duplicateSelected() {
        if (selectedIndex !in entities.indices) { message = "Select an entity first"; return }
        checkpoint(); entities.add(translated(entities[selectedIndex], Offset(10f, 10f))); selectedIndex = entities.lastIndex; message = "Copied entity +10,+10"
    }
    fun deleteSelected() {
        if (selectedIndex !in entities.indices) { message = "Select an entity first"; return }
        checkpoint(); entities.removeAt(selectedIndex); selectedIndex = -1; message = "Entity deleted"
    }
    fun modifySelected(scale: Float = 1f, rotation: Float = 0f, mirrorY: Boolean = false) {
        if (selectedIndex !in entities.indices) { message = "Select an entity first"; return }
        checkpoint(); entities[selectedIndex] = transformEntity(entities[selectedIndex], scale, rotation, mirrorY); message = "Entity modified"
    }
    fun straightSegments(excludeIndex: Int): List<CadLine> {
        val result = mutableListOf<CadLine>()
        entities.forEachIndexed { index, entity ->
            if (index == excludeIndex || entity.layer in hiddenLayers) return@forEachIndexed
            when (entity) {
                is CadEntity.Segment -> result += CadLine(CadPoint(entity.a.x.toDouble(), entity.a.y.toDouble()), CadPoint(entity.b.x.toDouble(), entity.b.y.toDouble()))
                is CadEntity.Rect -> {
                    val p = listOf(entity.a, Offset(entity.b.x, entity.a.y), entity.b, Offset(entity.a.x, entity.b.y))
                    p.indices.forEach { i -> val a = p[i]; val b = p[(i + 1) % p.size]; result += CadLine(CadPoint(a.x.toDouble(), a.y.toDouble()), CadPoint(b.x.toDouble(), b.y.toDouble())) }
                }
                is CadEntity.Polyline -> {
                    val count = if (entity.closed) entity.points.size else entity.points.size - 1
                    repeat(count) { i -> val a = entity.points[i]; val b = entity.points[(i + 1) % entity.points.size]; result += CadLine(CadPoint(a.x.toDouble(), a.y.toDouble()), CadPoint(b.x.toDouble(), b.y.toDouble())) }
                }
                else -> Unit
            }
        }
        return result
    }
    fun applyLineModify(operation: String, offsetDistance: Double = 10.0) {
        if (selectedIndex !in entities.indices) { message = "Select an entity first"; return }
        val selectedEntity = entities[selectedIndex]
        if (operation == "OFFSET") {
            val offsetEntity: CadEntity? = when (selectedEntity) {
                is CadEntity.Segment -> {
                    val line = CadLine(CadPoint(selectedEntity.a.x.toDouble(), selectedEntity.a.y.toDouble()), CadPoint(selectedEntity.b.x.toDouble(), selectedEntity.b.y.toDouble()))
                    val result = CadModifyOps.offset(line, offsetDistance)
                    CadEntity.Segment(Offset(result.start.x.toFloat(), result.start.y.toFloat()), Offset(result.end.x.toFloat(), result.end.y.toFloat()), selectedEntity.layer)
                }
                is CadEntity.Polyline -> {
                    val points = selectedEntity.points.map { CadPoint(it.x.toDouble(), it.y.toDouble()) }
                    val result = CadPolylineOps.offset(points, selectedEntity.closed, offsetDistance)
                    result?.let { CadEntity.Polyline(it.map { p -> Offset(p.x.toFloat(), p.y.toFloat()) }, selectedEntity.closed, selectedEntity.layer) }
                }
                is CadEntity.Circle -> (selectedEntity.radius.toDouble() + offsetDistance).takeIf { it > 0.0 && it.isFinite() }
                    ?.let { selectedEntity.copy(radius = it.toFloat()) }
                is CadEntity.Arc -> (selectedEntity.radius.toDouble() + offsetDistance).takeIf { it > 0.0 && it.isFinite() }
                    ?.let { selectedEntity.copy(radius = it.toFloat()) }
                is CadEntity.Rect -> {
                    val points = listOf(selectedEntity.a, Offset(selectedEntity.b.x, selectedEntity.a.y), selectedEntity.b, Offset(selectedEntity.a.x, selectedEntity.b.y))
                        .map { CadPoint(it.x.toDouble(), it.y.toDouble()) }
                    val result = CadPolylineOps.offset(points, true, offsetDistance)
                    result?.let { CadEntity.Polyline(it.map { p -> Offset(p.x.toFloat(), p.y.toFloat()) }, true, selectedEntity.layer) }
                }
            }
            if (offsetEntity == null) { message = "OFFSET cannot be applied at this distance or geometry"; return }
            checkpoint(); entities.add(offsetEntity); selectedIndex = entities.lastIndex
            message = "OFFSET completed · ${selectedEntity.layer} · ${offsetDistance} mm"
            return
        }
        val selected = selectedEntity as? CadEntity.Segment
        if (selected == null) { message = "$operation currently supports straight LINE entities only"; return }
        val line = CadLine(CadPoint(selected.a.x.toDouble(), selected.a.y.toDouble()), CadPoint(selected.b.x.toDouble(), selected.b.y.toDouble()))
        val others = straightSegments(selectedIndex)
        val result = when (operation) {
            "TRIM" -> CadModifyOps.trim(line, others, CadPoint(cursor.x.toDouble(), cursor.y.toDouble()))
            "EXTEND" -> CadModifyOps.extend(line, others, CadPoint(cursor.x.toDouble(), cursor.y.toDouble()))
            else -> null
        }
        if (result == null) { message = "$operation found no valid intersection"; return }
        checkpoint()
        entities[selectedIndex] = CadEntity.Segment(Offset(result.start.x.toFloat(), result.start.y.toFloat()), Offset(result.end.x.toFloat(), result.end.y.toFloat()), selected.layer)
        message = "$operation completed · ${selected.layer}"
    }
    fun applyPolylineCornerModify(operation: String, amount: Double = 5.0) {
        if (selectedIndex !in entities.indices) { message = "Select a polyline first"; return }
        val selected = entities[selectedIndex] as? CadEntity.Polyline
        if (selected == null) { message = "$operation currently supports polyline corners only"; return }
        if (selected.points.size < 3) { message = "Polyline needs at least 3 vertices"; return }
        val cornerIndex = selected.points.indices.minByOrNull { distance(selected.points[it], cursor) } ?: run { message = "No corner found"; return }
        val modelPoints = selected.points.map { CadPoint(it.x.toDouble(), it.y.toDouble()) }
        val result = when (operation) {
            "FILLET" -> CadPolylineOps.fillet(modelPoints, selected.closed, cornerIndex, amount, 8)
            "CHAMFER" -> CadPolylineOps.chamfer(modelPoints, selected.closed, cornerIndex, amount)
            else -> null
        }
        if (result == null) { message = "$operation cannot be applied at this corner · reduce radius/distance or choose another corner"; return }
        checkpoint()
        entities[selectedIndex] = selected.copy(points = result.map { Offset(it.x.toFloat(), it.y.toFloat()) })
        message = "$operation completed · corner ${cornerIndex + 1} · ${amount} mm"
    }

    fun fitDrawing() {
        if (entities.isEmpty() || canvasWidth <= 0f || canvasHeight <= 0f) { zoom = 1f; pan = Offset(canvasWidth / 2f, canvasHeight / 2f); return }
        val xs = mutableListOf<Float>(); val ys = mutableListOf<Float>()
        entities.filter { it.layer !in hiddenLayers }.forEach { entity -> when (entity) {
            is CadEntity.Segment -> { xs += entity.a.x; xs += entity.b.x; ys += entity.a.y; ys += entity.b.y }
            is CadEntity.Rect -> { xs += entity.a.x; xs += entity.b.x; ys += entity.a.y; ys += entity.b.y }
            is CadEntity.Circle -> { xs += entity.center.x - entity.radius; xs += entity.center.x + entity.radius; ys += entity.center.y - entity.radius; ys += entity.center.y + entity.radius }
            is CadEntity.Arc -> { xs += entity.center.x - entity.radius; xs += entity.center.x + entity.radius; ys += entity.center.y - entity.radius; ys += entity.center.y + entity.radius }
            is CadEntity.Polyline -> { entity.points.forEach { xs += it.x; ys += it.y } }
        } }
        if (xs.isEmpty()) return
        val minX = xs.minOrNull()!!; val maxX = xs.maxOrNull()!!; val minY = ys.minOrNull()!!; val maxY = ys.maxOrNull()!!
        val width = (maxX - minX).coerceAtLeast(1f); val height = (maxY - minY).coerceAtLeast(1f)
        zoom = min(canvasWidth / width, canvasHeight / height).times(0.85f).coerceIn(0.15f, 12f)
        pan = Offset(canvasWidth / 2f - ((minX + maxX) / 2f) * zoom, canvasHeight / 2f - ((minY + maxY) / 2f) * zoom)
    }
    // Rebuild snap primitives only when the drawing or visible-layer set changes.
    // Pointer movement no longer allocates a complete geometry list on every event.
    val snapGeometry = remember(entities.toList(), hiddenLayers.toList()) {
        entities.filter { it.layer !in hiddenLayers }.mapNotNull { entity ->
            fun model(p: Offset) = CadPoint(p.x.toDouble(), p.y.toDouble())
            when (entity) {
                is CadEntity.Segment -> CadObjectSnap.Geometry.Segment(model(entity.a), model(entity.b))
                is CadEntity.Rect -> CadObjectSnap.Geometry.Polyline(
                    listOf(entity.a, Offset(entity.b.x, entity.a.y), entity.b, Offset(entity.a.x, entity.b.y)).map(::model), true
                )
                is CadEntity.Polyline -> CadObjectSnap.Geometry.Polyline(entity.points.map(::model), entity.closed)
                is CadEntity.Circle -> CadObjectSnap.Geometry.Circle(model(entity.center), entity.radius.toDouble())
                is CadEntity.Arc -> CadObjectSnap.Geometry.Arc(model(entity.center), entity.radius.toDouble(), entity.startDegrees.toDouble(), entity.endDegrees.toDouble())
            }
        }
    }
    // Build a packed immutable AABB tree only when the drawing/visible layers change.
    // Pointer events query a small model-space window instead of scanning every entity.
    val snapSpatialIndex = remember(snapGeometry) { CadSpatialIndex.build(snapGeometry) }
    fun snap(point: Offset): Offset {
        if (zoom <= 0f) { snapResult = null; trackingResult = null; return point }
        val modelPoint = CadPoint(point.x.toDouble(), point.y.toDouble())
        val snapTolerance = 12.0 / zoom
        val localSnapGeometry = snapSpatialIndex.query(
            CadSpatialIndex.Bounds(modelPoint.x - snapTolerance, modelPoint.y - snapTolerance,
                modelPoint.x + snapTolerance, modelPoint.y + snapTolerance)
        )
        val objectSnap = if (snapOn) CadObjectSnap.resolve(modelPoint, localSnapGeometry, snapTolerance, snapModes) else null
        snapResult = objectSnap
        // Conventional object snaps win. Construction tracking is a fallback and
        // uses the previous command point (or last polyline vertex) as its anchor.
        if (objectSnap != null || !trackingOn) {
            trackingResult = null
            return objectSnap?.let { Offset(it.point.x.toFloat(), it.point.y.toFloat()) } ?: point
        }
        val anchor = (polylinePoints.lastOrNull() ?: pending)?.let { CadPoint(it.x.toDouble(), it.y.toDouble()) }
        val tracking = CadSnapTracking.resolve(modelPoint, anchor, snapGeometry, 12.0 / zoom)
        trackingResult = tracking
        return tracking?.let { Offset(it.point.x.toFloat(), it.point.y.toFloat()) } ?: point
    }
    fun applyDynamicCoordinate() {
        val base = pending ?: cursor
        val parsed = runCatching {
            CadDynamicInput.parsePoint(dynamicInputText, CadPoint(base.x.toDouble(), base.y.toDouble()))
        }.getOrElse { error -> message = error.message ?: "Invalid coordinate"; return }
        var point = Offset(parsed.x.toFloat(), parsed.y.toFloat())
        cursor = point
        snapResult = null; trackingResult = null // Explicit numeric coordinates are exact and bypass cursor snapping.
        when (tool) {
            Tool.LINE, Tool.CIRCLE, Tool.RECT -> {
                val first = pending
                if (first == null) {
                    pending = point
                    message = "First point set numerically · enter the next coordinate"
                } else {
                    // Explicit numeric coordinates take precedence over ORTHO and OSNAP.
                    val entity = when (tool) {
                        Tool.LINE -> if (distance(first, point) > 0.0001f) CadEntity.Segment(first, point, activeLayer) else null
                        Tool.CIRCLE -> if (distance(first, point) > 0.0001f) CadEntity.Circle(first, distance(first, point), activeLayer) else null
                        Tool.RECT -> if (abs(point.x-first.x) > 0.0001f && abs(point.y-first.y) > 0.0001f) CadEntity.Rect(first, point, activeLayer) else null
                        else -> null
                    }
                    if (entity != null) { checkpoint(); entities.add(entity); message = "${tool.label} created from numeric coordinates" }
                    else message = "Geometry is too small"
                    pending = null
                }
            }
            Tool.POLYLINE -> { polylinePoints.add(point); message = "Polyline · ${polylinePoints.size} points · numeric vertex added" }
            Tool.MEASURE -> {
                val first = pending
                if (first == null) { pending = point; message = "Measurement start set numerically" }
                else { message = "Distance: ${"%.3f".format(java.util.Locale.US, distance(first, point))} mm"; pending = null }
            }
            Tool.POLYGON -> {
                val center = pending
                if (center == null) { pending = point; message = "Polygon centre set numerically · enter radius point" }
                else {
                    val radius = distance(center, point)
                    if (radius > 0.0001f) {
                        val vertices = CadMeasurements.regularPolygon(center.x.toDouble(), center.y.toDouble(), radius.toDouble(), polygonSides)
                            .map { Offset(it.first.toFloat(), it.second.toFloat()) }
                        checkpoint(); entities.add(CadEntity.Polyline(vertices, true, activeLayer)); message = "$polygonSides-sided polygon created"
                    } else message = "Polygon radius must be positive"
                    pending = null
                }
            }
            Tool.ARC -> {
                val center = pending
                val startPoint = arcStart
                if (center == null) { pending = point; message = "Arc centre set numerically" }
                else if (startPoint == null) { arcStart = point; message = "Arc start set numerically · enter end point" }
                else {
                    val radius = distance(center, startPoint)
                    if (radius > 0.0001f) {
                        val a1 = Math.toDegrees(atan2((startPoint.y-center.y).toDouble(), (startPoint.x-center.x).toDouble())).toFloat()
                        val a2 = Math.toDegrees(atan2((point.y-center.y).toDouble(), (point.x-center.x).toDouble())).toFloat()
                        val angles = CadAngleMath.dxfToCanvas(-a1.toDouble(), -a2.toDouble())
                        checkpoint(); entities.add(CadEntity.Arc(center, radius, angles.startDegrees.toFloat(), angles.endDegrees.toFloat(), activeLayer)); message = "Arc created from numeric coordinates"
                    } else message = "Arc radius must be positive"
                    pending = null; arcStart = null
                }
            }
            Tool.SELECT, Tool.MOVE, Tool.PAN -> message = "Numeric point entry is not used by ${tool.label}"
        }
        dynamicInputText = ""
    }
    fun applyDxfImport(result: DxfImportResult) {
        val imported = result.entities.map { entity -> when (entity) {
            is DxfEntity.Line -> CadEntity.Segment(Offset(entity.x1.toFloat(), -entity.y1.toFloat()), Offset(entity.x2.toFloat(), -entity.y2.toFloat()), entity.layer)
            is DxfEntity.Circle -> CadEntity.Circle(Offset(entity.x.toFloat(), -entity.y.toFloat()), entity.radius.toFloat(), entity.layer)
            is DxfEntity.Arc -> {
                val angles = CadAngleMath.dxfToCanvas(entity.startDegrees, entity.endDegrees)
                CadEntity.Arc(Offset(entity.x.toFloat(), -entity.y.toFloat()), entity.radius.toFloat(), angles.startDegrees.toFloat(), angles.endDegrees.toFloat(), entity.layer)
            }
            is DxfEntity.Polyline -> CadEntity.Polyline(entity.points.map { Offset(it.first.toFloat(), -it.second.toFloat()) }, entity.closed, entity.layer)
        } }
        checkpoint(); entities.clear(); entities.addAll(imported); hiddenLayers.clear(); activeLayer = "0"; syncLayers(imported, result.layers); selectedIndex = -1; pending = null; arcStart = null; polylinePoints.clear()
        message = "DXF imported · ${CadDxfCodec.unitsLabel(result.unitsCode)} → mm · ${imported.size} entities · ${result.skippedTypes.values.sum()} omitted"
    }

    val saveDxfLauncher = rememberLauncherForActivityResult(ActivityResultContracts.CreateDocument("application/dxf")) { uri: Uri? ->
        if (uri != null) runCatching {
            val dxf = entities.map { entity -> when (entity) {
                is CadEntity.Segment -> DxfEntity.Line(entity.a.x.toDouble(), -entity.a.y.toDouble(), entity.b.x.toDouble(), -entity.b.y.toDouble(), entity.layer)
                is CadEntity.Circle -> DxfEntity.Circle(entity.center.x.toDouble(), -entity.center.y.toDouble(), entity.radius.toDouble(), entity.layer)
                is CadEntity.Rect -> DxfEntity.Polyline(listOf(entity.a.x.toDouble() to -entity.a.y.toDouble(), entity.b.x.toDouble() to -entity.a.y.toDouble(), entity.b.x.toDouble() to -entity.b.y.toDouble(), entity.a.x.toDouble() to -entity.b.y.toDouble()), true, entity.layer)
                is CadEntity.Polyline -> DxfEntity.Polyline(entity.points.map { it.x.toDouble() to -it.y.toDouble() }, entity.closed, entity.layer)
                is CadEntity.Arc -> {
                    val angles = CadAngleMath.canvasToDxf(entity.startDegrees.toDouble(), entity.endDegrees.toDouble())
                    DxfEntity.Arc(entity.center.x.toDouble(), -entity.center.y.toDouble(), entity.radius.toDouble(), angles.startDegrees, angles.endDegrees, entity.layer)
                }
            } }
            val data = CadDxfCodec.writeAscii(dxf, layerNames.toList())
            context.contentResolver.openOutputStream(uri)?.bufferedWriter()?.use { it.write(data) } ?: error("Cannot open destination")
        }.onSuccess { message = "DXF exported · ${entities.size} entities" }.onFailure { message = "DXF export failed: ${it.message}" }
    }
    val openDxfLauncher = rememberLauncherForActivityResult(ActivityResultContracts.OpenDocument()) { uri: Uri? ->
        if (uri != null) runCatching {
            val input = context.contentResolver.openInputStream(uri) ?: error("Cannot read file")
            input.use { stream ->
                val bounded = BoundedInputStream(stream, CadDxfCodec.MAX_FILE_BYTES)
                CadDxfCodec.parseAscii(java.io.InputStreamReader(bounded, Charsets.UTF_8).buffered())
            }
        }.onSuccess { result ->
            if (result.skippedTypes.isNotEmpty()) {
                pendingDxfImport = result
                message = "DXF contains unsupported objects · review before partial import"
            } else applyDxfImport(result)
        }.onFailure { message = "DXF import failed: ${it.message}" }
    }

    val saveLauncher = rememberLauncherForActivityResult(ActivityResultContracts.CreateDocument("application/json")) { uri: Uri? ->
        if (uri != null) runCatching { context.contentResolver.openOutputStream(uri)?.bufferedWriter()?.use { it.write(encodeDrawing(entities.toList(), layerNames.toList())) } ?: error("Cannot open destination") }
            .onSuccess { message = "Drawing saved" }.onFailure { message = "Save failed: ${it.message}" }
    }
    // Render the current editor snapshot through CADCraft. The Kotlin drawing is
    // exported to an in-memory DXF buffer, then opened in a short-lived native
    // document; the user's active drawing and undo history are never replaced.
    fun renderActiveNativePreview() {
        val entitySnapshot = entities.toList()
        val layerSnapshot = layerNames.toList()
        if (entitySnapshot.isEmpty()) {
            message = "Add geometry before requesting a native preview"
            return
        }
        val requestToken = nativePreviewGate.begin(entitySnapshot to layerSnapshot)
        message = "Rendering current drawing with CADCraft…"
        (context as? ComponentActivity)?.lifecycleScope?.launch {
            val rendered = withContext(Dispatchers.IO) {
                runCatching {
                    val dxfEntities = entitySnapshot.map { entity -> when (entity) {
                        is CadEntity.Segment -> DxfEntity.Line(entity.a.x.toDouble(), -entity.a.y.toDouble(), entity.b.x.toDouble(), -entity.b.y.toDouble(), entity.layer)
                        is CadEntity.Circle -> DxfEntity.Circle(entity.center.x.toDouble(), -entity.center.y.toDouble(), entity.radius.toDouble(), entity.layer)
                        is CadEntity.Rect -> DxfEntity.Polyline(listOf(entity.a.x.toDouble() to -entity.a.y.toDouble(), entity.b.x.toDouble() to -entity.a.y.toDouble(), entity.b.x.toDouble() to -entity.b.y.toDouble(), entity.a.x.toDouble() to -entity.b.y.toDouble()), true, entity.layer)
                        is CadEntity.Polyline -> DxfEntity.Polyline(entity.points.map { it.x.toDouble() to -it.y.toDouble() }, entity.closed, entity.layer)
                        is CadEntity.Arc -> {
                            val angles = CadAngleMath.canvasToDxf(entity.startDegrees.toDouble(), entity.endDegrees.toDouble())
                            DxfEntity.Arc(entity.center.x.toDouble(), -entity.center.y.toDouble(), entity.radius.toDouble(), angles.startDegrees, angles.endDegrees, entity.layer)
                        }
                    } }
                    val dxfText = CadDxfCodec.writeAscii(dxfEntities, layerSnapshot)
                    val bytes = dxfText.toByteArray(Charsets.UTF_8)
                    require(bytes.size <= CadDxfCodec.MAX_FILE_BYTES) { "Drawing exceeds the DXF preview size limit" }
                    val document = CadCraftDocument.open(bytes, "active-drawing.dxf")
                    try {
                        val hasWarnings = document.hasImportWarnings()
                        val png = document.renderPreviewPng(1600, 1000)
                        png to hasWarnings
                    } finally { document.close() }
                }
            }
            rendered.onSuccess { (png, hasWarnings) ->
                val currentSnapshot = entities.toList() to layerNames.toList()
                if (!nativePreviewGate.isCurrent(requestToken, currentSnapshot)) {
                    // A newer request or drawing edit superseded this result.
                } else {
                    val decoded = runCatching {
                        require(png.isNotEmpty() && png.size <= 64 * 1024 * 1024) { "Native PNG size is invalid" }
                        val bounds = BitmapFactory.Options().apply { inJustDecodeBounds = true }
                        BitmapFactory.decodeByteArray(png, 0, png.size, bounds)
                        require(bounds.outWidth in 1..4096 && bounds.outHeight in 1..4096 &&
                            bounds.outWidth.toLong() * bounds.outHeight.toLong() <= 16_777_216L) {
                            "Native PNG dimensions are invalid or exceed 16 megapixels"
                        }
                        BitmapFactory.decodeByteArray(png, 0, png.size) ?: error("Native renderer returned an invalid PNG")
                    }
                    decoded.onFailure { error -> message = "Native preview unavailable: ${error.message ?: "invalid PNG"}" }
                    decoded.getOrNull()?.let { bitmap ->
                        if (nativePreviewGate.isCurrent(requestToken, entities.toList() to layerNames.toList())) {
                            nativePreview = bitmap.asImageBitmap()
                            nativePreviewTitle = "Current drawing · CADCraft"
                            message = if (hasWarnings) "Native preview rendered · CADCraft reported import conversions" else "Native preview rendered · ${entitySnapshot.size} entities"
                        } else bitmap.recycle()
                    }
                }
            }.onFailure { error ->
                if (nativePreviewGate.isCurrent(requestToken, entities.toList() to layerNames.toList())) {
                    message = "Native preview unavailable: ${error.message ?: "render failed"}"
                }
            }
        } ?: run { message = "Native preview requires an active Android activity" }
    }

    val openLauncher = rememberLauncherForActivityResult(ActivityResultContracts.OpenDocument()) { uri: Uri? ->
        if (uri != null) runCatching { context.contentResolver.openInputStream(uri)?.let { readLimitedText(it, CadDxfCodec.MAX_FILE_BYTES) } ?: error("Cannot read file") }
            .mapCatching(::decodeDrawing)
            .onSuccess { loaded -> checkpoint(); entities.clear(); entities.addAll(loaded.entities); hiddenLayers.clear(); activeLayer = "0"; syncLayers(loaded.entities, loaded.layers); layerNames.clear(); layerNames.addAll(loaded.layers); selectedIndex = -1; pending = null; message = "Opened ${loaded.entities.size} entities" }
            .onFailure { message = "Open failed: ${it.message}" }
    }

    MaterialTheme(colorScheme = darkColorScheme(primary = Accent, background = Paper, surface = Panel)) {
        Scaffold(containerColor = Paper, topBar = {
            TopAppBar(
                title = { Column { Text("AR CAD", fontWeight = FontWeight.Bold, color = Color.White); Text("Drawing1 · mm · Layer: $activeLayer", fontSize = 11.sp, color = Color.LightGray) } },
                actions = {
                    TextButton(onClick = { if (entities.isEmpty()) startNewDrawing() else showNewDrawingDialog = true }) { Text("New", color = Accent) }
                    TextButton(onClick = { saveLauncher.launch("Drawing1.arcad.json") }) { Text("Save", color = Accent) }
                    TextButton(onClick = { openLauncher.launch(arrayOf("application/json", "text/*")) }) { Text("Open", color = Accent) }
                    TextButton(onClick = { showLayerManager = true }) { Text("Layers", color = Accent) }
                    TextButton(onClick = { saveDxfLauncher.launch("Drawing1.dxf") }) { Text("DXF ↓", color = Accent) }
                    TextButton(onClick = { openDxfLauncher.launch(arrayOf("*/*")) }) { Text("DXF ↑", color = Accent) }
                    TextButton(onClick = { renderActiveNativePreview() }) { Text("Native PNG", color = Accent) }
                    TextButton(onClick = { undoOnce() }, enabled = undo.isNotEmpty()) { Text("Undo", color = Color.White) }
                    TextButton(onClick = { redoOnce() }, enabled = redo.isNotEmpty()) { Text("Redo", color = Color.White) }
                }, colors = TopAppBarDefaults.topAppBarColors(containerColor = Navy)
            )
        }, bottomBar = {
            Column(Modifier.background(Navy)) {
                Row(Modifier.fillMaxWidth().horizontalScroll(rememberScrollState()).padding(horizontal = 8.dp, vertical = 6.dp), horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                    Tool.values().forEach { item ->
                        FilterChip(selected = tool == item, onClick = { tool = item; pending = null; arcStart = null; polylinePoints.clear(); message = "${item.label} tool active" }, label = { Text(item.label) })
                    }
                    TextButton(onClick = { duplicateSelected() }) { Text("Copy", color = Accent) }
                    TextButton(onClick = { modifySelected(rotation = 90f) }) { Text("Rotate 90°", color = Accent) }
                    TextButton(onClick = { modifySelected(mirrorY = true) }) { Text("Mirror", color = Accent) }
                    TextButton(onClick = { modifySelected(scale = 2f) }) { Text("Scale 2×", color = Accent) }
                    TextButton(onClick = { applyLineModify("TRIM") }) { Text("Trim", color = Accent) }
                    TextButton(onClick = { applyLineModify("EXTEND") }) { Text("Extend", color = Accent) }
                    TextButton(onClick = { applyLineModify("OFFSET", 10.0) }) { Text("Offset +10", color = Accent) }
                    TextButton(onClick = { applyLineModify("OFFSET", -10.0) }) { Text("Offset -10", color = Accent) }
                    TextButton(onClick = { applyPolylineCornerModify("FILLET", 5.0) }) { Text("Fillet 5", color = Accent) }
                    TextButton(onClick = { applyPolylineCornerModify("CHAMFER", 5.0) }) { Text("Chamfer 5", color = Accent) }
                    TextButton(onClick = { deleteSelected() }) { Text("Delete", color = Color(0xFFFF7777)) }
                    TextButton(onClick = { fitDrawing() }) { Text("Fit", color = Accent) }
                    if (tool == Tool.POLYGON) TextButton(onClick = { showPolygonSettings = true }) { Text("Sides: $polygonSides", color = Accent) }
                    if (tool == Tool.POLYLINE) TextButton(onClick = {
                        if (polylinePoints.size >= 2) { checkpoint(); entities.add(CadEntity.Polyline(polylinePoints.toList(), false, activeLayer)); message = "Polyline created · ${polylinePoints.size} vertices" }
                        else message = "Polyline needs at least 2 points"
                        polylinePoints.clear()
                    }) { Text("Finish PL", color = Accent) }
                }
                Row(Modifier.fillMaxWidth().padding(horizontal = 12.dp, vertical = 5.dp), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
                    Text(message, color = Color(0xFFB9C7D8), fontSize = 10.sp, modifier = Modifier.weight(1f))
                    Text("${entities.size} entities · ${(zoom * 100).toInt()}%", color = Accent, fontSize = 10.sp)
                }
            }
        }) { padding ->
            Column(Modifier.padding(padding).fillMaxSize()) {
                Row(Modifier.fillMaxWidth().background(Panel).padding(horizontal = 10.dp, vertical = 7.dp), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
                    Text("MODEL SPACE", color = Accent, fontSize = 10.sp, fontWeight = FontWeight.Bold)
                    Text("X ${"%.3f".format(java.util.Locale.US, cursor.x)}  Y ${"%.3f".format(java.util.Locale.US, cursor.y)}", color = Color.White, fontSize = 10.sp)
                    snapResult?.let { Text("${it.kind.name}", color = Color(0xFF63E6D0), fontSize = 9.sp, fontWeight = FontWeight.Bold) }
                    trackingResult?.let { Text("TRACK ${it.kind.name}", color = Color(0xFFFFD479), fontSize = 9.sp, fontWeight = FontWeight.Bold) }
                    pending?.let { base ->
                        val metrics = CadDynamicPreview.measure(CadPoint(base.x.toDouble(), base.y.toDouble()), CadPoint(cursor.x.toDouble(), cursor.y.toDouble()))
                        Text("ΔX ${"%.2f".format(java.util.Locale.US, metrics.dx)}  ΔY ${"%.2f".format(java.util.Locale.US, metrics.dy)}  L ${"%.2f".format(java.util.Locale.US, metrics.length)}  ∠ ${"%.1f".format(java.util.Locale.US, metrics.angleDegrees)}°", color = Color(0xFFFFD479), fontSize = 9.sp)
                    }
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        TextButton(onClick = { gridOn = !gridOn }) { Text(if (gridOn) "GRID ON" else "GRID OFF", color = if (gridOn) Accent else Color.Gray, fontSize = 9.sp) }
                        TextButton(onClick = { orthoOn = !orthoOn }) { Text(if (orthoOn) "ORTHO ON" else "ORTHO", color = if (orthoOn) Accent else Color.Gray, fontSize = 9.sp) }
                        TextButton(onClick = { snapOn = !snapOn }) { Text(if (snapOn) "OSNAP ON" else "OSNAP OFF", color = if (snapOn) Accent else Color.Gray, fontSize = 9.sp) }
                        TextButton(onClick = { trackingOn = !trackingOn; if (!trackingOn) trackingResult = null }) { Text(if (trackingOn) "TRACK ON" else "TRACK OFF", color = if (trackingOn) Color(0xFFFFD479) else Color.Gray, fontSize = 9.sp) }
                        TextButton(onClick = { showSnapSettings = true }) { Text("SNAP MODES", color = Accent, fontSize = 9.sp) }
                    }
                }
                Row(
                    Modifier.fillMaxWidth().background(Panel).padding(horizontal = 10.dp, vertical = 4.dp),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(8.dp)
                ) {
                    OutlinedTextField(
                        value = dynamicInputText,
                        onValueChange = { dynamicInputText = it.take(80) },
                        modifier = Modifier.weight(1f).height(54.dp),
                        singleLine = true,
                        label = { Text("Coordinate: x,y · @dx,dy · @dist<angle", fontSize = 10.sp) },
                        textStyle = androidx.compose.ui.text.TextStyle(fontSize = 13.sp),
                        colors = OutlinedTextFieldDefaults.colors(
                            focusedTextColor = Color.White, unfocusedTextColor = Color.White,
                            focusedBorderColor = Accent, unfocusedBorderColor = Color.Gray,
                            focusedLabelColor = Accent, unfocusedLabelColor = Color.LightGray
                        )
                    )
                    Button(onClick = { applyDynamicCoordinate() }, enabled = dynamicInputText.isNotBlank()) { Text("Enter") }
                }
                Canvas(Modifier.fillMaxSize().background(Paper).onSizeChanged { canvasWidth = it.width.toFloat(); canvasHeight = it.height.toFloat() }
                    .pointerInput(tool, pending, arcStart, polylinePoints.size, activeLayer, pan, zoom, entities.size, selectedIndex, orthoOn, snapOn, snapModes, trackingOn) {
                        detectTapGestures { screen ->
                            val raw = Offset((screen.x - pan.x) / zoom, (screen.y - pan.y) / zoom)
                            cursor = raw
                            if (tool == Tool.SELECT || tool == Tool.MOVE) {
                                snapResult = null
                                val hit = entities.indices.reversed().firstOrNull { entities[it].layer !in hiddenLayers && hitTest(entities[it], raw, 14f / zoom) }
                                selectedIndex = hit ?: -1
                                message = if (hit == null) "Nothing selected" else "Entity ${hit + 1} selected · ${selectedMeasurementSummary(hit)}"
                                return@detectTapGestures
                            }
                            if (tool == Tool.PAN) return@detectTapGestures
                            var point = snap(raw)
                            if (tool == Tool.POLYLINE) {
                                polylinePoints.add(point); message = "Polyline · ${polylinePoints.size} points · Finish PL to commit"; return@detectTapGestures
                            }
                            if (tool == Tool.POLYGON) {
                                val center = pending
                                if (center == null) {
                                    pending = point
                                    message = "Polygon: choose centre, then radius"
                                } else {
                                    val radius = distance(center, point)
                                    if (radius > 0.0001f) {
                                        val vertices = CadMeasurements.regularPolygon(center.x.toDouble(), center.y.toDouble(), radius.toDouble(), polygonSides)
                                            .map { Offset(it.first.toFloat(), it.second.toFloat()) }
                                        checkpoint()
                                        entities.add(CadEntity.Polyline(vertices, true, activeLayer))
                                        message = "$polygonSides-sided polygon created · ${selectedMeasurementSummary(entities.lastIndex)}"
                                    } else message = "Polygon radius must be positive"
                                    pending = null
                                }
                                return@detectTapGestures
                            }
                            if (tool == Tool.MEASURE) {
                                val first = pending
                                if (first == null) { pending = point; message = "Measure: select second point" }
                                else { message = "Distance: ${"%.3f".format(java.util.Locale.US, distance(first, point))} mm"; pending = null }
                                return@detectTapGestures
                            }
                            if (tool == Tool.ARC) {
                                val center = pending
                                val startPoint = arcStart
                                if (center == null) { pending = point; message = "Arc: select start point" }
                                else if (startPoint == null) { arcStart = point; message = "Arc: select end point" }
                                else {
                                    val radius = distance(center, startPoint)
                                    if (radius > 0.0001f) {
                                        val startAngle = Math.toDegrees(atan2((startPoint.y-center.y).toDouble(), (startPoint.x-center.x).toDouble())).toFloat()
                                        val endAngle = Math.toDegrees(atan2((point.y-center.y).toDouble(), (point.x-center.x).toDouble())).toFloat()
                                        val angles = CadAngleMath.dxfToCanvas(-startAngle.toDouble(), -endAngle.toDouble())
                                        checkpoint(); entities.add(CadEntity.Arc(center, radius, angles.startDegrees.toFloat(), angles.endDegrees.toFloat(), activeLayer)); message = "Arc created"
                                    } else message = "Arc radius must be positive"
                                    pending = null; arcStart = null
                                }
                                return@detectTapGestures
                            }
                            val first = pending
                            if (first == null) { pending = point; message = "First point set · Tap second point" }
                            else {
                                if (orthoOn && (tool == Tool.LINE || tool == Tool.RECT)) {
                                    val dx = abs(point.x - first.x); val dy = abs(point.y - first.y)
                                    point = if (dx >= dy) Offset(point.x, first.y) else Offset(first.x, point.y)
                                }
                                val entity = when (tool) {
                                    Tool.LINE -> if (distance(first, point) > 0.0001f) CadEntity.Segment(first, point, activeLayer) else null
                                    Tool.CIRCLE -> if (distance(first, point) > 0.0001f) CadEntity.Circle(first, distance(first, point), activeLayer) else null
                                    Tool.RECT -> if (abs(point.x-first.x) > 0.0001f && abs(point.y-first.y) > 0.0001f) CadEntity.Rect(first, point, activeLayer) else null
                                    else -> null
                                }
                                if (entity != null) { checkpoint(); entities.add(entity); message = "${tool.label} created" }
                                else message = "Geometry is too small"
                                pending = null
                            }
                        }
                    }
                    .pointerInput(tool, pan, zoom, snapOn, snapModes, trackingOn, snapGeometry, pending, polylinePoints.size) {
                        awaitEachGesture {
                            awaitFirstDown(requireUnconsumed = false)
                            do {
                                val event = awaitPointerEvent()
                                val pressed = event.changes.filter { it.pressed }
                                if (pressed.size == 1 && tool != Tool.PAN) {
                                    val screen = pressed.first().position
                                    val raw = Offset((screen.x - pan.x) / zoom, (screen.y - pan.y) / zoom)
                                    cursor = raw
                                    snap(raw) // Updates the live snap marker without consuming the gesture.
                                } else if (pressed.size >= 2) {
                                    snapResult = null
                                    trackingResult = null
                                }
                            } while (event.changes.any { it.pressed })
                        }
                    }
                    .pointerInput(tool, selectedIndex, zoom) {
                        detectDragGestures(onDragStart = {
                            if ((tool == Tool.SELECT || tool == Tool.MOVE) && selectedIndex in entities.indices) checkpoint()
                        }, onDrag = { change, drag ->
                            change.consume()
                            when (tool) {
                                Tool.PAN -> pan += drag
                                Tool.SELECT, Tool.MOVE -> if (selectedIndex in entities.indices) {
                                    if (drag != Offset.Zero) {
                                        entities[selectedIndex] = translated(entities[selectedIndex], drag / zoom)
                                        message = "Moving entity ${selectedIndex + 1}"
                                    }
                                }
                                else -> Unit
                            }
                        })
                    }
                    .pointerInput(Unit) {
                        awaitEachGesture {
                            awaitFirstDown(requireUnconsumed = false)
                            var previousCentroid: Offset? = null
                            var previousSpan = 0f
                            do {
                                val event = awaitPointerEvent()
                                val pressed = event.changes.filter { it.pressed }
                                if (pressed.size >= 2) {
                                    val centroid = Offset(pressed.map { it.position.x }.average().toFloat(), pressed.map { it.position.y }.average().toFloat())
                                    val span = pressed.map { distance(it.position, centroid) }.average().toFloat()
                                    val previous = previousCentroid
                                    if (previous != null && previousSpan > 0.01f && span > 0.01f) {
                                        val oldZoom = zoom
                                        val newZoom = (oldZoom * (span / previousSpan)).coerceIn(0.15f, 12f)
                                        val worldAtPreviousCentroid = Offset((previous.x - pan.x) / oldZoom, (previous.y - pan.y) / oldZoom)
                                        zoom = newZoom
                                        pan = Offset(centroid.x - worldAtPreviousCentroid.x * newZoom, centroid.y - worldAtPreviousCentroid.y * newZoom)
                                    }
                                    previousCentroid = centroid
                                    previousSpan = span
                                    event.changes.forEach { it.consume() }
                                } else {
                                    previousCentroid = null
                                    previousSpan = 0f
                                }
                            } while (event.changes.any { it.pressed })
                        }
                    }
                ) {
                    if (gridOn) {
                        val step = 40f * zoom.coerceAtLeast(0.15f)
                        var x = ((pan.x % step) + size.width) % step
                        var y = ((pan.y % step) + size.height) % step
                        while (x < size.width) { drawLine(Color(0xFF1A2A40), Offset(x, 0f), Offset(x, size.height), 1f); x += step }
                        while (y < size.height) { drawLine(Color(0xFF1A2A40), Offset(0f, y), Offset(size.width, y), 1f); y += step }
                    }
                    fun screen(p: Offset) = Offset(p.x * zoom + pan.x, p.y * zoom + pan.y)
                    entities.forEachIndexed { index, entity ->
                        if (entity.layer in hiddenLayers) return@forEachIndexed
                        val layerIndex = layerNames.indexOfFirst { it.equals(entity.layer, true) }.coerceAtLeast(0)
                        val color = if (index == selectedIndex) Color(0xFFFFC857) else LayerPalette[layerIndex % LayerPalette.size]
                        when (entity) {
                            is CadEntity.Segment -> drawLine(color, screen(entity.a), screen(entity.b), 2.5f, cap = StrokeCap.Round)
                            is CadEntity.Circle -> { val c = screen(entity.center); drawCircle(color, entity.radius * zoom, c, style = Stroke(2.5f)) }
                            is CadEntity.Rect -> { val a = screen(entity.a); val b = screen(entity.b); drawRect(color, Offset(min(a.x,b.x), min(a.y,b.y)), Size(abs(b.x-a.x), abs(b.y-a.y)), style = Stroke(2.5f)) }
                            is CadEntity.Polyline -> {
                                val pts = entity.points.map(::screen)
                                pts.zipWithNext().forEach { drawLine(color, it.first, it.second, 2.5f, cap = StrokeCap.Round) }
                                if (entity.closed && pts.size > 2) drawLine(color, pts.last(), pts.first(), 2.5f, cap = StrokeCap.Round)
                            }
                            is CadEntity.Arc -> {
                                val c = screen(entity.center); val rawSweep = entity.endDegrees - entity.startDegrees
                                val sweep = if (abs(rawSweep) < 0.0001f) 360f else rawSweep
                                drawArc(color, entity.startDegrees, sweep, false, Offset(c.x - entity.radius * zoom, c.y - entity.radius * zoom), Size(entity.radius * 2f * zoom, entity.radius * 2f * zoom), style = Stroke(2.5f))
                            }
                        }
                    }
                    pending?.let { base ->
                        drawCircle(Color.Yellow, 5f, screen(base)); drawCircle(Color.Yellow, 12f, screen(base), style = Stroke(1f))
                        val tip = screen(cursor)
                        when (tool) {
                            Tool.LINE, Tool.MEASURE, Tool.ARC -> drawLine(Color(0xFF63E6D0), screen(base), tip, 1.5f, pathEffect = androidx.compose.ui.graphics.PathEffect.dashPathEffect(floatArrayOf(8f, 6f)))
                            Tool.RECT -> drawRect(Color(0xFF63E6D0), Offset(min(screen(base).x, tip.x), min(screen(base).y, tip.y)), Size(abs(tip.x-screen(base).x), abs(tip.y-screen(base).y)), style = Stroke(1.5f))
                            Tool.CIRCLE, Tool.POLYGON -> drawCircle(Color(0xFF63E6D0), distance(base, cursor) * zoom, screen(base), style = Stroke(1.5f, pathEffect = androidx.compose.ui.graphics.PathEffect.dashPathEffect(floatArrayOf(8f, 6f))))
                            else -> Unit
                        }
                    }
                    if (polylinePoints.isNotEmpty()) {
                        val preview = polylinePoints.map(::screen)
                        preview.zipWithNext().forEach { drawLine(Color.Yellow, it.first, it.second, 2f, cap = StrokeCap.Round) }
                        drawLine(Color(0xFF63E6D0), preview.last(), screen(cursor), 1.5f, pathEffect = androidx.compose.ui.graphics.PathEffect.dashPathEffect(floatArrayOf(8f, 6f)))
                        preview.forEach { drawCircle(Color.Yellow, 3f, it) }
                    }
                    arcStart?.let { drawCircle(Color.Yellow, 4f, screen(it)) }
                    snapResult?.let { result ->
                        val p = screen(Offset(result.point.x.toFloat(), result.point.y.toFloat()))
                        val c = Color(0xFF63E6D0)
                        when (result.kind) {
                            CadObjectSnap.Kind.ENDPOINT -> drawRect(c, Offset(p.x - 6f, p.y - 6f), Size(12f, 12f), style = Stroke(2f))
                            CadObjectSnap.Kind.MIDPOINT -> {
                                drawLine(c, Offset(p.x, p.y - 7f), Offset(p.x - 7f, p.y + 6f), 2f)
                                drawLine(c, Offset(p.x - 7f, p.y + 6f), Offset(p.x + 7f, p.y + 6f), 2f)
                                drawLine(c, Offset(p.x + 7f, p.y + 6f), Offset(p.x, p.y - 7f), 2f)
                            }
                            CadObjectSnap.Kind.INTERSECTION -> {
                                drawCircle(c, 6f, p, style = Stroke(2f))
                                drawLine(c, Offset(p.x - 9f, p.y), Offset(p.x + 9f, p.y), 2f)
                                drawLine(c, Offset(p.x, p.y - 9f), Offset(p.x, p.y + 9f), 2f)
                            }
                            CadObjectSnap.Kind.CENTER -> {
                                drawCircle(c, 6f, p, style = Stroke(2f))
                                drawLine(c, Offset(p.x - 9f, p.y), Offset(p.x + 9f, p.y), 1.5f)
                                drawLine(c, Offset(p.x, p.y - 9f), Offset(p.x, p.y + 9f), 1.5f)
                            }
                            CadObjectSnap.Kind.QUADRANT -> {
                                drawLine(c, Offset(p.x, p.y - 7f), Offset(p.x + 7f, p.y), 2f)
                                drawLine(c, Offset(p.x + 7f, p.y), Offset(p.x, p.y + 7f), 2f)
                                drawLine(c, Offset(p.x, p.y + 7f), Offset(p.x - 7f, p.y), 2f)
                                drawLine(c, Offset(p.x - 7f, p.y), Offset(p.x, p.y - 7f), 2f)
                            }
                            CadObjectSnap.Kind.NEAREST -> {
                                drawLine(c, Offset(p.x - 6f, p.y - 6f), Offset(p.x + 6f, p.y + 6f), 2f)
                                drawLine(c, Offset(p.x - 6f, p.y + 6f), Offset(p.x + 6f, p.y - 6f), 2f)
                            }
                        }
                    }
                    trackingResult?.let { result ->
                        val guideColor = Color(0xFFFFD479)
                        result.guides.forEach { guide ->
                            drawLine(
                                guideColor,
                                screen(Offset(guide.start.x.toFloat(), guide.start.y.toFloat())),
                                screen(Offset(guide.end.x.toFloat(), guide.end.y.toFloat())),
                                1.2f,
                                pathEffect = androidx.compose.ui.graphics.PathEffect.dashPathEffect(floatArrayOf(5f, 5f))
                            )
                        }
                        val target = screen(Offset(result.point.x.toFloat(), result.point.y.toFloat()))
                        drawCircle(guideColor, 7f, target, style = Stroke(1.8f))
                        drawLine(guideColor, Offset(target.x - 10f, target.y), Offset(target.x + 10f, target.y), 1.5f)
                        drawLine(guideColor, Offset(target.x, target.y - 10f), Offset(target.x, target.y + 10f), 1.5f)
                    }
                    drawLine(Color(0xFFB94B55), Offset(pan.x - 9, pan.y), Offset(pan.x + 9, pan.y), 2f)
                    drawLine(Color(0xFF55A6D8), Offset(pan.x, pan.y - 9), Offset(pan.x, pan.y + 9), 2f)
                }
            }
        }
        if (showLayerManager) {
            AlertDialog(
                onDismissRequest = { showLayerManager = false },
                title = { Text("Layer Manager") },
                text = {
                    Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
                        Text("Active layer: $activeLayer", fontWeight = FontWeight.Bold)
                        layerNames.forEach { name ->
                            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.SpaceBetween, modifier = Modifier.fillMaxWidth()) {
                                TextButton(onClick = { activeLayer = name }) { Text(if (activeLayer == name) "● $name" else name) }
                                TextButton(onClick = {
                                    if (name in hiddenLayers) hiddenLayers.remove(name)
                                    else if (name == activeLayer) message = "Switch current layer before hiding it"
                                    else hiddenLayers.add(name)
                                }) { Text(if (name in hiddenLayers) "Show" else "Hide") }
                            }
                        }
                        OutlinedTextField(value = newLayerName, onValueChange = { newLayerName = it }, label = { Text("New layer name") }, singleLine = true)
                    }
                },
                confirmButton = { TextButton(onClick = { createLayer() }) { Text("Add layer") } },
                dismissButton = { TextButton(onClick = { showLayerManager = false }) { Text("Close") } }
            )
        }
        if (showNewDrawingDialog) {
            AlertDialog(
                onDismissRequest = { showNewDrawingDialog = false },
                title = { Text("Start a new drawing?") },
                text = { Text("The current drawing will be cleared. Save it first if you need to keep it.") },
                confirmButton = { TextButton(onClick = { startNewDrawing(); showNewDrawingDialog = false }) { Text("New drawing") } },
                dismissButton = { TextButton(onClick = { showNewDrawingDialog = false }) { Text("Cancel") } }
            )
        }
        if (showSnapSettings) {
            AlertDialog(
                onDismissRequest = { showSnapSettings = false },
                title = { Text("Object snap modes") },
                text = {
                    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                        CadObjectSnap.Kind.values().forEach { mode ->
                            Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
                                Text(mode.name.lowercase().replaceFirstChar { it.uppercase() })
                                Switch(checked = mode in snapModes, onCheckedChange = { enabled ->
                                    snapModes = if (enabled) snapModes + mode else snapModes - mode
                                })
                            }
                        }
                        Text("Only enabled modes can attract the cursor. Turn OSNAP off to temporarily bypass all modes.", style = MaterialTheme.typography.bodySmall, color = Color.Gray)
                    }
                },
                confirmButton = { TextButton(onClick = { showSnapSettings = false }) { Text("Done") } },
                dismissButton = { TextButton(onClick = { snapModes = CadObjectSnap.Kind.values().toSet() }) { Text("Reset") } }
            )
        }

        if (showPolygonSettings) {
            AlertDialog(
                onDismissRequest = { showPolygonSettings = false },
                title = { Text("Regular polygon") },
                text = {
                    OutlinedTextField(
                        value = polygonSidesText,
                        onValueChange = { value -> polygonSidesText = value.filter(Char::isDigit).take(3) },
                        label = { Text("Number of sides (3–128)") },
                        singleLine = true
                    )
                },
                confirmButton = {
                    TextButton(onClick = {
                        val parsed = polygonSidesText.toIntOrNull()
                        if (parsed == null || parsed !in 3..128) message = "Polygon sides must be between 3 and 128"
                        else { polygonSides = parsed; message = "Polygon configured · $polygonSides sides"; showPolygonSettings = false; pending = null }
                    }) { Text("Apply") }
                },
                dismissButton = { TextButton(onClick = { showPolygonSettings = false }) { Text("Cancel") } }
            )
        }
        nativePreview?.let { preview ->
            AlertDialog(
                onDismissRequest = { nativePreview = null; nativePreviewGate.invalidate() },
                title = { Text(nativePreviewTitle) },
                text = {
                    Column(Modifier.fillMaxWidth(), horizontalAlignment = Alignment.CenterHorizontally) {
                        Text("Rendered by the CADCraft native engine · 1600 × 1000", style = MaterialTheme.typography.bodySmall)
                        Spacer(Modifier.height(8.dp))
                        androidx.compose.foundation.Image(
                            bitmap = preview,
                            contentDescription = "Native CAD drawing preview",
                            modifier = Modifier.fillMaxWidth().heightIn(max = 420.dp),
                            contentScale = androidx.compose.ui.layout.ContentScale.Fit
                        )
                    }
                },
                confirmButton = { TextButton(onClick = { nativePreview = null; nativePreviewGate.invalidate() }) { Text("Close") } }
            )
        }
        pendingDxfImport?.let { result ->
            AlertDialog(
                onDismissRequest = { pendingDxfImport = null },
                title = { Text("Partial DXF import") },
                text = { Text("This drawing contains objects AR CAD cannot currently import: ${result.skippedTypes.entries.joinToString { "${it.key} × ${it.value}" }}. Importing supported entities will omit these objects. Your current drawing will only be replaced if you continue.") },
                confirmButton = { TextButton(onClick = { applyDxfImport(result); pendingDxfImport = null }) { Text("Import supported") } },
                dismissButton = { TextButton(onClick = { pendingDxfImport = null; message = "DXF import cancelled · current drawing unchanged" }) { Text("Cancel") } }
            )
        }
    }
}
