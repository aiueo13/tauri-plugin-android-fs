package com.plugin.android_fs

import android.content.Context
import android.content.res.AssetFileDescriptor.UNKNOWN_LENGTH
import android.net.Uri
import android.os.Build
import android.provider.DocumentsContract
import android.provider.MediaStore
import android.provider.OpenableColumns
import android.webkit.MimeTypeMap
import androidx.core.database.getLongOrNull
import androidx.core.database.getStringOrNull
import java.io.File
import java.io.FileOutputStream
import java.io.OutputStream

sealed class EntryType {
    data class File(val mimeType: String) : EntryType()
    object Dir : EntryType()
}

class AFUtils private constructor() { companion object {

    fun openFileWt(
        uri: Uri,
        ctx: Context
    ): OutputStream {

        // Android 9 以下の場合、w は既存の内容を必ず切り捨てる
        if (Build.VERSION.SDK_INT <= Build.VERSION_CODES.P) {
            return ctx.contentResolver.openOutputStream(uri, "w")
                ?: throw Exception("Failed to open file with w mode")
        }

        // Android 10 以上の場合、w は既存の内容を切り捨てるとは限らない
        // しかし wt に対応していない file provider もあるため、
        // フォールバックを用いてなるべく多くの状況に対応する。
        // https://issuetracker.google.com/issues/180526528

        for (mode in listOf("wt", "rwt", "w")) {
            try {
                val o = ctx.contentResolver.openOutputStream(uri, mode)
                if (o != null) {
                    if (mode == "w") {
                        if (o is FileOutputStream) {
                            try {
                                o.channel.truncate(0)
                                return o
                            } catch (ignore: Exception) {
                                o.close()
                            }
                        }
                        o.close()
                    } else {
                        return o
                    }
                }
            } catch (ignore: Exception) {
            }
        }

        throw Exception("Failed to open file with truncate and write")
    }

    fun getFileLength(uri: Uri, ctx: Context): Long {
        if (uri.scheme == "content") {
            ctx.contentResolver.query(uri, arrayOf(OpenableColumns.SIZE), null, null, null, null).use {
                if (it?.moveToFirst() == true) {
                    val id = it.getColumnIndex(OpenableColumns.SIZE)
                    val size = it.getLongOrNull(id)

                    if (size != null && UNKNOWN_LENGTH != size) return size
                }
            }
        }

        ctx.contentResolver.openAssetFileDescriptor(uri, "r").use {
            val size = it?.length
            if (size != null && UNKNOWN_LENGTH != size) return size
        }

        throw Exception("no file or permission $uri")
    }

    fun getMimeTypeOrNullFromExtension(ext: String): String? {
        return MimeTypeMap
            .getSingleton()
            .getMimeTypeFromExtension(ext)
    }

    fun getMimeTypeFromExtension(ext: String): String {
        if (ext.isEmpty()) {
            return "application/octet-stream"
        }

        return MimeTypeMap
            .getSingleton()
            .getMimeTypeFromExtension(ext)
            ?: "application/octet-stream"
    }

    fun getMimeTypeFromName(fileName: String): String {
        val ext = fileName.substringAfterLast('.', "").lowercase()
        return getMimeTypeFromExtension(ext)
    }

    fun getExtensionFromMimeType(mimeType: String): String? {
        return MimeTypeMap
            .getSingleton()
            .getExtensionFromMimeType(mimeType)
    }

    fun guessFileMimeTypeFromExtension(file: File): String {
        return guessFileMimeTypeFromExtensionOrNull(file) ?: "application/octet-stream"
    }

    fun guessFileMimeTypeFromExtensionOrNull(file: File): String? {
        val ext = file.extension

        if (ext.isEmpty()) {
            return null
        }

        return MimeTypeMap
            .getSingleton()
            .getMimeTypeFromExtension(ext)
    }

    fun getFileMimeType(
        fileUri: AFUri,
        ctx: Context
    ): String {

        return when (val entry = getEntryType(fileUri, ctx)) {
            is EntryType.File -> entry.mimeType
            else -> throw Exception("not a file: ${fileUri.uri}")
        }
    }

    fun getEntryType(
        fileUri: AFUri,
        ctx: Context
    ): EntryType {

        val uri = Uri.parse(fileUri.uri)

        if (uri.scheme == "file") {
            val entry = File(uri.path!!)
            return when (entry.isDirectory) {
                true -> EntryType.Dir
                else -> EntryType.File(guessFileMimeTypeFromExtension(entry))
            }
        }

        val columnMimeType = when (true) {
            (fileUri.documentTopTreeUri != null || DocumentsContract.isDocumentUri(ctx, uri)) -> {
                DocumentsContract.Document.COLUMN_MIME_TYPE
            }
            else -> {
                MediaStore.Files.FileColumns.MIME_TYPE
            }
        }

        ctx.contentResolver.query(
            uri,
            arrayOf(columnMimeType),
            null,
            null,
            null
        )?.use {

            if (it.moveToFirst()) {
                val mimeType = it.getStringOrNull(it.getColumnIndexOrThrow(columnMimeType))

                return when (mimeType) {
                    DocumentsContract.Document.MIME_TYPE_DIR -> EntryType.Dir
                    else -> EntryType.File(mimeType ?: "application/octet-stream")
                }
            }
        }

        throw Exception("Failed to find entry: $uri")
    }
}}