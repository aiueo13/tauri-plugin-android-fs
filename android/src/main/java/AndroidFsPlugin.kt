package com.plugin.android_fs

import android.Manifest
import android.annotation.SuppressLint
import android.app.Activity
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Build
import android.os.Environment
import android.os.storage.StorageManager
import android.provider.DocumentsContract
import android.provider.MediaStore
import android.util.Base64
import android.util.Size
import android.webkit.MimeTypeMap
import androidx.activity.result.ActivityResult
import androidx.activity.result.PickVisualMediaRequest
import androidx.activity.result.contract.ActivityResultContracts.PickMultipleVisualMedia
import androidx.activity.result.contract.ActivityResultContracts.PickVisualMedia
import androidx.core.app.ShareCompat
import app.tauri.PermissionState
import app.tauri.annotation.ActivityCallback
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.Permission
import app.tauri.annotation.PermissionCallback
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSArray
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import java.io.ByteArrayOutputStream
import java.io.File
import java.io.FileOutputStream
import java.io.OutputStream
import kotlin.io.copyTo


@InvokeArg
class GetFileDescriptorArgs {
    lateinit var mode: String
    lateinit var uri: FileUri
}

@InvokeArg
class GetFileDescriptorWithFallbackArgs {
    lateinit var modes: Array<String>
    lateinit var uri: FileUri
}

@InvokeArg
class GetNameArgs {
    lateinit var uri: FileUri
}

@InvokeArg
class GetLenArgs {
    lateinit var uri: FileUri
}

@InvokeArg
class GetThumbnailToFileArgs {
    lateinit var src: FileUri
    lateinit var dest: FileUri
    var width: Int = -1
    var height: Int = -1
    var quality: Int = -1
    lateinit var format: String
}

@InvokeArg
class GetThumbnailArgs {
    lateinit var uri: FileUri
    var width: Int = -1
    var height: Int = -1
    var quality: Int = -1
    lateinit var format: String
}

@InvokeArg
class ShowOpenFileDialogArgs {
    lateinit var mimeTypes: Array<String>
    var multiple: Boolean = false
    var localOnly: Boolean = false
    var initialLocation: FileUri? = null
}

@InvokeArg
class ShowOpenContentDialogArgs {
    lateinit var mimeTypes: Array<String>
    var multiple: Boolean = false
}

@InvokeArg
class ShowOpenVisualMediaDialogArgs {
    lateinit var target: String
    var localOnly: Boolean = false
    var multiple: Boolean = false
}

@InvokeArg
class ShowManageDirDialogArgs {
    var initialLocation: FileUri? = null
    var localOnly: Boolean = false
}

@InvokeArg
class ShowSaveFileDialogArgs {
    var initialLocation: FileUri? = null
    lateinit var initialFileName: String
    var localOnly: Boolean = false
    var mimeType: String? = null
}

@InvokeArg
enum class PersistableUriPermissionMode {
    Read,
    Write,
    ReadAndWrite,
    ReadOrWrite,
}

@InvokeArg
class GetStorageVolumeByPathArgs {
    var path: String? = null
}

@InvokeArg
class CheckStorageVolumeAvailableByPathArgs {
    var path: String? = null
}

@InvokeArg
class CheckMediaStoreVolumeNameAvailableArgs {
    var mediaStoreVolumeName: String? = null
}

@InvokeArg
class GetMimeTypeArgs {
    lateinit var uri: FileUri
}

@InvokeArg
class GetMetadataArgs {
    lateinit var uri: FileUri
}

@InvokeArg
class DeleteArgs {
    lateinit var uri: FileUri
}

@InvokeArg
class RenameArgs {
    lateinit var uri: FileUri
    lateinit var newName: String
}

@InvokeArg
class MoveArgs {
    lateinit var uri: FileUri
    lateinit var destDir: FileUri
    var newName: String? = null
}

@InvokeArg
class ReadDirArgs {
    lateinit var uri: FileUri
    lateinit var options: ReadDirEntryOptions
}

@InvokeArg
class ReadDirEntryOptions(
    val uri: Boolean = false,
    val name: Boolean = false,
    val lastModified: Boolean= false,
    val len: Boolean = false,
)

@InvokeArg
class CreateFileArgs {
    lateinit var dir: FileUri
    lateinit var relativePath: String
    var mimeType: String? = null
}

@InvokeArg
class CreateFileAndReturnRelativePathArgs {
    lateinit var dir: FileUri
    lateinit var relativePath: String
    var mimeType: String? = null
}

@InvokeArg
class CreateDirAllArgs {
    lateinit var dir: FileUri
    lateinit var relativePath: String
}

@InvokeArg
class CreateDirAllAndReturnRelativePathArgs {
    lateinit var dir: FileUri
    lateinit var relativePath: String
}

@InvokeArg
class FileUri {
    lateinit var uri: String
    var documentTopTreeUri: String? = null
}

@InvokeArg
class TakePersistableUriPermissionArgs {
    lateinit var uri: FileUri
}

@InvokeArg
class CheckPersistedUriPermissionArgs {
    lateinit var uri: FileUri
    lateinit var mode: PersistableUriPermissionMode
}

@InvokeArg
class ReleasePersistedUriPermissionArgs {
    lateinit var uri: FileUri
}

@InvokeArg
class CopyFileArgs {
    lateinit var src: FileUri
    lateinit var dest: FileUri
    var bufferSize: Int? = null
}
@InvokeArg
class CreateNewMediaStoreFileArgs {
    var volumeName: String? = null
    lateinit var relativePath: String
    var mimeType: String? = null
    // isPending と命名するとなぜか常にnullになる
    var pending: Boolean? = null
}

@InvokeArg
class SetMediaStoreFilePending {
    lateinit var uri: FileUri
    // isPending と命名するとなぜか常にnullになる
    var pending: Boolean? = null
}

@InvokeArg
class ScanFileToMediaStoreByPathArgs {
    lateinit var path: String
    var mimeType: String? = null
}

@InvokeArg
class ScanMediaStoreFileArgs {
    lateinit var uri: FileUri
}

@InvokeArg
class GetMediaStoreFileAbsolutePathArgs {
    lateinit var uri: FileUri
}

@InvokeArg
class FindFileUriArgs {
    lateinit var parentUri: FileUri
    lateinit var relativePath: String
}

@InvokeArg
class FindDirUriArgs {
    lateinit var parentUri: FileUri
    lateinit var relativePath: String
}


private const val ALIAS_LEGACY_WRITE_STORAGE_PERMISSION = "WRITE_EXTERNAL_STORAGE_MAX"
private const val ALIAS_LEGACY_READ_STORAGE_PERMISSION = "READ_EXTERNAL_STORAGE_MAX"

@TauriPlugin(
    permissions = [
        Permission(
            strings = [Manifest.permission.WRITE_EXTERNAL_STORAGE],
            alias = ALIAS_LEGACY_WRITE_STORAGE_PERMISSION
        ),
        Permission(
            strings = [Manifest.permission.READ_EXTERNAL_STORAGE],
            alias = ALIAS_LEGACY_READ_STORAGE_PERMISSION
        ),
    ]
)
class AndroidFsPlugin(private val activity: Activity) : Plugin(activity) {
    private val documentFileController = DocumentFileController(activity)
    private val mediaFileController = MediaFileController(activity)
    private val rawFileController = RawFileController()

    @Suppress("NAME_SHADOWING")
    private fun getFileController(uri: FileUri): FileController {
        val documentTopTreeUri = uri.documentTopTreeUri
        val uri = Uri.parse(uri.uri)

        return when (true) {
            (documentTopTreeUri != null || DocumentsContract.isDocumentUri(activity, uri)) -> {
                documentFileController
            }
            (uri.scheme == "content") -> {
                mediaFileController
            }
            (uri.scheme == "file") -> {
                rawFileController
            }
            else -> throw Exception("Unsupported uri: $uri")
        }
    }

    private fun getMimeTypeFromName(fileName: String): String {
        val ext = fileName.substringAfterLast('.', "").lowercase()

        if (ext.isEmpty()) {
            return "application/octet-stream"
        }

        return MimeTypeMap
            .getSingleton()
            .getMimeTypeFromExtension(ext)
            ?: "application/octet-stream"
    }

    private fun openFileWt(uri: Uri): OutputStream {
        // Android 9 以下の場合、w は既存の内容を必ず切り捨てる
        if (Build.VERSION.SDK_INT <= Build.VERSION_CODES.P) {
            return activity.contentResolver.openOutputStream(uri, "w")
                ?: throw Exception("Failed to open file with w mode")
        }

        // Android 10 以上の場合、w は既存の内容を切り捨てるとは限らない
        // しかし wt に対応していない file provider もあるため、
        // フォールバックを用いてなるべく多くの状況に対応する。
        // https://issuetracker.google.com/issues/180526528

        for (mode in listOf("wt", "rwt", "w")) {
            try {
                val o = activity.contentResolver.openOutputStream(uri, mode)
                if (o != null) {
                    if (mode == "w") {
                        if (o is FileOutputStream) {
                            try {
                                o.channel.truncate(0)
                                return o
                            }
                            catch (ignore: Exception) {
                                o.close()
                            }
                        }
                        o.close()
                    }
                    else {
                        return o
                    }
                }
            }
            catch (ignore: Exception) { }
        }

        throw Exception("Failed to open file with truncate and write")
    }

    private fun tryIntoSafInitialLocation(initialLocation: FileUri): Uri? {
        val documentTopTreeUri = initialLocation.documentTopTreeUri
        val uri = Uri.parse(initialLocation.uri)

        if (
            documentTopTreeUri != null ||
            DocumentsContract.isDocumentUri(activity, uri) ||
            initialLocation.uri.startsWith("content://com.android.externalstorage.documents/root/")
            ) {

            return uri
        }

        return null
    }

    @Command
    fun getConsts(invoke: Invoke) {
        try {
            val res = JSObject()
            res.put("buildVersionSdkInt", Build.VERSION.SDK_INT)
            res.put("envDirPictures", Environment.DIRECTORY_PICTURES)
            res.put("envDirDcim", Environment.DIRECTORY_DCIM)
            res.put("envDirMovies", Environment.DIRECTORY_MOVIES)
            res.put("envDirMusic", Environment.DIRECTORY_MUSIC)
            res.put("envDirAlarms", Environment.DIRECTORY_ALARMS)
            res.put("envDirNotifications", Environment.DIRECTORY_NOTIFICATIONS)
            res.put("envDirPodcasts", Environment.DIRECTORY_PODCASTS)
            res.put("envDirRingtones", Environment.DIRECTORY_RINGTONES)
            res.put("envDirDocuments", Environment.DIRECTORY_DOCUMENTS)
            res.put("envDirDownload", Environment.DIRECTORY_DOWNLOADS)
            // S は Android 12
            if (Build.VERSION_CODES.S <= Build.VERSION.SDK_INT) {
                res.put("envDirRecordings", Environment.DIRECTORY_RECORDINGS)
            }
            // Q は Android 10
            if (Build.VERSION_CODES.Q <= Build.VERSION.SDK_INT) {
                res.put("envDirAudiobooks", Environment.DIRECTORY_AUDIOBOOKS)
                res.put("mediaStorePrimaryVolumeName", MediaStore.VOLUME_EXTERNAL_PRIMARY)
            }

            invoke.resolve(res)
        }
        catch (e: Exception) {
            invoke.reject(e.message ?: "unknown")
        }
    }

    @Command
    fun isLegacyStorage(invoke: Invoke) {
        try {
            val isLegacyStorage = when {
                // Q は Android 10
                Build.VERSION_CODES.Q < Build.VERSION.SDK_INT -> false
                Build.VERSION_CODES.Q == Build.VERSION.SDK_INT -> Environment.isExternalStorageLegacy()
                else -> true
            }

            invoke.resolve(JSObject().apply { put("value", isLegacyStorage) })
        }
        catch (e: Exception) {
            invoke.reject(e.message ?: "unknown")
        }
    }

    @Command
    fun requestLegacyStoragePermission(invoke: Invoke) {
        try {
            val writeGranted = when (getPermissionState(ALIAS_LEGACY_WRITE_STORAGE_PERMISSION)) {
                PermissionState.GRANTED -> true
                else -> false
            }
            val readGranted = when (getPermissionState(ALIAS_LEGACY_READ_STORAGE_PERMISSION)) {
                PermissionState.GRANTED -> true
                else -> false
            }

            val permissions = mutableListOf<String>()
            if (!writeGranted) {
                permissions.add(ALIAS_LEGACY_WRITE_STORAGE_PERMISSION)
            }
            if (!readGranted) {
                permissions.add(ALIAS_LEGACY_READ_STORAGE_PERMISSION)
            }

            if (permissions.isEmpty()) {
                invoke.resolve(JSObject().apply {
                    put("granted", true)
                    put("prompted", false)
                })
            }
            else {
                requestPermissionForAliases(
                    permissions.toTypedArray(),
                    invoke,
                    "handleRequestLegacyStoragePermission"
                )
            }
        }
        catch (e: Exception) {
            invoke.reject(e.message ?: "unknown")
        }
    }

    @PermissionCallback
    fun handleRequestLegacyStoragePermission(invoke: Invoke) {
        try {
            val writeGranted = when (getPermissionState(ALIAS_LEGACY_WRITE_STORAGE_PERMISSION)) {
                PermissionState.GRANTED -> true
                else -> false
            }
            val readGranted = when (getPermissionState(ALIAS_LEGACY_READ_STORAGE_PERMISSION)) {
                PermissionState.GRANTED -> true
                else -> false
            }

            invoke.resolve(JSObject().apply {
                put("granted", writeGranted && readGranted)
                put("prompted", true)
            })
        }
        catch (e: Exception) {
            invoke.reject(e.message ?: "unknown")
        }
    }

    @Command
    fun hasLegacyStoragePermission(invoke: Invoke) {
        try {
            val writeGranted = when (getPermissionState(ALIAS_LEGACY_WRITE_STORAGE_PERMISSION)) {
                PermissionState.GRANTED -> true
                else -> false
            }
            val readGranted = when (getPermissionState(ALIAS_LEGACY_READ_STORAGE_PERMISSION)) {
                PermissionState.GRANTED -> true
                else -> false
            }

            invoke.resolve(JSObject().apply {
                put("granted", writeGranted && readGranted)
            })
        }
        catch (e: Exception) {
            invoke.reject(e.message ?: "unknown")
        }
    }

    @Command
    fun findSafFileUri(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(FindFileUriArgs::class.java)

            CoroutineScope(Dispatchers.IO).launch {
                try {
                    val uri = documentFileController.findFileUri(args.parentUri, args.relativePath)
                    withContext(Dispatchers.Main) {
                        invoke.resolve(uri)
                    }
                }
                catch (ex: Exception) {
                    withContext(Dispatchers.Main) {
                        invoke.reject(ex.message ?: "unknown")
                    }
                }
            }
        }
        catch (e: Exception) {
            invoke.reject(e.message ?: "unknown")
        }
    }

    @Command
    fun findSafDirUri(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(FindDirUriArgs::class.java)

            CoroutineScope(Dispatchers.IO).launch {
                try {
                    val uri = documentFileController.findDirUri(args.parentUri, args.relativePath)
                    withContext(Dispatchers.Main) {
                        invoke.resolve(uri)
                    }
                }
                catch (ex: Exception) {
                    withContext(Dispatchers.Main) {
                        invoke.reject(ex.message ?: "unknown")
                    }
                }
            }
        }
        catch (e: Exception) {
            invoke.reject(e.message ?: "unknown")
        }
    }

    @Command
    fun scanFileToMediaStoreByPath(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(ScanFileToMediaStoreByPathArgs::class.java)

            CoroutineScope(Dispatchers.IO).launch {
                try {

                    AFMediaStore.scanFile(
                        File(args.path),
                        args.mimeType,
                        { uri -> activity.runOnUiThread { invoke.resolve(JSObject().apply { put("uri", AFJSObject.createFileUri(uri)) }) } },
                        { err -> activity.runOnUiThread { invoke.reject(err.message ?: "unknown") } },
                        activity
                    )
                }
                catch (ex: Exception) {
                    withContext(Dispatchers.Main) {
                        invoke.reject(ex.message ?: "unknown")
                    }
                }
            }
        }
        catch (e: Exception) {
            invoke.reject(e.message ?: "unknown")
        }
    }

    @Command
    fun scanMediaStoreFile(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(ScanMediaStoreFileArgs::class.java)

            CoroutineScope(Dispatchers.IO).launch {
                try {
                    val e = AFMediaStore.getAbsolutePathAndMimeType(args.uri, activity)
                    val path = e.first
                    val mimeType = e.second
                    
                    AFMediaStore.scanFileWithIgnoringResult(
                        File(path),
                        mimeType,
                        activity
                    )

                    withContext(Dispatchers.Main) {
                        invoke.resolve()
                    }
                }
                catch (ex: Exception) {
                    withContext(Dispatchers.Main) {
                        invoke.reject(ex.message ?: "unknown")
                    }
                }
            }
        }
        catch (e: Exception) {
            invoke.reject(e.message ?: "unknown")
        }
    }

    @Command
    fun scanMediaStoreFileForResult(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(ScanMediaStoreFileArgs::class.java)

            CoroutineScope(Dispatchers.IO).launch {
                try {
                    val e = AFMediaStore.getAbsolutePathAndMimeType(args.uri, activity)
                    val path = e.first
                    val mimeType = e.second

                    AFMediaStore.scanFile(
                        File(path),
                        mimeType,
                        { activity.runOnUiThread { invoke.resolve() } },
                        { err -> activity.runOnUiThread { invoke.reject(err.message ?: "unknown") } },
                        activity
                    )
                }
                catch (ex: Exception) {
                    withContext(Dispatchers.Main) {
                        invoke.reject(ex.message ?: "unknown")
                    }
                }
            }
        }
        catch (e: Exception) {
            invoke.reject(e.message ?: "unknown")
        }
    }

    @Command
    fun getMediaStoreFileAbsolutePath(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(GetMediaStoreFileAbsolutePathArgs::class.java)

            CoroutineScope(Dispatchers.IO).launch {
                try {
                    val path: String = AFMediaStore.getAbsolutePath(args.uri, activity)

                    withContext(Dispatchers.Main) {
                        invoke.resolve(JSObject().apply {
                            put("path", path)
                        })
                    }
                }
                catch (ex: Exception) {
                    withContext(Dispatchers.Main) {
                        invoke.reject(ex.message ?: "unknown")
                    }
                }
            }
        }
        catch (e: Exception) {
            invoke.reject(e.message ?: "unknown")
        }
    }

    @Command
    fun setMediaStoreFilePending(invoke: Invoke) {
        try {
            if (Build.VERSION.SDK_INT < Build.VERSION_CODES.Q) {
                throw Exception("requires Android 10 (API level 29) or higher")
            }

            CoroutineScope(Dispatchers.IO).launch {
                try {
                    val args = invoke.parseArgs(SetMediaStoreFilePending::class.java)

                    AFMediaStore.setPending(
                        args.uri,
                        args.pending ?: throw Exception("missing value: isPending"),
                        activity
                    )

                    withContext(Dispatchers.Main) {
                        invoke.resolve()
                    }
                }
                catch (ex: Exception) {
                    withContext(Dispatchers.Main) {
                        invoke.reject(ex.message ?: "unknown")
                    }
                }
            }
        }
        catch (e: Exception) {
            invoke.reject(e.message ?: "unknown")
        }
    }

    @Command
    fun createNewMediaStoreFile(invoke: Invoke) {
        try {
            CoroutineScope(Dispatchers.IO).launch {
                try {
                    val args = invoke.parseArgs(CreateNewMediaStoreFileArgs::class.java)
                    val res = JSObject().apply {
                        put("uri", AFMediaStore.createNewFile(
                            args.volumeName,
                            args.relativePath,
                            args.mimeType,
                            args.pending ?: throw Exception("missing value: pending"),
                            activity
                        ))
                    }

                    withContext(Dispatchers.Main) {
                        invoke.resolve(res)
                    }
                }
                catch (ex: Exception) {
                    withContext(Dispatchers.Main) {
                        invoke.reject(ex.message ?: "unknown")
                    }
                }
            }
        }
        catch (e: Exception) {
            invoke.reject(e.message ?: "unknown")
        }
    }

    @Command
    fun getStorageVolumeByPath(invoke: Invoke) {
        try {
            CoroutineScope(Dispatchers.IO).launch {
                try {
                    val args = invoke.parseArgs(GetStorageVolumeByPathArgs::class.java)
                    val res = JSObject().apply {
                        val sv = AFStorageVolume.getStorageVolumeByFileIfAvailable(File(args.path!!), activity)
                        val svJsObj: JSObject? = sv?.let { AFJSObject.createStorageVolumeJSObject(it) }
                        put("volume", svJsObj)
                    }

                    withContext(Dispatchers.Main) {
                        invoke.resolve(res)
                    }
                }
                catch (ex: Exception) {
                    val message = ex.message ?: "unknown"
                    withContext(Dispatchers.Main) {
                        invoke.reject(message)
                    }
                }
            }
        }
        catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke getStorageVolumeByPath."
            invoke.reject(message)
        }
    }

    @Command
    fun getPrimaryStorageVolumeIfAvailable(invoke: Invoke) {
        try {
            CoroutineScope(Dispatchers.IO).launch {
                try {
                    val res = JSObject().apply {
                        val sv = AFStorageVolume.getPrimaryStorageVolumeIfAvailable(activity)
                        val svJsObj: JSObject? = sv?.let { AFJSObject.createStorageVolumeJSObject(it) }
                        put("volume", svJsObj)
                    }

                    withContext(Dispatchers.Main) {
                        invoke.resolve(res)
                    }
                }
                catch (ex: Exception) {
                    val message = ex.message ?: "Failed to invoke getPrimaryStorageVolumeIfAvailable"
                    withContext(Dispatchers.Main) {
                        invoke.reject(message)
                    }
                }
            }
        }
        catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke getPrimaryStorageVolumeIfAvailable"
            invoke.reject(message)
        }
    }

    @Command
    fun getAvailableStorageVolumes(invoke: Invoke) {
        try {
            CoroutineScope(Dispatchers.IO).launch {
                try {
                    val res = JSObject().apply {
                        val svs = AFStorageVolume.getAvailableStorageVolumes(activity)
                        val svsObj: JSArray = JSArray().apply {
                            for (sv in svs) {
                                put(AFJSObject.createStorageVolumeJSObject(sv))
                            }
                        }
                        put("volumes", svsObj)
                    }

                    withContext(Dispatchers.Main) {
                        invoke.resolve(res)
                    }
                }
                catch (ex: Exception) {
                    val message = ex.message ?: "Failed to invoke getAvailableStorageVolumes"

                    withContext(Dispatchers.Main) {
                        invoke.reject(message)
                    }
                }
            }
        }
        catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke getAvailableStorageVolumes"
            invoke.reject(message)
        }
    }

    @Command
    fun checkStorageVolumeAvailableByPath(invoke: Invoke) {
        try {
            CoroutineScope(Dispatchers.IO).launch {
                try {
                    val args = invoke.parseArgs(CheckStorageVolumeAvailableByPathArgs::class.java)
                    val res = JSObject().apply {
                        val ok: Boolean = AFStorageVolume.checkStorageVolumeAvailableByFile(File(args.path!!) ,activity)
                        put("value", ok)
                    }

                    withContext(Dispatchers.Main) {
                        invoke.resolve(res)
                    }
                }
                catch (ex: Exception) {
                    val message = ex.message ?: "Failed to invoke checkStorageVolumeAvailableByPath"

                    withContext(Dispatchers.Main) {
                        invoke.reject(message)
                    }
                }
            }
        }
        catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke checkStorageVolumeAvailableByPath"
            invoke.reject(message)
        }
    }

    @Command
    fun checkMediaStoreVolumeNameAvailable(invoke: Invoke) {
        try {
            CoroutineScope(Dispatchers.IO).launch {
                try {
                    val args = invoke.parseArgs(CheckMediaStoreVolumeNameAvailableArgs::class.java)
                    val res = JSObject().apply {
                        val ok: Boolean = AFStorageVolume.checkMediaStoreVolumeNameAvailable(args.mediaStoreVolumeName!! ,activity)
                        put("value", ok)
                    }

                    withContext(Dispatchers.Main) {
                        invoke.resolve(res)
                    }
                }
                catch (ex: Exception) {
                    val message = ex.message ?: "Failed to invoke checkMediaStoreVolumeNameAvailable"

                    withContext(Dispatchers.Main) {
                        invoke.reject(message)
                    }
                }
            }
        }
        catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke checkMediaStoreVolumeNameAvailable"
            invoke.reject(message)
        }
    }

    @Command
    fun getAllPersistedUriPermissions(invoke: Invoke) {
        try {
            val items = JSArray()

            activity.contentResolver.persistedUriPermissions.forEach {
                val uri = it.uri
                val item = when {
                    DocumentsContract.isTreeUri(uri) -> {
                        val builtUri = DocumentsContract.buildDocumentUriUsingTree(
                            uri,
                            DocumentsContract.getTreeDocumentId(uri)
                        )

                        JSObject().apply {
                            put("uri", JSObject().apply {
                                put("uri", builtUri.toString())
                                put("documentTopTreeUri", uri.toString())
                            })
                            put("r", it.isReadPermission)
                            put("w", it.isWritePermission)
                            put("d", true)
                        }
                    }
                    else -> {
                        JSObject().apply {
                            put("uri", JSObject().apply {
                                put("uri", uri.toString())
                                put("documentTopTreeUri", null)
                            })
                            put("r", it.isReadPermission)
                            put("w", it.isWritePermission)
                            put("d", false)
                        }
                    }
                };
                items.put(item)
            }

            val res = JSObject().apply {
                put("items", items)
            }

            invoke.resolve(res)
        }
        catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke getAllPersistedUriPermissions."
            invoke.reject(message)
        }
    }

    @Command
    fun releaseAllPersistedUriPermissions(invoke: Invoke) {
        try {
            activity.contentResolver.persistedUriPermissions.forEach {
                val flag = when {
                    it.isReadPermission && it.isWritePermission -> Intent.FLAG_GRANT_READ_URI_PERMISSION or Intent.FLAG_GRANT_WRITE_URI_PERMISSION
                    it.isReadPermission -> Intent.FLAG_GRANT_READ_URI_PERMISSION
                    it.isWritePermission -> Intent.FLAG_GRANT_WRITE_URI_PERMISSION
                    else -> null
                }
            
                if (flag != null) {
                    activity.contentResolver.releasePersistableUriPermission(it.uri, flag)
                }
            }
            invoke.resolve()
        }
        catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke releaseAllPersistedUriPermissions."
            invoke.reject(message)
        }
    }

    @Command
    fun releasePersistedUriPermission(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(ReleasePersistedUriPermissionArgs::class.java)
            val uri = if (args.uri.documentTopTreeUri != null) {
                Uri.parse(args.uri.documentTopTreeUri)
            }
            else {
                Uri.parse(args.uri.uri)
            }

            activity.contentResolver.persistedUriPermissions.find { it.uri == uri }?.let {
                val flag = when {
                    it.isReadPermission && it.isWritePermission -> Intent.FLAG_GRANT_READ_URI_PERMISSION or Intent.FLAG_GRANT_WRITE_URI_PERMISSION
                    it.isReadPermission -> Intent.FLAG_GRANT_READ_URI_PERMISSION
                    it.isWritePermission -> Intent.FLAG_GRANT_WRITE_URI_PERMISSION
                    else -> null
                }
            
                if (flag != null) {
                    activity.contentResolver.releasePersistableUriPermission(it.uri, flag)
                }
            }

            invoke.resolve()
        }
        catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke releasePersistedUriPermission."
            invoke.reject(message)
        }
    }

    @Command
    fun takePersistableUriPermission(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(TakePersistableUriPermissionArgs::class.java)

            val uri = if (args.uri.documentTopTreeUri != null) {
                Uri.parse(args.uri.documentTopTreeUri)
            }
            else {
                Uri.parse(args.uri.uri)
            }

            try {
                activity.contentResolver.takePersistableUriPermission(uri, Intent.FLAG_GRANT_READ_URI_PERMISSION)    
            }
            catch (ignore: Exception) {}
            try {
                activity.contentResolver.takePersistableUriPermission(uri, Intent.FLAG_GRANT_WRITE_URI_PERMISSION)    
            }
            catch (ignore: Exception) {}

            invoke.resolve()
        }
        catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke takePersistableUriPermission."
            invoke.reject(message)
        }
    }

    @Command
    fun checkPersistedUriPermission(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(CheckPersistedUriPermissionArgs::class.java)

            val uri = if (args.uri.documentTopTreeUri != null) {
                Uri.parse(args.uri.documentTopTreeUri)
            }
            else {
                Uri.parse(args.uri.uri)
            }

            val p = activity.contentResolver.persistedUriPermissions.find { it.uri == uri }
            if (p != null) {
                 val value = when (args.mode) {
                    PersistableUriPermissionMode.Read -> p.isReadPermission
                    PersistableUriPermissionMode.Write -> p.isWritePermission
                    PersistableUriPermissionMode.ReadAndWrite -> p.isReadPermission && p.isWritePermission
                    PersistableUriPermissionMode.ReadOrWrite -> p.isReadPermission || p.isWritePermission
                }

                invoke.resolve(JSObject().apply {
                    put("value", value)
                })
            }
            else {
                invoke.resolve(JSObject().apply {
                    put("value", false)
                })
            }
        }
        catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke checkPersistedUriPermission."
            invoke.reject(message)
        }
    }

    @Command
    fun createFile(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(CreateFileArgs::class.java)

            CoroutineScope(Dispatchers.IO).launch {
                try {
                    val fileName = args.relativePath.substringAfterLast('/', args.relativePath)
                    val mimeType = args.mimeType ?: getMimeTypeFromName(fileName)
                    val res = getFileController(args.dir)
                        .createFile(args.dir, args.relativePath, mimeType)

                    withContext(Dispatchers.Main) {
                        invoke.resolve(res) 
                    }
                }
                catch (ex: Exception) {
                    withContext(Dispatchers.Main) {
                        val message = ex.message ?: "Failed to invoke createFile."
                        invoke.reject(message)
                    }
                }
            }
        }
        catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke createFile."
            invoke.reject(message)
        }
    }

    @Command
    fun createFileAndReturnRelativePath(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(CreateFileAndReturnRelativePathArgs::class.java)

            CoroutineScope(Dispatchers.IO).launch {
                try {
                    val fileName = args.relativePath.substringAfterLast('/', args.relativePath)
                    val mimeType = args.mimeType ?: getMimeTypeFromName(fileName)
                    val res = getFileController(args.dir)
                        .createFileAndReturnRelativePath(args.dir, args.relativePath, mimeType)

                    withContext(Dispatchers.Main) {
                        invoke.resolve(res)
                    }
                }
                catch (ex: Exception) {
                    withContext(Dispatchers.Main) {
                        val message = ex.message ?: "Failed to invoke createFile."
                        invoke.reject(message)
                    }
                }
            }
        }
        catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke createFile."
            invoke.reject(message)
        }
    }

    @Command
    fun createDirAll(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(CreateDirAllArgs::class.java)

            CoroutineScope(Dispatchers.IO).launch {
                try {
                    val res = getFileController(args.dir)
                        .createDirAll(args.dir, args.relativePath)

                    withContext(Dispatchers.Main) {
                        invoke.resolve(res)
                    }
                }
                catch (ex: Exception) {
                    withContext(Dispatchers.Main) {
                        val message = ex.message ?: "Failed to invoke createDirAll."
                        invoke.reject(message)
                    }
                }
            }
        }
        catch(ex: Exception) {
            val message = ex.message ?: "Failed to invoke createDirAll."
            invoke.reject(message)
        }
    }

    @Command
    fun createDirAllAndReturnRelativePath(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(CreateDirAllAndReturnRelativePathArgs::class.java)

            CoroutineScope(Dispatchers.IO).launch {
                try {
                    val res = getFileController(args.dir)
                        .createDirAllAndReturnRelativePath(args.dir, args.relativePath)

                    withContext(Dispatchers.Main) {
                        invoke.resolve(res)
                    }
                }
                catch (ex: Exception) {
                    withContext(Dispatchers.Main) {
                        val message = ex.message ?: "Failed to invoke createDirAll."
                        invoke.reject(message)
                    }
                }
            }
        }
        catch(ex: Exception) {
            val message = ex.message ?: "Failed to invoke createDirAll."
            invoke.reject(message)
        }
    }

    @Command
    fun readDir(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(ReadDirArgs::class.java)
            
            CoroutineScope(Dispatchers.IO).launch {
                try {
                    val res = JSObject().apply {
                        put("entries", getFileController(args.uri).readDir(args.uri, args.options))
                    }

                    withContext(Dispatchers.Main) {
                        invoke.resolve(res)
                    }
                }
                catch (ex: Exception) {
                    withContext(Dispatchers.Main) {
                        invoke.reject(ex.message ?: "unknown")
                    }
                }
            }
        }
        catch (ex: Exception) {
            invoke.reject(ex.message ?: "unknown")
        }
    }

    @Command
    fun getName(invoke: Invoke) {
        try {
            CoroutineScope(Dispatchers.IO).launch {
                try {
                    val args = invoke.parseArgs(GetNameArgs::class.java)
                    val res = JSObject()
                    res.put("name", getFileController(args.uri).getName(args.uri))

                    withContext(Dispatchers.Main) {
                        invoke.resolve(res)
                    }
                }
                catch (e: Exception) {
                    withContext(Dispatchers.Main) {
                        invoke.reject(e.message ?: "Unknown exception")
                    }
                }
            }
        }
        catch (e: Exception) {
            invoke.reject(e.message ?: "Unknown exception")
        }
    }

    @Command
    fun getLen(invoke: Invoke) {
        try {
            CoroutineScope(Dispatchers.IO).launch {
                try {
                    val args = invoke.parseArgs(GetLenArgs::class.java)
                    val res = JSObject()
                    res.put("len", getFileController(args.uri).getLen(args.uri))

                    withContext(Dispatchers.Main) {
                        invoke.resolve(res)
                    }
                }
                catch (e: Exception) {
                    withContext(Dispatchers.Main) {
                        invoke.reject(e.message ?: "Unknown exception")
                    }
                }
            }
        } catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke getLen."
            invoke.reject(message)
        }
    }

    @Command
    fun getMetadata(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(GetMetadataArgs::class.java)

            CoroutineScope(Dispatchers.IO).launch {
                try {
                    val res = getFileController(args.uri).getMetadata(args.uri)

                    withContext(Dispatchers.Main) {
                        invoke.resolve(res)
                    }
                }
                catch (ex: Exception) {
                    withContext(Dispatchers.Main) {
                        val message = ex.message ?: "Failed to invoke getMetadata."
                        invoke.reject(message)
                    }
                }
            }
        }
        catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke getMetadata."
            invoke.reject(message)
        }
    }

    @Command
    fun getThumbnailToFile(invoke: Invoke) {
        try {
            CoroutineScope(Dispatchers.Default).launch {
                try {
                    val args = invoke.parseArgs(GetThumbnailToFileArgs::class.java)

                    val ok = AFThumbnails.loadThumbnail(
                        fileUri = args.src,
                        preferredSize = Size(args.width, args.height),
                        format = args.format,
                        quality = args.quality,
                        output = { openFileWt(Uri.parse(args.dest.uri)) },
                        useThumbnail = { true },
                        ctx = activity
                    ) ?: false

                    withContext(Dispatchers.Main) {
                        invoke.resolve(JSObject().apply {
                            put("value", ok)
                        })
                    }
                }
                catch (ex: Exception) {
                    val message = ex.message ?: "Failed to invoke getThumbnail."
                    withContext(Dispatchers.Main) {
                        invoke.reject(message)
                    }
                }
            }
        }
        catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke getThumbnail."
            invoke.reject(message)
        }
    }

    @Command
    fun getThumbnail(invoke: Invoke) {
        try {
            CoroutineScope(Dispatchers.Default).launch {
                try {
                    val args = invoke.parseArgs(GetThumbnailArgs::class.java)

                    val base64 = AFThumbnails.loadThumbnail(
                        fileUri = args.uri,
                        preferredSize = Size(args.width, args.height),
                        format = args.format,
                        quality = args.quality,
                        output = { ByteArrayOutputStream() },
                        useThumbnail = { Base64.encodeToString(it.toByteArray(), Base64.NO_WRAP) },
                        ctx = activity
                    )

                    val res = JSObject().apply {
                        put("bytes", base64)
                    }

                    withContext(Dispatchers.Main) {
                        invoke.resolve(res)
                    }
                }
                catch (ex: Exception) {
                    val message = ex.message ?: "Failed to invoke getThumbnail."

                    withContext(Dispatchers.Main) {
                        invoke.reject(message)
                    }
                }
            }
        }
        catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke getThumbnail."
            invoke.reject(message)
        }
    }

    @Command
    fun deleteFile(invoke: Invoke) {
        try {
            CoroutineScope(Dispatchers.IO).launch {
                try {
                    val args = invoke.parseArgs(DeleteArgs::class.java)
                    getFileController(args.uri).deleteFile(args.uri)

                    withContext(Dispatchers.Main) {
                        invoke.resolve()
                    }
                }
                catch (e: Exception) {
                    withContext(Dispatchers.Main) {
                        invoke.reject(e.message ?: "Unknown exception")
                    }
                }
            }
        } catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke deleteFile."
            invoke.reject(message)
        }
    }

    @Command
    fun deleteEmptyDir(invoke: Invoke) {
        try {
            CoroutineScope(Dispatchers.IO).launch {
                try {
                    val args = invoke.parseArgs(DeleteArgs::class.java)
                    getFileController(args.uri).deleteEmptyDir(args.uri)

                    withContext(Dispatchers.Main) {
                        invoke.resolve()
                    }
                }
                catch (e: Exception) {
                    withContext(Dispatchers.Main) {
                        invoke.reject(e.message ?: "Unknown exception")
                    }
                }
            }
        } catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke deleteEmptyDir."
            invoke.reject(message)
        }
    }

    @Command
    fun deleteDirAll(invoke: Invoke) {
        try {
            CoroutineScope(Dispatchers.IO).launch {
                try {
                    val args = invoke.parseArgs(DeleteArgs::class.java)
                    getFileController(args.uri).deleteDirAll(args.uri)

                    withContext(Dispatchers.Main) {
                        invoke.resolve()
                    }
                }
                catch (e: Exception) {
                    withContext(Dispatchers.Main) {
                        invoke.reject(e.message ?: "Unknown exception")
                    }
                }
            }
        } catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke deleteDirAll."
            invoke.reject(message)
        }
    }

    @Command
    fun rename(invoke: Invoke) {
        try {
            CoroutineScope(Dispatchers.IO).launch {
                try {
                    val args = invoke.parseArgs(RenameArgs::class.java)
                    val uri = getFileController(args.uri).rename(args.uri, args.newName)

                    withContext(Dispatchers.Main) {
                        invoke.resolve(uri)
                    }
                }
                catch (e: Exception) {
                    withContext(Dispatchers.Main) {
                        invoke.reject(e.message ?: "Unknown exception")
                    }
                }
            }
        } catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke rename."
            invoke.reject(message)
        }
    }

    @Command
    fun move(invoke: Invoke) {
        try {
            CoroutineScope(Dispatchers.IO).launch {
                try {
                    val args = invoke.parseArgs(MoveArgs::class.java)
                    val uri = getFileController(args.uri).move(args.uri, args.destDir, args.newName)

                    withContext(Dispatchers.Main) {
                        invoke.resolve(uri)
                    }
                }
                catch (e: Exception) {
                    withContext(Dispatchers.Main) {
                        invoke.reject(e.message ?: "Unknown exception")
                    }
                }
            }
        } catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke move."
            invoke.reject(message)
        }
    }

    @Command
    fun shareFiles(invoke: Invoke) {
        @InvokeArg
        class Args {
            lateinit var uris: Array<FileUri>
        }

        try {
            val args = invoke.parseArgs(Args::class.java)
            val uris = args.uris.map { Uri.parse(it.uri) }

            if (uris.isEmpty()) throw IllegalArgumentException("uris must not be empty")

            val mimeTypes = uris.map { activity.contentResolver.getType(it) ?: "*/*" }
            val commonMimeType = getCommonMimeType(mimeTypes)
            val builder = ShareCompat.IntentBuilder(activity)
                .setType(commonMimeType)

            for (uri in uris) {
                builder.addStream(uri)
            }

            val intentChooser = Intent.createChooser(
                builder.intent
                    .addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
                    .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK),
                ""
            ).apply {
                addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
                putExtra(Intent.EXTRA_EXCLUDE_COMPONENTS, arrayOf(activity.componentName))
            }

            activity.applicationContext.startActivity(intentChooser)
            invoke.resolve()
        }
        catch (e: Exception) {
            invoke.reject(e.message ?: "Unknown")
        }
    }

    @Command
    fun viewDir(invoke: Invoke) {
        @InvokeArg
        class Args {
            lateinit var uri: FileUri
        }

        try {
            val args = invoke.parseArgs(Args::class.java)
            val uri = Uri.parse(args.uri.uri)
            val mimeType = DocumentsContract.Document.MIME_TYPE_DIR

            val intentChooser = Intent.createChooser(
                Intent(Intent.ACTION_VIEW)
                    .setDataAndType(uri, mimeType)
                    .addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
                    .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK),
                ""
            ).apply {
                addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
                putExtra(Intent.EXTRA_EXCLUDE_COMPONENTS, arrayOf(activity.componentName))
            }

            activity.applicationContext.startActivity(intentChooser)
            invoke.resolve()
        }
        catch (e: Exception) {
            invoke.reject(e.message ?: "Unknown")
        }
    }

    @Command
    fun viewFile(invoke: Invoke) {
        @InvokeArg
        class Args {
            lateinit var uri: FileUri
        }

        try {
            val args = invoke.parseArgs(Args::class.java)
            val uri = Uri.parse(args.uri.uri)
            val mimeType = activity.contentResolver.getType(uri)

            val intentChooser =  Intent.createChooser(
                Intent(Intent.ACTION_VIEW)
                    .setDataAndType(uri, mimeType)
                    .addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
                    .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK),
                ""
            ).apply {
                addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
                putExtra(Intent.EXTRA_EXCLUDE_COMPONENTS, arrayOf(activity.componentName))
            }

            activity.applicationContext.startActivity(intentChooser)
            invoke.resolve()
        }
        catch (e: Exception) {
            invoke.reject(e.message ?: "Unknown")
        }
    }

    @Command
    fun editFile(invoke: Invoke) {
        @InvokeArg
        class Args {
            lateinit var uri: FileUri
        }

        try {
            val args = invoke.parseArgs(Args::class.java)
            val uri = Uri.parse(args.uri.uri)
            val mimeType = activity.contentResolver.getType(uri)

            val intentChooser = Intent.createChooser(
                Intent(Intent.ACTION_EDIT)
                    .setDataAndType(uri, mimeType)
                    .addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
                    .addFlags(Intent.FLAG_GRANT_WRITE_URI_PERMISSION)
                    .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK),
                ""
            ).apply {
                addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
                addFlags(Intent.FLAG_GRANT_WRITE_URI_PERMISSION)
                putExtra(Intent.EXTRA_EXCLUDE_COMPONENTS, arrayOf(activity.componentName))
            }

            activity.applicationContext.startActivity(intentChooser)
            invoke.resolve()
        }
        catch (e: Exception) {
            invoke.reject(e.message ?: "Unknown")
        }
    }

    @Command
    fun showManageDirDialog(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(ShowManageDirDialogArgs::class.java)
            val initialLocation = args.initialLocation?.let { tryIntoSafInitialLocation(it) }
            val initialLocationStr = initialLocation?.toString()
            val initialLocationIsStorageVolumeRoot = initialLocationStr?.startsWith("content://com.android.externalstorage.documents/root/") ?: false

            var intent: Intent? = null

            // RootUri が指定された場合は StorageVolume.createOpenDocumentTreeIntent を用いる。
            // これは DocumentsContract.EXTRA_INITIAL_URI で指定するよりもアクセシビリティの最適化が行われる。
            if (initialLocation != null && Build.VERSION_CODES.Q <= Build.VERSION.SDK_INT && initialLocationIsStorageVolumeRoot) {
                val id = initialLocationStr!!.removePrefix("content://com.android.externalstorage.documents/root/")
                val sm = activity.getSystemService(Context.STORAGE_SERVICE) as StorageManager

                when (id) {
                    "primary" -> intent = sm.primaryStorageVolume.createOpenDocumentTreeIntent()
                    else -> {
                        for (vol in sm.storageVolumes) {
                            if (vol.uuid.equals(id, true)) {
                                intent = vol.createOpenDocumentTreeIntent()
                                break
                            }
                        }
                    }
                }
            }

            if (intent == null) {
                intent = Intent(Intent.ACTION_OPEN_DOCUMENT_TREE).apply {
                    if (Build.VERSION_CODES.O <= Build.VERSION.SDK_INT && initialLocation != null) {
                        putExtra(DocumentsContract.EXTRA_INITIAL_URI, initialLocation)
                    }
                }
            }

            if (args.localOnly) {
                intent.putExtra(Intent.EXTRA_LOCAL_ONLY, true)
            }

            startActivityForResult(invoke, intent, "handleShowManageDirDialog")
        }
        catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke showManageDirDialog."
            invoke.reject(message)
        }
    }

    @ActivityCallback
    private fun handleShowManageDirDialog(invoke: Invoke, result: ActivityResult) {
        try {
            val res = JSObject()

            val uri = result.data?.data
            if (uri != null) {
                val builtUri = DocumentsContract.buildDocumentUriUsingTree(
                    uri,
                    DocumentsContract.getTreeDocumentId(uri)
                )

                val obj = JSObject()
                obj.put("uri", builtUri.toString())
                obj.put("documentTopTreeUri", uri.toString())

                res.put("uri", obj)
            } else {
                res.put("uri", null)
            }

            invoke.resolve(res)
        } catch (ex: java.lang.Exception) {
            val message = ex.message ?: "Failed to invoke dirDialogResult."
            invoke.reject(message)
        }
    }

    @Command
    fun getPrivateBaseDirAbsolutePaths(invoke: Invoke) {
        try {
            val res = JSObject()
            res.put("data", activity.filesDir.absolutePath)
            res.put("cache", activity.cacheDir.absolutePath)
            res.put("noBackupData", activity.noBackupFilesDir.absolutePath)
            invoke.resolve(res)
        } catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke getPrivateBaseDirAbsolutePaths."
            invoke.reject(message)
        }
    }

    @Command
    fun getMimeType(invoke: Invoke) {
        try {
            CoroutineScope(Dispatchers.IO).launch {
                try {
                    val args = invoke.parseArgs(GetMimeTypeArgs::class.java)
                    val res = JSObject()
                    res.put("value", getFileController(args.uri).getMimeType(args.uri))

                    withContext(Dispatchers.Main) {
                        invoke.resolve(res)
                    }
                }
                catch (e: Exception) {
                    withContext(Dispatchers.Main) {
                        invoke.reject(e.message ?: "Unknown exception")
                    }
                }
            }
        }
        catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke getMimeType."
            invoke.reject(message)
        }
    }

    @Command
    fun showOpenFileDialog(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(ShowOpenFileDialogArgs::class.java)
            val intent = createFilePickerIntent(args.mimeTypes, args.multiple)

            args.initialLocation?.let { uri ->
                tryIntoSafInitialLocation(uri)?.let { dUri ->
                    if (Build.VERSION_CODES.O <= Build.VERSION.SDK_INT){
                        intent.putExtra(DocumentsContract.EXTRA_INITIAL_URI, dUri)
                    }
                }
            }

            if (args.localOnly) {
                intent.putExtra(Intent.EXTRA_LOCAL_ONLY, true)
            }

            startActivityForResult(invoke, intent, "handleShowOpenFileAndVisualMediaDialog")
        }
        catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke showOpenFileDialog."
            invoke.reject(message)
        }
    }

    @Command
    fun showOpenContentDialog(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(ShowOpenContentDialogArgs::class.java)
            val intent = createContentPickerIntent(args.mimeTypes, args.multiple)

            startActivityForResult(invoke, intent, "handleShowOpenFileAndVisualMediaDialog")
        } catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke showOpenContentDialog."
            invoke.reject(message)
        }
    }

    @Command
    fun showOpenVisualMediaDialog(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(ShowOpenVisualMediaDialogArgs::class.java)
            val intent = createVisualMediaPickerIntent(args.multiple, args.target)

            if (args.localOnly) {
                intent.putExtra(Intent.EXTRA_LOCAL_ONLY, true)
            }

            startActivityForResult(invoke, intent, "handleShowOpenFileAndVisualMediaDialog")
        } catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke showOpenVisualMediaDialog."
            invoke.reject(message)
        }
    }

    @Command
    fun showSaveFileDialog(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(ShowSaveFileDialogArgs::class.java)

            val intent = Intent(Intent.ACTION_CREATE_DOCUMENT)

            intent.setType(args.mimeType ?: getMimeTypeFromName(args.initialFileName))
            intent.addCategory(Intent.CATEGORY_OPENABLE)
            intent.putExtra(Intent.EXTRA_TITLE, args.initialFileName)
            
            args.initialLocation?.let { uri ->
                tryIntoSafInitialLocation(uri)?.let { dUri ->
                    if (Build.VERSION_CODES.O <= Build.VERSION.SDK_INT){
                        intent.putExtra(DocumentsContract.EXTRA_INITIAL_URI, dUri)
                    }
                }
            }

            if (args.localOnly) {
                intent.putExtra(Intent.EXTRA_LOCAL_ONLY, true)
            }

            startActivityForResult(invoke, intent, "handleShowSaveFileDialog")
        } catch (ex: Exception) {
            val message = ex.message ?: "Failed to pick save file"
            invoke.reject(message)
        }
    }

    @ActivityCallback
    fun handleShowSaveFileDialog(invoke: Invoke, result: ActivityResult) {
        try {
            when (result.resultCode) {
                Activity.RESULT_OK -> {
                    val callResult = JSObject()
                    val intent: Intent? = result.data
                    if (intent != null) {
                        val uri = intent.data

                        if (uri == null) {
                            callResult.put("uri", null)
                        }
                        else {
                            val o = JSObject()
                            o.put("uri", uri.toString())
                            o.put("documentTopTreeUri", null)
                            callResult.put("uri", o)
                        }
                    }
                    invoke.resolve(callResult)
                }
                Activity.RESULT_CANCELED -> {
                    val callResult = JSObject()
                    callResult.put("uri", null)
                    invoke.resolve(callResult)
                }
                else -> invoke.reject("Failed to pick files")
            }
        } catch (ex: java.lang.Exception) {
            val message = ex.message ?: "Failed to read file pick result"
            invoke.reject(message)
        }
    }

    @Command
    fun isVisualMediaDialogAvailable(invoke: Invoke) {
        try {
            val res = JSObject()
            res.put("value", PickVisualMedia.isPhotoPickerAvailable())
            invoke.resolve(res)
        }
        catch (e: Exception) {
            invoke.reject(e.message ?: "Unknown")
        }
    }

    @SuppressLint("Recycle")
    @Command
    fun getFileDescriptor(invoke: Invoke) {
        try {
            CoroutineScope(Dispatchers.IO).launch {
                try {
                    val args = invoke.parseArgs(GetFileDescriptorArgs::class.java)
                    val fd: Int = AFFileDescriptor
                        .getPfd(Uri.parse(args.uri.uri), args.mode, activity)
                        .detachFd()

                    val res = JSObject()
                    res.put("fd", fd)

                    withContext(Dispatchers.Main) {
                        invoke.resolve(res)
                    }
                }
                catch (ex: Exception) {
                    val message = ex.message ?: "Failed to invoke getFileDescriptor."

                    withContext(Dispatchers.Main) {
                        invoke.reject(message)
                    }
                }
            }
        }
        catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke getFileDescriptor."
            invoke.reject(message)
        }
    }

    @SuppressLint("Recycle")
    @Command
    fun getFileDescriptorWithFallback(invoke: Invoke) {
        try {
            CoroutineScope(Dispatchers.IO).launch {
                try {
                    val args = invoke.parseArgs(GetFileDescriptorWithFallbackArgs::class.java)
                    val uri = Uri.parse(args.uri.uri)

                    var fd: Int? = null
                    var mode: String? = null
                    for (m in args.modes) {
                        try {
                            mode = m
                            fd = AFFileDescriptor
                                .getPfd(uri, m, activity)
                                .detachFd()
                        }
                        catch (ignore: Exception) {}

                        if (fd != null) break
                    }
                    
                    val res = JSObject()
                    res.put("fd", fd ?: throw Exception("No file or permission, or unavailable"))
                    res.put("mode", mode!!)

                    withContext(Dispatchers.Main) {
                        invoke.resolve(res)
                    }
                }
                catch (ex: Exception) {
                    val message = ex.message ?: "Failed to invoke getFileDescriptor."
                    withContext(Dispatchers.Main) {
                        invoke.reject(message)
                    }
                }
            }
        }
        catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke getFileDescriptor."
            invoke.reject(message)
        }
    }

    @ActivityCallback
    fun handleShowOpenFileAndVisualMediaDialog(invoke: Invoke, result: ActivityResult) {
        try {
            when (result.resultCode) {
                Activity.RESULT_OK -> {
                    val callResult = createPickFilesResult(result.data)
                    invoke.resolve(callResult)
                }
                Activity.RESULT_CANCELED -> {
                    val callResult = createPickFilesResult(null)
                    invoke.resolve(callResult)
                }
            }
        } catch (ex: java.lang.Exception) {
            val message = ex.message ?: "Failed to read file pick result"
            invoke.reject(message)
        }
    }

    private fun createPickFilesResult(data: Intent?): JSObject {
        val callResult = JSObject()
        if (data == null) {
            callResult.put("uris", JSArray())
            return callResult
        }
        val uris: MutableList<Uri?> = ArrayList()
        if (data.clipData == null) {
            val uri: Uri? = data.data
            uris.add(uri)
        }
        else {
            for (i in 0 until data.clipData!!.itemCount) {
                val uri: Uri = data.clipData!!.getItemAt(i).uri
                uris.add(uri)
            }
        }

        val buffer = JSArray()
        for (uri in uris) {
            if (uri != null) {
                val o = JSObject()
                o.put("uri", uri.toString())
                o.put("documentTopTreeUri", null)
                buffer.put(o)
            }
        }

        callResult.put("uris", buffer)
        return callResult
    }

    private fun createFilePickerIntent(mimeTypes: Array<String>, multiple: Boolean): Intent {
        val intent = Intent(Intent.ACTION_OPEN_DOCUMENT)
            .addCategory(Intent.CATEGORY_OPENABLE)
            .putExtra(Intent.EXTRA_ALLOW_MULTIPLE, multiple)

        if (mimeTypes.isEmpty()) {
            return intent.setType("*/*")
        } else if (mimeTypes.size == 1) {
            return intent.setType(mimeTypes[0])
        }

        return intent.setType("*/*").putExtra(Intent.EXTRA_MIME_TYPES, mimeTypes)
    }

    private fun createContentPickerIntent(mimeTypes: Array<String>, multiple: Boolean): Intent {
        val intent = Intent(Intent.ACTION_GET_CONTENT)
            .addCategory(Intent.CATEGORY_OPENABLE)
            .putExtra(Intent.EXTRA_ALLOW_MULTIPLE, multiple)

        if (mimeTypes.isEmpty()) {
            intent.setType("*/*")
        } 
        else if (mimeTypes.size == 1) {
            intent.setType(mimeTypes[0])
        }
        else {
            intent.setType("*/*")
            intent.putExtra(Intent.EXTRA_MIME_TYPES, mimeTypes)
        }

        if (intent.resolveActivity(activity.packageManager) != null) {
            return Intent.createChooser(intent, "")
        }
        else {
            return createFilePickerIntent(mimeTypes, multiple)
        }
    }

    private fun createVisualMediaPickerIntent(
        multiple: Boolean,
        targetMimeType: String
    ): Intent {

        val req = PickVisualMediaRequest(
            when {
                targetMimeType == "image/*" -> PickVisualMedia.ImageOnly
                targetMimeType == "video/*" -> PickVisualMedia.VideoOnly
                targetMimeType.startsWith("image/") -> PickVisualMedia.SingleMimeType(targetMimeType)
                targetMimeType.startsWith("video/") -> PickVisualMedia.SingleMimeType(targetMimeType)
                else -> PickVisualMedia.ImageAndVideo
            }
        )

        return when (multiple) {
            true -> PickMultipleVisualMedia().createIntent(activity, req)
            false -> PickVisualMedia().createIntent(activity, req)
        }
    }
    
    private fun getCommonMimeType(mimeTypes: List<String>): String {
        if (mimeTypes.isEmpty()) return "*/*"

        // 最初の MIME タイプを基準に分割
        val firstParts = mimeTypes[0].split("/")
        if (firstParts.size != 2) throw Exception("Illegal mimeType format: ${mimeTypes[0]}")

        val type = firstParts[0]
        var subtype = firstParts[1]

        for (mime in mimeTypes.drop(1)) {
            val parts = mime.split("/")
            if (parts.size != 2) throw Exception("Illegal mimeType format: ${mime}")

            // typeが共通でなければ共通MIMEはなし
            if (parts[0] != type) return "*/*"

            // subtypeが異なる場合はワイルドカードに
            if (parts[1] != subtype) {
                subtype = "*"
            }
        }

        return "$type/$subtype"
    }
}