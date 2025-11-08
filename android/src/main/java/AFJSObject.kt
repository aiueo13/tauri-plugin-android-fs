package com.plugin.android_fs

import android.content.Context
import android.net.Uri
import android.os.Build
import android.os.Environment
import android.provider.DocumentsContract
import androidx.annotation.RequiresApi
import app.tauri.plugin.JSObject

class AFJSObject private constructor() { companion object {

    fun createFileUri(uri: String, documentTopTreeUri: String?): JSObject {
        return JSObject().apply {
            put("uri", uri)
            put("documentTopTreeUri", documentTopTreeUri)
        }
    }

    fun createFileUri(uri: Uri): JSObject {
        return createFileUri(uri.toString(), null)
    }

    fun createFileUri(uri: String, documentTopTreeUri: Uri): JSObject {
        return createFileUri(uri, documentTopTreeUri.toString())
    }

    fun createFileUri(uri: Uri, documentTopTreeUri: String): JSObject {
        return createFileUri(uri.toString(), documentTopTreeUri)
    }

    fun createFileUri(uri: Uri, documentTopTreeUri: Uri): JSObject {
        return createFileUri(uri.toString(), documentTopTreeUri.toString())
    }

    @RequiresApi(Build.VERSION_CODES.N)
    fun createStorageVolumeJSObject(sv: AFStorageVolume.StorageVolume): JSObject {
        // アプリ専用フォルダはシステムに不安定と判断された StorageVolume に存在しない
        val isStable = sv.externalFilesDir != null || sv.externalCacheDir != null || sv.externalMediaDir != null
        
        val isAvailableForPublicStorage = when {

            // Q は Android 10
            Build.VERSION_CODES.Q <= Build.VERSION.SDK_INT -> sv.mediaStoreVolumeName != null

            // Android 9 以下の場合、secondary storage の操作は SAF でしか行えない
            else -> sv.isPrimary
        }

        return JSObject().apply {
            put("id", JSObject().apply {
                put("topDirPath", sv.topDir?.absolutePath)
                put("mediaStoreVolumeName", sv.mediaStoreVolumeName)
                put("appDataDirPath", sv.externalFilesDir?.absolutePath)
                put("appCacheDirPath", sv.externalCacheDir?.absolutePath)
                put("appMediaDirPath", sv.externalMediaDir?.absolutePath)
                put("uid", sv.uuid)
                put("storageUuid", sv.storageUuid?.toString())
            })
            put("description", sv.description)
            put("isPrimary", sv.isPrimary)
            put("isRemovable", sv.isRemovable)
            put("isStable", isStable)
            put("isEmulated", sv.isEmulated)
            put("isReadonly", sv.state == Environment.MEDIA_MOUNTED_READ_ONLY)
            put("isAvailableForPublicStorage", isAvailableForPublicStorage)
            put("isAvailableForAppStorage", isStable)
        }
    }
}}