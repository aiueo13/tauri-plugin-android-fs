package com.plugin.android_fs

import android.content.ContentValues
import android.content.Context
import android.media.MediaScannerConnection
import android.net.Uri
import android.os.Build
import android.os.Environment
import android.provider.MediaStore
import androidx.annotation.RequiresApi
import app.tauri.plugin.JSObject
import java.io.File
import java.io.FileNotFoundException

// Q は Android 10
@RequiresApi(Build.VERSION_CODES.Q)
@Suppress("NAME_SHADOWING")
class AFMediaStore private constructor() { companion object {

    fun createNewFile(
        volumeName: String,
        relativePath: String,
        mimeType: String?,
        isPending: Boolean,
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

        val mimeType = mimeType ?: AFUtils.guessFileMimeTypeFromExtension(entry)

        val uri = ctx.contentResolver.insert(
            getContentUri(volumeName, relativePath, mimeType),
            ContentValues().apply {
                put(MediaStore.MediaColumns.DISPLAY_NAME, displayName)
                put(MediaStore.MediaColumns.MIME_TYPE, mimeType)
                put(MediaStore.MediaColumns.RELATIVE_PATH, "$parentRelativePath/")
                if (isPending) {
                    put(MediaStore.MediaColumns.IS_PENDING, 1)
                }
            }
        ) ?: throw Exception("Failed to create file")

        return AFJSObject.createFileUri(uri)
    }

    fun setPending(
        fileUri: FileUri,
        isPending: Boolean,
        ctx: Context
    ) {

        val uri = Uri.parse(fileUri.uri)
        val pending = if (isPending) { 1 } else { 0 }

        val updated = ctx.contentResolver.update(
            uri,
            ContentValues().apply {
                put(MediaStore.MediaColumns.IS_PENDING, pending)
            },
            null,
            null
        )

        if (updated < 1) {
            val p = arrayOf(MediaStore.MediaColumns.IS_PENDING)
            ctx.contentResolver.query(uri, p, null, null)?.use {
                val ci = it.getColumnIndexOrThrow(MediaStore.MediaColumns.IS_PENDING)
                if (it.getInt(ci) == pending) {
                    return
                }
            }

            throw Exception("No file or permission: ${fileUri.uri}")
        }
    }

    fun getRelativePath(
        fileUri: FileUri,
        ctx: Context
    ): String {

        val uri = Uri.parse(fileUri.uri)

        val projection = arrayOf(
            MediaStore.MediaColumns.RELATIVE_PATH,
            MediaStore.MediaColumns.DISPLAY_NAME,
        )
        ctx.contentResolver.query(uri, projection, null, null, null)?.use {
            if (it.moveToFirst()) {
                val dirRelativePathCi= it.getColumnIndexOrThrow(MediaStore.MediaColumns.RELATIVE_PATH)
                val nameCi = it.getColumnIndexOrThrow(MediaStore.MediaColumns.DISPLAY_NAME)
                val dirRelativePath = it.getString(dirRelativePathCi).trimEnd('/')
                val name = it.getString(nameCi)

                return "$dirRelativePath/$name"
            }
        }

        throw FileNotFoundException("Failed to find file: ${fileUri.uri}")
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

    return MediaStore.Files.getContentUri(volumeName)
}