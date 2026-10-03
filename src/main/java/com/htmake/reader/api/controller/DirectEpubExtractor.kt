package com.htmake.reader.api.controller

import mu.KotlinLogging
import java.io.File
import java.nio.file.Files
import java.nio.file.LinkOption
import java.nio.file.Path
import java.nio.file.SimpleFileVisitor
import java.nio.file.FileVisitResult
import java.nio.file.attribute.BasicFileAttributes
import me.ag2s.epublib.util.EpubArchivePolicy
import java.util.UUID

/** Extracts a directly imported EPUB into an owned cache, never into its source directory. */
object DirectEpubExtractor {
    private val logger = KotlinLogging.logger {}
    private const val MARKER = ".reader-epub-source"

    fun extract(source: File, destination: File, force: Boolean, bookUrl: String): Boolean {
        return extractCache(source, destination, force, bookUrl, false)
    }

    /** Preserve an old unmarked legacy cache in a recoverable sibling, never delete it. */
    fun extractLegacy(source: File, destination: File, force: Boolean, bookUrl: String): Boolean {
        return extractCache(source, destination, force, bookUrl, true)
    }

    private fun extractCache(source: File, destination: File, force: Boolean, bookUrl: String, legacy: Boolean): Boolean {
        val staging = File(destination.parentFile, destination.name + ".tmp-" + UUID.randomUUID())
        var previous: Path? = null
        var previousOwned = false
        var installed = false
        try {
            EpubArchivePolicy.noSymbolicParents(destination.toPath())
            val destinationPath = destination.toPath().toAbsolutePath().normalize()
            if (source.canonicalFile.toPath().startsWith(destinationPath)) return false
            val marker = File(destination, MARKER)
            val markerText = if (marker.isFile && marker.length() <= 16384 && !Files.isSymbolicLink(marker.toPath())) marker.readText() else ""
            previousOwned = markerText.startsWith("$bookUrl\n")
            if (destination.exists() && !previousOwned && !legacy) return false
            val identity = "$bookUrl\n${source.length()}\n${source.lastModified()}\narchive-policy-v1"
            // Preflight even cache hits, but don't re-inflate a previously validated unchanged archive.
            EpubArchivePolicy.open(source, EpubArchivePolicy.EPUB).use {
                EpubArchivePolicy.validateMetadata(it, EpubArchivePolicy.EPUB)
                if (destination.exists() && !previousOwned && legacy && !force) {
                    // Original-layout edited caches remain user data; don't silently regenerate them.
                    EpubArchivePolicy.validate(it)
                    for (entry in EpubArchivePolicy.validateMetadata(it, EpubArchivePolicy.EPUB).values) {
                        val target = EpubArchivePolicy.extractionTarget(destinationPath, entry.name).toFile()
                        if (target.isFile && target.length() > EpubArchivePolicy.EPUB.entryBytes) return false
                    }
                    return true
                }
            }
            if (destination.exists() && previousOwned && !force && markerText == identity) return true
            Files.createDirectories(destination.parentFile.toPath())
            if (!staging.mkdirs()) return false
            val stagingPath = staging.toPath().toAbsolutePath().normalize()
            EpubArchivePolicy.open(source, EpubArchivePolicy.EPUB).use { archive ->
                val entries = EpubArchivePolicy.validateMetadata(archive, EpubArchivePolicy.EPUB).values
                val targets = entries.associateWith { EpubArchivePolicy.extractionTarget(stagingPath, it.name) }
                if (targets.values.toSet().size != targets.size || targets.values.any { it.fileName.toString() == MARKER }) return false
                val budget = EpubArchivePolicy.Budget(EpubArchivePolicy.EPUB.expandedBytes)
                val buffer = ByteArray(8192)
                for (entry in entries) {
                    val target = targets.getValue(entry)
                    EpubArchivePolicy.entryStream(archive, entry, EpubArchivePolicy.EPUB, budget).use { input ->
                        if (entry.isDirectory) {
                            while (input.read(buffer) >= 0) { /* Verify directory payload too. */ }
                            Files.createDirectories(target)
                        } else {
                            Files.createDirectories(target.parent)
                            Files.newOutputStream(target, java.nio.file.StandardOpenOption.CREATE_NEW,
                                java.nio.file.StandardOpenOption.WRITE, LinkOption.NOFOLLOW_LINKS).use { output ->
                                input.copyTo(output, 8192)
                            }
                        }
                    }
                }
            }
            File(staging, MARKER).writeText(identity)
            if (destination.exists()) {
                previous = File(destination.parentFile, destination.name + ".previous-" + UUID.randomUUID()).toPath()
                Files.move(destination.toPath(), previous)
            }
            Files.move(staging.toPath(), destination.toPath())
            installed = true
            if (previousOwned && previous != null) {
                try { deleteOwnedTree(previous!!) }
                catch (e: Exception) { logger.warn("Previous owned EPUB cache retained after cleanup failure", e) }
            }
            return true
        } catch (e: Exception) {
            if (!installed && previous != null && !Files.exists(destination.toPath(), LinkOption.NOFOLLOW_LINKS)) {
                try { Files.move(previous, destination.toPath()) }
                catch (rollback: Exception) { logger.warn("Previous EPUB cache retained at its recoverable sibling", rollback) }
            }
            logger.warn("Unable to safely extract EPUB cache", e)
            return false
        } finally {
            if (Files.exists(staging.toPath(), LinkOption.NOFOLLOW_LINKS)) deleteOwnedTree(staging.toPath())
        }
    }

    /** walkFileTree never follows symbolic links, unlike a File-based recursive walk. */
    private fun deleteOwnedTree(root: Path) {
        Files.walkFileTree(root, object : SimpleFileVisitor<Path>() {
            override fun visitFile(file: Path, attributes: BasicFileAttributes): FileVisitResult {
                Files.delete(file)
                return FileVisitResult.CONTINUE
            }
            override fun postVisitDirectory(directory: Path, failure: java.io.IOException?): FileVisitResult {
                if (failure != null) throw failure
                Files.delete(directory)
                return FileVisitResult.CONTINUE
            }
        })
    }
}
