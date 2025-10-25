package com.plugin.android_fs

import android.annotation.SuppressLint
import android.content.Context
import android.os.Build
import android.os.Environment
import android.os.storage.StorageManager
import android.os.storage.StorageVolume
import android.provider.MediaStore
import androidx.annotation.RequiresApi
import app.tauri.plugin.JSArray
import app.tauri.plugin.JSObject
import java.io.File
import java.util.Locale


// N は Android 7
@RequiresApi(Build.VERSION_CODES.N)
class AFStorageVolume private constructor() { companion object {

    fun getAvailableStorageVolumes(ctx: Context): JSArray {
        val sm = storageManager(ctx)
        val privateDirs = getPrivateDataAndCacheDirsWithStorageVolume(ctx, sm)
        val availableMediaStoreVolumeNames = when {
            Build.VERSION_CODES.Q <= Build.VERSION.SDK_INT -> MediaStore.getExternalVolumeNames(ctx)
            else -> emptySet()
        }

        val buffer = JSArray()
        for (storageVolume in sm.storageVolumes) {
            if (!isAvailable(storageVolume)) continue

            val topDirectoryPath = getTopDirectoryPath(storageVolume)
            val privateDirEntry = privateDirs[storageVolume]
            val privateDataDir = privateDirEntry?.first
            val privateCacheDir = privateDirEntry?.second

            val mediaStoreVolumeName = when {
                // Q は Android 10
                Build.VERSION_CODES.Q <= Build.VERSION.SDK_INT ->
                    getMediaStoreVolumeName(storageVolume, availableMediaStoreVolumeNames)

                else -> null
            }

            buffer.put(createStorageVolumeJSObject(
                storageVolume = storageVolume,
                topDirectoryPath = topDirectoryPath,
                mediaStoreVolumeName = mediaStoreVolumeName,
                privateDataDirPath = privateDataDir?.absolutePath,
                privateCacheDirPath = privateCacheDir?.absolutePath,
                context = ctx
            ))
        }

        return buffer
    }

    fun getPrimaryStorageVolumeIfAvailable(ctx: Context): JSObject? {
        val sm = storageManager(ctx)
        val storageVolume = sm.storageVolumes.find { it.isPrimary } ?: return null

        return createStorageVolumeJSObjectFromStorageVolumeIfAvailable(
            storageVolume,
            ctx,
            sm
        )
    }

    fun getStorageVolumeByFileIfAvailable(
        file: File,
        ctx: Context
    ): JSObject? {

        val sm = storageManager(ctx)
        val storageVolume = sm.getStorageVolume(file) ?: return null

        return createStorageVolumeJSObjectFromStorageVolumeIfAvailable(
            storageVolume,
            ctx,
            sm
        )
    }

    fun checkMediaStoreVolumeNameAvailable(
        mediaStoreVolumeName: String,
        ctx: Context
    ): Boolean {

        return when {
            // Q は Android 10
            Build.VERSION_CODES.Q <= Build.VERSION.SDK_INT ->
                MediaStore.getExternalVolumeNames(ctx).any { it == mediaStoreVolumeName }

            else -> false
        }
    }

    fun checkStorageVolumeAvailableByFile(
        entryInStorageVolume: File,
        ctx: Context
    ): Boolean {

        val sm = storageManager(ctx)
        val storageVolume = sm.getStorageVolume(entryInStorageVolume)
        return storageVolume != null && isAvailable(storageVolume)
    }
}}


// N は Android 7
@RequiresApi(Build.VERSION_CODES.N)
private fun createStorageVolumeJSObjectFromStorageVolumeIfAvailable(
    storageVolume: StorageVolume,
    ctx: Context,
    sm: StorageManager
): JSObject? {

    if (!isAvailable(storageVolume)) return null
    val topDirectoryPath = getTopDirectoryPath(storageVolume)
    val privateDataDir = getPrivateDataDir(storageVolume, ctx, sm)
    val privateCacheDir = getPrivateCacheDir(storageVolume, ctx, sm)
    val mediaStoreVolumeName = when {
        // Q は Android 10
        Build.VERSION_CODES.Q <= Build.VERSION.SDK_INT -> {
            val availableVolumeNames = MediaStore.getExternalVolumeNames(ctx)
            getMediaStoreVolumeName(storageVolume, availableVolumeNames)
        }

        else -> null
    }

    return createStorageVolumeJSObject(
        storageVolume = storageVolume,
        topDirectoryPath = topDirectoryPath,
        mediaStoreVolumeName = mediaStoreVolumeName,
        privateDataDirPath = privateDataDir?.absolutePath,
        privateCacheDirPath = privateCacheDir?.absolutePath,
        context = ctx
    )
}

// N は Android 7
@RequiresApi(Build.VERSION_CODES.N)
private fun createStorageVolumeJSObject(
    storageVolume: StorageVolume,
    topDirectoryPath: String?,
    mediaStoreVolumeName: String?,
    privateDataDirPath: String?,
    privateCacheDirPath: String?,
    context: Context
): JSObject {

    // アプリ専用フォルダはシステムに不安定と判断された StorageVolume に存在しない
    val isStable = privateDataDirPath != null || privateCacheDirPath != null

    val isAvailableForPublicStorage = when {

        // Q は Android 10
        Build.VERSION_CODES.Q <= Build.VERSION.SDK_INT -> mediaStoreVolumeName != null

        // Android 9 以下の場合、primary storage volume 以外の操作は SAF でしか行えない
        else -> storageVolume.isPrimary
    }

    return JSObject().apply {
        put("id", JSObject().apply {
            put("topDirectoryPath", topDirectoryPath)
            put("privateDataDirPath", privateDataDirPath)
            put("privateCacheDirPath", privateCacheDirPath)
            put("uuid", storageVolume.uuid)
            put("mediaStoreVolumeName", mediaStoreVolumeName)
        })
        put("description", storageVolume.getDescription(context))
        put("isPrimary", storageVolume.isPrimary)
        put("isRemovable", storageVolume.isRemovable)
        put("isStable", isStable)
        put("isEmulated", storageVolume.isEmulated)
        put("isReadonly", storageVolume.state == Environment.MEDIA_MOUNTED_READ_ONLY)
        put("isAvailableForPublicStorage", isAvailableForPublicStorage)
        put("isAvailableForPrivateStorage", isStable)
    }
}


/**
 * Note:
 * アプリ固有ディレクトリは現在有効な StorageVolume でも存在するとは限らない。
 * USB フラッシュドライバーのような「一時的なデバイス」の場合、アプリ固有ディレクトリは存在しない。
 */
// N は Android 7
@RequiresApi(Build.VERSION_CODES.N)
private fun getPrivateDataAndCacheDirsWithStorageVolume(
    ctx: Context,
    sm: StorageManager
): Map<StorageVolume, Pair<File?, File?>> {

    val entries = mutableMapOf<StorageVolume, Pair<File?, File?>>()

    for (dataDir in ctx.getExternalFilesDirs(null).filterNotNull()) {
        val sv = sm.getStorageVolume(dataDir)
        if (sv != null) {
            entries[sv] = (entries[sv] ?: Pair(null, null)).copy(first = dataDir)
        }
    }
    for (cacheDir in ctx.externalCacheDirs.filterNotNull()) {
        val sv = sm.getStorageVolume(cacheDir)
        if (sv != null) {
            entries[sv] = (entries[sv] ?: Pair(null, null)).copy(second = cacheDir)
        }
    }

    return entries
}

/**
 * Note:
 * アプリ固有ディレクトリは現在有効な StorageVolume でも存在するとは限らない。
 * USB フラッシュドライバーのような「一時的なデバイス」の場合、アプリ固有ディレクトリは存在しない。
 */
// N は Android 7
@RequiresApi(Build.VERSION_CODES.N)
private fun getPrivateDataDir(
    storageVolume: StorageVolume,
    ctx: Context,
    sm: StorageManager,
): File? {

    if (storageVolume.isPrimary) {
        return ctx.getExternalFilesDir(null)
    }

    return ctx.getExternalFilesDirs(null)
        .filterNotNull()
        .find { sm.getStorageVolume(it) == storageVolume }
}

/**
 * Note:
 * アプリ固有ディレクトリは現在有効な StorageVolume でも存在するとは限らない。
 * USB フラッシュドライバーのような「一時的なデバイス」の場合、アプリ固有ディレクトリは存在しない。
 */
// N は Android 7
@RequiresApi(Build.VERSION_CODES.N)
private fun getPrivateCacheDir(
    storageVolume: StorageVolume,
    ctx: Context,
    sm: StorageManager
): File? {

    if (storageVolume.isPrimary) {
        return ctx.externalCacheDir
    }

    return ctx.externalCacheDirs
        .filterNotNull()
        .find { sm.getStorageVolume(it) == storageVolume }
}

// N は Android 7
@RequiresApi(Build.VERSION_CODES.N)
private fun isAvailable(sv: StorageVolume): Boolean {
    // これは StorageVolume.getDirectory で使われる判定処理と同じである
    // https://android.googlesource.com/platform/frameworks/base/+/HEAD/core/java/android/os/storage/StorageVolume.java

    return when (sv.state) {
        Environment.MEDIA_MOUNTED,
        Environment.MEDIA_MOUNTED_READ_ONLY -> true

        else -> false
    }
}

// N は Android 7
@RequiresApi(Build.VERSION_CODES.N)
private fun getTopDirectoryPath(sv: StorageVolume): String? {
    // この関数内で使用する StorageVolume.getDirectory は現在有効でない場合に null を返すのでこの動作に統一する
    if (!isAvailable(sv)) {
        return null
    }

    // Q は Android 10
    if (Build.VERSION.SDK_INT <= Build.VERSION_CODES.Q) {
        if (sv.isPrimary) {
            return Environment.getExternalStorageDirectory().absolutePath
        }

        return try {
            // https://qiita.com/wa2c/items/4b3bacfec9667a5a99d7
            // https://android.googlesource.com/platform/frameworks/base/+/HEAD/core/java/android/os/storage/StorageVolume.java
            @SuppressLint("PrivateApi")
            val getPath = StorageVolume::class.java.getDeclaredMethod("getPath")

            getPath.invoke(sv) as String?
        }
        catch (_: Exception) {
            null
        }
    }

    return sv.directory?.absolutePath
}

// Q は Android 10
@RequiresApi(Build.VERSION_CODES.Q)
private fun getMediaStoreVolumeName(
    sv: StorageVolume,
    availableMediaStoreVolumeNames: Set<String>
): String? {

    val volumeName: String = when {
        sv.isPrimary -> MediaStore.VOLUME_EXTERNAL_PRIMARY

        // R は Android 11
        Build.VERSION_CODES.R <= Build.VERSION.SDK_INT -> sv.mediaStoreVolumeName ?: return null

        // https://android.googlesource.com/platform/frameworks/base/+/HEAD/core/java/android/os/storage/StorageVolume.java
        // の getMediaStoreVolumeName の実装をそのまま使用
        else -> sv.uuid?.lowercase(Locale.US) ?: return null
    }

    return availableMediaStoreVolumeNames.find { it == volumeName }
}

private fun storageManager(ctx: Context): StorageManager {
    return ctx.getSystemService(Context.STORAGE_SERVICE) as StorageManager
}