package okayu.tauri.plugin.android.fs

import android.content.ContentProvider
import android.content.ContentValues
import android.content.Context
import android.database.Cursor
import android.database.MatrixCursor
import android.net.Uri
import android.os.Build
import android.os.Handler
import android.os.HandlerThread
import android.os.ParcelFileDescriptor
import android.os.ProxyFileDescriptorCallback
import android.os.storage.StorageManager
import android.provider.DocumentsContract
import android.provider.MediaStore
import android.provider.OpenableColumns
import android.system.ErrnoException
import android.system.OsConstants
import androidx.annotation.RequiresApi
import app.tauri.Logger
import java.io.FileNotFoundException
import java.io.IOException
import java.lang.IllegalStateException

class AFCustomFileProvider: ContentProvider() {

    override fun onCreate(): Boolean {
        return true
    }

    override fun openFile(uri: Uri, mode: String): ParcelFileDescriptor {
        try {
            // mode が r, w, wa, wt, rw, rwt 以外の場合はここでエラーになる。
            val fileResult = CustomFileProviderBridge.open(uri.toString(), mode)
            val fileType = fileResult[0]
            val fileValue = fileResult[1]

            when (fileType) {
                // raw fd
                0 -> return ParcelFileDescriptor.adoptFd(fileValue)
                // custom file
                1 -> {
                    var fileId: Int? = null
                    var token: ThreadToken? = null
                    try {
                        fileId = fileValue

                        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.O) {
                            throw UnsupportedOperationException("Custom file provider is only for Android 8 or higher")
                        }

                        val sm = context!!.getSystemService(Context.STORAGE_SERVICE) as StorageManager
                        token = ThreadHandlerManager.acquire()

                        return sm.openProxyFileDescriptor(
                            when (mode) {
                                "w", "wa", "wt" -> ParcelFileDescriptor.MODE_WRITE_ONLY
                                "r" -> ParcelFileDescriptor.MODE_READ_ONLY
                                else -> ParcelFileDescriptor.MODE_READ_WRITE
                            },
                            AFCustomFile(
                                fileId = fileId,
                                read = mode.contains("r"),
                                write = mode.contains("w"),
                                onRelease = { ThreadHandlerManager.release(token) },
                            ),
                            token.handler
                        )
                    }
                    catch (e: Throwable) {
                        try {
                            if (token != null) {
                                ThreadHandlerManager.release(token)
                            }
                        }
                        catch (e: Throwable) {
                            Logger.error(LOG_TAG_FOR_CUSTOM_FILE_PROVIDER, "openFile: Unexpected error in ThreadHandlerManager.release", e)
                        }

                        try {
                            if (fileId != null) {
                                CustomFileBridge.close(fileId)
                            }
                        }
                        catch (e: Throwable) {
                            Logger.error(LOG_TAG_FOR_CUSTOM_FILE_PROVIDER, "openFile: Unexpected error in CustomFileBridge.close", e)
                        }

                        throw e
                    }
                }
                else -> throw IllegalStateException("invalid file type: $fileType")
            }
        }
        catch (e: Throwable) {
            when (e) {
                is FileNotFoundException -> {}
                is IOException -> Logger.error(LOG_TAG_FOR_CUSTOM_FILE_PROVIDER, "openFile: IOException: ${e.message}", null)
                is UnsupportedOperationException -> Logger.error(LOG_TAG_FOR_CUSTOM_FILE_PROVIDER, "openFile: UnsupportedOperationException: ${e.message}", null)
                else -> Logger.error(LOG_TAG_FOR_CUSTOM_FILE_PROVIDER, "openFile: Unexpected error", e)
            }

            throw FileNotFoundException("File unavailable")
        }
    }

    override fun delete(uri: Uri, selection: String?, selectionArgs: Array<out String>?): Int {
        return try {
            CustomFileProviderBridge.delete(uri.toString())
            1
        }
        catch (e: Throwable) {
            when (e) {
                is UnsupportedOperationException -> {}
                is FileNotFoundException -> {}
                is IOException -> Logger.error(LOG_TAG_FOR_CUSTOM_FILE_PROVIDER, "delete: IOException: ${e.message}", null)
                else -> Logger.error(LOG_TAG_FOR_CUSTOM_FILE_PROVIDER, "delete: Unexpected error", e)
            }

            0
        }
    }

    override fun getType(uri: Uri): String? {
        return try {
            CustomFileProviderBridge.getMimeType(uri.toString()) ?: "application/octet-stream"
        }
        catch (e: Throwable) {
            when (e) {
                is FileNotFoundException -> {}
                is IOException -> Logger.error(LOG_TAG_FOR_CUSTOM_FILE_PROVIDER, "getType: IOException: ${e.message}", null)
                else -> Logger.error(LOG_TAG_FOR_CUSTOM_FILE_PROVIDER, "getType: Unexpected error", e)
            }

            null
        }
    }

    override fun getTypeAnonymous(uri: Uri): String {
        return "application/octet-stream"
    }

    override fun query(
        uri: Uri,
        projection: Array<out String>?,
        selection: String?,
        selectionArgs: Array<out String>?,
        sortOrder: String?
    ): Cursor {

        fun<T> getOrNull(targetValueName: String, get: () -> T?): T? {
            return try {
                get()
            }
            catch (e: FileNotFoundException) {
                throw e
            }
            catch (e: IOException) {
                Logger.error(LOG_TAG_FOR_CUSTOM_FILE_PROVIDER, "query: Failed to get $targetValueName: ${e.message}", null)
                null
            }
        }

        val requestedColumns = projection ?: arrayOf(
            OpenableColumns.DISPLAY_NAME,
            OpenableColumns.SIZE,
            MediaStore.MediaColumns.MIME_TYPE,
            MediaStore.MediaColumns.DATE_MODIFIED,
            DocumentsContract.Document.COLUMN_LAST_MODIFIED
        )

        val columns = requestedColumns.filter {
            when (it) {
                OpenableColumns.DISPLAY_NAME,
                OpenableColumns.SIZE,
                MediaStore.MediaColumns.MIME_TYPE,
                MediaStore.MediaColumns.DATE_MODIFIED,
                DocumentsContract.Document.COLUMN_LAST_MODIFIED -> true
                else -> false
            }
        }.toTypedArray()

        try {
            val uriStr = uri.toString()
            val mimeType: String by lazy { getOrNull("mime type") { CustomFileProviderBridge.getMimeType(uriStr) } ?: "application/octet-stream" }
            val name: String by lazy {
                getOrNull("name") { CustomFileProviderBridge.getName(uriStr) }
                    ?: uri.lastPathSegment
                    ?: Uri.decode(uriStr).split("/").last()
            }
            val len: Long? by lazy { getOrNull("len") { CustomFileProviderBridge.getLen(uriStr).takeIf { 0L <= it } } }
            val lastModifiedMs: Long? by lazy { getOrNull("last modified") { CustomFileProviderBridge.getLastModified(uriStr).takeIf { 0L <= it } } }

            val cursor = MatrixCursor(columns)

            cursor.addRow(
                columns.map {
                    when (it) {
                        OpenableColumns.DISPLAY_NAME -> name
                        OpenableColumns.SIZE -> len
                        MediaStore.MediaColumns.MIME_TYPE -> mimeType
                        MediaStore.MediaColumns.DATE_MODIFIED -> lastModifiedMs?.div(1000L)
                        DocumentsContract.Document.COLUMN_LAST_MODIFIED -> lastModifiedMs
                        else -> null
                    }
                }.toTypedArray()
            )

            return cursor
        }
        catch (e: Throwable) {
            when (e) {
                is FileNotFoundException -> {}
                is IOException -> Logger.error(LOG_TAG_FOR_CUSTOM_FILE_PROVIDER, "query: IOException: ${e.message}", null)
                else -> Logger.error(LOG_TAG_FOR_CUSTOM_FILE_PROVIDER, "query: Unexpected error", e)
            }

            return MatrixCursor(columns)
        }
    }


    override fun update(
        uri: Uri,
        values: ContentValues?,
        selection: String?,
        selectionArgs: Array<out String>?
    ): Int {

        throw UnsupportedOperationException()
    }

    override fun insert(uri: Uri, values: ContentValues?): Uri? {
        throw UnsupportedOperationException();
    }
}

@RequiresApi(Build.VERSION_CODES.O)
private class AFCustomFile(
    private val fileId: Int,
    private val read: Boolean,
    private val write: Boolean,
    private val onRelease: (() -> Unit)?
): ProxyFileDescriptorCallback() {

    override fun onFsync() {
        try {
            if (write) {
                CustomFileBridge.fsync(fileId)
            }
        }
        catch (e: CustomFileErrnoException) {
            Logger.error(LOG_TAG_FOR_CUSTOM_FILE_CALLBACK, "onFsync: CustomFileErrnoException: ${e.message}", null)
            throw ErrnoException("onFsync", e.errno)
        }
        catch (e: Throwable) {
            Logger.error(LOG_TAG_FOR_CUSTOM_FILE_CALLBACK, "onFsync: Unexpected error", e)
            throw ErrnoException("onFsync", OsConstants.EIO)
        }
    }

    override fun onGetSize(): Long {
        try {
            val size = CustomFileBridge.len(fileId)
            if (size < 0) {
                throw IllegalStateException("invalid negative size: $size")
            }

            return size
        }
        catch (e: CustomFileErrnoException) {
            Logger.error(LOG_TAG_FOR_CUSTOM_FILE_CALLBACK, "onGetSize: CustomFileErrnoException: ${e.message}", null)
            throw ErrnoException("onGetSize", e.errno)
        }
        catch (e: Throwable) {
            Logger.error(LOG_TAG_FOR_CUSTOM_FILE_CALLBACK, "onGetSize: Unexpected error", e)
            throw ErrnoException("onGetSize", OsConstants.EIO)
        }
    }

    override fun onRead(offset: Long, size: Int, data: ByteArray): Int {
        try {
            if (!read) throw ErrnoException("onRead", OsConstants.EBADF)
            if (size == 0) return 0

            return CustomFileBridge.readAt(fileId, offset, size, data)
        }
        catch (e: ErrnoException) {
            throw e
        }
        catch (e: CustomFileErrnoException) {
            Logger.error(LOG_TAG_FOR_CUSTOM_FILE_CALLBACK, "onRead: CustomFileErrnoException: ${e.message}", null)
            throw ErrnoException("onRead", e.errno)
        }
        catch (e: Throwable) {
            Logger.error(LOG_TAG_FOR_CUSTOM_FILE_CALLBACK, "onRead: Unexpected error", e)
            throw ErrnoException("onRead", OsConstants.EIO)
        }
    }

    override fun onWrite(offset: Long, size: Int, data: ByteArray): Int {
        try {
            if (!write) throw ErrnoException("onWrite", OsConstants.EBADF)
            if (size == 0) return 0

            return CustomFileBridge.writeAt(fileId, offset, size, data)
        }
        catch (e: ErrnoException) {
            throw e
        }
        catch (e: CustomFileErrnoException) {
            Logger.error(LOG_TAG_FOR_CUSTOM_FILE_CALLBACK, "onWrite: CustomFileErrnoException: ${e.message}", null)
            throw ErrnoException("onWrite", e.errno)
        }
        catch (e: Throwable) {
            Logger.error(LOG_TAG_FOR_CUSTOM_FILE_CALLBACK, "onWrite: Unexpected error", e)
            throw ErrnoException("onWrite", OsConstants.EIO)
        }
    }

    override fun onRelease() {
        try {
            CustomFileBridge.close(fileId)
        }
        catch (e: CustomFileErrnoException) {
            Logger.error(LOG_TAG_FOR_CUSTOM_FILE_CALLBACK, "onRelease: CustomFileErrnoException: ${e.message}", null)
            throw ErrnoException("onRelease", e.errno)
        }
        catch (e: Throwable) {
            Logger.error(LOG_TAG_FOR_CUSTOM_FILE_CALLBACK, "onRelease: Unexpected error", e)
            throw ErrnoException("onRelease", OsConstants.EIO)
        }
        finally {
            try {
                onRelease?.invoke()
            }
            catch (e: Exception) {
                Logger.error(LOG_TAG_FOR_CUSTOM_FILE_CALLBACK, "onRelease: Unexpected error in onRelease.invoke", e)
            }
        }
    }

}

private class ThreadToken(val handler: Handler, val slot: Int) {
    var released = false
}

private object ThreadHandlerManager {

    /** コールバックを同時実行できる最大スレッド数 */
    private const val POOL_SIZE = 4

    private val threads = arrayOfNulls<HandlerThread>(POOL_SIZE)
    private val handlers = arrayOfNulls<Handler>(POOL_SIZE)
    private val usages = IntArray(POOL_SIZE)  // 各スレッドに割り当て済みの未クローズファイル数
    private var openCount = 0

    /**
     * openProxyFileDescriptor の前に必ず呼ぶ。
     * 同じファイルのコールバックは必ず token.handler に集約される（ファイル単位で直列化）。
     */
    @Synchronized
    fun acquire(): ThreadToken {
        openCount++

        var leastLoaded = -1
        var deadSlot = -1
        for (i in 0 until POOL_SIZE) {
            val t = threads[i]
            if (t == null || !t.isAlive) {
                if (deadSlot == -1) deadSlot = i
                continue
            }
            if (usages[i] == 0) { // 空いてる確保済みスレッドを再利用
                usages[i] = 1
                return ThreadToken(handlers[i]!!, i)
            }
            if (leastLoaded == -1 || usages[i] < usages[leastLoaded]) leastLoaded = i
        }

        // 生存スレッドが全て使われていて、まだスロットが余っている場合はプールを拡張
        if (deadSlot != -1) {
            val t = HandlerThread("AFCustomFileThread-$deadSlot").apply { start() }
            threads[deadSlot] = t
            handlers[deadSlot] = Handler(t.looper)
            usages[deadSlot] = 1
            return ThreadToken(handlers[deadSlot]!!, deadSlot)
        }

        // プール満杯 → 最小負荷のスレッドを共有（オープン自体はブロックしない）
        usages[leastLoaded]++
        return ThreadToken(handlers[leastLoaded]!!, leastLoaded)
    }

    @Synchronized
    fun release(token: ThreadToken) {
        if (token.released) return
        token.released = true

        val slot = token.slot
        if (slot !in usages.indices) return
        if (usages[slot] > 0) usages[slot]--
        if (openCount > 0) openCount--

        if (openCount == 0) {
            threads.forEach { it?.quitSafely() }
            threads.fill(null)
            handlers.fill(null)
            usages.fill(0)
        }
    }
}