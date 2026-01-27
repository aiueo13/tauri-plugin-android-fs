@file:Suppress("NAME_SHADOWING")

package com.plugin.android_fs

import android.content.ContentResolver
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
import java.io.IOException


class AFMediaStore private constructor() { companion object {

    @Synchronized
    fun createNewFile(
        volumeName: String?,
        relativePath: String,
        mimeType: String?,
        isPending: Boolean,
        ctx: Context
    ): JSObject {

        val uri = when {
            // Q は Android 10
            Build.VERSION_CODES.Q <= Build.VERSION.SDK_INT -> {
                _createNewFile(
                    volumeName ?: MediaStore.VOLUME_EXTERNAL_PRIMARY,
                    relativePath,
                    mimeType,
                    isPending,
                    ctx
                )
            }
            else -> {
                if (volumeName != null) {
                    throw Exception("volume name is available for Android 10 or higher")
                }

                _createNewFileLegacy(relativePath, mimeType, ctx)
            }
        }

        return AFJSObject.createFileUri(uri)
    }

    @Synchronized
    fun delete(
        uri: AFUri,
        ctx: Context
    ) {

        when {
            // Q は Android 10
            Build.VERSION_CODES.Q <= Build.VERSION.SDK_INT -> {
                _delete(uri, ctx)
            }
            else -> {
                _deleteLegacy(uri, ctx)
            }
        }
    }

    @Synchronized
    fun rename(
        uri: AFUri,
        newName: String,
        ctx: Context
    ) {

        when {
            // Q は Android 10
            Build.VERSION_CODES.Q <= Build.VERSION.SDK_INT -> {
                _rename(uri, newName, ctx)
            }
            else -> {
                _renameLegacy(uri, newName, ctx)
            }
        }
    }

    // Q は Android 10
    @RequiresApi(Build.VERSION_CODES.Q)
    fun setPending(
        fileUri: AFUri,
        isPending: Boolean,
        ctx: Context
    ) {

        // Android 10 でも IS_PENDING は導入されているが、
        // - pending 中のファイルが他アプリから参照可能な場合がある
        // - pending のエントリが存在すると、同名ファイル作成時に 自動リネームされずエラーになる場合がある
        // などという問題がある。
        // そのため、IS_PENDING は挙動が安定している Android 11 (R) 以降でのみ使用する。
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.R) {
            return
        }

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
                if (it.moveToFirst()) {
                    val ci = it.getColumnIndexOrThrow(MediaStore.MediaColumns.IS_PENDING)
                    if (it.getInt(ci) == pending) {
                        return
                    }
                }
            }

            throw Exception("no file or permission: ${fileUri.uri}")
        }
    }

    fun getDisplayName(uri: AFUri, ctx: Context): String {
        ctx.contentResolver.query(
            Uri.parse(uri.uri),
            arrayOf(MediaStore.MediaColumns.DISPLAY_NAME),
            null,
            null,
            null
        )?.use {

            if (it.moveToFirst()) {
                return it.getString(it.getColumnIndexOrThrow(MediaStore.MediaColumns.DISPLAY_NAME))
            }
        }

        throw Exception("no file or permission: ${uri.uri}")
    }

    fun getMimeType(uri: AFUri, ctx: Context): String {
        ctx.contentResolver.query(
            Uri.parse(uri.uri),
            arrayOf(MediaStore.MediaColumns.MIME_TYPE),
            null,
            null,
            null
        )?.use {

            if (it.moveToFirst()) {
                return it.getString(it.getColumnIndexOrThrow(MediaStore.MediaColumns.MIME_TYPE))
            }
        }

        throw Exception("no file or permission: ${uri.uri}")
    }

    fun getAbsolutePath(
        fileUri: AFUri,
        ctx: Context
    ): String {

        val uri = Uri.parse(fileUri.uri)

        val projection = arrayOf(MediaStore.MediaColumns.DATA)

        ctx.contentResolver.query(uri, projection, null, null, null)?.use {
            if (it.moveToFirst()) {
                val ci = it.getColumnIndexOrThrow(MediaStore.MediaColumns.DATA)
                return it.getString(ci)
            }
        }

        throw Exception("no file or permission: $uri")
    }

    fun getAbsolutePathAndMimeType(
        fileUri: AFUri,
        ctx: Context
    ): Pair<String, String> {

        val uri = Uri.parse(fileUri.uri)

        val projection = arrayOf(
            MediaStore.MediaColumns.DATA,
            MediaStore.MediaColumns.MIME_TYPE
        )

        ctx.contentResolver.query(uri, projection, null, null, null)?.use {
            if (it.moveToFirst()) {
                val pathC = it.getColumnIndexOrThrow(MediaStore.MediaColumns.DATA)
                val mimeTypeC = it.getColumnIndexOrThrow(MediaStore.MediaColumns.MIME_TYPE)
                return Pair(it.getString(pathC), it.getString(mimeTypeC))
            }
        }

        throw Exception("no file or permission: $uri")
    }

    fun scanFileWithIgnoringResult(
        file: File,
        mimeType: String?,
        ctx: Context
    ) {

        val mimeType = mimeType ?: AFUtils.guessFileMimeTypeFromExtensionOrNull(file)
        MediaScannerConnection.scanFile(ctx, arrayOf(file.path), arrayOf(mimeType), null)
    }

    fun scanFile(
        file: File,
        mimeType: String?,
        callback: (Uri) -> Unit,
        errCallback: (Exception) -> Unit,
        ctx: Context
    ) {

        // static method の MediaScannerConnection.scanFile だと失敗時に callback が呼ばれないことがあるので使わない

        try {
            val path = file.absolutePath
            val mimeType = mimeType ?: AFUtils.guessFileMimeTypeFromExtensionOrNull(file)

            lateinit var ms: MediaScannerConnection
            val client = object : MediaScannerConnection.MediaScannerConnectionClient {

                override fun onMediaScannerConnected() {
                    // クラッシュを防ぐために例外は握りつぶす

                    try {
                        ms.scanFile(path, mimeType)
                    }
                    catch (e: Exception) {
                        try {
                            errCallback(e)
                        }
                        catch (_: Exception) {}

                        try {
                            ms.disconnect()
                        }
                        catch (_: Exception) {}
                    }
                }

                override fun onScanCompleted(path: String, uri: Uri?) {
                    // クラッシュを防ぐために例外は握りつぶす

                    try {
                        if (uri != null) {
                            callback(uri)
                        }
                        else {
                            errCallback(Exception("Media scan failed for $path. The file may not exist, you may lack permission, or the format is unsupported."))
                        }
                    }
                    catch (_: Exception) {}

                    try {
                        ms.disconnect()
                    }
                    catch (_: Exception) {}
                }
            }

            ms = MediaScannerConnection(ctx, client)
            ms.connect()
        }
        catch (e: Exception) {
            errCallback(e)
        }
    }

    fun isMediaStoreFile(uri: Uri): Boolean {
        return uri.scheme == ContentResolver.SCHEME_CONTENT
                && uri.authority == MediaStore.AUTHORITY
    }
}}

/**
 * Android 10 ~
 */
@RequiresApi(Build.VERSION_CODES.Q)
fun _delete(
    uri: AFUri,
    ctx: Context
) {

    if (ctx.contentResolver.delete(Uri.parse(uri.uri), null, null) <= 0) {
        throw Exception("No file or permission: ${uri.uri}")
    }
}

/**
 * Android 7 ~ 9
 */
fun _deleteLegacy(
    uri: AFUri,
    ctx: Context
) {

    val path = AFMediaStore.getAbsolutePath(uri, ctx)

    if (!File(path).delete()) {
        throw Exception("No file or permission: ${uri.uri}, $path")
    }
    if (ctx.contentResolver.delete(Uri.parse(uri.uri), null, null) <= 0) {
        throw Exception("No file or permission: ${uri.uri}, $path")
    }
}

/**
 * Android 10 ~
 */
@RequiresApi(Build.VERSION_CODES.Q)
fun _rename(
    uri: AFUri,
    newName: String,
    ctx: Context
) {

    if (AFMediaStore.getDisplayName(uri, ctx) != newName) {
        val updated = ctx.contentResolver.update(
            Uri.parse(uri.uri),
            ContentValues().apply {
                put(MediaStore.MediaColumns.DISPLAY_NAME, newName)
            },
            null,
            null
        )

        if (updated == 0) {
            throw Exception("No file or permission: ${uri.uri}")
        }
    }
}

/**
 * Android 7 ~ 9
 */
fun _renameLegacy(
    uri: AFUri,
    newName: String,
    ctx: Context
) {

    // Q は Android 10
    val e = AFMediaStore.getAbsolutePathAndMimeType(uri, ctx)
    val srcPath = e.first
    val mimeType = e.second
    val srcFile = File(srcPath)

    val destPath = when {
        srcFile.name != newName -> {
            val result = useNewFilePathWithNameSuffixFallback(
                File(srcFile.parent, newName).path,
                mimeType,
                { path ->
                    val renamed = srcFile.renameTo(File(path))
                    Result.Ok(Pair(renamed, path))
                }
            )
            if (!result.first) {
                throw Exception("no file or permission: ${uri.uri}, $srcPath")
            }
            result.second
        }
        else -> {
            srcFile.path
        }
    }

    val newName = File(destPath).name

    if (srcPath != destPath || AFMediaStore.getDisplayName(uri, ctx) != newName) {
        val updated = ctx.contentResolver.update(
            Uri.parse(uri.uri),
            ContentValues().apply {
                put(MediaStore.MediaColumns.DATA, destPath)
                put(MediaStore.MediaColumns.DISPLAY_NAME, newName)
            },
            null,
            null
        )

        if (updated == 0) {
            throw Exception("No file or permission: ${uri.uri}")
        }

        AFMediaStore.scanFileWithIgnoringResult(File(destPath), mimeType, ctx)
    }
}

/**
 * Android 10 ~
 */
@RequiresApi(Build.VERSION_CODES.Q)
private fun _createNewFile(
    volumeName: String,
    relativePath: String,
    mimeType: String?,
    isPending: Boolean,
    ctx: Context
): Uri {

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

    return ctx.contentResolver.insert(
        getContentUri(volumeName, relativePath, mimeType),
        ContentValues().apply {
            put(MediaStore.MediaColumns.DISPLAY_NAME, displayName)
            put(MediaStore.MediaColumns.MIME_TYPE, mimeType)
            put(MediaStore.MediaColumns.RELATIVE_PATH, "$parentRelativePath/")

            // Android 10 でも IS_PENDING は導入されているが、
            // - pending 中のファイルが他アプリから参照可能な場合がある
            // - pending のエントリが存在すると、同名ファイル作成時に 自動リネームされずエラーになる場合がある
            // などという問題がある。
            // そのため、IS_PENDING は挙動が安定している Android 11 (R) 以降でのみ使用する。
            if (isPending && Build.VERSION_CODES.R <= Build.VERSION.SDK_INT) {
                put(MediaStore.MediaColumns.IS_PENDING, 1)
            }
        }
    ) ?: throw Exception("Failed to create file")
}

/**
 * Android 7 ~ 9
 */
private fun _createNewFileLegacy(
    relativePath: String,
    mimeType: String?,
    ctx: Context
): Uri {

    val relativePath = relativePath.trimStart('/')
    val path = Environment.getExternalStorageDirectory().absolutePath + "/" + relativePath
    val mimeType = mimeType ?: AFUtils.guessFileMimeTypeFromExtension(File(path))
    val baseContentUri = getBaseContentUriLegacy(relativePath, mimeType)

    return useNewFilePathWithNameSuffixFallback(
        path,
        mimeType,
        { path ->
            val name = File(path).name
            val uri = ctx.contentResolver.insert(
                baseContentUri,
                ContentValues().apply {
                    put(MediaStore.MediaColumns.DISPLAY_NAME, name)
                    put(MediaStore.MediaColumns.MIME_TYPE, mimeType)
                    put(MediaStore.MediaColumns.DATA, path)
                }
            )

            when (uri != null) {
                true -> Result.Ok(uri)
                false -> Result.No
            }
        },
        // 上の usePath では File.exists により存在しないことが確認されたパスが使われるが、
        // TOCTOU により既に存在する可能性もある。
        // このとき contentResolver.insert は null を返すが、これが10回続けば例外を投げるようにする。
        // ただしこれは滅多に起こらないはず。
        // 制限をつけるのは何らかのエラーで contentResolver.insert が常に null を返すことがあるかもしれないため。
        10
    )
}

// Q は Android 10
@RequiresApi(Build.VERSION_CODES.Q)
private fun getContentUri(volumeName: String, relativePath: String, mimeType: String): Uri {
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

private fun getBaseContentUriLegacy(relativePath: String, mimeType: String): Uri {
    val topDir = relativePath.trimStart('/').split("/").firstOrNull() ?: ""

    // MediaStore.Images.Media.getContentUri(volumeName) などは対応するフォルダ ( Pictures, DCIM など) 用なので、
    // それ以外のフォルダを用いる場合は MediaStore.Downloads か MediaStore.Files の URI を用いる。
    if (Environment.DIRECTORY_DOWNLOADS == topDir) {
        return MediaStore.Files.getContentUri("external")
    }
    if (Environment.DIRECTORY_DOCUMENTS == topDir) {
        return MediaStore.Files.getContentUri("external")
    }

    // DCIM と Pictures フォルダが画像と動画の両方に対応しているのでフォルダからではなく MIME type から判定する
    if (mimeType.startsWith("image/")) {
        return MediaStore.Images.Media.EXTERNAL_CONTENT_URI
    }
    if (mimeType.startsWith("video/")) {
        return MediaStore.Video.Media.EXTERNAL_CONTENT_URI
    }
    if (mimeType.startsWith("audio/")) {
        return MediaStore.Audio.Media.EXTERNAL_CONTENT_URI
    }

    return MediaStore.Files.getContentUri("external")
}

sealed class Result<out T> {
    data class Ok<out T>(val value: T) : Result<T>()
    object No : Result<Nothing>()
}

private fun <T> useNewFilePathWithNameSuffixFallback(
    path: String,
    mimeType: String? = null,
    usePath: (String) -> Result<T>,
    limitOfUsePath: Int? = null
): T {

    val file = File(path)
    val parentDirPath = file.parent ?: throw IllegalArgumentException("Parent dir not exists: $path")
    val parentDir = File(parentDirPath)
    if (!parentDir.exists() && !parentDir.mkdirs()) {
        throw IOException("Failed to create parent directories: $parentDirPath")
    }

    val fileBaseName = file.nameWithoutExtension
    val e = file.extension
    val fileExt = if (e.isEmpty() && mimeType != null) AFUtils.getExtensionFromMimeType(mimeType) else e

    fun buildPath(i: Int): String {
        var name = fileBaseName
        if (i != 0) name += "($i)"
        if (!fileExt.isNullOrEmpty()) name += ".$fileExt"
        return File(parentDirPath, name).path
    }

    var usePathCount = 0
    var i = 0
    var pathToCreate = buildPath(0)

    while (true) {
        if (!File(pathToCreate).exists()) {
            val t = usePath(pathToCreate)
            if (t is Result.Ok<T>) return t.value

            usePathCount++
            if (limitOfUsePath != null && usePathCount >= limitOfUsePath) {
                throw Exception("Exceeded the limit of $limitOfUsePath attempts to find an unused path")
            }
        }

        i++
        pathToCreate = buildPath(i)
    }
}