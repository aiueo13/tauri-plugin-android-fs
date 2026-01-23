package com.plugin.android_fs

import android.content.Context
import android.net.Uri
import android.provider.DocumentsContract
import android.provider.MediaStore
import android.webkit.MimeTypeMap
import androidx.core.database.getStringOrNull
import java.io.File

sealed class EntryType {
    data class File(val mimeType: String) : EntryType()
    object Dir : EntryType()
}

class AFUtils private constructor() { companion object {

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
        fileUri: FileUri,
        ctx: Context
    ): String {

        return when (val entry = getEntryType(fileUri, ctx)) {
            is EntryType.File -> entry.mimeType
            else -> throw Exception("not a file: ${fileUri.uri}")
        }
    }

    fun getEntryType(
        fileUri: FileUri,
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