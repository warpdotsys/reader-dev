package io.legado.app.utils

import java.io.File
import java.io.InterruptedIOException
import java.nio.file.AccessDeniedException
import java.nio.file.Files
import java.nio.file.LinkOption
import java.nio.file.StandardCopyOption

internal const val CACHE_PUBLICATION_TEMP_PREFIX = ".reader-acache-"

/** Publish complete bytes in the same directory; readers never see an in-place truncate/write. */
internal fun publishCacheBytes(target: File, bytes: ByteArray) {
    val temporary = Files.createTempFile(target.parentFile.toPath(), CACHE_PUBLICATION_TEMP_PREFIX, ".tmp")
    var primary: Throwable? = null
    try {
        Files.write(temporary, bytes)
        // Unsupported atomic replacement is a cache-write failure, not permission
        // to publish partially copied bytes. The previously published file remains.
        for (attempt in 0..3) {
            try {
                Files.move(temporary, target.toPath(), StandardCopyOption.ATOMIC_MOVE, StandardCopyOption.REPLACE_EXISTING)
                return
            } catch (error: AccessDeniedException) {
                // Windows file filters can briefly hold an existing cache file.
                // Retry only complete atomic replacement, with <=15ms total wait;
                // never retry a directory/symlink target or fall back to a copy.
                if (attempt == 3 || !Files.isRegularFile(target.toPath(), LinkOption.NOFOLLOW_LINKS)) throw error
                try { Thread.sleep(5) } catch (interrupted: InterruptedException) {
                    Thread.currentThread().interrupt()
                    throw InterruptedIOException("Cache publication interrupted").apply { initCause(interrupted) }
                }
            }
        }
    } catch (error: Throwable) {
        primary = error
        throw error
    } finally {
        try { Files.deleteIfExists(temporary) } catch (cleanup: Throwable) {
            if (primary != null) primary.addSuppressed(cleanup) else throw cleanup
        }
    }
}
