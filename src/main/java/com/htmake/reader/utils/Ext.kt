@file:JvmName("ExtKt")
@file:JvmMultifileClass

package com.htmake.reader.utils

import java.io.File
import java.io.OutputStream
import java.io.InputStream
import java.io.FileOutputStream
import java.io.FileInputStream
import java.util.zip.ZipFile
import java.util.zip.ZipEntry
import java.util.zip.ZipOutputStream
import okhttp3.HttpUrl.Companion.toHttpUrl
import me.ag2s.epublib.util.EpubArchivePolicy
import java.nio.file.Files
import java.nio.file.Paths
import java.nio.file.StandardOpenOption
import java.nio.file.LinkOption

/**
 * @Date: 2019-07-19 23:43
 * @Description:
 */

fun String.url(): String {
    if (this.startsWith("//")) {
        return ("http:" + this).toHttpUrl().toString()
    } else if (this.startsWith("http")) {
        return this.toHttpUrl().toString()
    }
    return this
}

fun File.deleteRecursively() {
    if (this.exists()) {
        if (this.isFile() ) {
            this.delete();
        } else {
            this.listFiles().forEach{
                it.deleteRecursively()
            }
            this.delete()
        }
    }
}

fun File.unzip(descDir: String): Boolean {
    if (!this.exists()) {
        return false
    }
    try {
        val root = Paths.get(descDir).toAbsolutePath().normalize()
        EpubArchivePolicy.noSymbolicParents(root)
        EpubArchivePolicy.open(this, EpubArchivePolicy.GENERAL_ZIP).use { archive ->
            val entries = EpubArchivePolicy.validateMetadata(archive, EpubArchivePolicy.GENERAL_ZIP).values
            val targets = entries.associateWith { EpubArchivePolicy.extractionTarget(root, it.name) }
            // Validate every path before creating a file; Path equality also rejects Windows case aliases.
            if (targets.values.toSet().size != targets.size) {
                throw EpubArchivePolicy.ArchiveException("ZIP 解包路径存在冲突")
            }
            val budget = EpubArchivePolicy.Budget(EpubArchivePolicy.GENERAL_ZIP.expandedBytes)
            val buffer = ByteArray(8192)
            for (entry in entries) {
                val target = targets.getValue(entry)
                EpubArchivePolicy.extractionTarget(root, entry.name)
                EpubArchivePolicy.entryStream(archive, entry, EpubArchivePolicy.GENERAL_ZIP, budget).use { input ->
                    if (entry.isDirectory) {
                        while (input.read(buffer) >= 0) { /* Verify directory CRC/size too. */ }
                        Files.createDirectories(target)
                    } else {
                        Files.createDirectories(target.parent)
                        Files.newOutputStream(target, StandardOpenOption.CREATE, StandardOpenOption.TRUNCATE_EXISTING,
                            StandardOpenOption.WRITE, LinkOption.NOFOLLOW_LINKS).use { output ->
                            input.copyTo(output, 8192)
                        }
                    }
                }
            }
        }
        return true
    } catch(e: Exception) {
        e.printStackTrace()
    }
    return false
}

fun File.zip(zipFilePath: String): Boolean {
    if (!this.exists()) {
        return false
    }
    if (this.isDirectory()) {
        val files = this.listFiles()
        val filesList: List<File> = files.toList()
        return zip(filesList, zipFilePath)
    } else {
        return zip(arrayListOf(this), zipFilePath)
    }
}

fun zip(files: List<File>, zipFilePath: String): Boolean {
    if (files.isEmpty()) {
        return false
    }

    val zipFile = createFile(zipFilePath)
    val buffer = ByteArray(1024)
    var zipOutputStream: ZipOutputStream? = null
    var inputStream: FileInputStream? = null
    try {
        zipOutputStream = ZipOutputStream(FileOutputStream(zipFile))
        for (file in files) {
            if (!file.exists()) continue
            zipOutputStream.putNextEntry(ZipEntry(file.name))
            inputStream = FileInputStream(file)
            var len: Int
            while (inputStream.read(buffer).also { len = it } > 0) {
                zipOutputStream.write(buffer, 0, len)
            }
            zipOutputStream.closeEntry()
        }
        return true
    } catch(e: Exception) {
        e.printStackTrace()
    } finally {
        inputStream?.close()
        zipOutputStream?.close()
    }
    return false
}

fun createDir(filePath: String): File {
    val file = File(filePath)
    logger.debug("createDir filePath {}", filePath)
    if (!file.exists()) {
        file.mkdirs()
    }
    return file
}

fun createFile(filePath: String): File {
    val file = File(filePath)
    val parentFile = file.parentFile!!
    logger.debug("createFile filePath {}", filePath)
    if (!parentFile.exists()) {
        parentFile.mkdirs()
    }
    if (!file.exists()) {
        file.createNewFile()
    }
    return file
}
