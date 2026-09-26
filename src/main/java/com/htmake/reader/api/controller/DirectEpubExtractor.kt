package com.htmake.reader.api.controller

import mu.KotlinLogging
import java.io.File
import java.nio.file.Files
import java.util.UUID
import java.util.zip.ZipFile

/** Extracts a directly imported EPUB into an owned cache, never into its source directory. */
object DirectEpubExtractor {
    private val logger = KotlinLogging.logger {}
    private const val MAX_ENTRIES = 10_000
    private const val MAX_UNCOMPRESSED_BYTES = 1_073_741_824L
    private const val MARKER = ".reader-epub-source"

    fun extract(source: File, destination: File, force: Boolean, bookUrl: String): Boolean {
        val marker = File(destination, MARKER)
        val identity = "$bookUrl\n${source.length()}\n${source.lastModified()}"
        if (destination.exists()) {
            if (!marker.isFile || !marker.readText().startsWith("$bookUrl\n")) return false
            if (!force && marker.readText() == identity) return true
        }

        val staging = File(destination.parentFile, destination.name + ".tmp-" + UUID.randomUUID())
        try {
            if (!staging.mkdirs()) return false
            val stagingPath = staging.canonicalFile.toPath()
            ZipFile(source).use { archive ->
                val entries = archive.entries()
                var count = 0
                var totalBytes = 0L
                val buffer = ByteArray(8192)
                while (entries.hasMoreElements()) {
                    val entry = entries.nextElement()
                    if (++count > MAX_ENTRIES || entry.size < 0 ||
                        entry.size > MAX_UNCOMPRESSED_BYTES - totalBytes) return false
                    val target = File(staging, entry.name).canonicalFile
                    if (!target.toPath().startsWith(stagingPath) ||
                        target.toPath() == stagingPath || target.name == MARKER) return false
                    if (entry.isDirectory) {
                        if (!target.mkdirs() && !target.isDirectory) return false
                    } else {
                        if (!target.parentFile.mkdirs() && !target.parentFile.isDirectory) return false
                        var written = 0L
                        archive.getInputStream(entry).use { input ->
                            target.outputStream().use { output ->
                                while (true) {
                                    val read = input.read(buffer)
                                    if (read < 0) break
                                    written += read
                                    // The central-directory length is untrusted, so also cap real output.
                                    if (written > entry.size || written > MAX_UNCOMPRESSED_BYTES - totalBytes) {
                                        return false
                                    }
                                    output.write(buffer, 0, read)
                                }
                            }
                        }
                        if (written != entry.size) return false
                        totalBytes += written
                    }
                }
            }
            File(staging, MARKER).writeText(identity)
            if (destination.exists()) destination.deleteRecursively()
            if (destination.exists()) return false
            Files.move(staging.toPath(), destination.toPath())
            return true
        } catch (e: Exception) {
            logger.warn("Unable to extract direct EPUB: ${source.path}", e)
            return false
        } finally {
            if (staging.exists()) staging.deleteRecursively()
        }
    }
}
