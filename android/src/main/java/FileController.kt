package com.plugin.android_fs

import android.graphics.Bitmap
import app.tauri.plugin.JSArray
import app.tauri.plugin.JSObject

interface FileController {

    fun getMimeType(uri: FileUri): String?

    fun getName(uri: FileUri): String

    fun getLen(uri: FileUri): Long

    fun readDir(dirUri: FileUri, options: ReadDirEntryOptions): JSArray

    fun getMetadata(uri: FileUri): JSObject

    fun createFile(dirUri: FileUri, relativePath: String, mimeType: String): JSObject

    fun createFileAndReturnRelativePath(dirUri: FileUri, relativePath: String, mimeType: String): JSObject

    fun createDirAll(dirUri: FileUri, relativePath: String): JSObject

    fun createDirAllAndReturnRelativePath(dirUri: FileUri, relativePath: String): JSObject

    fun deleteFile(uri: FileUri)

    fun deleteEmptyDir(uri: FileUri)

    fun deleteDirAll(uri: FileUri)

    fun rename(uri: FileUri, newName: String): JSObject
}