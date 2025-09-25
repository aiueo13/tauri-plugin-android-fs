package com.plugin.android_fs

import android.webkit.MimeTypeMap
import java.io.File

class AFUtils private constructor() { companion object {

    fun guessMimeTypeFromExtension(file: File): String {
        val ext = file.extension

        if (ext.isEmpty()) {
            return "application/octet-stream"
        }

        return MimeTypeMap
            .getSingleton()
            .getMimeTypeFromExtension(ext)
            ?: "application/octet-stream"
    }
}}