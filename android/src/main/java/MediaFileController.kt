package com.plugin.android_fs

import android.app.Activity
import android.content.ContentValues
import android.net.Uri
import android.provider.MediaStore
import androidx.core.database.getStringOrNull
import android.graphics.Bitmap
import android.os.Build
import android.util.Size
import app.tauri.plugin.JSArray
import app.tauri.plugin.JSObject

class MediaFileController(private val activity: Activity): FileController {

    // フォルダが指定されることは想定していない
    override fun getMimeType(uri: FileUri): String {
        activity.contentResolver.query(
            Uri.parse(uri.uri),
            arrayOf(MediaStore.Files.FileColumns.MIME_TYPE),
            null,
            null,
            null
        )?.use {

            if (it.moveToFirst()) {
                return it.getStringOrNull(it.getColumnIndexOrThrow(MediaStore.Files.FileColumns.MIME_TYPE))
                    ?: "application/octet-stream"
            }
        }

        throw Exception("Failed to find entry: ${uri.uri}")
    }

    override fun getName(uri: FileUri): String {
        activity.contentResolver.query(
            Uri.parse(uri.uri),
            arrayOf(MediaStore.MediaColumns.DISPLAY_NAME),
            null,
            null,
            null
        )?.use {

            if (it.moveToFirst()) {
                return it.getString(it.getColumnIndexOrThrow(MediaStore.MediaColumns.DISPLAY_NAME))
            }
        }

        throw Exception("Failed to find entry: ${uri.uri}")
    }

    override fun deleteFile(uri: FileUri) {
        if (activity.contentResolver.delete(Uri.parse(uri.uri), null, null) <= 0) {
            throw Exception("Failed to delete file: ${uri.uri}")
        }
    }

    override fun getThumbnail(uri: FileUri, width: Int, height: Int): Bitmap? {
        try {
            if (Build.VERSION_CODES.Q <= Build.VERSION.SDK_INT) {
                return activity.contentResolver.loadThumbnail(
                    Uri.parse(uri.uri),
                    Size(width, height),
                    null
                )
            }
        }
        catch (ignore: Exception) {}

        return null
    }

    override fun rename(uri: FileUri, newName: String): JSObject {
        if (getName(uri) != newName) {
            val updated = activity.contentResolver.update(
                Uri.parse(uri.uri),
                ContentValues().apply {
                    put(MediaStore.MediaColumns.DISPLAY_NAME, newName)
                },
                null,
                null
            )

            if (updated == 0) {
                throw Exception("Failed to rename: ${uri.uri}")
            }
        }

        val res = JSObject()
        res.put("uri", uri.uri)
        res.put("documentTopTreeUri", uri.documentTopTreeUri)
        return res
    }



    override fun createFile(dirUri: FileUri, relativePath: String, mimeType: String): JSObject {
        throw Exception("Unsupported operation for ${dirUri.uri}")
    }

    override fun createDirAll(dirUri: FileUri, relativePath: String): JSObject {
        throw Exception("Unsupported operation for ${dirUri.uri}")
    }

    override fun deleteEmptyDir(uri: FileUri) {
        throw Exception("Unsupported operation for ${uri.uri}")
    }

    override fun deleteDirAll(uri: FileUri) {
        throw Exception("Unsupported operation for ${uri.uri}")
    }

    override fun readDir(dirUri: FileUri, options: ReadDirEntryOptions): JSArray {
        throw Exception("Unsupported operation for ${dirUri.uri}")
    }
}
