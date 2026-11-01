package com.arcad.app

private class WarningCadApi : CadNativeApi {
    var notes = """{"ok":true,"result":{"notes":[]}}"""
    var savedName = ""
    override fun create(metric: Boolean) = 77L
    override fun open(bytes: ByteArray, filename: String) = 77L
    override fun close(handle: Long) = Unit
    override fun entityCount(handle: Long) = 0L
    override fun readNotes(handle: Long) = notes
    override fun execute(handle: Long, command: String, paramsJson: String) = "{}"
    override fun undo(handle: Long) = "{}"
    override fun redo(handle: Long) = "{}"
    override fun save(handle: Long, filename: String): ByteArray { savedName = filename; return "saved".toByteArray() }
}

fun main() {
    val api = WarningCadApi()
    val doc = CadCraftDocument.create(api = api)
    check(!doc.hasImportWarnings())
    check(doc.save("clean.dxf").decodeToString() == "saved")
    check(api.savedName == "clean.dxf")

    api.notes = """{"ok":true,"result":{"notes":["Unsupported proxy entity was skipped"]}}"""
    check(doc.hasImportWarnings())
    var blocked = false
    try { doc.save("original.dwg") } catch (_: IllegalStateException) { blocked = true }
    check(blocked) { "Saving with import warnings must require explicit acknowledgement" }
    check(api.savedName == "clean.dxf") { "Blocked save must not reach native writer" }
    check(doc.save("reviewed-copy.dxf", acknowledgeReadNotes = true).decodeToString() == "saved")
    check(api.savedName == "reviewed-copy.dxf")
    check(doc.exportPreviewSvg().decodeToString() == "saved")
    check(api.savedName == "ar-cad-preview.svg") { "Preview export must use CADCraft SVG export path" }

    api.notes = "not-json"
    check(doc.hasImportWarnings()) { "Malformed diagnostics must fail closed" }
    var malformedBlocked = false
    try { doc.save("bad.dxf") } catch (_: IllegalStateException) { malformedBlocked = true }
    check(malformedBlocked)
    doc.close()
    println("CadNativeSaveSafetyTest: diagnostics gate, explicit acknowledgement, SVG export path and fail-closed parsing passed")
}
