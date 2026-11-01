package com.arcad.app

private class FakeCadNativeApi : CadNativeApi {
    var nextHandle = 41L
    var closed = 0
    var lastCommand = ""
    var lastFilename = ""
    var lastInteractiveCommand = ""
    var lastInteractiveInput = ""
    var lastViewport = Pair(0.0, 0.0)
    var lastPointer = Triple(0.0, 0.0, false)
    var lastClick = Triple(0.0, 0.0, false)
    var count = 7L
    override fun create(metric: Boolean): Long = nextHandle
    override fun open(bytes: ByteArray, filename: String): Long { lastFilename = filename; return nextHandle }
    override fun close(handle: Long) { check(handle == nextHandle); closed++ }
    override fun entityCount(handle: Long): Long { check(handle == nextHandle); return count }
    override fun readNotes(handle: Long): String { check(handle == nextHandle); return """{"ok":true,"result":{"notes":[]}}""" }
    override fun execute(handle: Long, command: String, paramsJson: String): String { lastCommand = command; return """{"ok":true}""" }
    override fun startCommand(handle: Long, command: String): String { lastInteractiveCommand = command; return """{"ok":true,"result":{"commandActive":true}}""" }
    override fun inputCommand(handle: Long, inputJson: String): String { lastInteractiveInput = inputJson; return """{"ok":true,"result":{"commandActive":false}}""" }
    override fun setViewport(handle: Long, widthPx: Double, heightPx: Double): String { lastViewport = widthPx to heightPx; return """{"ok":true}""" }
    override fun pointerMove(handle: Long, x: Double, y: Double, shift: Boolean, nowSeconds: Double): String { lastPointer = Triple(x, y, shift); return """{"ok":true}""" }
    override fun pointerClick(handle: Long, x: Double, y: Double, shift: Boolean, nowSeconds: Double): String { lastClick = Triple(x, y, shift); return """{"ok":true}""" }
    override fun undo(handle: Long): String = """{"ok":true}"""
    override fun redo(handle: Long): String = """{"ok":true}"""
    override fun save(handle: Long, filename: String): ByteArray { lastFilename = filename; return byteArrayOf(1, 2, 3) }
}

private inline fun expectFailure(block: () -> Unit) {
    var failed = false
    try { block() } catch (_: IllegalArgumentException) { failed = true } catch (_: IllegalStateException) { failed = true }
    check(failed) { "Expected operation to fail" }
}

fun main() {
    val api = FakeCadNativeApi()
    val doc = CadCraftDocument.create(metric = true, api = api)
    check(doc.entityCount() == 7L)
    check(doc.readNotes().contains("notes"))
    check(doc.execute("LINE", "{}") == """{"ok":true}""" && api.lastCommand == "LINE")
    check(doc.startCommand("line").contains("commandActive") && api.lastInteractiveCommand == "line")
    val pointInput = """{"kind":"point","value":{"x":12.5,"y":-3.0}}"""
    check(doc.inputCommand(pointInput).contains("commandActive") && api.lastInteractiveInput == pointInput)
    check(doc.setViewport(1080.0, 1920.0).contains("ok") && api.lastViewport == (1080.0 to 1920.0))
    check(doc.pointerMove(12.25, -9.5, shift = true, nowSeconds = 5.0).contains("ok"))
    check(api.lastPointer == Triple(12.25, -9.5, true))
    check(doc.pointerClick(13.0, 4.0, shift = false, nowSeconds = 5.1).contains("ok"))
    check(api.lastClick == Triple(13.0, 4.0, false))
    expectFailure { doc.setViewport(0.0, 100.0) }
    expectFailure { doc.pointerMove(Double.NaN, 0.0) }
    expectFailure { doc.pointerClick(1.0e13, 0.0) }
    expectFailure { doc.pointerMove(0.0, 0.0, nowSeconds = -1.0) }
    check(doc.save("plan.dxf").contentEquals(byteArrayOf(1, 2, 3)))
    check(api.lastFilename == "plan.dxf")
    doc.close()
    doc.close()
    check(api.closed == 1) { "Native handle must be released exactly once" }
    expectFailure { doc.entityCount() }
    expectFailure { doc.startCommand("line") }
    expectFailure { doc.inputCommand("{}") }
    expectFailure { doc.setViewport(100.0, 100.0) }
    expectFailure { doc.pointerMove(0.0, 0.0) }
    expectFailure { doc.pointerClick(0.0, 0.0) }

    val opened = CadCraftDocument.open(byteArrayOf(10), "existing.dxf", api)
    check(api.lastFilename == "existing.dxf")
    opened.close()
    expectFailure { CadCraftDocument.open(ByteArray(0), "empty.dxf", api) }
    expectFailure { CadCraftDocument.create(api = object : CadNativeApi by api { override fun create(metric: Boolean) = 0L }) }
    println("CadNativeDocumentTest: lifecycle, serialization boundary, limits and close semantics passed")
}
