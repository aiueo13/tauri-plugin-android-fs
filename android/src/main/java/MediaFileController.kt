package com.plugin.android_fs

import android.app.Activity;
import android.content.ContentValues
import android.net.Uri
import android.provider.MediaStore
import androidx.core.database.getStringOrNull
import android.graphics.Bitmap
import android.graphics.ImageDecoder
import android.media.MediaMetadataRetriever
import android.media.MediaMetadataRetriever.OPTION_PREVIOUS_SYNC
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

    @Suppress("NAME_SHADOWING")
    override fun createFile(dirUri: FileUri, relativePath: String, mimeType: String): JSObject {
        val relativePath = relativePath.trimStart('/')
        val relativeDirPath = relativePath.substringBeforeLast("/", "")
        val fileName = relativePath.substringAfterLast("/", relativePath)

        val content = ContentValues().apply {
            val columns = getColumns(mimeType)

            put(columns.displayName, fileName)
            put(columns.mimeType, mimeType)
            if (relativeDirPath.isNotEmpty()) {
                put(columns.relativePath, "$relativeDirPath/")
            }
        }

        val uri = activity.contentResolver.insert(Uri.parse(dirUri.uri), content)
            ?: throw Exception("Failed to create file")

        val res = JSObject()
        res.put("uri", uri)
        res.put("documentTopTreeUri", null)
        return res
    }

    override fun deleteFile(uri: FileUri) {
        if (activity.contentResolver.delete(Uri.parse(uri.uri), null, null) <= 0) {
            throw Exception("Failed to delete file: ${uri.uri}")
        }
    }

    override fun deleteEmptyDir(uri: FileUri) {
        throw Exception("Unsupported operation for ${uri.uri}")
    }

    override fun deleteDirAll(uri: FileUri) {
        throw Exception("Unsupported operation for ${uri.uri}")
    }

    override fun readDir(dirUri: FileUri): JSArray {
        throw Exception("Unsupported or not dir: ${dirUri.uri}")
    }

    override fun getThumbnail(uri: FileUri, width: Int, height: Int): Bitmap? {
        try {
            return activity.contentResolver.loadThumbnail(
                Uri.parse(uri.uri), 
                Size(width, height), 
                null
            )
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
    

    private data class Columns(
        val displayName: String,
        val mimeType: String,
        val relativePath: String
    )

    private fun getColumns(mimeType: String): Columns {
        return when {
            mimeType.startsWith("image/") -> Columns(
                displayName = MediaStore.Images.Media.DISPLAY_NAME,
                mimeType = MediaStore.Images.Media.MIME_TYPE,
                relativePath = MediaStore.Images.Media.RELATIVE_PATH
            )
            mimeType.startsWith("video/") -> Columns(
                displayName = MediaStore.Video.Media.DISPLAY_NAME,
                mimeType = MediaStore.Video.Media.MIME_TYPE,
                relativePath = MediaStore.Video.Media.RELATIVE_PATH
            )
            mimeType.startsWith("audio/") -> Columns(
                displayName = MediaStore.Audio.Media.DISPLAY_NAME,
                mimeType = MediaStore.Audio.Media.MIME_TYPE,
                relativePath = MediaStore.Audio.Media.RELATIVE_PATH
            )
            else -> Columns(
                displayName = MediaStore.Files.FileColumns.DISPLAY_NAME,
                mimeType = MediaStore.Files.FileColumns.MIME_TYPE,
                relativePath = MediaStore.Files.FileColumns.RELATIVE_PATH
            )
        }
    }
}
