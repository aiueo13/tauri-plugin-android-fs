package okayu.tauri.plugin.android.fs

import android.app.Activity
import android.net.Uri
import android.provider.MediaStore
import android.provider.MediaStore.PickerMediaColumns
import androidx.core.database.getLongOrNull
import app.tauri.plugin.JSArray
import app.tauri.plugin.JSObject

class MediaFileController(private val activity: Activity): FileController {

    // フォルダが指定されることは想定していない
    override fun getMimeType(uri: AFUri): String {
        return AFMediaStore.getMimeType(uri, activity)
    }

    override fun getName(uri: AFUri): String {
        return AFMediaStore.getDisplayName(uri, activity)
    }

    override fun getLen(uri: AFUri): Long {
        val cursor = activity.contentResolver.query(
            Uri.parse(uri.uri),
            arrayOf(MediaStore.MediaColumns.SIZE),
            null,
            null,
            null
        )

        cursor?.use {
            val sizeColumnIndex = it.getColumnIndex(MediaStore.MediaColumns.SIZE)

            while (it.moveToNext()) {
                return it.getLong(sizeColumnIndex)
            }
        }

        throw Exception("No permission or entry: $uri")
    }

    override fun getLastModified(uri: AFUri): Long {
        val cursor = activity.contentResolver.query(
            Uri.parse(uri.uri),
            arrayOf(
                MediaStore.MediaColumns.DATE_MODIFIED,
                MediaStore.MediaColumns.DATE_ADDED,
                MediaStore.MediaColumns.DATE_TAKEN, // PickerMediaColumns.DATE_TAKEN
            ),
            null,
            null,
            null
        )

        cursor?.use {
            val dateModifiedColumnIndex = it.getColumnIndex(MediaStore.MediaColumns.DATE_MODIFIED)
            val dateAddedColumnIndex = it.getColumnIndex(MediaStore.MediaColumns.DATE_ADDED)
            val dateTakenColumnIndex = it.getColumnIndex(MediaStore.MediaColumns.DATE_TAKEN)

            while (it.moveToNext()) {
                // MediaStore.MediaColumns.DATE_MODIFIED や DATE_ADDED は秒単位だが、
                // DATE_TAKEN はミリ秒単位であることに注意。
                return it.getLongOrNull(dateModifiedColumnIndex)?.times(1000L)
                    ?: it.getLongOrNull(dateAddedColumnIndex)?.times(1000L)
                    ?: it.getLongOrNull(dateTakenColumnIndex)
                    ?: 0
            }
        }

        throw Exception("No permission or entry: $uri")
    }

    override fun deleteFile(uri: AFUri) {
        AFMediaStore.delete(uri, activity)
    }

    override fun getMetadata(uri: AFUri): JSObject {
        val cursor = activity.contentResolver.query(
            Uri.parse(uri.uri),
            arrayOf(
                MediaStore.MediaColumns.MIME_TYPE,
                MediaStore.MediaColumns.DISPLAY_NAME,
                MediaStore.MediaColumns.SIZE,
                MediaStore.MediaColumns.DATE_MODIFIED,
                MediaStore.MediaColumns.DATE_ADDED,
                MediaStore.MediaColumns.DATE_TAKEN // PickerMediaColumns.DATE_TAKEN
            ),
            null,
            null,
            null
        )

        cursor?.use {
            val mimeTypeColumnIndex = it.getColumnIndex(MediaStore.MediaColumns.MIME_TYPE)
            val nameColumnIndex = it.getColumnIndex(MediaStore.MediaColumns.DISPLAY_NAME)
            val dateModifiedColumnIndex = it.getColumnIndex(MediaStore.MediaColumns.DATE_MODIFIED)
            val dateAddedColumnIndex = it.getColumnIndex(MediaStore.MediaColumns.DATE_ADDED)
            val dateTakenColumnIndex = it.getColumnIndex(MediaStore.MediaColumns.DATE_TAKEN)
            val sizeColumnIndex = it.getColumnIndex(MediaStore.MediaColumns.SIZE)

            while (it.moveToNext()) {
                val obj = JSObject()

                obj.put("uri", JSObject().apply {
                    put("uri", uri.uri)
                    put("documentTopTreeUri", uri.documentTopTreeUri)
                })
                obj.put("name", it.getString(nameColumnIndex))

                // MediaStore.MediaColumns.DATE_MODIFIED や DATE_ADDED は秒単位だが、
                // DATE_TAKEN はミリ秒単位であることに注意。
                val lastModified = it.getLongOrNull(dateModifiedColumnIndex)?.times(1000L)
                    ?: it.getLongOrNull(dateAddedColumnIndex)?.times(1000L)
                    ?: it.getLongOrNull(dateTakenColumnIndex)

                obj.put("lastModified", lastModified ?: 0)

                val mimeType = it.getString(mimeTypeColumnIndex)
                obj.put("mimeType", mimeType)
                obj.put("len", it.getLong(sizeColumnIndex))

                return obj
            }
        }

        throw Exception("No permission or entry: $uri")
    }

    override fun rename(uri: AFUri, newName: String): JSObject {
        AFMediaStore.rename(uri, newName, activity)

        val res = JSObject()
        res.put("uri", uri.uri)
        res.put("documentTopTreeUri", uri.documentTopTreeUri)
        return res
    }

    override fun createNewFile(dirUri: AFUri, relativePath: String, mimeType: String): JSObject {
        throw Exception("Unsupported operation for ${dirUri.uri}")
    }

    override fun createNewFileAndReturnRelativePath(dirUri: AFUri, relativePath: String, mimeType: String): JSObject {
        throw Exception("Unsupported operation for ${dirUri.uri}")
    }

    override fun createDirAll(dirUri: AFUri, relativePath: String): JSObject {
        throw Exception("Unsupported operation for ${dirUri.uri}")
    }

    override fun createDirAllAndReturnRelativePath(dirUri: AFUri, relativePath: String): JSObject {
        throw Exception("Unsupported operation for ${dirUri.uri}")
    }

    override fun createNewDir(dirUri: AFUri, relativePath: String): JSObject {
        throw Exception("Unsupported operation for ${dirUri.uri}")
    }

    override fun createNewDirAndReturnRelativePath(dirUri: AFUri, relativePath: String): JSObject {
        throw Exception("Unsupported operation for ${dirUri.uri}")
    }

    override fun deleteEmptyDir(uri: AFUri) {
        throw Exception("Unsupported operation for ${uri.uri}")
    }

    override fun deleteDirAll(uri: AFUri) {
        throw Exception("Unsupported operation for ${uri.uri}")
    }

    override fun readDir(dirUri: AFUri, options: ReadDirEntryOptions, offset: ULong, limit: ULong?): JSArray {
        throw Exception("Unsupported operation for ${dirUri.uri}")
    }
}
