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
        return AFMediaStore.getMimeType(uri, activity)
    }

    override fun getName(uri: FileUri): String {
        return AFMediaStore.getDisplayName(uri, activity)
    }

    override fun deleteFile(uri: FileUri) {
        AFMediaStore.delete(uri, activity)
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
