package com.arcad.app

import java.io.File
import java.nio.file.Files

fun main() {
    val directory = Files.createTempDirectory("arcad-recovery-test-").toFile()
    try {
        val store = CadRecoveryStore(directory, 1024)
        check(store.load { it.startsWith("snapshot:") } == null)
        store.save("snapshot:first")
        check(store.load { it.startsWith("snapshot:") } == "snapshot:first")
        store.save("snapshot:second")
        check(store.load { it.startsWith("snapshot:") } == "snapshot:second")
        File(directory, "recovery.arcad.json").writeText("corrupt", Charsets.UTF_8)
        check(store.load { it.startsWith("snapshot:") } == "snapshot:first") { "Corrupt current snapshot should fall back to previous valid snapshot" }
        check(runCatching { store.save("x".repeat(1025)) }.isFailure) { "Recovery byte limit was not enforced" }
        store.clear()
        check(store.load { true } == null)
        println("Recovery store tests passed: atomic snapshot, backup fallback, size guard and clear")
    } finally {
        directory.deleteRecursively()
    }
}
