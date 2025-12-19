package com.plugin.android_fs

import android.annotation.SuppressLint
import android.content.Context
import android.os.Build
import android.os.Environment
import android.os.storage.StorageManager
import android.os.storage.StorageVolume
import android.provider.MediaStore
import java.io.File
import java.util.Locale
import java.util.UUID


class AFStorageVolume private constructor() {

    data class Metadata(
        val mediaStoreVolumeName: String?,
        val topDir: File?,
        val externalFilesDir: File?,
        val externalCacheDir: File?,
        val externalMediaDir: File?,
        val description: String,
        val storageUuid: UUID?,
        val uuid: String?,
        val isPrimary: Boolean,
        val isRemovable: Boolean,
        val isEmulated: Boolean,
        val state: String
    )

    companion object {

        fun getAvailableStorageVolumes(ctx: Context): List<Metadata> {
            val svc = StorageVolumeContext(ctx)
            val externalDirs = getExternalFilesCacheMediaDirsWithStorageVolume(svc)
            val buf = mutableListOf<Metadata>()

            for (sv in svc.storageVolumes) {
                if (!svc.checkAvailable(sv)) continue

                val externalDirEntry = externalDirs[sv]

                buf.add(
                    Metadata(
                        mediaStoreVolumeName = svc.getMediaStoreVolumeName(sv),
                        topDir = svc.getTopDir(sv),
                        externalFilesDir = externalDirEntry?.first,
                        externalCacheDir = externalDirEntry?.second,
                        externalMediaDir = externalDirEntry?.third,
                        storageUuid = svc.getStorageUuid(sv),
                        uuid = sv.uuid,
                        description = svc.getDescription(sv),
                        isPrimary = sv.isPrimary,
                        isRemovable = sv.isRemovable,
                        isEmulated = sv.isEmulated,
                        state = sv.state
                    )
                )
            }

            return buf
        }

        fun getPrimaryStorageVolumeIfAvailable(ctx: Context): Metadata? {
            val svc = StorageVolumeContext(ctx)
            val sv = svc.storageVolumes.find { it.isPrimary } ?: return null
            return createStorageVolumeFromRawIfAvailable(sv, svc)
        }

        fun getStorageVolumeByFileIfAvailable(file: File, ctx: Context): Metadata? {
            val svc = StorageVolumeContext(ctx)
            val sv = svc.storageManager.getStorageVolume(file) ?: return null
            return createStorageVolumeFromRawIfAvailable(sv, svc)
        }

        fun checkMediaStoreVolumeNameAvailable(
            mediaStoreVolumeName: String,
            ctx: Context
        ): Boolean {

            val svc = StorageVolumeContext(ctx)

            for (sv in svc.storageVolumes) {
                if (mediaStoreVolumeName == svc.getMediaStoreVolumeName(sv)) {
                    return svc.checkAvailable(sv)
                }
            }

            return false
        }

        fun checkStorageVolumeAvailableByFile(
            file: File,
            ctx: Context
        ): Boolean {

            val svc = StorageVolumeContext(ctx)
            val sv = svc.storageManager.getStorageVolume(file)
            return sv != null && svc.checkAvailable(sv)
        }

        fun getStorageVolumeTopDirsAndMediaStoreVolumeNames(ctx: Context): List<Pair<File, String>> {
            val svc = StorageVolumeContext(ctx)
            val buf = mutableListOf<Pair<File, String>>()
            for (vol in svc.storageVolumes) {
                val topDir = svc.getTopDir(vol) ?: continue
                val volumeName = svc.getMediaStoreVolumeName(vol) ?: continue
                buf.add(Pair(topDir, volumeName))
            }
            return buf.toList()
        }
    }
}

private fun createStorageVolumeFromRawIfAvailable(
    sv: StorageVolume,
    svc: StorageVolumeContext
): AFStorageVolume.Metadata? {

    if (!svc.checkAvailable(sv)) return null

    return AFStorageVolume.Metadata(
        mediaStoreVolumeName = svc.getMediaStoreVolumeName(sv),
        topDir = svc.getTopDir(sv),
        externalFilesDir = svc.getExternalFilesDir(sv),
        externalCacheDir = svc.getExternalCacheDir(sv),
        externalMediaDir = svc.getExternalMediaDir(sv),
        storageUuid = svc.getStorageUuid(sv),
        uuid = sv.uuid,
        description = svc.getDescription(sv),
        isPrimary = sv.isPrimary,
        isRemovable = sv.isRemovable,
        isEmulated = sv.isEmulated,
        state = sv.state
    )
}

private fun getExternalFilesCacheMediaDirsWithStorageVolume(
    svc: StorageVolumeContext
): Map<StorageVolume, Triple<File?, File?, File?>> {

    val entries = mutableMapOf<StorageVolume, Triple<File?, File?, File?>>()

    for (dataDir in svc.externalFilesDirs) {
        val sv = svc.storageManager.getStorageVolume(dataDir)
        if (sv != null) {
            entries[sv] = (entries[sv] ?: Triple(null, null, null)).copy(first = dataDir)
        }
    }
    for (cacheDir in svc.externalCacheDirs) {
        val sv = svc.storageManager.getStorageVolume(cacheDir)
        if (sv != null) {
            entries[sv] = (entries[sv] ?: Triple(null, null, null)).copy(second = cacheDir)
        }
    }
    for (mediaDir in svc.externalMediaDirs) {
        val sv = svc.storageManager.getStorageVolume(mediaDir)
        if (sv != null) {
            entries[sv] = (entries[sv] ?: Triple(null, null, null)).copy(third = mediaDir)
        }
    }

    return entries
}

private class StorageVolumeContext(val ctx: Context) {

    val storageManager: StorageManager by lazy {
        ctx.getSystemService(Context.STORAGE_SERVICE) as StorageManager
    }

    val storageVolumes: List<StorageVolume> by lazy {
        storageManager.storageVolumes
    }

    val mediaStoreVolumeNames: Set<String> by lazy {
        when {
            // 一部のデバイスでは getExternalVolumeNames ではなく getRecentExternalVolumeNames でないと
            // Primary Storage Volume 以外を検知できないバグがある。
            // https://forum.developer.samsung.com/t/usb-massive-storage-not-recognized-in-galaxy-s20-android-11/9758/6
            Build.VERSION_CODES.R <= Build.VERSION.SDK_INT->
                MediaStore.getRecentExternalVolumeNames(ctx)

            Build.VERSION_CODES.Q <= Build.VERSION.SDK_INT ->
                MediaStore.getExternalVolumeNames(ctx)

            else -> emptySet()
        }
    }

    val externalFilesDirs: List<File> by lazy {
        ctx.getExternalFilesDirs(null).filterNotNull()
    }

    val externalCacheDirs: List<File> by lazy {
        ctx.externalCacheDirs.filterNotNull()
    }

    val externalFilesDir: File? by lazy {
        ctx.getExternalFilesDir(null)
    }

    val externalCacheDir: File? by lazy {
        ctx.externalCacheDir
    }

    val externalMediaDirs: List<File> by lazy {
        try {
            @Suppress("DEPRECATION")
            ctx.externalMediaDirs.filterNotNull()
        }
        catch (_: Exception) {
            listOf()
        }
    }

    fun checkAvailable(sv: StorageVolume): Boolean {
        // これは StorageVolume.getDirectory で使われる判定処理と同じである
        // https://android.googlesource.com/platform/frameworks/base/+/HEAD/core/java/android/os/storage/StorageVolume.java

        return when (sv.state) {
            Environment.MEDIA_MOUNTED,
            Environment.MEDIA_MOUNTED_READ_ONLY -> true

            else -> false
        }
    }

    fun getExternalFilesDir(sv: StorageVolume): File? {
        if (sv.isPrimary) {
            return externalFilesDir
        }

        return externalFilesDirs.find { storageManager.getStorageVolume(it) == sv }
    }

    fun getExternalCacheDir(sv: StorageVolume): File? {
        if (sv.isPrimary) {
            return externalCacheDir
        }

        return externalCacheDirs.find { storageManager.getStorageVolume(it) == sv }
    }

    fun getExternalMediaDir(sv: StorageVolume): File? {
        return externalMediaDirs.find { storageManager.getStorageVolume(it) == sv }
    }

    fun getMediaStoreVolumeName(sv: StorageVolume): String? {
        // Q は Android 10
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.Q) {
            return null
        }

        val volumeName: String = when {
            sv.isPrimary -> MediaStore.VOLUME_EXTERNAL_PRIMARY

            // R は Android 11
            Build.VERSION_CODES.R <= Build.VERSION.SDK_INT -> sv.mediaStoreVolumeName ?: return null

            // https://android.googlesource.com/platform/frameworks/base/+/HEAD/core/java/android/os/storage/StorageVolume.java
            // の getMediaStoreVolumeName の実装をそのまま使用
            else -> sv.uuid?.lowercase(Locale.US) ?: return null
        }

        return mediaStoreVolumeNames.find { it == volumeName }
    }

    fun getTopDir(sv: StorageVolume): File? {
        // この関数内で使用する StorageVolume.getDirectory は現在有効でない場合に null を返すのでこの動作に統一する
        if (!checkAvailable(sv)) {
            return null
        }

        // Q は Android 10
        if (Build.VERSION.SDK_INT <= Build.VERSION_CODES.Q) {
            if (sv.isPrimary) {
                return Environment.getExternalStorageDirectory()
            }

            return try {
                // https://qiita.com/wa2c/items/4b3bacfec9667a5a99d7
                // https://android.googlesource.com/platform/frameworks/base/+/HEAD/core/java/android/os/storage/StorageVolume.java
                @SuppressLint("PrivateApi")
                val getPath = StorageVolume::class.java.getDeclaredMethod("getPath")
                val path = getPath.invoke(sv) as String?
                return path?.let { File(it) }
            }
            catch (_: Exception) {
                null
            }
        }

        return sv.directory
    }

    fun getStorageUuid(sv: StorageVolume): UUID? {
        return when {
            Build.VERSION_CODES.S <= Build.VERSION.SDK_INT -> sv.storageUuid
            else -> null
        }
    }

    fun getDescription(sv: StorageVolume): String {
        return sv.getDescription(ctx)
    }
}