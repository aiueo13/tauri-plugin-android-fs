package com.plugin.android_fs

import android.content.Context
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.graphics.ImageDecoder
import android.graphics.Point
import android.media.MediaMetadataRetriever
import android.media.ThumbnailUtils
import android.net.Uri
import android.os.Build
import android.provider.DocumentsContract
import android.util.Size
import java.io.File
import java.io.FileNotFoundException
import java.io.OutputStream
import kotlin.math.ceil
import kotlin.math.min


class AFThumbnails private constructor() { companion object {

    /**
     * @param format: "jpeg", "png", "webp"
     * @param quality: 0 ~ 100
     */
    fun <T, O: OutputStream>loadThumbnail(
        fileUri: FileUri,
        preferredSize: Size,
        format: String,
        quality: Int,
        output: () -> O,
        useThumbnail: (O) -> T,
        ctx: Context
    ): T? {

        var thumbnail: Bitmap? = null
        try {
            thumbnail = fetchThumbnail(fileUri, preferredSize, ctx)
            if (thumbnail == null) {
                return null
            }

            val rw = preferredSize.width
            val rh = preferredSize.height
            val w = thumbnail.width
            val h = thumbnail.height
            if ((w > (rw * 2) + 3 || h > (rh * 2) + 3)) {
                val ratio = minOf(rw.toFloat() / w, rh.toFloat() / h)
                val newThumbnail = Bitmap.createScaledBitmap(
                    thumbnail,
                    (w * ratio).toInt(),
                    (h * ratio).toInt(),
                    false
                )
                val oldThumbnail = thumbnail
                thumbnail = newThumbnail

                // Bitmap.createScaledBitmap は元の Bitmap を返すことがある。
                if (oldThumbnail !== thumbnail) {
                    oldThumbnail.recycle()
                }
            }

            val compressFormat = when (format.lowercase()) {
                "jpeg", "jpg" -> Bitmap.CompressFormat.JPEG
                "png" -> Bitmap.CompressFormat.PNG
                "webp" -> {
                    if (Build.VERSION.SDK_INT < Build.VERSION_CODES.R) {
                        Bitmap.CompressFormat.WEBP
                    }
                    else {
                        Bitmap.CompressFormat.WEBP_LOSSY
                    }
                }
                else -> throw Exception("Illegal format: $format")
            }

            val t = output().use {
                if (!thumbnail.compress(compressFormat, quality.coerceIn(0, 100), it)) {
                    throw Exception("Failed to compress bitmap")
                }
                it.flush()
                useThumbnail(it)
            }

            return t
        }
        finally {
            thumbnail?.recycle()
        }
    }
}}


private fun fetchThumbnail(
    fileUri: FileUri,
    preferredSize: Size,
    ctx: Context
): Bitmap? {

    val uri = Uri.parse(fileUri.uri)

    when (true) {
        (uri.scheme == "file") -> {
            fetchThumbnailFromFile(File(uri.path!!), preferredSize)?.let {
                return it
            }
        }
        (uri.scheme == "content") -> {
            if (fileUri.documentTopTreeUri != null || DocumentsContract.isDocumentUri(ctx, uri)) {
                fetchThumbnailFromDocumentFile(uri, preferredSize, ctx)?.let {
                    return it
                }
            }
            else {
                fetchThumbnailFromContentFile(uri, preferredSize, ctx)?.let {
                    return it
                }
            }

            val mimeType = AFUtils.getFileMimeType(fileUri, ctx)
            if (mimeType.startsWith("video/")) {
                createVideoThumbnailWithoutResizeFromContentFile(uri, preferredSize, ctx)?.let {
                    return it
                }
            }
            else if (mimeType.startsWith("image/")) {
                createImageThumbnailWithoutResizeFromContentFile(uri, preferredSize, ctx)?.let {
                    return it
                }
            }
        }
        else -> {}
    }

    return null
}


private fun fetchThumbnailFromContentFile(
    fileUri: Uri,
    preferredSize: Size,
    ctx: Context
): Bitmap? {

    try {
        // Q は Android 10
        if (Build.VERSION_CODES.Q <= Build.VERSION.SDK_INT) {
            return ctx.contentResolver.loadThumbnail(
                fileUri,
                preferredSize,
                null
            )
        }
    }
    catch (e: FileNotFoundException) {
        throw FileNotFoundException("file not found: $fileUri")
    }
    catch (ignore: Exception) {}

    return null
}

private fun fetchThumbnailFromDocumentFile(
    fileUri: Uri,
    preferredSize: Size,
    ctx: Context
): Bitmap? {

    try {
        return DocumentsContract.getDocumentThumbnail(
            ctx.contentResolver,
            fileUri,
            Point(preferredSize.width, preferredSize.height),
            null
        )
    }
    catch (e: FileNotFoundException) {
        throw FileNotFoundException("file not found: $fileUri")
    }
    catch (_: Exception) {}

    return null
}

private fun fetchThumbnailFromFile(file: File, preferredSize: Size): Bitmap? {
    val mimeType = AFUtils.guessFileMimeTypeFromExtension(file)

    try {
        if (Build.VERSION_CODES.Q <= Build.VERSION.SDK_INT) {
            when {
                mimeType.startsWith("image/") -> return ThumbnailUtils.createImageThumbnail(
                    file,
                    preferredSize,
                    null
                )
                mimeType.startsWith("video/") -> return ThumbnailUtils.createVideoThumbnail(
                    file,
                    preferredSize,
                    null
                )
                mimeType.startsWith("audio/") -> return ThumbnailUtils.createAudioThumbnail(
                    file,
                    preferredSize,
                    null
                )
            }
        }
    }
    catch (e: FileNotFoundException) {
        throw FileNotFoundException("file not found: ${file.path}")
    }
    catch (_: Exception) {}

    return null
}


// https://developer.android.com/social-and-messaging/guides/media-thumbnails?hl=ja#create-thumbnail
// を改変したもの (Apache 2.0 ライセンス)
private fun createImageThumbnailWithoutResizeFromContentFile(uri: Uri, size: Size, context: Context): Bitmap?{
    try {
        // P は Android 9
        if (Build.VERSION_CODES.P <= Build.VERSION.SDK_INT) {
            val source = ImageDecoder.createSource(context.contentResolver, uri)

            return ImageDecoder.decodeBitmap(source) { decoder, info, _ ->
                val widthSample = ceil(info.size.width / size.width.toDouble()).toInt()
                val heightSample = ceil(info.size.height / size.height.toDouble()).toInt()
                val sample = min(widthSample, heightSample)
                if (sample > 1) decoder.setTargetSampleSize(sample)
            }
        }

        val options = context.contentResolver.openInputStream(uri)?.use {
            val options = BitmapFactory.Options()
            options.inJustDecodeBounds = true
            BitmapFactory.decodeStream(it, null, options)
            options
        } ?: return null

        if ( options.outHeight != 0 ) {
            val widthSample = options.outWidth / size.width
            val heightSample = options.outHeight / size.height
            val sample = min(widthSample, heightSample)
            if (sample > 1) {
                options.inSampleSize = sample
            }
            options.inJustDecodeBounds = false
            val decodeStream = context.contentResolver.openInputStream(uri)
            val bitmap =  BitmapFactory.decodeStream(decodeStream, null, options)
            decodeStream?.close()
            return bitmap
        }
    }
    catch (_: Exception) {}

    return null
}

// https://developer.android.com/social-and-messaging/guides/media-thumbnails?hl=ja#create-thumbnail
// を改変したもの (Apache 2.0 ライセンス)
private fun createVideoThumbnailWithoutResizeFromContentFile(
    uri: Uri,
    preferredSize: Size,
    ctx: Context
): Bitmap? {

    val width = preferredSize.width
    val height = preferredSize.height

    try {
        MediaMetadataRetriever().use { mediaMetadataRetriever ->
            mediaMetadataRetriever.setDataSource(ctx, uri)
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
                        MediaMetadataRetriever.OPTION_PREVIOUS_SYNC,
                        (vw * ratio).toInt(),
                        (vh * ratio).toInt()
                    )
                }
            }

            return mediaMetadataRetriever.frameAtTime
        }
    }
    catch (e: FileNotFoundException) {
        throw FileNotFoundException("file not found: $uri")
    }
    catch (_: Exception) {}

    return null
}