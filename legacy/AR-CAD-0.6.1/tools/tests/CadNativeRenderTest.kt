package com.arcad.app

private class RenderCadApi : CadNativeApi {
    var closed = false
    var lastSize = 0 to 0
    override fun create(metric: Boolean) = 91L
    override fun open(bytes: ByteArray, filename: String) = 91L
    override fun close(handle: Long) { closed = true }
    override fun entityCount(handle: Long) = 0L
    override fun readNotes(handle: Long) = """{"ok":true,"result":{"notes":[]}}"""
    override fun execute(handle: Long, command: String, paramsJson: String) = "{}"
    override fun undo(handle: Long) = "{}"
    override fun redo(handle: Long) = "{}"
    override fun save(handle: Long, filename: String) = byteArrayOf()
    override fun renderPng(handle: Long, width: Int, height: Int): ByteArray {
        check(!closed && handle == 91L)
        lastSize = width to height
        return byteArrayOf(0x89.toByte(), 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a)
    }
}

fun main() {
    val api = RenderCadApi()
    val doc = CadCraftDocument.create(api = api)
    val png = doc.renderPreviewPng()
    check(png.take(8).map { it.toInt() and 0xff } == listOf(137, 80, 78, 71, 13, 10, 26, 10))
    check(api.lastSize == (1600 to 1000))
    doc.renderPreviewPng(320, 240)
    check(api.lastSize == (320 to 240))
    var badDimensions = false
    try { doc.renderPreviewPng(8, 240) } catch (_: IllegalArgumentException) { badDimensions = true }
    check(badDimensions)
    var tooLarge = false
    try { doc.renderPreviewPng(4097, 4096) } catch (_: IllegalArgumentException) { tooLarge = true }
    check(tooLarge)
    doc.close()
    var closedRejected = false
    try { doc.renderPreviewPng() } catch (_: IllegalStateException) { closedRejected = true }
    check(closedRejected)
    println("CadNativeRenderTest: PNG signature, default/custom dimensions, pixel limits and closed-handle rejection passed")
}
