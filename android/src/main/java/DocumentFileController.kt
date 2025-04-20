package com.plugin.android_fs

import android.app.Activity
import android.content.Context
import android.net.Uri
import android.provider.DocumentsContract
import androidx.core.database.getStringOrNull
import android.provider.MediaStore
import app.tauri.plugin.JSArray
import app.tauri.plugin.JSObject

class DocumentFileController(private val activity: Activity): FileController {

    override fun getMimeType(uri: FileUri): String? {
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

        throw Error("Failed to get name from ${uri.uri}")
    }

    override fun getName(uri: FileUri): String {
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

        throw Error("Failed to get name from ${uri.uri}")
    }

    override fun readDir(dirUri: FileUri): JSArray {
        val topTreeUri = Uri.parse(dirUri.documentTopTreeUri!!)
        val cursor = activity.contentResolver.query(
            DocumentsContract.buildChildDocumentsUriUsingTree(
                topTreeUri,
                DocumentsContract.getDocumentId(Uri.parse(dirUri.uri))
            ),
            arrayOf(
                DocumentsContract.Document.COLUMN_DOCUMENT_ID,
                DocumentsContract.Document.COLUMN_MIME_TYPE,
                DocumentsContract.Document.COLUMN_DISPLAY_NAME,
                DocumentsContract.Document.COLUMN_LAST_MODIFIED,
                DocumentsContract.Document.COLUMN_SIZE,
            ),
            null,
            null,
            null
        )

        val buffer = JSArray()

        cursor?.use {
            val idColumnIndex = cursor.getColumnIndex(DocumentsContract.Document.COLUMN_DOCUMENT_ID)
            val mimeTypeColumnIndex = cursor.getColumnIndex(DocumentsContract.Document.COLUMN_MIME_TYPE)
            val nameColumnIndex = cursor.getColumnIndex(DocumentsContract.Document.COLUMN_DISPLAY_NAME)
            val lastModifiedColumnIndex = cursor.getColumnIndex(DocumentsContract.Document.COLUMN_LAST_MODIFIED)
            val sizeColumnIndex = cursor.getColumnIndex(DocumentsContract.Document.COLUMN_SIZE)
            val topTreeUriString = topTreeUri.toString()

            while (cursor.moveToNext()) {
                val id = cursor.getString(idColumnIndex)
                val uri = DocumentsContract.buildDocumentUriUsingTree(topTreeUri, id)

                val uriObj = JSObject()
                uriObj.put("uri", uri)
                uriObj.put("documentTopTreeUri", topTreeUriString)

                var mimeType: String? = cursor.getStringOrNull(mimeTypeColumnIndex)
                if (mimeType == DocumentsContract.Document.MIME_TYPE_DIR) {
                    mimeType = null
                }
                else {
                    mimeType = mimeType ?: "application/octet-stream"
                }

                val obj = JSObject()
                obj.put("uri", uriObj)
                obj.put("mimeType", mimeType)
                obj.put("name", cursor.getString(nameColumnIndex))
                obj.put("lastModified", cursor.getLong(lastModifiedColumnIndex))
                obj.put("byteSize", cursor.getLong(sizeColumnIndex))
                buffer.put(obj)
            }
        }

        return buffer
    }

    override fun createFile(dirUri: FileUri, relativePath: String, mimeType: String): JSObject {
        if (relativePath.endsWith('/')) {
            throw Error("Illegal file path format, ends with '/'. $relativePath")
        }
        if (relativePath.isEmpty()) {
            throw Error("Relative path is empty.")
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
        ) ?: throw Error("Failed to create file: { parent: $parentUri, fileName: $fileName, mimeType: $mimeType }")

        val res = JSObject()
        res.put("uri", uri)
        res.put("documentTopTreeUri", dirUri.documentTopTreeUri)
        return res
    }

    override fun deleteFile(uri: FileUri) {
        if (getMimeType(uri) == null) {
            throw Error("This is dir, not file: ${uri.uri}")
        }
        if (!DocumentsContract.deleteDocument(activity.contentResolver, Uri.parse(uri.uri))) {
            throw Error("Failed to delete file: ${uri.uri}")
        }
    }

    override fun deleteDirAll(uri: FileUri) {
        if (getMimeType(uri) != null) {
            throw Error("This is file, not dir: ${uri.uri}")
        }
        if (!DocumentsContract.deleteDocument(activity.contentResolver, Uri.parse(uri.uri))) {
            throw Error("Failed to delete file: ${uri.uri}")
        }
    }

    override fun deleteEmptyDir(uri: FileUri) {
        if (getMimeType(uri) != null) {
            throw Error("This is file, not dir: ${uri.uri}")
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
                throw Error("Dir is not empty: ${uri.uri}")
            }
        }

        if (!DocumentsContract.deleteDocument(activity.contentResolver, Uri.parse(uri.uri))) {
            throw Error("Failed to delete file: ${uri.uri}")
        }
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

    private fun createOrGetDir(dirUri: FileUri, relativePath: String): Uri {
        val topTreeUri = Uri.parse(dirUri.documentTopTreeUri!!)
        var parentId = DocumentsContract.getDocumentId(Uri.parse(dirUri.uri))

        // フォルダが存在しなければ再帰的に作成する
        for (dirName in relativePath.split("/").filter { it.isNotEmpty() }) {
            parentId = findIdFromName(activity, topTreeUri, parentId, dirName) ?: DocumentsContract.getDocumentId(
                DocumentsContract.createDocument(
                    activity.contentResolver,
                    DocumentsContract.buildDocumentUriUsingTree(topTreeUri, parentId),
                    DocumentsContract.Document.MIME_TYPE_DIR,
                    dirName
                )
            )
        }

        return DocumentsContract.buildDocumentUriUsingTree(topTreeUri, parentId)
    }
}