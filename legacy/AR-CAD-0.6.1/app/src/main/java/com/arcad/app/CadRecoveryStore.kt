package com.arcad.app

import java.io.File
import java.io.FileOutputStream
import java.nio.file.AtomicMoveNotSupportedException
import java.nio.file.Files
import java.nio.file.StandardCopyOption

/** Crash-resistant local recovery snapshots. Keeps the previous valid snapshot as fallback. */
class CadRecoveryStore(private val directory: File, private val maxBytes: Int = 25 * 1024 * 1024) {
    private val current get() = File(directory, "recovery.arcad.json")
    private val temporary get() = File(directory, "recovery.arcad.json.tmp")
    private val backup get() = File(directory, "recovery.arcad.json.bak")

    @Synchronized
    fun save(contents: String) {
        val bytes = contents.toByteArray(Charsets.UTF_8)
        require(bytes.size <= maxBytes) { "Recovery snapshot exceeds size limit" }
        check(directory.exists() || directory.mkdirs()) { "Cannot create recovery directory" }

        try {
            FileOutputStream(temporary).use { stream ->
                stream.write(bytes)
                stream.fd.sync()
            }
            if (current.isFile) {
                Files.copy(current.toPath(), backup.toPath(), StandardCopyOption.REPLACE_EXISTING)
                FileOutputStream(backup, true).use { it.fd.sync() }
            }
            try {
                Files.move(temporary.toPath(), current.toPath(), StandardCopyOption.ATOMIC_MOVE, StandardCopyOption.REPLACE_EXISTING)
            } catch (_: AtomicMoveNotSupportedException) {
                Files.move(temporary.toPath(), current.toPath(), StandardCopyOption.REPLACE_EXISTING)
            }
        } catch (failure: Exception) {
            temporary.delete()
            throw failure
        }
    }

    /** Returns the newest snapshot accepted by [isValid], falling back to the previous snapshot. */
    @Synchronized
    fun load(isValid: (String) -> Boolean): String? {
        for (candidate in listOf(current, backup)) {
            if (!candidate.isFile || candidate.length() > maxBytes) continue
            val contents = runCatching { candidate.readBytes().toString(Charsets.UTF_8) }.getOrNull() ?: continue
            if (runCatching { isValid(contents) }.getOrDefault(false)) return contents
        }
        return null
    }

    @Synchronized
    fun clear() {
        current.delete()
        temporary.delete()
        backup.delete()
    }
}
