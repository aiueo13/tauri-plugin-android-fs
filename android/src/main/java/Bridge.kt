package okayu.tauri.plugin.android.fs

import androidx.annotation.Keep
import java.io.IOException

@Keep
object CustomFileProviderBridge {

    @Keep
    @JvmStatic
    external fun getMimeType(uri: String): String?

    @Keep
    @JvmStatic
    external fun getName(uri: String): String?

    /** 不明なら -1 を返す */
    @Keep
    @JvmStatic
    external fun getLen(uri: String): Long

    /** 不明なら -1 を返す */
    @Keep
    @JvmStatic
    external fun getLastModified(uri: String): Long

    @Keep
    @JvmStatic
    external fun open(uri: String, mode: String): IntArray

    @Keep
    @JvmStatic
    external fun delete(uri: String)
}

@Keep
class CustomFileErrnoException(message: String, val errno: Int): IOException(message)

@Keep
object CustomFileBridge {

    @Keep
    @JvmStatic
    external fun close(fileId: Int)

    @Keep
    @JvmStatic
    external fun readAt(fileId: Int, offset: Long, len: Int, buf: ByteArray): Int

    @Keep
    @JvmStatic
    external fun writeAt(fileId: Int, offset: Long, len: Int, buf: ByteArray): Int

    /** 不明なら -1 を返す */
    @JvmStatic
    @Keep
    external fun len(fileId: Int): Long

    @Keep
    @JvmStatic
    external fun fsync(fileId: Int)
}