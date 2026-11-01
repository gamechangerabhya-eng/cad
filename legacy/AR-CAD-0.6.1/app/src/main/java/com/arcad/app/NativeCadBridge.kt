package com.arcad.app

/** JNI entry points. Loading is explicit so the interim Kotlin editor can still start
 * while native ABI libraries are being built or are unavailable on a device. */
internal object NativeCadBridge {
    @Volatile private var loadFailure: Throwable? = null
    @Volatile private var loaded = false

    @Synchronized
    fun ensureLoaded(): Boolean {
        if (loaded) return true
        if (loadFailure != null) return false
        return try {
            System.loadLibrary("arcad_jni")
            if (!nativeIsAvailable()) {
                loadFailure = IllegalStateException("CADCraft native engine is not bundled. Run native/build-android.sh with Rust and Android NDK installed.")
                false
            } else {
                loaded = true
                true
            }
        } catch (error: UnsatisfiedLinkError) {
            loadFailure = error
            false
        } catch (error: SecurityException) {
            loadFailure = error
            false
        }
    }

    external fun nativeIsAvailable(): Boolean
    external fun nativeVersion(): String
    external fun nativeCreate(metric: Boolean): Long
    external fun nativeOpen(bytes: ByteArray, filename: String): Long
    external fun nativeClose(handle: Long)
    external fun nativeEntityCount(handle: Long): Long
    external fun nativeReadNotes(handle: Long): String
    external fun nativeExecute(handle: Long, command: String, paramsJson: String): String
    external fun nativeStartCommand(handle: Long, command: String): String
    external fun nativeInputCommand(handle: Long, inputJson: String): String
    external fun nativeSetViewport(handle: Long, widthPx: Double, heightPx: Double): String
    external fun nativePointerMove(handle: Long, x: Double, y: Double, shift: Boolean, nowSeconds: Double): String
    external fun nativePointerClick(handle: Long, x: Double, y: Double, shift: Boolean, nowSeconds: Double): String
    external fun nativeUndo(handle: Long): String
    external fun nativeRedo(handle: Long): String
    external fun nativeSave(handle: Long, filename: String): ByteArray
    external fun nativeRenderPng(handle: Long, width: Int, height: Int): ByteArray
}

/** Small testable contract around JNI. All methods on one document are serialized. */
internal interface CadNativeApi {
    fun create(metric: Boolean): Long
    fun open(bytes: ByteArray, filename: String): Long
    fun close(handle: Long)
    fun entityCount(handle: Long): Long
    fun readNotes(handle: Long): String
    fun execute(handle: Long, command: String, paramsJson: String): String
    fun startCommand(handle: Long, command: String): String = throw UnsupportedOperationException("Interactive native commands are not available")
    fun inputCommand(handle: Long, inputJson: String): String = throw UnsupportedOperationException("Interactive native commands are not available")
    fun setViewport(handle: Long, widthPx: Double, heightPx: Double): String = throw UnsupportedOperationException("Native viewport integration is not available")
    fun pointerMove(handle: Long, x: Double, y: Double, shift: Boolean, nowSeconds: Double): String = throw UnsupportedOperationException("Native pointer integration is not available")
    fun pointerClick(handle: Long, x: Double, y: Double, shift: Boolean, nowSeconds: Double): String = throw UnsupportedOperationException("Native pointer integration is not available")
    fun undo(handle: Long): String
    fun redo(handle: Long): String
    fun save(handle: Long, filename: String): ByteArray
    fun renderPng(handle: Long, width: Int, height: Int): ByteArray =
        throw UnsupportedOperationException("Native CADCraft renderer is not available")
}

internal object JniCadNativeApi : CadNativeApi {
    private fun requireLibrary() {
        check(NativeCadBridge.ensureLoaded()) {
            "AR CAD native engine is unavailable. Build the CADCraft Android libraries first."
        }
    }

    override fun create(metric: Boolean): Long { requireLibrary(); return NativeCadBridge.nativeCreate(metric) }
    override fun open(bytes: ByteArray, filename: String): Long { requireLibrary(); return NativeCadBridge.nativeOpen(bytes, filename) }
    override fun close(handle: Long) { requireLibrary(); NativeCadBridge.nativeClose(handle) }
    override fun entityCount(handle: Long): Long { requireLibrary(); return NativeCadBridge.nativeEntityCount(handle) }
    override fun readNotes(handle: Long): String { requireLibrary(); return NativeCadBridge.nativeReadNotes(handle) }
    override fun execute(handle: Long, command: String, paramsJson: String): String { requireLibrary(); return NativeCadBridge.nativeExecute(handle, command, paramsJson) }
    override fun startCommand(handle: Long, command: String): String { requireLibrary(); return NativeCadBridge.nativeStartCommand(handle, command) }
    override fun inputCommand(handle: Long, inputJson: String): String { requireLibrary(); return NativeCadBridge.nativeInputCommand(handle, inputJson) }
    override fun setViewport(handle: Long, widthPx: Double, heightPx: Double): String { requireLibrary(); return NativeCadBridge.nativeSetViewport(handle, widthPx, heightPx) }
    override fun pointerMove(handle: Long, x: Double, y: Double, shift: Boolean, nowSeconds: Double): String { requireLibrary(); return NativeCadBridge.nativePointerMove(handle, x, y, shift, nowSeconds) }
    override fun pointerClick(handle: Long, x: Double, y: Double, shift: Boolean, nowSeconds: Double): String { requireLibrary(); return NativeCadBridge.nativePointerClick(handle, x, y, shift, nowSeconds) }
    override fun undo(handle: Long): String { requireLibrary(); return NativeCadBridge.nativeUndo(handle) }
    override fun redo(handle: Long): String { requireLibrary(); return NativeCadBridge.nativeRedo(handle) }
    override fun save(handle: Long, filename: String): ByteArray { requireLibrary(); return NativeCadBridge.nativeSave(handle, filename) }
    override fun renderPng(handle: Long, width: Int, height: Int): ByteArray { requireLibrary(); return NativeCadBridge.nativeRenderPng(handle, width, height) }
}

/** Owns exactly one opaque CADCraft document handle. Never expose the pointer to UI code. */
internal class CadCraftDocument private constructor(
    private val api: CadNativeApi,
    handle: Long
) : java.io.Closeable {
    private val lock = java.util.concurrent.locks.ReentrantLock()
    private var nativeHandle: Long = handle

    init { check(handle != 0L) { "Native CAD document creation failed" } }

    private fun liveHandle(): Long {
        check(nativeHandle != 0L) { "CAD document is already closed" }
        return nativeHandle
    }

    fun entityCount(): Long = lock.runLocked { api.entityCount(liveHandle()) }
    /** Read diagnostics must be shown before a user overwrites the original drawing. */
    fun readNotes(): String = lock.runLocked { api.readNotes(liveHandle()) }
    fun execute(command: String, paramsJson: String = "{}"): String = lock.runLocked {
        require(command.isNotBlank() && command.length <= 256) { "Invalid CAD command name" }
        require(paramsJson.toByteArray(Charsets.UTF_8).size <= 1024 * 1024) { "Command parameters exceed 1 MB" }
        api.execute(liveHandle(), command, paramsJson)
    }
    /** Starts an interactive CADCraft command and returns its prompt/state JSON. */
    fun startCommand(command: String): String = lock.runLocked {
        require(command.isNotBlank() && command.length <= 256) { "Invalid CAD command name" }
        api.startCommand(liveHandle(), command)
    }
    /** Feeds one prompt input. JSON is validated and interpreted by the Rust bridge. */
    fun inputCommand(inputJson: String): String = lock.runLocked {
        require(inputJson.isNotBlank() && inputJson.toByteArray(Charsets.UTF_8).size <= 1024 * 1024) { "Invalid or oversized interactive input" }
        api.inputCommand(liveHandle(), inputJson)
    }
    /** Update the native view's pixel dimensions; world coordinates remain f64 in CADCraft. */
    fun setViewport(widthPx: Double, heightPx: Double): String = lock.runLocked {
        require(widthPx.isFinite() && heightPx.isFinite() && widthPx in 1.0..16384.0 && heightPx in 1.0..16384.0) { "Invalid native viewport dimensions" }
        api.setViewport(liveHandle(), widthPx, heightPx)
    }

    /** Move the cursor through CADCraft's OSNAP, grid, polar tracking and ORTHO pipeline. */
    fun pointerMove(x: Double, y: Double, shift: Boolean = false, nowSeconds: Double = System.nanoTime() / 1_000_000_000.0): String = lock.runLocked {
        requireWorldPoint(x, y); require(nowSeconds.isFinite() && nowSeconds >= 0.0) { "Invalid monotonic timestamp" }
        api.pointerMove(liveHandle(), x, y, shift, nowSeconds)
    }

    /** Send a tap to native selection or the active command prompt. */
    fun pointerClick(x: Double, y: Double, shift: Boolean = false, nowSeconds: Double = System.nanoTime() / 1_000_000_000.0): String = lock.runLocked {
        requireWorldPoint(x, y); require(nowSeconds.isFinite() && nowSeconds >= 0.0) { "Invalid monotonic timestamp" }
        api.pointerClick(liveHandle(), x, y, shift, nowSeconds)
    }

    private fun requireWorldPoint(x: Double, y: Double) {
        require(x.isFinite() && y.isFinite() && kotlin.math.abs(x) <= 1.0e12 && kotlin.math.abs(y) <= 1.0e12) { "Invalid world coordinates" }
    }

    fun undo(): String = lock.runLocked { api.undo(liveHandle()) }
    fun redo(): String = lock.runLocked { api.redo(liveHandle()) }
    /**
     * Saves through CADCraft's format-aware writer. Import diagnostics are checked first:
     * a drawing that contained unsupported/converted source objects cannot be overwritten
     * accidentally. Pass acknowledgeReadNotes=true only after the UI has shown the warnings.
     */
    /** Render a read-only PNG preview from the authoritative native document. */
    fun renderPreviewPng(width: Int = 1600, height: Int = 1000): ByteArray = lock.runLocked {
        require(width in 16..4096 && height in 16..4096) { "Preview dimensions must be between 16 and 4096 pixels" }
        require(width.toLong() * height.toLong() <= 16_777_216L) { "Preview cannot exceed 16 megapixels" }
        api.renderPng(liveHandle(), width, height)
    }

    fun save(filename: String, acknowledgeReadNotes: Boolean = false): ByteArray = lock.runLocked {
        require(filename.isNotBlank() && filename.length <= 4096 && '\u0000' !in filename) { "Invalid CAD filename" }
        val diagnostics = api.readNotes(liveHandle())
        if (hasNonEmptyNotesArray(diagnostics) && !acknowledgeReadNotes) {
            throw IllegalStateException("This drawing has import warnings. Review them before saving; pass acknowledgeReadNotes=true only after explicit user confirmation.")
        }
        api.save(liveHandle(), filename)
    }

    /** Exports a read-only SVG preview using CADCraft's own renderer/export pipeline. */
    fun exportPreviewSvg(): ByteArray = save("ar-cad-preview.svg", acknowledgeReadNotes = true)

    /** True when CADCraft reported one or more unsupported or converted source objects. */
    fun hasImportWarnings(): Boolean = lock.runLocked { hasNonEmptyNotesArray(api.readNotes(liveHandle())) }

    override fun close() = lock.runLocked {
        val handle = nativeHandle
        if (handle != 0L) {
            nativeHandle = 0L
            api.close(handle)
        }
    }

    companion object {
        fun create(metric: Boolean = true, api: CadNativeApi = JniCadNativeApi): CadCraftDocument =
            CadCraftDocument(api, api.create(metric))

        fun open(bytes: ByteArray, filename: String, api: CadNativeApi = JniCadNativeApi): CadCraftDocument {
            require(bytes.isNotEmpty() && bytes.size <= 64 * 1024 * 1024) { "CAD file must be between 1 byte and 64 MB" }
            require(filename.isNotBlank() && filename.length <= 4096 && '\u0000' !in filename) { "Invalid CAD filename" }
            return CadCraftDocument(api, api.open(bytes, filename))
        }
    }
}

/** Small bounded JSON-field probe for CADCraft's stable {result:{notes:[...]}} response.
 * It intentionally fails closed: malformed or unexpected diagnostics are treated as warnings.
 */
private fun hasNonEmptyNotesArray(json: String): Boolean {
    if (json.length > 1024 * 1024) return true
    val key = Regex("""\"notes\"\s*:""").find(json) ?: return true
    var index = key.range.last + 1
    while (index < json.length && json[index].isWhitespace()) index++
    if (index >= json.length || json[index] != '[') return true
    index++
    while (index < json.length && json[index].isWhitespace()) index++
    return index >= json.length || json[index] != ']'
}

private inline fun <T> java.util.concurrent.locks.ReentrantLock.runLocked(block: () -> T): T {
    lock()
    try { return block() } finally { unlock() }
}
