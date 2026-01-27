package com.plugin.android_fs

import app.tauri.plugin.JSArray
import app.tauri.plugin.JSObject

interface FileController {

    fun getMimeType(uri: AFUri): String?

    fun getName(uri: AFUri): String

    fun getLen(uri: AFUri): Long

    fun readDir(dirUri: AFUri, options: ReadDirEntryOptions): JSArray

    fun getMetadata(uri: AFUri): JSObject

    fun createFile(dirUri: AFUri, relativePath: String, mimeType: String): JSObject

    fun createFileAndReturnRelativePath(dirUri: AFUri, relativePath: String, mimeType: String): JSObject

    fun createDirAll(dirUri: AFUri, relativePath: String): JSObject

    fun createDirAllAndReturnRelativePath(dirUri: AFUri, relativePath: String): JSObject

    fun deleteFile(uri: AFUri)

    fun deleteEmptyDir(uri: AFUri)

    fun deleteDirAll(uri: AFUri)

    fun rename(uri: AFUri, newName: String): JSObject
}