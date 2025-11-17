Note: **I’m using a translation tool, so there may be some inappropriate expressions.**

# Overview

The Android file system is strict and complex. This plugin was created to provide practical file operations. By default (default-feature), this does not use any options that require additional permissions or review, so you can submit your app for Google Play review with peace of mind.

# Setup
Register this plugin in your Tauri project:

`src-tauri/src/lib.rs`

```rust
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // Add following
        .plugin(tauri_plugin_android_fs::init())
        //
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

# Usage
This section explains how to use this plugin on the Rust side. If you need JavaScript bindings on the frontend, please see [this link](https://www.npmjs.com/package/tauri-plugin-android-fs-api?activeTab=readme).

### 1. Dialog

Opens the file/folder picker to read and write user-selected entries.

```rust
use tauri_plugin_android_fs::{AndroidFsExt, ImageFormat, Result, Size};

async fn file_picker_example(&app: tauri::AppHandle<impl tauri::Runtime>) -> Result<()> {
    let api = app.android_fs_async();
    
    // Pick files to read and write
    let selected_files = api
        .file_picker()
        .pick_files(
            None, // Initial location
            &["*/*"], // Target MIME types
            false, // If true, only files on local device
        )
        .await?;

    if selected_files.is_empty() {
        // Handle cancel
    }
    else {
        for uri in selected_files {
            // Converts a URI into a path usable by Tauri's official FS plugin.
            // 
            // This is FilePath::Url(..)
            // Not FilePath::Path(..)
            let file_path: tauri_plugin_fs::FilePath = uri.clone().into();

            let file_type = api.get_mime_type(&uri).await?;
            let file_name = api.get_name(&uri).await?;
            let file_thumbnail = api.get_thumbnail(
                &uri, 
                Size { width: 200, height: 200}, 
                ImageFormat::Jpeg
            ).await?;

            {
                // Handle readonly file.
                let file: std::fs::File = api.open_file_readable(&uri).await?;
            }
        }
    }
    Ok(())
}
```

```rust
use tauri_plugin_android_fs::{AndroidFsExt, PublicImageDir, Result};

async fn file_saver_example(app: &tauri::AppHandle<impl tauri::Runtime>) -> Result<()> {
    let api = app.android_fs_async();

    // Directory on file picker launch
    // 
    // ~/Pictures/MyApp/2025-10-22/
    let initial_location = api
        .public_storage()
        .resolve_initial_location(
            None, // Storage volume (e.g. internal storage, SD card). If none, primary one
            PublicImageDir::Pictures, // Base directory
            "MyApp/2025-10-22", // Relative path
            true // Create direcotries if missing
        ).await?;

    // Pick/create file to save contents
    let selected_file = api
        .file_picker()
        .save_file(
            Some(&initial_location), // Initial location
            "my-image.jpg", // Initial file name
            Some("image/jpeg"), // MIME type
            false, // If true, only files on local device
        )
        .await?;

    if let Some(uri) = selected_file {

        // Handle writeonly file.
        // 
        // NOTE:
        // This truncate existing contents
        let file: std::fs::File = api.open_file_writable(&uri).await?;
    }
    else {
        // Handle cancel
    }

    Ok(())
}
```

```rust
use tauri_plugin_android_fs::{AndroidFsExt, Entry, Result};

async fn dir_picker_example(app: &tauri::AppHandle<impl tauri::Runtime>) -> Result<()> {
    let api = app.android_fs_async();

    // Pick directory to read and write
    let selected = api
        .file_picker()
        .pick_dir(
            None, // Initial location
            false, // If true, only directory on local device
        )
        .await?;

    if let Some(dir_uri) = selected {
        // Persist access permission across app/device restarts.
        api.take_persistable_uri_permission(&dir_uri).await?;
        
        // Read the directory
        for entry in api.read_dir(&dir_uri).await? {
            match entry {
                Entry::File { uri, name, .. } => {
                    // Handle a file
                },
                Entry::Dir { uri, name, .. } => {
                    // Handle a direcotry
                },
            }
        }

        // Create a new file in the directory.
        // The parent directories will be created recursively.
        let file_uri = api
            .create_new_file(
                &dir_uri, 
                "MyApp/2025-1021/file.txt", 
                Some("text/plain")
            )
            .await?;
    } 
    else {
        // Handle cancel
    }
    
    Ok(())
}
```

### 2. Public Storage
File storage that is available to other applications and users.  
For Android 9 (API level 29) or lower, please enable `legacy_storage_permission` feature.  

```rust
use tauri_plugin_android_fs::{AndroidFsExt, Error, PublicGeneralPurposeDir, PublicImageDir, Result};

async fn example(app: &tauri::AppHandle<impl tauri::Runtime>) -> Result<()> {
    let api = app.android_fs_async();

    // Request permission for PublicStorage
    //
    // NOTE:
    // Please enable 'legacy_storage_permission' feature,
    // for Android 9 or lower.
    if !api.public_storage().request_permission().await? {
        return Err(Error::with("Permission denied by user"))
    }

    // Save to new file.
    // 
    // Destination:
    // ~/Pictures/MyApp/my-image.png
    api.public_storage().write_new(
        // Storage volume (e.g. internal storage, SD card). 
        // If None, use primary storage volume
        None, 

        // Base directory. 
        // One of: PublicImageDir, PublicVideoDir, PublicAudioDir, PublicGeneralPurposeDir
        PublicImageDir::Pictures, 

        // Relative file path.
        // The parent directories will be created recursively.
        "MyApp/my-image.png",

        // Mime type.
        Some("image/png"),

        // Contents to save
        &[]
    ).await?;



    // Get any available volume other than the primary one if possible.
    // e.g. SD card, USB drive
    let volume = api
        .public_storage()
        .get_volumes().await?
        .into_iter()
        .find(|v| !v.is_primary && !v.is_readonly);

    // Create an empty file
    // and mark it pending (hidden from other apps).
    let uri = api
        .public_storage()
        .create_new_file_with_pending(
            volume.as_ref().map(|v| &v.id),
            PublicGeneralPurposeDir::Documents,
            "MyApp/2025-9-14/data.txt",
            Some("text/plain")
        ).await?;

    let mut file: std::fs::File = api.open_file_writable(&uri).await?;

    // Write content in blocking thread
    let result = tauri::async_runtime::spawn_blocking(move || -> Result<()> {
        use std::io::Write;

        // Write content
        file.write_all(&[])?;

        Ok(())
    }).await.map_err(Into::into).and_then(|r| r);

    // Handle error
    if let Err(err) = result {
        api.remove_file(&uri).await.ok();
        return Err(err)
    }

    // Clear pending state
    api.public_storage().set_pending(&uri, false).await?;

    // Register with Gallery
    api.public_storage().scan(&uri).await?;

    Ok(())
}
```

### 3. Private Storage
File storage intended for the app’s use only.

```rust
use tauri_plugin_android_fs::{AndroidFsExt, PrivateDir, Result};

async fn example(app: tauri::AppHandle<impl tauri::Runtime>) -> Result<()> {
    let ps = app
        .android_fs_async()
        .private_storage();

    // Get the absolute path.
    // Apps can fully manage entries within those directories with 'std::fs'.
    let cache_dir_path: std::path::PathBuf = ps.resolve_path(PrivateDir::Cache).await?;
    let data_dir_path: std::path::PathBuf = ps.resolve_path(PrivateDir::Data).await?;

    // Since these locations may contain files created by other Tauri plugins or webview systems, 
    // it is recommended to add a subdirectory with a unique name.
    let cache_dir_path = cache_dir_path.join("01K6049FVCD4SAGMAB6X20SA5S");
    let data_dir_path = data_dir_path.join("01K6049FVCD4SAGMAB6X20SA5S");

    Ok(())
}
```

# Link
- [Changelog](https://github.com/aiueo13/tauri-plugin-android-fs/blob/main/CHANGES.md)

# License
This project is licensed under either of

 * MIT license
 * Apache License (Version 2.0)

at your option.