package io.legado.app.adapters

/**
 * Singleton helper holding the current ReaderAdapterInterface instance.
 */
object ReaderAdapterHelper {

    var readerAdapter: ReaderAdapterInterface = DefaultAdpater()

    fun setAdapter(adapter: ReaderAdapterInterface) {
        readerAdapter = adapter
    }

    fun getAdapter(): ReaderAdapterInterface {
        return readerAdapter
    }
}
