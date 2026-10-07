Note: **I’m using a translation tool, so some expressions may be awkward or inaccurate.**

# Overview
This plugin provides a unified file system API across all Android versions supported by Tauri.

By default (with default features enabled), this plugin does not add permissions to the application or enable any options that require additional review by Google Play.

**NOTE**: This page explains how to use this plugin in the Rust backend. For JavaScript bindings in the frontend, see [tauri-plugin-android-fs-api on npm](https://www.npmjs.com/package/tauri-plugin-android-fs-api?activeTab=readme).

# Setup
Register this plugin in your Tauri project:

`src-tauri/src/lib.rs`

```rust
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_android_fs::builder().build()) // This
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

# Usage
In many cases, this plugin operates on files and directories using URIs instead of file paths. These URIs are represented as [tauri_plugin_android_fs::FsUri](https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/struct.FsUri.html).

The plugin provides the following APIs:

| API | Description |
| --- | --- |
| [AndroidFs](https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.AndroidFs.html) | Provides core file and directory operations, such as metadata retrieval and reading and writing files. |
| [Picker](https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.Picker.html) | Provides file selection, directory selection, save dialogs, and permission management for selected entries. |
| [Opener](https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.Opener.html) | Provides opening files and directories in other applications. |
| [PublicStorage](https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.PublicStorage.html) | Provides access to public directories accessible to other applications and users. |
| [PrivateStorage](https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.PrivateStorage.html) | Provides access to app-specific private directories. |
| [AppStorage](https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.AppStorage.html) | Provides access to directories used by the app. Compared with `PrivateStorage`, it provides additional functionality, but the directories may be more easily accessible to other applications. |
| [⁠CustomFileProvider](https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/struct.Builder.html#method.register_custom_file_provider) | Defines files backed by sources such as network shares, cloud storage, or other non-filesystem sources. |

For documentation and examples, see the API links above.

# License
This project is licensed under either of

 * MIT license
 * Apache License (Version 2.0)

at your option.