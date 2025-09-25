package com.plugin.android_fs

import android.content.ContentValues
import android.content.Context
import android.net.Uri
import android.os.Build
import android.os.Environment
import android.provider.MediaStore
import androidx.annotation.RequiresApi
import app.tauri.plugin.JSObject
import java.io.File

// Q は Android 10
@RequiresApi(Build.VERSION_CODES.Q)
@Suppress("NAME_SHADOWING")
class AFMediaStore private constructor() { companion object {

    fun createNewFile(
        volumeName: String,
        relativePath: String,
        mimeType: String?,
        ctx: Context
    ): JSObject {

        val entry = File(relativePath)
        if (entry.isAbsolute) {
            throw IllegalArgumentException("absolute path is not supported")
        }

        val displayName = entry.name
        val parentRelativePath = entry.parent
        if (parentRelativePath.isNullOrEmpty()) {
            throw IllegalArgumentException("need parent directory")
        }

        val mimeType = mimeType ?: AFUtils.guessMimeTypeFromExtension(entry)

        val uri = ctx.contentResolver.insert(
            getContentUri(volumeName, relativePath, mimeType),
            ContentValues().apply {
                put(MediaStore.MediaColumns.DISPLAY_NAME, displayName)
                put(MediaStore.MediaColumns.MIME_TYPE, mimeType)
                put(MediaStore.MediaColumns.RELATIVE_PATH, "$parentRelativePath/")
            }
        ) ?: throw Exception("Failed to create file")

        return AFJSObject.createFileUri(uri)
    }
}}

// Q は Android 10
@RequiresApi(Build.VERSION_CODES.Q)
fun getContentUri(volumeName: String, relativePath: String, mimeType: String): Uri {
    val topDir = relativePath.trimStart('/').split("/").firstOrNull() ?: ""

    // MediaStore.Images.Media.getContentUri(volumeName) などは対応するフォルダ ( Pictures, DCIM など) 用なので、
    // それ以外のフォルダを用いる場合は MediaStore.Downloads か MediaStore.Files の URI を用いる。
    if (Environment.DIRECTORY_DOWNLOADS == topDir) {
        return MediaStore.Downloads.getContentUri(volumeName)
    }
    if (Environment.DIRECTORY_DOCUMENTS == topDir) {
        return MediaStore.Files.getContentUri(volumeName)
    }

    // DCIM と Pictures フォルダが画像と動画の両方に対応しているのでフォルダからではなく MIME type から判定する
    if (mimeType.startsWith("image/")) {
        return MediaStore.Images.Media.getContentUri(volumeName)
    }
    if (mimeType.startsWith("video/")) {
        return MediaStore.Video.Media.getContentUri(volumeName)
    }
    if (mimeType.startsWith("audio/")) {
        return MediaStore.Audio.Media.getContentUri(volumeName)
    }

    throw IllegalArgumentException(
        "The top-level directory '$topDir' is not a valid for ${mimeType}."
    )
}