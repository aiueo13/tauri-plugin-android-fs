package com.plugin.android_fs

import android.content.Context
import android.net.Uri
import android.os.Build
import android.os.Handler
import android.os.HandlerThread
import android.os.ParcelFileDescriptor
import android.os.ProxyFileDescriptorCallback
import android.os.storage.StorageManager
import android.system.ErrnoException
import android.system.OsConstants
import androidx.annotation.RequiresApi
import java.io.OutputStream

class AFFileDescriptor private constructor() {

    companion object {

        @Suppress("Recycle")
        fun getPfd(uri: Uri, mode: String, ctx: Context): ParcelFileDescriptor {
            if (isWritableMode(mode) && needWriteViaOutputStream(uri)) {
                // O は Android 8
                if (Build.VERSION.SDK_INT < Build.VERSION_CODES.O) {
                    throw Exception("Unsupported mode: $mode")
                }
                if (isReadableMode(mode)) {
                    throw Exception("Unsupported mode: $mode")
                }

                val output = ctx.contentResolver.openAssetFileDescriptor(uri, mode) ?: throw Exception("Failed to open file: $uri")
                val outputLen = when (mode) {
                    "wt" -> 0
                    else -> output.length
                }

                val sm = ctx.getSystemService(Context.STORAGE_SERVICE) as StorageManager

                return sm.openProxyFileDescriptor(
                    ParcelFileDescriptor.MODE_WRITE_ONLY,
                    UnseekableWriteonlyFdBehavior(output.createOutputStream(), outputLen) {
                        HandlerManager.notifyTaskEnd()
                    },
                    HandlerManager.getHandlerAndNotifyTaskAdd()
                )
            }

            return ctx.contentResolver
                .openAssetFileDescriptor(uri, mode)
                ?.parcelFileDescriptor ?: throw Exception("Failed to open file: $uri")
        }
    }
}


private fun isWritableMode(mode: String): Boolean {
    // r, rw, w, wa, wt, rwt
    return mode != "r"
}

private fun isReadableMode(mode: String): Boolean {
    // r, rw, w, wa, wt, rwt
    return mode == "r" || mode == "rw" || mode == "rwt"
}

private fun needWriteViaOutputStream(uri: Uri): Boolean {
    // - https://issuetracker.google.com/issues/200201777
    // - https://stackoverflow.com/questions/51015513/fileoutputstream-writes-0-bytes-to-google-drive
    // - https://stackoverflow.com/questions/51490194/file-written-using-action-create-document-is-empty-on-google-drive-but-not-local
    // - https://community.latenode.com/t/csv-export-to-google-drive-results-in-empty-file-but-local-storage-works-fine
    //
    // Intent.ACTION_OPEN_DOCUMENT や Intent.ACTION_CREATE_DOCUMENT などの SAF で
    // 取得した Google Drive のファイルに対して生の FD を用いて書き込んだ場合、
    // それが反映されず空のファイルのみが残ることがある。
    // これの対処法として Context.openOutputStream から得た OutputStream で書き込んだ後
    // flush 関数を使うことで反映させることができる。
    // このプラグインでは Context.openAssetFileDescriptor から FD を取得して操作しているが
    // これはハック的な手法ではなく公式の doc でも SAF の例として用いられている手法であるため
    // この動作は仕様ではなく GoogleDrive 側のバグだと考えていいと思う。
    //
    // また Web を調べたが GoogleDrive 以外でこのような問題が起こるのは見つけれなかった。
    // 実際、試した限りでは DropBox で書き込んだものが普通に反映された。
    // もしかしたら他のクラウドストレージアプリでは起こるかもしれないが、
    // それは仕様ではなく FileProvider 側のバグ？だと思うのでこちら側ではコストを考え
    // ホワイトリスト方式ではなくブラックリスト方式を用いて判定する。

    val targetUriPrefixes = arrayOf(
        "content://com.google.android.apps.docs" // Google Drive
    )

    val uriString = uri.toString()

    return targetUriPrefixes.any { uriString.startsWith(it) }
}


class HandlerManager {
    companion object {
        private var handlerThread: HandlerThread? = null
        private var handler: Handler? = null
        private var taskCount = 0

        @Synchronized
        fun getHandlerAndNotifyTaskAdd(): Handler {
            taskCount++

            // 既に存在していて動作中なら再利用
            handlerThread?.let { thread ->
                val currentHandler = handler
                if (thread.isAlive && currentHandler != null) return currentHandler
            }

            // 新規起動
            handlerThread = HandlerThread("ProxyFDThread").apply { start() }
            handler = Handler(handlerThread!!.looper)
            return handler!!
        }

        @Synchronized
        fun notifyTaskEnd() {
            taskCount--
            if (taskCount <= 0) {
                handlerThread?.quitSafely()
                handlerThread = null
                handler = null
                taskCount = 0
            }
        }
    }
}

@RequiresApi(Build.VERSION_CODES.O)
private class UnseekableWriteonlyFdBehavior(
    private val dest: OutputStream,
    private val destInitLen: Long,
    private val onRelease: (() -> Unit)?
) : ProxyFileDescriptorCallback() {

    private var currentPosition: Long = 0

    override fun onRead(offset: Long, size: Int, data: ByteArray): Int {
        throw ErrnoException("read", OsConstants.EBADF)
    }

    override fun onWrite(offset: Long, size: Int, data: ByteArray?): Int {
        try {
            if (data == null) return 0

            // シーク不可
            if (offset != currentPosition) {
                throw ErrnoException("write", OsConstants.ESPIPE)
            }

            val writeSize = size.coerceAtMost(data.size)
            dest.write(data, 0, writeSize)
            currentPosition = offset + writeSize

            return writeSize
        }
        catch (e: ErrnoException) {
            throw e
        }
        catch (e: Exception) {
            throw ErrnoException("write", OsConstants.EIO, e)
        }
    }

    override fun onFsync() {
        try {
            dest.flush()
        }
        catch (e: Exception) {
            throw ErrnoException("fsync", OsConstants.EIO, e)
        }
    }

    override fun onRelease() {
        try {
            dest.flush()
        }
        catch (_: Exception) {}

        try {
            dest.close()
        }
        catch (_: Exception) {}

        try {
            onRelease?.invoke()
        }
        catch (_: Exception) {}
    }

    override fun onGetSize(): Long {
        if (destInitLen < 0) {
            return -1
        }

        return destInitLen + currentPosition
    }
}