package com.plugin.android_fs

import android.net.Uri
import android.graphics.Bitmap
import android.webkit.MimeTypeMap
import android.util.Size
import android.media.ThumbnailUtils
import app.tauri.plugin.JSArray
import app.tauri.plugin.JSObject
import java.io.File

class RawFileController: FileController {

    override fun getMimeType(uri: FileUri): String? {
        return _getMimeType(File(Uri.parse(uri.uri).path!!))
    }

    override fun getName(uri: FileUri): String {
        return File(Uri.parse(uri.uri).path!!).name
    }

    override fun readDir(dirUri: FileUri): JSArray {
        val dir = File(Uri.parse(dirUri.uri).path!!)
        val buffer = JSArray()

        for (file in dir.listFiles()!!) {
            val uriObj = JSObject()
            uriObj.put("uri", file.toURI())
            uriObj.put("documentTopTreeUri", null)

            val obj = JSObject()
            obj.put("uri", uriObj)
            obj.put("mimeType", _getMimeType(file))
            obj.put("name", file.name)
            obj.put("lastModified", file.lastModified())
            obj.put("byteSize", file.length())
            buffer.put(obj)
        }

        return buffer
    }

    // この関数が返すUriは他のアプリに共有できない
    @Synchronized
    override fun createFile(dirUri: FileUri, relativePath: String, mimeType: String): JSObject {
        val dir = File(Uri.parse(dirUri.uri).path!!)
        val baseFile = File(dir.path + "/" + relativePath.trimStart('/'))
        val fileName = baseFile.nameWithoutExtension
        val fileExtension = baseFile.extension
    
        var file = baseFile
        var counter = 1
    
        // 同じ名前のファイルが既に存在する場合、連番を追加してファイル名を変更
        while (file.exists()) {
            val newFileName = if (fileExtension.isEmpty()) {
                "$fileName($counter)"
            } else {
                "$fileName($counter).$fileExtension"
            }
            file = File(baseFile.parentFile, newFileName)
            counter++
        }
    
        file.parentFile?.mkdirs()
        file.createNewFile()

        val res = JSObject()
        res.put("uri", Uri.fromFile(file))
        res.put("documentTopTreeUri", null)
        return res
    }

    override fun deleteFile(uri: FileUri) {
        val file = File(Uri.parse(uri.uri).path!!)
        if (!file.isFile) {
            throw Exception("This is not file: ${uri.uri}")
        }
        if (!file.delete()) {
            throw Exception("Failed to delete file: ${uri.uri}")
        }
    }

    override fun deleteEmptyDir(uri: FileUri) {
        val file = File(Uri.parse(uri.uri).path!!)
        if (!file.isDirectory) {
            throw Exception("This is not dir: ${uri.uri}")
        }
        if (!file.delete()) {
            throw Exception("Failed to delete file: ${uri.uri}")
        }
    }

    override fun deleteDirAll(uri: FileUri) {
        val file = File(Uri.parse(uri.uri).path!!)
        if (!file.isDirectory) {
            throw Exception("This is not dir: ${uri.uri}")
        }
        
        if (!deleteRecursive(file)) {
            throw Exception("Failed to delete file: ${uri.uri}")
        }
    }

    override fun getThumbnail(uri: FileUri, width: Int, height: Int): Bitmap? {
        val file = File(Uri.parse(uri.uri).path!!)
        val mimeType = _getMimeType(file)!!
        val size = Size(width, height)

        try {
            when {
                mimeType.startsWith("image/") -> return ThumbnailUtils.createImageThumbnail(file, size, null)
                mimeType.startsWith("video/") -> return ThumbnailUtils.createVideoThumbnail(file, size, null)
                mimeType.startsWith("audio/") -> return ThumbnailUtils.createAudioThumbnail(file, size, null)
            }
        }
        catch (ignore: Exception) {}

        return null
    }

    override fun rename(uri: FileUri, newName: String): JSObject {
        val file = File(Uri.parse(uri.uri).path!!)
        val newFile = File(file.parentFile, newName)

        if (newFile.exists()) {
            throw Exception("File already exists: ${newFile.path}")
        }

        if (!file.renameTo(newFile)) {
            throw Exception("Failed to rename file: ${uri.uri}")
        }

        val res = JSObject()
        res.put("uri", Uri.fromFile(newFile).toString())
        res.put("documentTopTreeUri", uri.documentTopTreeUri)
        return res
    }

    private fun deleteRecursive(fileOrDirectory: File): Boolean {
        if (fileOrDirectory.isDirectory) {
            val children = fileOrDirectory.listFiles()
            if (children != null) {
                for (child in children) {
                    deleteRecursive(child)
                }
            }
        }
        return fileOrDirectory.delete()
    }

    // フォルダの場合のみnullを返す
    private fun _getMimeType(file: File): String? {
        if (file.isDirectory) {
            return null
        }

        return MimeTypeMap
            .getSingleton()
            .getMimeTypeFromExtension(file.extension)
            ?: "application/octet-stream"
    }
}