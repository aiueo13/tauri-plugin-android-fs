package com.plugin.android_fs

import android.annotation.SuppressLint
import android.app.Activity
import android.content.Intent
import android.content.ActivityNotFoundException
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.graphics.ImageDecoder
import android.net.Uri
import android.os.Build
import android.os.Environment
import android.provider.DocumentsContract
import android.provider.MediaStore
import androidx.activity.result.ActivityResult
import androidx.activity.result.PickVisualMediaRequest
import androidx.activity.result.contract.ActivityResultContracts.PickMultipleVisualMedia
import androidx.activity.result.contract.ActivityResultContracts.PickVisualMedia
import androidx.core.app.ShareCompat
import android.webkit.MimeTypeMap
import android.media.MediaMetadataRetriever
import android.media.MediaMetadataRetriever.OPTION_PREVIOUS_SYNC
import android.util.Base64
import app.tauri.annotation.ActivityCallback
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSArray
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import java.io.ByteArrayOutputStream
import java.io.File
import kotlin.math.min
import kotlin.io.DEFAULT_BUFFER_SIZE
import kotlin.io.copyTo
import java.io.FileOutputStream
import java.io.OutputStream

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
    var multiple: Boolean = false
}

@InvokeArg
class ShowManageDirDialogArgs {
    var initialLocation: FileUri? = null
}

@InvokeArg
class ShowSaveFileDialogArgs {
    var initialLocation: FileUri? = null
    lateinit var initialFileName: String
    var mimeType: String? = null
}

@InvokeArg
enum class PersistableUriPermissionMode {
    Read,
    Write,
    ReadAndWrite
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
class DeleteArgs {
    lateinit var uri: FileUri
}

@InvokeArg
class RenameArgs {
    lateinit var uri: FileUri
    lateinit var newName: String
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
class CreateFileInDirArgs {
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
class ShareFilesArgs {
    lateinit var uris: Array<FileUri>
    var commonMimeType: String? = null
    var useAppChooser: Boolean = true
    var excludeSelfFromAppChooser: Boolean = true
}

@InvokeArg
class CanShareFilesArgs {
    lateinit var uris: Array<FileUri>
    var commonMimeType: String? = null
}

@InvokeArg
class ViewFileArgs {
    lateinit var uri: FileUri
    var mimeType: String? = null
    var useAppChooser: Boolean = true
    var excludeSelfFromAppChooser: Boolean = true
}

@InvokeArg
class CanViewFileArgs {
    lateinit var uri: FileUri
    var mimeType: String? = null
}

@InvokeArg
class ViewDirArgs {
    lateinit var uri: FileUri
    var useAppChooser: Boolean = true
    var excludeSelfFromAppChooser: Boolean = true
}

@InvokeArg
class EditFileArgs {
    lateinit var uri: FileUri
    var mimeType: String? = null
    var useAppChooser: Boolean = true
    var excludeSelfFromAppChooser: Boolean = true
}

@InvokeArg
class CanEditFileArgs {
    lateinit var uri: FileUri
    var mimeType: String? = null
}

@InvokeArg
class CreateNewMediaStoreFileArgs {
    lateinit var mediaStoreVolumeName: String
    lateinit var relativePath: String
    var mimeType: String? = null
}


@TauriPlugin
class AndroidFsPlugin(private val activity: Activity) : Plugin(activity) {
    private val isVisualMediaPickerAvailable = PickVisualMedia.isPhotoPickerAvailable()
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

    @Suppress("NAME_SHADOWING")
    private fun tryAsDocumentUri(uri: FileUri): Uri? {
        val documentTopTreeUri = uri.documentTopTreeUri
        val uri = Uri.parse(uri.uri)

        if (documentTopTreeUri != null || DocumentsContract.isDocumentUri(activity, uri)) {
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
    fun createNewMediaStoreFile(invoke: Invoke) {
        try {
            if (Build.VERSION.SDK_INT < Build.VERSION_CODES.Q) {
                throw Exception("requires Android 10 (API level 29) or higher")
            }

            CoroutineScope(Dispatchers.IO).launch {
                try {
                    val args = invoke.parseArgs(CreateNewMediaStoreFileArgs::class.java)
                    val res = JSObject().apply {
                        put("uri", AFMediaStore.createNewFile(
                            args.mediaStoreVolumeName,
                            args.relativePath,
                            args.mimeType,
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
            // Tauri は Android7 未満をサポートしていないので本来これはいらないが、警告を消すために書く
            if (Build.VERSION.SDK_INT < Build.VERSION_CODES.N) {
                throw Exception("requires API level 24 or higher")
            }

            CoroutineScope(Dispatchers.IO).launch {
                try {
                    val args = invoke.parseArgs(GetStorageVolumeByPathArgs::class.java)
                    val res = JSObject().apply {
                        put("volume", AFStorageVolume.getStorageVolumeByFileIfAvailable(File(args.path!!), activity))
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
            // Tauri は Android7 未満をサポートしていないので本来これはいらないが、警告を消すために書く
            if (Build.VERSION.SDK_INT < Build.VERSION_CODES.N) {
                throw Exception("requires API level 24 or higher")
            }

            CoroutineScope(Dispatchers.IO).launch {
                try {
                    val res = JSObject().apply {
                        put("volume", AFStorageVolume.getPrimaryStorageVolumeIfAvailable(activity))
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
            // Tauri は Android7 未満をサポートしていないので本来これはいらないが、警告を消すために書く
            if (Build.VERSION.SDK_INT < Build.VERSION_CODES.N) {
                throw Exception("requires API level 24 or higher")
            }

            CoroutineScope(Dispatchers.IO).launch {
                try {
                    val res = JSObject().apply {
                        put("volumes", AFStorageVolume.getAvailableStorageVolumes(activity))
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
            // Tauri は Android7 未満をサポートしていないので本来これはいらないが、警告を消すために書く
            if (Build.VERSION.SDK_INT < Build.VERSION_CODES.N) {
                throw Exception("requires API level 24 or higher")
            }

            CoroutineScope(Dispatchers.IO).launch {
                try {
                    val args = invoke.parseArgs(CheckStorageVolumeAvailableByPathArgs::class.java)
                    val res = JSObject().apply {
                        put("value", AFStorageVolume.checkStorageVolumeAvailableByFile(File(args.path!!) ,activity))
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
            // Tauri は Android7 未満をサポートしていないので本来これはいらないが、警告を消すために書く
            if (Build.VERSION.SDK_INT < Build.VERSION_CODES.N) {
                throw Exception("requires API level 24 or higher")
            }

            CoroutineScope(Dispatchers.IO).launch {
                try {
                    val args = invoke.parseArgs(CheckMediaStoreVolumeNameAvailableArgs::class.java)
                    val res = JSObject().apply {
                        put("value", AFStorageVolume.checkMediaStoreVolumeNameAvailable(args.mediaStoreVolumeName!! ,activity))
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
            // Tauri は Android7 未満をサポートしていないので本来これはいらないが、警告を消すために書く
            if (Build.VERSION.SDK_INT < Build.VERSION_CODES.N) {
                throw Exception("requires API level 24 or higher")
            }

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
            val args = invoke.parseArgs(CreateFileInDirArgs::class.java)

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
            val args = invoke.parseArgs(GetNameArgs::class.java)

            val res = JSObject()
            res.put("name", getFileController(args.uri).getName(args.uri))
            invoke.resolve(res)
        } catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke getFileName."
            invoke.reject(message)
        }
    }

    @Command
    fun getThumbnailToFile(invoke: Invoke) {
        try {
            CoroutineScope(Dispatchers.IO).launch {
                try {
                    val args = invoke.parseArgs(GetThumbnailToFileArgs::class.java)

                    val ok = openFileWt(Uri.parse(args.dest.uri)).use {
                        _getThumbnail(
                            src = args.src,
                            out = it,
                            width = args.width,
                            height = args.height,
                            quality = args.quality,
                            format = args.format
                        )
                    }

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
            CoroutineScope(Dispatchers.IO).launch {
                try {
                    val args = invoke.parseArgs(GetThumbnailArgs::class.java)

                    val base64 = ByteArrayOutputStream().use {
                        if (_getThumbnail(
                            src = args.uri,
                            out = it,
                            width = args.width,
                            height = args.height,
                            quality = args.quality,
                            format = args.format
                        )) {
                            Base64.encodeToString(it.toByteArray(), Base64.NO_WRAP)
                        }
                        else {
                            null
                        }
                    }

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
    
    @Suppress("DEPRECATION")
    private fun _getThumbnail(
        src: FileUri,
        out: OutputStream,
        width: Int,
        height: Int,
        format: String,
        quality: Int,
    ): Boolean {

        var thumbnail: Bitmap? = null
        var img: Bitmap? = null

        try {
            val compressFormat = when (format.lowercase()) {
                "jpeg" -> Bitmap.CompressFormat.JPEG
                "png" -> Bitmap.CompressFormat.PNG
                "webp" -> {
                    if (Build.VERSION_CODES.R < Build.VERSION.SDK_INT) {
                        Bitmap.CompressFormat.WEBP_LOSSY
                    }
                    else {
                        Bitmap.CompressFormat.WEBP
                    }
                }
                else -> throw Exception("Illegal format: $format")
            }

            img = getFileController(src).getThumbnail(
                src,
                width,
                height
            )

            val srcUri = Uri.parse(src.uri)
            if (img == null && srcUri.scheme == "content") {
                try {
                    val mimeType = getFileController(src).getMimeType(src)
                    if (mimeType != null && mimeType.startsWith("video/")) {
                        img = getVideoThumbnail(
                            srcUri, 
                            width, 
                            height
                        )
                    }
                }
                catch (ignore: Exception) {}
            }
            if (img == null) {
                return false
            }

            thumbnail = if (img.width > (width * 2) + 3 || img.height > (height * 2) + 3) {
                val ratio = minOf(width.toFloat() / img.width, height.toFloat() / img.height)
                Bitmap.createScaledBitmap(
                    img,
                    (img.width * ratio).toInt(),
                    (img.height * ratio).toInt(),
                    false
                )
            }
            else {
                img
            }

            if (!thumbnail.compress(compressFormat, quality.coerceIn(0, 100), out)) {
                throw Exception("Failed to compress bitmap")
            }
            out.flush()
            
            return true
        }
        finally {
            thumbnail?.recycle()
            img?.recycle()
        }
    }

    private fun getVideoThumbnail(uri: Uri, width: Int, height: Int): Bitmap? {
        MediaMetadataRetriever().use { mediaMetadataRetriever ->
            mediaMetadataRetriever.setDataSource(activity, uri)
            val thumbnailBytes = mediaMetadataRetriever.embeddedPicture
            thumbnailBytes?.let {
                return if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
                    ImageDecoder.decodeBitmap(ImageDecoder.createSource(it))
                } else {
                    BitmapFactory.decodeByteArray(it, 0, it.size)
                }
            }

            val vw = mediaMetadataRetriever.extractMetadata(MediaMetadataRetriever.METADATA_KEY_VIDEO_WIDTH)?.toFloat()
            val vh = mediaMetadataRetriever.extractMetadata(MediaMetadataRetriever.METADATA_KEY_VIDEO_HEIGHT)?.toFloat()
            if (vw != null && vh != null && (width < vw || height < vh)) {
                val wr = width.toFloat() / vw
                val hr = height.toFloat() / vh
                val ratio = min(wr, hr)
                
                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P) {
                    return mediaMetadataRetriever.getScaledFrameAtTime(
                        -1,
                        OPTION_PREVIOUS_SYNC,
                        (vw * ratio).toInt(),
                        (vh * ratio).toInt()
                    )
                }
            }

            return mediaMetadataRetriever.frameAtTime
        }
    }

    @Command
    fun deleteFile(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(DeleteArgs::class.java)
            getFileController(args.uri).deleteFile(args.uri)
            invoke.resolve()
        } catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke deleteFile."
            invoke.reject(message)
        }
    }

    @Command
    fun deleteEmptyDir(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(DeleteArgs::class.java)
            getFileController(args.uri).deleteEmptyDir(args.uri)
            invoke.resolve()
        } catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke deleteEmptyDir."
            invoke.reject(message)
        }
    }

    @Command
    fun deleteDirAll(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(DeleteArgs::class.java)
            getFileController(args.uri).deleteDirAll(args.uri)
            invoke.resolve()
        } catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke deleteDirAll."
            invoke.reject(message)
        }
    }

    @Command
    fun rename(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(RenameArgs::class.java)
            val uri = getFileController(args.uri).rename(args.uri, args.newName)
            invoke.resolve(uri)
        } catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke rename."
            invoke.reject(message)
        }
    }

    @Command
    fun copyFile(invoke: Invoke) {
        try {
            CoroutineScope(Dispatchers.IO).launch {
                try {
                    val args = invoke.parseArgs(CopyFileArgs::class.java)

                    // グーグルドライブなどの場合、
                    // OutputStream を介して書き込まないと正しく反映されないことがある。
                    // ( 正確にはOutputStream.flush()が必要 )
                    //
                    // https://community.latenode.com/t/csv-export-to-google-drive-results-in-empty-file-but-local-storage-works-fine/10822/3
                    // https://community.latenode.com/t/csv-file-exports-to-local-storage-but-appears-empty-when-saved-to-google-drive-using-action-create-document/29264/4
                    // https://stackoverflow.com/questions/51490194/file-written-using-action-create-document-is-empty-on-google-drive-but-not-local
                    // https://issuetracker.google.com/issues/126362828

                    activity.contentResolver.openInputStream(Uri.parse(args.src.uri))?.use { input ->
                        openFileWt(Uri.parse(args.dest.uri)).use { output ->
                            input.copyTo(output, args.bufferSize ?: DEFAULT_BUFFER_SIZE)
                            output.flush()
                        }
                    }

                    withContext(Dispatchers.Main) {
                        invoke.resolve() 
                    }
                }
                catch (ex: Exception) {
                    withContext(Dispatchers.Main) {
                        val message = ex.message ?: "Failed to invoke copyFile."
                        invoke.reject(message)
                    }
                }
            }
        } 
        catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke copyFile."
            invoke.reject(message)
        }
    }

    @Command
    fun shareFiles(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(ShareFilesArgs::class.java)
            var intent = createShareFilesIntent(
                args.uris.map { Uri.parse(it.uri) },
                args.commonMimeType
            )

            if (args.useAppChooser) {
                intent = createShareFilesIntentChooser(intent, args.excludeSelfFromAppChooser)
            }

            // 対応できるアプリがないときExceptionになる。
            // resolveActivityやqueryIntentActivitiesなどによる判定はAndroid11以降特別な権限が必要。
            try {
                activity.applicationContext.startActivity(intent)
            }
            catch (_: ActivityNotFoundException) {}

            invoke.resolve()
        }
        catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke shareFiles."
            invoke.reject(message)
        }
    }

    @Command
    fun canShareFiles(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(CanShareFilesArgs::class.java)
            val uris = args.uris.map { Uri.parse(it.uri) }

            var ok = true
            for (uri in uris) {
                if (uri.scheme == "file") {
                    ok = false
                    break
                }
            }

            if (ok) {
                val intent = createShareFilesIntent(uris, args.commonMimeType)
                ok = intent.resolveActivity(activity.packageManager) != null
            }

            val res = JSObject()
            res.put("value", ok)
            invoke.resolve(res)
        }
        catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke canShareFiles."
            invoke.reject(message)
        }
    }

    @Command
    fun viewDir(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(ViewDirArgs::class.java)
            var intent = createViewDirIntent(
                Uri.parse(args.uri.uri)
            )

            if (args.useAppChooser) {
                intent = createViewDirIntentChooser(intent, args.excludeSelfFromAppChooser)
            }

            // 対応できるアプリがないときExceptionになる。
            // resolveActivityやqueryIntentActivitiesなどによる判定はAndroid11以降特別な権限が必要。
            try {
                activity.applicationContext.startActivity(intent)
            }
            catch (_: ActivityNotFoundException) {}

            invoke.resolve()
        }
        catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke viewDir."
            invoke.reject(message)
        }
    }

    @Command
    fun viewFile(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(ViewFileArgs::class.java)
            var intent = createViewFileIntent(
                Uri.parse(args.uri.uri),
                args.mimeType
            ) 

            if (args.useAppChooser) {
                intent = createViewFileIntentChooser(intent, args.excludeSelfFromAppChooser)
            }

            // 対応できるアプリがないときExceptionになる。
            // resolveActivityやqueryIntentActivitiesなどによる判定はAndroid11以降特別な権限が必要。
            try {
                activity.applicationContext.startActivity(intent)
            }
            catch (_: ActivityNotFoundException) {}

            invoke.resolve()
        }
        catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke viewFile."
            invoke.reject(message)
        }
    }

    @Command
    fun canViewFile(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(CanViewFileArgs::class.java)
            val uri = Uri.parse(args.uri.uri)
            val ok = when {
                uri.scheme == "file" -> false
                else -> {
                    val intent = createViewFileIntent(uri, args.mimeType)
                    intent.resolveActivity(activity.packageManager) != null
                }
            }

            val res = JSObject()
            res.put("value", ok)
            invoke.resolve(res)
        }
        catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke cabViewFile."
            invoke.reject(message)
        }
    }

    @Command
    fun editFile(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(EditFileArgs::class.java)
            var intent = createEditFileIntent(
                Uri.parse(args.uri.uri),
                args.mimeType
            ) 

            if (args.useAppChooser) {
                intent = createEditFileIntentChooser(intent, args.excludeSelfFromAppChooser)
            }

            // 対応できるアプリがないときExceptionになる。
            // resolveActivityやqueryIntentActivitiesなどによる判定はAndroid11以降特別な権限が必要。
            try {
                activity.applicationContext.startActivity(intent)
            }
            catch (_: ActivityNotFoundException) {}

            invoke.resolve()
        }
        catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke editFile."
            invoke.reject(message)
        }
    }

    @Command
    fun canEditFile(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(CanEditFileArgs::class.java)
            val uri = Uri.parse(args.uri.uri)
            val ok = when {
                uri.scheme == "file" -> false
                else -> {
                    val intent = createEditFileIntent(uri, args.mimeType)
                    intent.resolveActivity(activity.packageManager) != null
                }
            }

            val res = JSObject()
            res.put("value", ok)
            invoke.resolve(res)
        }
        catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke canEditFile."
            invoke.reject(message)
        }
    }

    @Command
    fun showManageDirDialog(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(ShowManageDirDialogArgs::class.java)
            val intent = Intent(Intent.ACTION_OPEN_DOCUMENT_TREE)
            
            args.initialLocation?.let { uri ->
                tryAsDocumentUri(uri)?.let { dUri ->
                    if (Build.VERSION_CODES.O <= Build.VERSION.SDK_INT){
                        intent.putExtra(DocumentsContract.EXTRA_INITIAL_URI, dUri)
                    }
                }
            }

            startActivityForResult(invoke, intent, "handleShowManageDirDialog")
        } catch (ex: Exception) {
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
            val args = invoke.parseArgs(GetMimeTypeArgs::class.java)

            val res = JSObject()
            res.put("value", getFileController(args.uri).getMimeType(args.uri))
            invoke.resolve(res)
        } catch (ex: Exception) {
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
                tryAsDocumentUri(uri)?.let { dUri ->
                    if (Build.VERSION_CODES.O <= Build.VERSION.SDK_INT){
                        intent.putExtra(DocumentsContract.EXTRA_INITIAL_URI, dUri)
                    }
                }
            }

            startActivityForResult(invoke, intent, "handleShowOpenFileAndVisualMediaDialog")
        } catch (ex: Exception) {
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
                tryAsDocumentUri(uri)?.let { dUri ->
                    if (Build.VERSION_CODES.O <= Build.VERSION.SDK_INT){
                        intent.putExtra(DocumentsContract.EXTRA_INITIAL_URI, dUri)
                    }
                }
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
            res.put("value", isVisualMediaPickerAvailable)
            invoke.resolve(res)
        } catch (ex: java.lang.Exception) {
            val message = ex.message ?: "Failed to invoke isVisualMediaDialogAvailable."
            invoke.reject(message)
        }
    }

    @SuppressLint("Recycle")
    @Command
    fun getFileDescriptor(invoke: Invoke) {
        try {
            CoroutineScope(Dispatchers.IO).launch {
                try {
                    val args = invoke.parseArgs(GetFileDescriptorArgs::class.java)
                    val fd = activity.contentResolver
                        .openAssetFileDescriptor(Uri.parse(args.uri.uri), args.mode)!!
                        .parcelFileDescriptor
                        .detachFd()

                    val res = JSObject()
                    res.put("fd", fd)

                    withContext(Dispatchers.Main) {
                        invoke.resolve(res)
                    }
                } catch (ex: Exception) {
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
                            val afd = activity.contentResolver.openAssetFileDescriptor(uri, m)
                            fd = afd?.parcelFileDescriptor?.detachFd()
                        }
                        catch (ignore: Exception) {}

                        if (fd != null) break
                    }
                    
                    val res = JSObject()
                    res.put("fd", fd ?: throw Exception("Failed to get FileDescriptor with ${args.modes.toString()}"))
                    res.put("mode", mode!!)

                    withContext(Dispatchers.Main) {
                        invoke.resolve(res)
                    }
                } catch (ex: Exception) {
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

    @Command
    fun getApiLevel(invoke: Invoke) {
        try {
            val res = JSObject()
            res.put("value", Build.VERSION.SDK_INT)
            invoke.resolve(res)
        } 
        catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke getApiLevel."
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

    private fun createViewDirIntent(
        uri: Uri,
    ): Intent {

        val mimeType = DocumentsContract.Document.MIME_TYPE_DIR
        var isAvailable = DocumentsContract.isDocumentUri(activity, uri)
        if (!isAvailable && Build.VERSION_CODES.N <= Build.VERSION.SDK_INT) {
            isAvailable = DocumentsContract.isTreeUri(uri)
        }

        if (!isAvailable) {
            throw Exception("This is not a available type: $uri")
        }

        return Intent(Intent.ACTION_VIEW)
            .setDataAndType(uri, mimeType)
            .addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
            .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
    }

    private fun createViewDirIntentChooser(
        viewDirIntent: Intent,
        excludeSelfFromAppChooser: Boolean
    ): Intent {

        val chooser = Intent.createChooser(viewDirIntent, "")
            .addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
            .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)

        if (excludeSelfFromAppChooser && Build.VERSION.SDK_INT >= Build.VERSION_CODES.N) {
            chooser.putExtra(Intent.EXTRA_EXCLUDE_COMPONENTS, arrayOf(activity.componentName))
        }

        return chooser
    }

    private fun createViewFileIntent(
        uri: Uri,
        mimeType: String?
    ): Intent {

        return Intent(Intent.ACTION_VIEW)
            .setDataAndType(uri, mimeType ?: activity.contentResolver.getType(uri))
            .addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
            .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
    }

    private fun createViewFileIntentChooser(
        viewFileIntent: Intent,
        excludeSelfFromAppChooser: Boolean
    ): Intent {

        val chooser = Intent.createChooser(viewFileIntent, "")
            .addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
            .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)

        if (excludeSelfFromAppChooser && Build.VERSION.SDK_INT >= Build.VERSION_CODES.N) {
            chooser.putExtra(Intent.EXTRA_EXCLUDE_COMPONENTS, arrayOf(activity.componentName))
        }

        return chooser
    }

    private fun createShareFilesIntent(
        uris: List<Uri>,
        commonMimeType: String? = null
    ): Intent {

        if (uris.isEmpty()) throw IllegalArgumentException("uris must not be empty")

        val mimeType = commonMimeType ?: run {
            val types = uris.map { activity.contentResolver.getType(it) ?: "*/*" }
            getCommonMimeType(types)
        }

        val builder = ShareCompat.IntentBuilder(activity)
            .setType(mimeType)

        for (uri in uris) {
            builder.addStream(uri)
        }

        return builder.intent
            .addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
            .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
    }

    private fun createShareFilesIntentChooser(
        shareFileIntent: Intent,
        excludeSelfFromAppChooser: Boolean
    ): Intent {

        val chooser = Intent.createChooser(shareFileIntent, "")
            .addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
            .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)

        if (excludeSelfFromAppChooser && Build.VERSION.SDK_INT >= Build.VERSION_CODES.N) {
            chooser.putExtra(Intent.EXTRA_EXCLUDE_COMPONENTS, arrayOf(activity.componentName))
        }

        return chooser
    }

    private fun createEditFileIntent(
        uri: Uri,
        mimeType: String?
    ): Intent {

        return Intent(Intent.ACTION_EDIT)
            .setDataAndType(uri, mimeType ?: activity.contentResolver.getType(uri))
            .addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
            .addFlags(Intent.FLAG_GRANT_WRITE_URI_PERMISSION)
            .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
    }

    private fun createEditFileIntentChooser(
        editFileIntent: Intent,
        excludeSelfFromAppChooser: Boolean
    ): Intent {

        val chooser = Intent.createChooser(editFileIntent, "")
            .addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
            .addFlags(Intent.FLAG_GRANT_WRITE_URI_PERMISSION)
            .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)

        if (excludeSelfFromAppChooser && Build.VERSION.SDK_INT >= Build.VERSION_CODES.N) {
            chooser.putExtra(Intent.EXTRA_EXCLUDE_COMPONENTS, arrayOf(activity.componentName))
        }

        return chooser
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