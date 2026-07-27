package com.htmake.reader.utils

import java.lang.reflect.ParameterizedType
import java.lang.reflect.Type

/**
 * Runtime implementation of ParameterizedType for Gson generic type resolution.
 */
class ParameterizedTypeImpl(
    private val rawType: Type,
    @Suppress("UNCHECKED_CAST")
    private vararg val typeArguments: Type
) : ParameterizedType {

    @Suppress("UNCHECKED_CAST")
    override fun getActualTypeArguments(): Array<Type> {
        return typeArguments as Array<Type>
    }

    override fun getRawType(): Type {
        return rawType
    }

    override fun getOwnerType(): Type? {
        return null
    }

    override fun equals(other: Any?): Boolean {
        if (this === other) return true
        if (other !is ParameterizedType) return false
        return rawType == other.rawType &&
                typeArguments.contentEquals(other.actualTypeArguments)
    }

    override fun hashCode(): Int {
        return rawType.hashCode() xor typeArguments.contentHashCode()
    }

    override fun toString(): String {
        val sb = StringBuilder()
        sb.append(rawType.typeName)
        if (typeArguments.isNotEmpty()) {
            sb.append("<")
            sb.append(typeArguments.joinToString(", ") { it.typeName })
            sb.append(">")
        }
        return sb.toString()
    }
}
