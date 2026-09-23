package com.htmake.reader.api.controller

/** Source indices are zero-based; the source count is exclusive. */
object SourceScanCursor {
    @JvmStatic
    fun isComplete(lastIndex: Int, sourceCount: Int): Boolean =
        sourceCount > 0 && lastIndex >= sourceCount - 1
}
