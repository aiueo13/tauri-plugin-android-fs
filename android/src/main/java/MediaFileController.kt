package com.plugin.android_fs

import android.app.Activity
import android.content.ContentValues
import android.net.Uri
import android.provider.MediaStore
import androidx.core.database.getStringOrNull
import android.graphics.Bitmap
import android.os.Build
import android.provider.DocumentsContract
import android.provider.MediaStore.PickerMediaColumns
import android.util.Size
import androidx.core.database.getLongOrNull
import app.tauri.plugin.JSArray
import app.tauri.plugin.JSObject

class MediaFileController(private val activity: Activity): FileController {

    // フォルダが指定されることは想定していない
    override fun getMimeType(uri: FileUri): String {
        return AFMediaStore.getMimeType(uri, activity)
    }

    override fun getName(uri: FileUri): String {
        return AFMediaStore.getDisplayName(uri, activity)
    }

    override fun deleteFile(uri: FileUri) {
        AFMediaStore.delete(uri, activity)
    }

    override fun getMetadata(uri: FileUri): JSObject {
        val cursor = activity.contentResolver.query(
            Uri.parse(uri.uri),
            arrayOf(
                MediaStore.MediaColumns.MIME_TYPE,
                MediaStore.MediaColumns.DISPLAY_NAME,
                MediaStore.MediaColumns.SIZE,
                MediaStore.MediaColumns.DATE_MODIFIED,
                MediaStore.MediaColumns.DATE_TAKEN
            ),
            null,
            null,
            null
        )

        cursor?.use {
            val mimeTypeColumnIndex = it.getColumnIndex(MediaStore.MediaColumns.MIME_TYPE)
            val nameColumnIndex = it.getColumnIndex(MediaStore.MediaColumns.DISPLAY_NAME)
            val lastModifiedColumnIndex = it.getColumnIndex(MediaStore.MediaColumns.DATE_MODIFIED)
            val dateTakenColumnIndex = it.getColumnIndex(MediaStore.MediaColumns.DATE_TAKEN)
            val sizeColumnIndex = it.getColumnIndex(MediaStore.MediaColumns.SIZE)

            while (it.moveToNext()) {
                val obj = JSObject()

                obj.put("uri", JSObject().apply {
                    put("uri", uri.uri)
                    put("documentTopTreeUri", uri.documentTopTreeUri)
                })
                obj.put("name", it.getString(nameColumnIndex))

                val lastModified = it.getLongOrNull(lastModifiedColumnIndex)
                    ?: it.getLongOrNull(dateTakenColumnIndex)

                obj.put("lastModified", lastModified ?: 0)

                val mimeType = it.getString(mimeTypeColumnIndex)
                obj.put("mimeType", mimeType)
                obj.put("len", it.getLong(sizeColumnIndex))

                return obj
            }
        }

        throw Exception("No permission or entry: $uri")
    }

    override fun rename(uri: FileUri, newName: String): JSObject {
        AFMediaStore.rename(uri, newName, activity)

        val res = JSObject()
        res.put("uri", uri.uri)
        res.put("documentTopTreeUri", uri.documentTopTreeUri)
        return res
    }

    override fun createFile(dirUri: FileUri, relativePath: String, mimeType: String): JSObject {
        throw Exception("Unsupported operation for ${dirUri.uri}")
    }

    override fun createFileAndReturnRelativePath(dirUri: FileUri, relativePath: String, mimeType: String): JSObject {
        throw Exception("Unsupported operation for ${dirUri.uri}")
    }

    override fun createDirAll(dirUri: FileUri, relativePath: String): JSObject {
        throw Exception("Unsupported operation for ${dirUri.uri}")
    }

    override fun createDirAllAndReturnRelativePath(dirUri: FileUri, relativePath: String): JSObject {
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
