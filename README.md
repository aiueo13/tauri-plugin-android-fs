Note: **I’m using a translation tool, so there may be some inappropriate expressions.**

# Overview

The Android file system is strict and complex. This plugin was created to provide practical file operations. You don’t need to set special permissions or configurations. 

And this does not use any options that require additional permissions or review, so you can submit your app for Google Play review with peace of mind.

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
This plugin only provides a Rust-side API. 
If you need to use file data on frontend, consider using Tauri’s custom protocols for efficient transmission. Or convert `tauri_plugin_android_fs::FileUri` to `tauri_plugin_fs::FilePath` and use tauri_plugin_fs functions on frontend. 

### 1. Dialog

Opens the file/folder picker to read and write user-selected entries.

```rust
use tauri_plugin_android_fs::{AndroidFsExt, ImageFormat, Result, Size};

async fn file_picker_example(app: tauri::AppHandle<impl tauri::Runtime>) -> Result<()> {
    let api = app.android_fs_async();
    
    // Pick files to read and write
    let selected_files = api
        .file_picker()
        .pick_files(
            None, // Initial location
            &["*/*"], // Target MIME types
        )
        .await?;

    if selected_files.is_empty() {
        // Handle cancel
    }
    else {
        for uri in selected_files {
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

            {
                // Handle writeonly file. 
                // This truncate existing contents.
                let file: std::fs::File = api.open_file_writable(&uri).await?;


                // But, writing files via file picker,
                // consider using 'open_writable_stream' instead.
                // See document of 'open_file_writable' for reason.
                // https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_sync/struct.AndroidFs.html#method.open_file_writable
                
                use std::io::{BufWriter, Write as _};

                let mut stream = api.open_writable_stream(&uri).await?;
                // Write contents in blocking thread
                let stream = tauri::async_runtime::spawn_blocking(move || -> Result<_> {
                    stream.write_all(&[])?;
                    Ok(stream)
                }).await??;
                // Finish writing
                // This is required
                stream.reflect().await?;

                let stream = api.open_writable_stream(&uri).await?;
                tauri::async_runtime::spawn_blocking(move || -> Result<()> {
                    let mut buf_stream = BufWriter::new(stream);

                    for i in 0..100 {
                        buf_stream.write(&[i])?;
                    }

                    buf_stream.flush()?;

                    let stream = buf_stream
                        .into_inner()?
                        .into_sync(); // Into synchronous WritableStream for follwing functions

                    stream.sync_all()?; // This is optional
                    stream.reflect()?; // This is required

                    Ok(())
                }).await??;
            }
        }
    }
    Ok(())
}
```
```rust
use tauri_plugin_android_fs::{AndroidFsExt, Entry, Result};

async fn dir_picker_example(app: tauri::AppHandle<impl tauri::Runtime>) -> Result<()> {
    let api = app.android_fs_async();

    // Pick directory to read and write
    let selected = api
        .file_picker()
        .pick_dir(
            None, // Initial location
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
This is for Android 10 (API level 29) or higher.  

```rust
use tauri_plugin_android_fs::{AndroidFsExt, PublicGeneralPurposeDir, PublicImageDir, Result};

async fn example(app: tauri::AppHandle<impl tauri::Runtime>) -> Result<()> {
    let api = app.android_fs_async();

    // Create an empty file
    // and mark it pending (hidden from other apps).
    // 
    // ~/Pictures/MyApp/my-image.png
    let uri = api
        .public_storage()
        .create_new_file_with_pending(
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
            Some("image/png") 
        ).await?;

    // Write the contents to the file
    if let Err(e) = api.write(&uri, &[]).await {
        // Handle err
        api.remove_file(&uri).await.ok();
        return Err(e)
    }

    // Clear pending state
    api.public_storage()
        .set_pending(&uri, false).await?;



    // Get any available volume other than the primary one.
    // e.g. SD card, USB drive
    let volume = api
        .public_storage()
        .get_volumes().await?
        .into_iter()
        .find(|v| !v.is_primary && !v.is_readonly);

    let uri = api
        .public_storage()
        .create_new_file(
            volume.as_ref().map(|v| &v.id),
            PublicGeneralPurposeDir::Documents,
            "MyApp/2025-9-14/data.txt",
            Some("text/plain")
        ).await?;

    let mut file: std::fs::File = api.open_file_writable(&uri).await?;

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
MIT OR Apache-2.0