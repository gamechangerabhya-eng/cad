package com.arcad.app

/**
 * Prevents an older asynchronous native preview from replacing a newer request or
 * displaying a render after the source drawing has changed.
 */
internal class NativePreviewGate<T> {
    private var nextToken = 0L
    private var activeToken = 0L
    private var activeSnapshot: T? = null

    @Synchronized
    fun begin(snapshot: T): Long {
        check(nextToken < Long.MAX_VALUE) { "Native preview request counter exhausted" }
        nextToken += 1
        activeToken = nextToken
        activeSnapshot = snapshot
        return activeToken
    }

    @Synchronized
    fun isCurrent(token: Long, snapshot: T): Boolean =
        token != 0L && token == activeToken && activeSnapshot == snapshot

    @Synchronized
    fun invalidate() {
        activeToken = 0L
        activeSnapshot = null
    }
}
