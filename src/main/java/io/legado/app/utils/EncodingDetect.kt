package io.legado.app.utils

import io.legado.app.lib.icu4j.CharsetDetector
import org.jsoup.Jsoup
import java.io.File
import java.io.FileInputStream

/**
 * 自动获取文件的编码
 * */
@Suppress("MemberVisibilityCanBePrivate", "unused")
object EncodingDetect {

    fun getHtmlEncode(bytes: ByteArray): String {
        try {
            val htmlStr = String(bytes, Charsets.UTF_8)
            val doc = Jsoup.parse(htmlStr)
            val metaTags = doc.getElementsByTag("meta")
            var charsetStr: String
            for (metaTag in metaTags) {
                charsetStr = metaTag.attr("charset")
                if (charsetStr.isNotEmpty()) {
                    return charsetStr
                }
                val httpEquiv = metaTag.attr("http-equiv")
                if (httpEquiv.equals("content-type", true)) {
                    val content = metaTag.attr("content")
                    if (content.toLowerCase().contains("charset")) {
                        charsetStr = content.substring(
                            content.toLowerCase().indexOf("charset") + "charset=".length
                        )
                    } else {
                        charsetStr = content.substring(content.toLowerCase().indexOf(";") + 1)
                    }
                    if (charsetStr.isNotEmpty()) {
                        return charsetStr
                    }
                }
            }
        } catch (ignored: Exception) {
        }
        return getEncode(bytes)
    }

    fun getEncode(bytes: ByteArray): String {
        val match = CharsetDetector().setText(bytes).detect()
        return match?.name ?: "UTF-8"
    }

    /**
     * 得到文件的编码
     */
    fun getEncode(filePath: String): String {
        return getEncode(File(filePath))
    }

    /**
     * 得到文件的编码
     */
    fun getEncode(file: File): String {
        val tempByte = getFileBytes(file)
        return getEncode(tempByte)
    }

    private fun getFileBytes(file: File?): ByteArray {
        val byteArray = ByteArray(8000)
        try {
            FileInputStream(file).use {
                it.read(byteArray)
            }
        } catch (e: Exception) {
            System.err.println("Error: $e")
        }
        return byteArray
    }
}
