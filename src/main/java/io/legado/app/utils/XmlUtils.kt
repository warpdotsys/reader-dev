package io.legado.app.utils

import java.io.InputStream
import java.util.LinkedHashMap
import javax.xml.parsers.DocumentBuilderFactory
import org.w3c.dom.Node
import org.w3c.dom.NodeList
import org.xml.sax.InputSource

object XmlUtils {

    fun xml2map(source: Any): MutableMap<String, Any> {
        val doc = LinkedHashMap<String, Any>()
        return try {
            val builder = DocumentBuilderFactory.newInstance().newDocumentBuilder()
            when (source) {
                is String -> parseNode(builder.parse(source).childNodes)
                is InputStream -> parseNode(builder.parse(source).childNodes)
                is InputSource -> parseNode(builder.parse(source).childNodes)
                else -> doc
            }
        } catch (e: Exception) {
            e.printStackTrace()
            doc
        }
    }

    fun parseNode(list: NodeList): MutableMap<String, Any> {
        val doc = LinkedHashMap<String, Any>()
        for (index in 0 until list.length) {
            val node = list.item(index)
            if (node.nodeType != Node.ELEMENT_NODE) continue

            val children = node.childNodes
            if (children.length == 1 && node.firstChild.nodeType == Node.TEXT_NODE) {
                doc[node.nodeName] = node.firstChild.nodeValue
            } else if (children.length > 1) {
                doc[node.nodeName] = parseNode(children)
            }
        }
        return doc
    }
}
