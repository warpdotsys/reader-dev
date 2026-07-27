package com.htmake.reader.utils

class AttemptResult<T>(val value: T?, val error: Exception?) {

    val isSuccess: Boolean get() = error == null

    val isFailure: Boolean get() = error != null

    fun <R> then(block: (T) -> AttemptResult<R>): AttemptResult<R> {
        return if (value != null) {
            try {
                block(value)
            } catch (e: Exception) {
                AttemptResult(null, e)
            }
        } else {
            AttemptResult(null, error)
        }
    }

    companion object {
        fun <T> success(value: T): AttemptResult<T> {
            return AttemptResult(value, null)
        }

        fun <T> failure(error: Exception): AttemptResult<T> {
            return AttemptResult(null, error)
        }
    }
}
