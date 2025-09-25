package com.plugin.android_fs

import android.net.Uri
import app.tauri.plugin.JSObject

class AFJSObject private constructor() { companion object {

    fun createFileUri(uri: String, documentTopTreeUri: String?): JSObject {
        return JSObject().apply {
            put("uri", uri)
            put("documentTopTreeUri", documentTopTreeUri)
        }
    }

    fun createFileUri(uri: Uri): JSObject {
        return createFileUri(uri.toString(), null)
    }

    fun createFileUri(uri: String, documentTopTreeUri: Uri): JSObject {
        return createFileUri(uri, documentTopTreeUri.toString())
    }

    fun createFileUri(uri: Uri, documentTopTreeUri: String): JSObject {
        return createFileUri(uri.toString(), documentTopTreeUri)
    }

    fun createFileUri(uri: Uri, documentTopTreeUri: Uri): JSObject {
        return createFileUri(uri.toString(), documentTopTreeUri.toString())
    }
}}