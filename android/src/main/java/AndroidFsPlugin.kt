package com.plugin.android_fs

import android.annotation.SuppressLint
import android.app.Activity
import android.content.Intent
import android.net.Uri
import android.os.Build
import android.os.Environment
import android.provider.DocumentsContract
import android.provider.MediaStore
import androidx.activity.result.ActivityResult
import androidx.activity.result.PickVisualMediaRequest
import androidx.activity.result.contract.ActivityResultContracts.PickMultipleVisualMedia
import androidx.activity.result.contract.ActivityResultContracts.PickVisualMedia
import app.tauri.Logger
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

@InvokeArg
class GetFileDescriptorArgs {
    lateinit var mode: String
    lateinit var uri: FileUri
}

@InvokeArg
class GetNameArgs {
    lateinit var uri: FileUri
}

@InvokeArg
class ShowOpenFileDialogArgs {
    lateinit var mimeTypes: Array<String>
    var multiple: Boolean = false
    var initialLocation: FileUri? = null
}

@InvokeArg
class ShowOpenVisualMediaDialogArgs {
    lateinit var target: VisualMediaPickerType
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
    lateinit var mimeType: String
}

@InvokeArg
enum class PersistableUriPermissionMode {
    Read,
    Write,
    ReadAndWrite
}

@InvokeArg
enum class VisualMediaPickerType {
    ImageOnly,
    VideoOnly,
    ImageAndVideo
}

@InvokeArg
class GetPublicDirInfo {
    lateinit var dir: BaseDir
    lateinit var dirType: ContentType
}

@InvokeArg
class GetMimeTypeArgs {
    lateinit var uri: FileUri
}

@InvokeArg
enum class ContentType {
    Image,
    Video,
    Audio,
    GeneralPurpose
}

@InvokeArg
enum class BaseDir {
    Pictures,
    Movies,
    Music,
    Alarms,
    Audiobooks,
    Notifications,
    Podcasts,
    Ringtones,
    Recordings,
    DCIM,
    Documents,
    Download,
}

@InvokeArg
class RemoveFileArgs {
    lateinit var uri: FileUri
}

@InvokeArg
class ReadDirArgs {
    lateinit var uri: FileUri
}

@InvokeArg
class CreateFileInDirArgs {
    lateinit var dir: FileUri
    lateinit var relativePath: String
    lateinit var mimeType: String
}

@InvokeArg
class FileUri {
    lateinit var uri: String
    var documentTopTreeUri: String? = null
}

@InvokeArg
class TakePersistableUriPermissionArgs {
    lateinit var uri: FileUri
    lateinit var mode: PersistableUriPermissionMode
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

@TauriPlugin
class AndroidFsPlugin(private val activity: Activity) : Plugin(activity) {
    private val isVisualMediaPickerAvailable = PickVisualMedia.isPhotoPickerAvailable()
    private val documentFileController = DocumentFileController(activity)
    private val mediaFileController = MediaFileController(activity)
    private val rawFileController = RawFileController()

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
            else -> throw Error("Unsupported uri: $uri")
        }
    }

    private fun tryAsDocumentUri(uri: FileUri): Uri? {
        val documentTopTreeUri = uri.documentTopTreeUri
        val uri = Uri.parse(uri.uri)

        when {
            (documentTopTreeUri != null || DocumentsContract.isDocumentUri(activity, uri)) -> {
                return uri
            }
            (uri.authority == MediaStore.AUTHORITY) -> {
                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                    try {
                        return MediaStore.getDocumentUri(activity, uri)
                    }
                    catch (ignore: Exception) {}
                }
            }
        }

        return null
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
            Logger.error(message)
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
            Logger.error(message)
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
            Logger.error(message)
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

            val flag = when (args.mode) {
                PersistableUriPermissionMode.Read -> Intent.FLAG_GRANT_READ_URI_PERMISSION
                PersistableUriPermissionMode.Write -> Intent.FLAG_GRANT_WRITE_URI_PERMISSION
                PersistableUriPermissionMode.ReadAndWrite -> Intent.FLAG_GRANT_READ_URI_PERMISSION or Intent.FLAG_GRANT_WRITE_URI_PERMISSION
            }

            activity.contentResolver.takePersistableUriPermission(uri, flag)    

            invoke.resolve()
        }
        catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke takePersistableUriPermission."
            Logger.error(message)
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
            Logger.error(message)
            invoke.reject(message)
        }
    }

    @Command
    fun getPublicDirInfo(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(GetPublicDirInfo::class.java)

            val dirName = when (args.dir) {
                BaseDir.Pictures -> Environment.DIRECTORY_PICTURES
                BaseDir.DCIM -> Environment.DIRECTORY_DCIM
                BaseDir.Movies -> Environment.DIRECTORY_MOVIES
                BaseDir.Music -> Environment.DIRECTORY_MUSIC
                BaseDir.Alarms -> Environment.DIRECTORY_ALARMS
                BaseDir.Audiobooks -> if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                    Environment.DIRECTORY_AUDIOBOOKS
                } else {
                    throw Error("Environment.DIRECTORY_AUDIOBOOKS isn't available on Android 9 (API level 28) and lower.")
                }
                BaseDir.Notifications -> Environment.DIRECTORY_NOTIFICATIONS
                BaseDir.Podcasts -> Environment.DIRECTORY_PODCASTS
                BaseDir.Ringtones -> Environment.DIRECTORY_RINGTONES
                BaseDir.Documents -> Environment.DIRECTORY_DOCUMENTS
                BaseDir.Download -> Environment.DIRECTORY_DOWNLOADS
                BaseDir.Recordings -> if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
                    Environment.DIRECTORY_RECORDINGS
                } else {
                    throw Error("Environment.DIRECTORY_RECORDINGS isn't available on Android 11 (API level 30) and lower.")
                }
            }

            val res = JSObject()
            res.put("name", dirName)

            val uri = when (args.dirType) {
                ContentType.Image -> {
                    if (Build.VERSION_CODES.Q <= Build.VERSION.SDK_INT) {
                        MediaStore.Images.Media.getContentUri(MediaStore.VOLUME_EXTERNAL_PRIMARY)
                    } else {
                        MediaStore.Images.Media.EXTERNAL_CONTENT_URI
                    }
                }
                ContentType.Video -> {
                    if (Build.VERSION_CODES.Q <= Build.VERSION.SDK_INT) {
                        MediaStore.Video.Media.getContentUri(MediaStore.VOLUME_EXTERNAL_PRIMARY)
                    } else {
                        MediaStore.Video.Media.EXTERNAL_CONTENT_URI
                    }
                }
                ContentType.Audio -> {
                    if (Build.VERSION_CODES.Q <= Build.VERSION.SDK_INT) {
                        MediaStore.Audio.Media.getContentUri(MediaStore.VOLUME_EXTERNAL_PRIMARY)
                    } else {
                        MediaStore.Audio.Media.EXTERNAL_CONTENT_URI
                    }
                }
                ContentType.GeneralPurpose -> {
                    if (Build.VERSION_CODES.Q <= Build.VERSION.SDK_INT) {
                        MediaStore.Files.getContentUri(MediaStore.VOLUME_EXTERNAL_PRIMARY)
                    } else {
                        MediaStore.Files.getContentUri("external")
                    }
                }
            }

            res.put("uri", uri)

            invoke.resolve(res)
        } catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke getPublicDirInfo"
            Logger.error(message)
            invoke.reject(message)
        }
    }

    @Command
    fun createFile(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(CreateFileInDirArgs::class.java)

            CoroutineScope(Dispatchers.IO).launch {
                try {
                   val res = getFileController(args.dir)
                        .createFile(args.dir, args.relativePath, args.mimeType)

                    // 必要ないかもしれないが念の為
                    withContext(Dispatchers.Main) {
                        invoke.resolve(res) 
                    }
                }
                catch (ex: Exception) {
                    withContext(Dispatchers.Main) {
                        val message = ex.message ?: "Failed to invoke createFile."
                        Logger.error(message)
                        invoke.reject(message)
                    }
                }
            }
        } catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke createFileInDir."
            Logger.error(message)
            invoke.reject(message)
        }
    }

    @Command
    fun readDir(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(ReadDirArgs::class.java)
            
            CoroutineScope(Dispatchers.IO).launch {
                try {
                    val res = JSObject()
                    res.put("entries", getFileController(args.uri).readDir(args.uri))
                   
                    // 必要ないかもしれないが念の為
                    withContext(Dispatchers.Main) {
                        invoke.resolve(res) 
                    }
                }
                catch (ex: Exception) {
                    withContext(Dispatchers.Main) {
                        val message = ex.message ?: "Failed to invoke readDir."
                        Logger.error(message)
                        invoke.reject(message)
                    }
                }
            }
        } catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke readDir."
            Logger.error(message)
            invoke.reject(message)
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
            Logger.error(message)
            invoke.reject(message)
        }
    }

    @Command
    fun delete(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(RemoveFileArgs::class.java)
            getFileController(args.uri).delete(args.uri)
            invoke.resolve()
        } catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke removeFile."
            Logger.error(message)
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
                    intent.putExtra(DocumentsContract.EXTRA_INITIAL_URI, dUri)
                }
            }

            startActivityForResult(invoke, intent, "handleShowManageDirDialog")
        } catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke showManageDirDialog."
            Logger.error(message)
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
            Logger.error(message)
            invoke.reject(message)
        }
    }

    @Command
    fun getPrivateBaseDirAbsolutePaths(invoke: Invoke) {
        try {
            val res = JSObject()
            res.put("data", activity.filesDir.absolutePath)
            res.put("cache", activity.cacheDir.absolutePath)
            invoke.resolve(res)
        } catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke getPrivateBaseDirAbsolutePaths."
            Logger.error(message)
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
            Logger.error(message)
            invoke.reject(message)
        }
    }

    @Command
    fun showOpenFileDialog(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(ShowOpenFileDialogArgs::class.java)
            var intent = createFilePickerIntent(args.mimeTypes, args.multiple)

            args.initialLocation?.let { uri ->
                tryAsDocumentUri(uri)?.let { dUri ->
                    intent = intent.putExtra(DocumentsContract.EXTRA_INITIAL_URI, dUri)
                }
            }

            startActivityForResult(invoke, intent, "handleShowOpenFileAndVisualMediaDialog")
        } catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke showOpenFileDialog."
            Logger.error(message)
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
            Logger.error(message)
            invoke.reject(message)
        }
    }

    @Command
    fun showSaveFileDialog(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(ShowSaveFileDialogArgs::class.java)

            val intent = Intent(Intent.ACTION_CREATE_DOCUMENT)

            intent.setType(args.mimeType)
            intent.addCategory(Intent.CATEGORY_OPENABLE)
            intent.putExtra(Intent.EXTRA_TITLE, args.initialFileName)
            
            args.initialLocation?.let { uri ->
                tryAsDocumentUri(uri)?.let { dUri ->
                    intent.putExtra(DocumentsContract.EXTRA_INITIAL_URI, dUri)
                }
            }

            startActivityForResult(invoke, intent, "handleShowSaveFileDialog")
        } catch (ex: Exception) {
            val message = ex.message ?: "Failed to pick save file"
            Logger.error(message)
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

                        // file descriptor を介して google drive 上のファイルに書き込めない(読み取りはできる)ため、nullを返す。
                        // google drive 上のファイルに書き込みたい場合は output streamを用いる必要がある。
                        if (uri == null || uri.authority == "com.google.android.apps.docs.storage") {
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
            Logger.error(message)
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
            Logger.error(message)
            invoke.reject(message)
        }
    }

    @SuppressLint("Recycle")
    @Command
    fun getFileDescriptor(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(GetFileDescriptorArgs::class.java)
            val fd = activity.contentResolver
                .openAssetFileDescriptor(Uri.parse(args.uri.uri), args.mode)!!
                .parcelFileDescriptor
                .detachFd()

            val res = JSObject()
            res.put("fd", fd)
            invoke.resolve(res)
        } catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke getFileDescriptor."
            Logger.error(message)
            invoke.reject(message)
        }
    }

    @Command
    fun isAudiobooksDirAvailable(invoke: Invoke) {
        try {
            val res = JSObject()
            res.put("value", Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q)
            invoke.resolve(res)
        } catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke isAudiobooksDirAvailable."
            Logger.error(message)
            invoke.reject(message)
        }
    }

    @Command
    fun isRecordingsDirAvailable(invoke: Invoke) {
        try {
            val res = JSObject()
            res.put("value", Build.VERSION.SDK_INT >= Build.VERSION_CODES.S)
            invoke.resolve(res)
        } catch (ex: Exception) {
            val message = ex.message ?: "Failed to invoke isRecordingsDirAvailable."
            Logger.error(message)
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
            Logger.error(message)
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

    private fun createVisualMediaPickerIntent(
        multiple: Boolean,
        target: VisualMediaPickerType
    ): Intent {

        val req = PickVisualMediaRequest(
            when (target) {
                VisualMediaPickerType.ImageOnly -> PickVisualMedia.ImageOnly
                VisualMediaPickerType.VideoOnly -> PickVisualMedia.VideoOnly
                VisualMediaPickerType.ImageAndVideo -> PickVisualMedia.ImageAndVideo
            }
        )

        return when (multiple) {
            true -> PickMultipleVisualMedia().createIntent(activity, req)
            false -> PickVisualMedia().createIntent(activity, req)
        }
    }
}