package com.plugin.android_fs

import android.app.Activity
import android.content.Context
import android.net.Uri
import android.provider.DocumentsContract
import androidx.core.database.getLongOrNull
import androidx.core.database.getStringOrNull
import app.tauri.plugin.JSArray
import app.tauri.plugin.JSObject

class DocumentFileController(private val activity: Activity): FileController {

    override fun getMimeType(uri: AFUri): String? {
        activity.contentResolver.query(
            Uri.parse(uri.uri),
            arrayOf(DocumentsContract.Document.COLUMN_MIME_TYPE),
            null,
            null,
            null
        )?.use {

            if (it.moveToFirst()) {
                val mimeType = it.getStringOrNull(it.getColumnIndexOrThrow(DocumentsContract.Document.COLUMN_MIME_TYPE))

                if (mimeType == DocumentsContract.Document.MIME_TYPE_DIR) {
                    return null
                }
                return mimeType ?: "application/octet-stream"
            }
        }

        throw Exception("Failed to find entry: ${uri.uri}")
    }

    override fun getName(uri: AFUri): String {
        activity.contentResolver.query(
            Uri.parse(uri.uri),
            arrayOf(DocumentsContract.Document.COLUMN_DISPLAY_NAME),
            null,
            null,
            null
        )?.use {

            if (it.moveToFirst()) {
                return it.getString(it.getColumnIndexOrThrow(DocumentsContract.Document.COLUMN_DISPLAY_NAME))
            }
        }

        throw Exception("No permission or entry: ${uri.uri}")
    }

    override fun getLen(uri: AFUri): Long {
        activity.contentResolver.query(
            Uri.parse(uri.uri),
            arrayOf(
                DocumentsContract.Document.COLUMN_SIZE,
                DocumentsContract.Document.COLUMN_MIME_TYPE
            ),
            null,
            null,
            null
        )?.use {

            if (it.moveToFirst()) {
                val mimeType = it.getStringOrNull(it.getColumnIndexOrThrow(DocumentsContract.Document.COLUMN_MIME_TYPE))

                if (mimeType == DocumentsContract.Document.MIME_TYPE_DIR) {
                    throw Exception("This is a directory: ${uri.uri}")
                }
                
                return it.getLongOrNull(it.getColumnIndexOrThrow(DocumentsContract.Document.COLUMN_SIZE)) ?: 0
            }
        }

        throw Exception("No permission or file: ${uri.uri}")
    }

    override fun readDir(dirUri: AFUri, options: ReadDirEntryOptions): JSArray {
        val queryTarget = mutableListOf(DocumentsContract.Document.COLUMN_MIME_TYPE)
        if (options.uri) {
            queryTarget.add(DocumentsContract.Document.COLUMN_DOCUMENT_ID)
        }
        if (options.lastModified) {
            queryTarget.add(DocumentsContract.Document.COLUMN_LAST_MODIFIED)
        }
        if (options.name) {
            queryTarget.add(DocumentsContract.Document.COLUMN_DISPLAY_NAME)
        }
        if (options.len) {
            queryTarget.add(DocumentsContract.Document.COLUMN_SIZE)
        }

        val topTreeUriString = dirUri.documentTopTreeUri!!
        val topTreeUri = Uri.parse(dirUri.documentTopTreeUri!!)
        val cursor = activity.contentResolver.query(
            DocumentsContract.buildChildDocumentsUriUsingTree(
                topTreeUri,
                DocumentsContract.getDocumentId(Uri.parse(dirUri.uri))
            ),
            queryTarget.toTypedArray(),
            null,
            null,
            null
        )

        val buffer = JSArray()

        cursor?.use {
            val idColumnIndex = it.getColumnIndex(DocumentsContract.Document.COLUMN_DOCUMENT_ID)
            val mimeTypeColumnIndex = it.getColumnIndex(DocumentsContract.Document.COLUMN_MIME_TYPE)
            val nameColumnIndex = it.getColumnIndex(DocumentsContract.Document.COLUMN_DISPLAY_NAME)
            val lastModifiedColumnIndex = it.getColumnIndex(DocumentsContract.Document.COLUMN_LAST_MODIFIED)
            val sizeColumnIndex = it.getColumnIndex(DocumentsContract.Document.COLUMN_SIZE)

            while (it.moveToNext()) {
                val obj = JSObject()

                if (options.uri) {
                    obj.put("uri", JSObject().apply {
                        val id = it.getString(idColumnIndex)
                        val uri = DocumentsContract.buildDocumentUriUsingTree(topTreeUri, id)
                        put("uri", uri.toString())
                        put("documentTopTreeUri", topTreeUriString)
                    })
                }
                if (options.name) {
                    obj.put("name", it.getString(nameColumnIndex))
                }
                if (options.lastModified) {
                    obj.put("lastModified", it.getLongOrNull(lastModifiedColumnIndex) ?: 0)
                }

                val mimeType = it.getStringOrNull(mimeTypeColumnIndex) ?: "application/octet-stream"
                if (mimeType != DocumentsContract.Document.MIME_TYPE_DIR) {
                    obj.put("mimeType", mimeType)
                    if (options.len) {
                        obj.put("len", it.getLongOrNull(sizeColumnIndex) ?: 0)
                    }
                }

                buffer.put(obj)
            }
        }

        return buffer
    }

    override fun getMetadata(uri: AFUri): JSObject {
        val cursor = activity.contentResolver.query(
            Uri.parse(uri.uri),
            arrayOf(
                DocumentsContract.Document.COLUMN_MIME_TYPE,
                DocumentsContract.Document.COLUMN_DISPLAY_NAME,
                DocumentsContract.Document.COLUMN_LAST_MODIFIED,
                DocumentsContract.Document.COLUMN_SIZE
            ),
            null,
            null,
            null
        )

        cursor?.use {
            val mimeTypeColumnIndex = it.getColumnIndex(DocumentsContract.Document.COLUMN_MIME_TYPE)
            val nameColumnIndex = it.getColumnIndex(DocumentsContract.Document.COLUMN_DISPLAY_NAME)
            val lastModifiedColumnIndex = it.getColumnIndex(DocumentsContract.Document.COLUMN_LAST_MODIFIED)
            val sizeColumnIndex = it.getColumnIndex(DocumentsContract.Document.COLUMN_SIZE)

            while (it.moveToNext()) {
                val obj = JSObject()

                obj.put("uri", JSObject().apply {
                    put("uri", uri.uri)
                    put("documentTopTreeUri", uri.documentTopTreeUri)
                })
                obj.put("name", it.getString(nameColumnIndex))
                obj.put("lastModified", it.getLongOrNull(lastModifiedColumnIndex) ?: 0)

                val mimeType = it.getStringOrNull(mimeTypeColumnIndex) ?: "application/octet-stream"
                if (mimeType != DocumentsContract.Document.MIME_TYPE_DIR) {
                    obj.put("mimeType", mimeType)
                    obj.put("len", it.getLongOrNull(sizeColumnIndex) ?: 0)
                }

                return obj
            }
        }

        throw Exception("No permission or entry: $uri")
    }

    @Synchronized
    override fun createFile(dirUri: AFUri, relativePath: String, mimeType: String): JSObject {
        if (relativePath.endsWith('/')) {
            throw Exception("Illegal file path format, ends with '/'. $relativePath")
        }
        if (relativePath.isEmpty()) {
            throw Exception("Relative path is empty.")
        }

        val _relativePath = relativePath.trimStart('/')
        val relativeDirPath = _relativePath.substringBeforeLast("/", "")
        val fileName = _relativePath.substringAfterLast("/", _relativePath)

        val parentUri = createOrGetDir(dirUri, relativeDirPath)

        val uri =  DocumentsContract.createDocument(
            activity.contentResolver,
            parentUri,
            mimeType,
            fileName
        ) ?: throw Exception("Failed to create file: { parent: $parentUri, fileName: $fileName, mimeType: $mimeType }")

        val res = JSObject()
        res.put("uri", uri)
        res.put("documentTopTreeUri", dirUri.documentTopTreeUri)
        return res
    }

    @Synchronized
    override fun createFileAndReturnRelativePath(dirUri: AFUri, relativePath: String, mimeType: String): JSObject {
        if (relativePath.endsWith('/')) {
            throw Exception("Illegal file path format, ends with '/'. $relativePath")
        }
        if (relativePath.isEmpty()) {
            throw Exception("Relative path is empty.")
        }

        val _relativePath = relativePath.trimStart('/')
        val relativeDirPath = _relativePath.substringBeforeLast("/", "")
        val fileName = _relativePath.substringAfterLast("/", _relativePath)

        val entry = createOrGetDirAndReturnRelativePath(dirUri, relativeDirPath)
        val parentUri = entry.first
        val parentActualRelativePath = entry.second

        val uri = DocumentsContract.createDocument(
            activity.contentResolver,
            parentUri,
            mimeType,
            fileName
        ) ?: throw Exception("Failed to create file: { parent: $parentUri, fileName: $fileName, mimeType: $mimeType }")
        val actualFileName = _getName(uri)
        val actualRelativePath = "${parentActualRelativePath.trim('/')}/${actualFileName}"

        return JSObject().apply {
            put("relativePath", actualRelativePath)
            put("uri", JSObject().apply {
                put("uri", uri)
                put("documentTopTreeUri", dirUri.documentTopTreeUri)
            })
        }
    }

    @Synchronized
    override fun createDirAll(dirUri: AFUri, relativePath: String): JSObject {
        if (relativePath.endsWith('/')) {
            throw Exception("Illegal file path format, ends with '/'. $relativePath")
        }
        if (relativePath.isEmpty()) {
            throw Exception("Relative path is empty.")
        }

        val uri = createOrGetDir(dirUri, relativePath)

        val res = JSObject()
        res.put("uri", uri)
        res.put("documentTopTreeUri", dirUri.documentTopTreeUri)
        return res
    }

    @Synchronized
    override fun createDirAllAndReturnRelativePath(dirUri: AFUri, relativePath: String): JSObject {
        if (relativePath.endsWith('/')) {
            throw Exception("Illegal file path format, ends with '/'. $relativePath")
        }
        if (relativePath.isEmpty()) {
            throw Exception("Relative path is empty.")
        }

        val entry = createOrGetDirAndReturnRelativePath(dirUri, relativePath)
        val uri = entry.first
        val actualRelativePath = entry.second

        return JSObject().apply {
            put("relativePath", actualRelativePath)
            put("uri", JSObject().apply {
                put("uri", uri)
                put("documentTopTreeUri", dirUri.documentTopTreeUri)
            })
        }
    }

    override fun deleteFile(uri: AFUri) {
        if (getMimeType(uri) == null) {
            throw Exception("This is dir, not file: ${uri.uri}")
        }
        if (!DocumentsContract.deleteDocument(activity.contentResolver, Uri.parse(uri.uri))) {
            throw Exception("Failed to delete file: ${uri.uri}")
        }
    }

    override fun deleteDirAll(uri: AFUri) {
        if (getMimeType(uri) != null) {
            throw Exception("This is file, not dir: ${uri.uri}")
        }
        if (!DocumentsContract.deleteDocument(activity.contentResolver, Uri.parse(uri.uri))) {
            throw Exception("Failed to delete file: ${uri.uri}")
        }
    }

    override fun deleteEmptyDir(uri: AFUri) {
        if (getMimeType(uri) != null) {
            throw Exception("This is file, not dir: ${uri.uri}")
        }

        val topTreeUri = Uri.parse(uri.documentTopTreeUri!!)
        val childrenUri = DocumentsContract.buildChildDocumentsUriUsingTree(
            topTreeUri,
            DocumentsContract.getDocumentId(Uri.parse(uri.uri))
        )
        val cursor = activity.contentResolver.query(
            childrenUri,
            arrayOf(),
            null,
            null,
            null
        )
        cursor?.use {
            if (it.moveToFirst()) {
                throw Exception("Dir is not empty: ${uri.uri}")
            }
        }

        if (!DocumentsContract.deleteDocument(activity.contentResolver, Uri.parse(uri.uri))) {
            throw Exception("Failed to delete file: ${uri.uri}")
        }
    }

    override fun rename(uri: AFUri, newName: String): JSObject {
        val documentUri = Uri.parse(uri.uri)
        val updatedUri = DocumentsContract.renameDocument(
            activity.contentResolver, 
            documentUri, 
            newName
        )

        val res = JSObject()
        res.put("uri", updatedUri.toString())
        res.put("documentTopTreeUri", uri.documentTopTreeUri)
        return res
    }

    private fun findDirIdFromName(
        activity: Context,
        dir_topTreeUri: Uri,
        dir_id: String,
        name: String,
    ): String? {

        val cursor = activity.contentResolver.query(
            DocumentsContract.buildChildDocumentsUriUsingTree(
                dir_topTreeUri,
                dir_id
            ),
            arrayOf(
                DocumentsContract.Document.COLUMN_DISPLAY_NAME,
                DocumentsContract.Document.COLUMN_DOCUMENT_ID,
                DocumentsContract.Document.COLUMN_MIME_TYPE
            ),
            null,
            null,
            null
        )

        cursor?.use {
            val nameColumnIndex = cursor.getColumnIndex(DocumentsContract.Document.COLUMN_DISPLAY_NAME)
            val idColumnIndex = cursor.getColumnIndex(DocumentsContract.Document.COLUMN_DOCUMENT_ID)
            val mimeTypeColumnIndex = cursor.getColumnIndex(DocumentsContract.Document.COLUMN_MIME_TYPE)

            while (cursor.moveToNext()) {
                if (name == cursor.getString(nameColumnIndex)) {
                    if (DocumentsContract.Document.MIME_TYPE_DIR != cursor.getStringOrNull(mimeTypeColumnIndex)) {
                        return null
                    }

                    return cursor.getString(idColumnIndex)
                }
            }
        }

        return null
    }

    private fun findIdFromName(
        activity: Context,
        dir_topTreeUri: Uri,
        dir_id: String,
        name: String,
    ): String? {

        val cursor = activity.contentResolver.query(
            DocumentsContract.buildChildDocumentsUriUsingTree(
                dir_topTreeUri,
                dir_id
            ),
            arrayOf(
                DocumentsContract.Document.COLUMN_DISPLAY_NAME,
                DocumentsContract.Document.COLUMN_DOCUMENT_ID
            ),
            null,
            null,
            null
        )

        cursor?.use {
            val nameColumnIndex = cursor.getColumnIndex(DocumentsContract.Document.COLUMN_DISPLAY_NAME)
            val idColumnIndex = cursor.getColumnIndex(DocumentsContract.Document.COLUMN_DOCUMENT_ID)

            while (cursor.moveToNext()) {
                if (name == cursor.getString(nameColumnIndex)) {
                    return cursor.getString(idColumnIndex)
                }
            }
        }

        return null
    }

    private fun createOrGetDir(dirUri: AFUri, relativePath: String): Uri {
        val topTreeUri = Uri.parse(dirUri.documentTopTreeUri!!)
        var parentId = DocumentsContract.getDocumentId(Uri.parse(dirUri.uri))

        // フォルダが存在しなければ再帰的に作成する
        for (dirName in relativePath.split("/").filter { it.isNotEmpty() }) {
            parentId = findDirIdFromName(activity, topTreeUri, parentId, dirName) ?: DocumentsContract.getDocumentId(
                DocumentsContract.createDocument(
                    activity.contentResolver,
                    DocumentsContract.buildDocumentUriUsingTree(topTreeUri, parentId),
                    DocumentsContract.Document.MIME_TYPE_DIR,
                    dirName
                )!!
            )
        }

        return DocumentsContract.buildDocumentUriUsingTree(topTreeUri, parentId)
    }

    private fun createOrGetDirAndReturnRelativePath(dirUri: AFUri, relativePath: String): Pair<Uri, String> {
        val topTreeUri = Uri.parse(dirUri.documentTopTreeUri!!)
        var parentId = DocumentsContract.getDocumentId(Uri.parse(dirUri.uri))
        var actualRelativePath = ""

        // フォルダが存在しなければ再帰的に作成する
        for (dirName in relativePath.split("/").filter { it.isNotEmpty() }) {
            val id = findDirIdFromName(activity, topTreeUri, parentId, dirName)
            if (id != null) {
                parentId = id
                actualRelativePath += dirName
                actualRelativePath += "/"
            }
            else {
                val newUri = DocumentsContract.createDocument(
                    activity.contentResolver,
                    DocumentsContract.buildDocumentUriUsingTree(topTreeUri, parentId),
                    DocumentsContract.Document.MIME_TYPE_DIR,
                    dirName
                )!!
                val newId = DocumentsContract.getDocumentId(newUri)
                val actualDirName = _getName(newUri)

                parentId = newId
                actualRelativePath += actualDirName
                actualRelativePath += "/"
            }
        }

        return Pair(
            DocumentsContract.buildDocumentUriUsingTree(topTreeUri, parentId),
            actualRelativePath.trim('/')
        )
    }

    private fun findUri(dirUri: AFUri, relativePath: String): Uri {
        val topTreeUri = Uri.parse(dirUri.documentTopTreeUri!!)
        var id = DocumentsContract.getDocumentId(Uri.parse(dirUri.uri))

        for (name in relativePath.split("/").filter { it.isNotEmpty() }) {
            val i = findIdFromName(activity, topTreeUri, id, name)
                ?: throw Exception("Part of the file or directory path was not found or permission denied")

            id = i
        }

        return DocumentsContract.buildDocumentUriUsingTree(topTreeUri, id)
    }

    private fun isDir(uri: Uri): Boolean {
        activity.contentResolver.query(
            uri,
            arrayOf(DocumentsContract.Document.COLUMN_MIME_TYPE),
            null,
            null,
            null
        )?.use {

            if (it.moveToFirst()) {
                val mimeType = it.getStringOrNull(it.getColumnIndexOrThrow(DocumentsContract.Document.COLUMN_MIME_TYPE))

                return mimeType == DocumentsContract.Document.MIME_TYPE_DIR
            }
        }

        throw Exception("No file or permission: $uri")
    }

    fun findFileUri(dirUri: AFUri, relativePath: String): JSObject {
        if (relativePath.startsWith('/')) {
            throw Exception("Illegal file path format, starts with '/'.")
        }
        if (relativePath.isEmpty()) {
            throw Exception("Relative path is empty.")
        }

        val uri = findUri(dirUri, relativePath)
        if (isDir(uri)) {
            throw Exception("This is a directory: $uri")
        }

        val res = JSObject()
        res.put("uri", uri)
        res.put("documentTopTreeUri", dirUri.documentTopTreeUri)
        return res
    }

    fun findDirUri(dirUri: AFUri, relativePath: String): JSObject {
        if (relativePath.startsWith('/')) {
            throw Exception("Illegal file path format, starts with '/'.")
        }
        if (relativePath.isEmpty()) {
            throw Exception("Relative path is empty.")
        }

        val uri = findUri(dirUri, relativePath)
        if (!isDir(uri)) {
            throw Exception("This is a file: $uri")
        }

        val res = JSObject()
        res.put("uri", uri)
        res.put("documentTopTreeUri", dirUri.documentTopTreeUri)
        return res
    }

    private fun _getName(uri: Uri): String {
        activity.contentResolver.query(
            uri,
            arrayOf(DocumentsContract.Document.COLUMN_DISPLAY_NAME),
            null,
            null,
            null
        )?.use {

            if (it.moveToFirst()) {
                return it.getString(it.getColumnIndexOrThrow(DocumentsContract.Document.COLUMN_DISPLAY_NAME))
            }
        }

        throw Exception("Failed to get name from $uri")
    }
}