package okayu.tauri.plugin.android.fs

import android.content.ContentResolver
import android.content.Context
import android.database.Cursor
import android.net.Uri
import android.os.Build
import android.os.Bundle
import android.provider.MediaStore
import androidx.annotation.RequiresApi
import androidx.core.database.getBlobOrNull
import androidx.core.database.getDoubleOrNull
import androidx.core.database.getLongOrNull
import androidx.core.database.getStringOrNull

object AFMediaStoreQuery {

    sealed class StorageVolume {
        object External: StorageVolume()
        object ExternalPrimary: StorageVolume()
        data class Specific(val volumeName: String): StorageVolume()
    }

    enum class Table {
        Image,
        Video,
        Audio,
        File,
        Download,
    }

    sealed class Column(open val name: kotlin.String, open val minApiLevel: Int) {
        data class String(
            override val name: kotlin.String,
            override val minApiLevel: Int,
            val mapValue: ((kotlin.String) -> Any)? = null
        ): Column(name, minApiLevel)

        data class Integer(
            override val name: kotlin.String,
            override val minApiLevel: Int,
            val mapValue: ((kotlin.Long) -> Any)? = null
        ): Column(name, minApiLevel)

        data class Float(
            override val name: kotlin.String,
            override val minApiLevel: Int,
            val mapValue: ((kotlin.Double) -> Any)? = null
        ): Column(name, minApiLevel)

        data class Blob(
            override val name: kotlin.String,
            override val minApiLevel: Int,
            val mapValue: ((kotlin.ByteArray) -> Any)? = null
        ): Column(name, minApiLevel)
    }


    data class QueriedFile(
        val id: Long,
        val columnValues: Array<Any?>
    )

    fun queryMediaStoreFiles(
        table: Table,
        volume: StorageVolume,
        columns: Array<Column>,
        selection: String?,
        selectionArgs: Array<String>,
        sortColumns: Array<String>,
        sortDirectionIsDesc: Boolean?,
        offset: Int?,
        limit: Int?,
        matchPending: Int?,
        matchFavorite: Int?,
        matchTrashed: Int?,
        ctx: Context,
        onQueryFile: (QueriedFile) -> Unit
    ): Uri {

        val projection = columns
            .filter { it.minApiLevel <= Build.VERSION.SDK_INT }
            .map { it.name }
            .toMutableSet()
            .apply {
                add(MediaStore.MediaColumns._ID)
                for (sortColumn in sortColumns) {
                    add(sortColumn)
                }
            }
            .toTypedArray()

        val collectionUri = resolveCollectionUri(table, volume)
        val result = if (Build.VERSION_CODES.R <= Build.VERSION.SDK_INT) {
            val queryArgs = resolveBundle(selection, selectionArgs,  sortColumns, sortDirectionIsDesc, offset, limit, matchPending, matchFavorite, matchTrashed)
            ctx.contentResolver.query(collectionUri, projection, queryArgs, null)
        }
        else {
            val sortOrder = resolveSortOrder(sortColumns, sortDirectionIsDesc, offset, limit)
            ctx.contentResolver.query(collectionUri, projection, selection, selectionArgs, sortOrder, null)
        }

        result?.use {
            val columnIndexes = columns.map { column ->
                if (column.minApiLevel <= Build.VERSION.SDK_INT) {
                    it.getColumnIndex(column.name)
                }
                else {
                    -1
                }
            }
            val idIndex = it.getColumnIndexOrThrow(MediaStore.MediaColumns._ID)

            while (it.moveToNext()) {
                val id = it.getLong(idIndex)

                val columnValues = Array<Any?>(columns.size) { null }
                for (i in columns.indices) {
                    val column = columns[i]
                    val columnIdx = columnIndexes[i]
                    if (columnIdx != -1) {
                        columnValues[i] = getCursorColumnValue(it, column, columnIdx)
                    }
                }

                onQueryFile(QueriedFile(id, columnValues))
            }
        }

        return collectionUri
    }

    fun queryMediaStoreFile(
        uri: Uri,
        columns: Array<Column>,
        ctx: Context,
    ): QueriedFile? {

        val projection = columns
            .filter { it.minApiLevel <= Build.VERSION.SDK_INT }
            .map { it.name }
            .toMutableSet()
            .apply {
                add(MediaStore.MediaColumns._ID)
            }
            .toTypedArray()

        ctx.contentResolver.query(uri, projection, null, null, null)?.use {
            val columnIndexes = columns.map { column ->
                if (column.minApiLevel <= Build.VERSION.SDK_INT) {
                    it.getColumnIndex(column.name)
                }
                else {
                    -1
                }
            }
            val idIndex = it.getColumnIndexOrThrow(MediaStore.MediaColumns._ID)

            if (it.moveToFirst()) {
                val id = it.getLong(idIndex)

                val columnValues = Array<Any?>(columns.size) { null }
                for (i in columns.indices) {
                    val column = columns[i]
                    val columnIdx = columnIndexes[i]
                    if (columnIdx != -1) {
                        columnValues[i] = getCursorColumnValue(it, column, columnIdx)
                    }
                }

                return QueriedFile(id, columnValues)
            }
        }

        return null
    }
}


private fun getCursorColumnValue(
    cursor: Cursor,
    column: AFMediaStoreQuery.Column,
    columnIdx: Int,
): Any? {

    return when (column) {
        is AFMediaStoreQuery.Column.String -> cursor.getStringOrNull(columnIdx)?.let {
            column.mapValue?.invoke(it) ?: it
        }
        is AFMediaStoreQuery.Column.Integer -> cursor.getLongOrNull(columnIdx)?.let {
            column.mapValue?.invoke(it) ?: it
        }
        is AFMediaStoreQuery.Column.Float -> cursor.getDoubleOrNull(columnIdx)?.let {
            column.mapValue?.invoke(it) ?: it
        }
        is AFMediaStoreQuery.Column.Blob -> cursor.getBlobOrNull(columnIdx)?.let {
            column.mapValue?.invoke(it) ?: it
        }
        else -> throw IllegalArgumentException("invalid column type")
    }
}

@RequiresApi(Build.VERSION_CODES.R)
private fun resolveBundle(
    selection: String?,
    selectionArgs: Array<String>?,
    sortColumns: Array<String>?,
    sortDirectionIsDesc: Boolean?,
    offset: Int?,
    limit: Int?,
    matchPending: Int?,
    matchFavorite: Int?,
    matchTrashed: Int?,
): Bundle {

    // 以下を参考
    // <https://proandroiddev.com/kotlin-flow-contentresolver-and-mediastore-the-key-to-effortless-media-access-in-android-fad56db16fdd>

    return Bundle().apply {
        // MediaStore が特定の QUERY_ARG_* に対応しているかに注意。
        // これらは少なくとも Android 11 以降であれば MediaStore は対応している。

        if (matchPending != null) {
            putInt(MediaStore.QUERY_ARG_MATCH_PENDING, matchPending)
        }
        if (matchFavorite != null) {
            putInt(MediaStore.QUERY_ARG_MATCH_FAVORITE, matchFavorite)
        }
        if (matchTrashed != null) {
            putInt(MediaStore.QUERY_ARG_MATCH_TRASHED, matchTrashed)
        }

        if (selection != null) {
            putString(ContentResolver.QUERY_ARG_SQL_SELECTION, selection)
        }
        if (!selectionArgs.isNullOrEmpty()) {
            putStringArray(ContentResolver.QUERY_ARG_SQL_SELECTION_ARGS, selectionArgs)
        }

        val hasSortOrder = !sortColumns.isNullOrEmpty()
        if (hasSortOrder) {
            putStringArray(ContentResolver.QUERY_ARG_SORT_COLUMNS, sortColumns!!)
            if (sortDirectionIsDesc != null) {
                putInt(ContentResolver.QUERY_ARG_SORT_DIRECTION, when (sortDirectionIsDesc) {
                    true -> ContentResolver.QUERY_SORT_DIRECTION_DESCENDING
                    false -> ContentResolver.QUERY_SORT_DIRECTION_ASCENDING
                })
            }
        }

        val hasOffset = offset != null && 0 < offset
        val hasLimit = limit != null && 0 < limit
        if (hasOffset) {
            putInt(ContentResolver.QUERY_ARG_OFFSET, offset!!)
        }
        if (hasLimit) {
            putInt(ContentResolver.QUERY_ARG_LIMIT, limit!!)
        }
        // limit や offset があるにも関わらず sort order がない場合は sort order を指定する
        if ((hasOffset || hasLimit) && !hasSortOrder) {
            putStringArray(ContentResolver.QUERY_ARG_SORT_COLUMNS, arrayOf(MediaStore.MediaColumns._ID))
        }
    }
}

/**
 * Android 11 以降で offset と limit を指定するとエラーになることに注意。
 */
private fun resolveSortOrder(
    sortColumns: Array<String>?,
    sortDirectionIsDesc: Boolean?,
    offset: Int?,
    limit: Int?
): String? {

    val sortClause = sortColumns
        ?.takeIf { it.isNotEmpty() }
        ?.joinToString(", ") { column ->
            when (sortDirectionIsDesc) {
                true -> "$column DESC"
                false -> "$column ASC"
                null -> column
            }
        }

    val effectiveOffset = offset?.takeIf { 0 <= it} ?: 0
    val effectiveLimit = limit?.takeIf { 0 <= it } ?: -1

    val paging = when {
        effectiveOffset == 0 && 0 <= effectiveLimit -> "LIMIT $effectiveLimit"
        effectiveOffset == 0 -> null
        else -> "LIMIT $effectiveLimit OFFSET $effectiveOffset"
    }

    return when {
        paging == null -> sortClause
        // LIMIT を付けるなら ORDER BY 式が必要
        else -> "${sortClause ?: MediaStore.MediaColumns._ID} $paging"
    }
}

private fun resolveCollectionUri(
    table: AFMediaStoreQuery.Table,
    volume: AFMediaStoreQuery.StorageVolume,
): Uri {

    return if (Build.VERSION.SDK_INT < Build.VERSION_CODES.Q) {
        // Android 9 以下では Media Store は External Volume のうち External Primary Volume にしかアクセスできない
        when (volume) {
            is AFMediaStoreQuery.StorageVolume.External -> {}
            is AFMediaStoreQuery.StorageVolume.ExternalPrimary -> {}
            is AFMediaStoreQuery.StorageVolume.Specific -> throw IllegalArgumentException("invalid volume")
            else -> throw IllegalArgumentException("invalid volume")
        }

        when (table) {
            AFMediaStoreQuery.Table.File -> MediaStore.Files.getContentUri("external")
            AFMediaStoreQuery.Table.Image -> MediaStore.Images.Media.EXTERNAL_CONTENT_URI
            AFMediaStoreQuery.Table.Video -> MediaStore.Video.Media.EXTERNAL_CONTENT_URI
            AFMediaStoreQuery.Table.Audio -> MediaStore.Audio.Media.EXTERNAL_CONTENT_URI
            AFMediaStoreQuery.Table.Download -> throw IllegalArgumentException("download table is only for Android 10 or higher")
        }
    }
    else {
        val volumeName = when (volume) {
            is AFMediaStoreQuery.StorageVolume.External -> MediaStore.VOLUME_EXTERNAL
            is AFMediaStoreQuery.StorageVolume.ExternalPrimary -> MediaStore.VOLUME_EXTERNAL_PRIMARY
            is AFMediaStoreQuery.StorageVolume.Specific -> volume.volumeName
            else -> throw IllegalArgumentException("invalid volume")
        }

        when (table) {
            AFMediaStoreQuery.Table.File -> MediaStore.Files.getContentUri(volumeName)
            AFMediaStoreQuery.Table.Image -> MediaStore.Images.Media.getContentUri(volumeName)
            AFMediaStoreQuery.Table.Video -> MediaStore.Video.Media.getContentUri(volumeName)
            AFMediaStoreQuery.Table.Audio -> MediaStore.Audio.Media.getContentUri(volumeName)
            AFMediaStoreQuery.Table.Download -> MediaStore.Downloads.getContentUri(volumeName)
        }
    }
}