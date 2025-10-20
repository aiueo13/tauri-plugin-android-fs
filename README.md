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

fn file_picker_example(app: tauri::AppHandle<impl tauri::Runtime>) -> Result<()> {
    let api = app.android_fs();
    
    // Pick files to read and write
    let selected_files = api.file_picker().pick_files(
        None, // Initial location
        &["*/*"], // Target MIME types
    )?;

    if selected_files.is_empty() {
        // Handle cancel
    }
    else {
        for uri in selected_files {
            // This is FilePath::Url(..)
            // Not FilePath::Path(..)
            let file_path: tauri_plugin_fs::FilePath = uri.clone().into();

            let file_type = api.get_mime_type(&uri)?;
            let file_name = api.get_name(&uri)?;
            let file_thumbnail = api.get_thumbnail(
                &uri, 
                Size { width: 200, height: 200}, 
                ImageFormat::Jpeg
            )?;

            {
                // Handle readonly file.
                let file: std::fs::File = api.open_file_readable(&uri)?;
            }

            {
                // Handle writeonly file. 
                // This truncate existing contents.
                let file: std::fs::File = api.open_file_writable(&uri)?;


                // But if you can, use 'open_writable_stream' instead.
                // See document of open_file_writable for details.
                // https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_sync/struct.AndroidFs.html#method.open_file_writable

                use std::io::{BufWriter, Write as _};

                let mut stream = api.open_writable_stream(&uri)?;
                stream.write_all(&[])?;
                stream.reflect()?;

                let mut stream = BufWriter::new(api.open_writable_stream(&uri)?);
                stream.write_all(&[])?;
                stream.into_inner()?.reflect()?;
            }
        }
    }
    Ok(())
}
```
```rust
use tauri_plugin_android_fs::{AndroidFsExt, Entry, Result};

fn dir_picker_example(app: tauri::AppHandle<impl tauri::Runtime>) -> Result<()> {
    let api = app.android_fs();

    // Pick directory to read and write
    let selected = api.file_picker().pick_dir(
        None, // Initial location
    )?;

    if let Some(dir_uri) = selected {
        // Persist access permission across app restarts.
        api.take_persistable_uri_permission(&dir_uri)?;
        
        // Read the directory
        for entry in api.read_dir(&dir_uri)? {
            match entry {
                Entry::File { uri, name, .. } => {
                    // Handle file
                },
                Entry::Dir { uri, name, .. } => {
                    // Handle directory
                },
            }
        }

        // Create a new file in the directory
        let file_uri = api.create_new_file(
            &dir_uri, 
            "MyApp/file.txt", 
            Some("text/plain")
        )?;
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

fn example(app: tauri::AppHandle<impl tauri::Runtime>) -> Result<()> {
    let api = app.android_fs();
    let storage = api.public_storage();

    // Create an empty PNG file
    // and mark it pending (hidden from other apps).
    // 
    // ~/Pictures/MyApp/my-image.png
    let uri = storage.create_new_file_with_pending(
        // Storage volume (e.g. internal storage, SD card). 
        // If None, use primary storage volume
        None, 

        // Base directory. 
        // One of: PublicImageDir, PublicVideoDir, PublicAudioDir, PublicGeneralPurposeDir
        PublicImageDir::Pictures, 

        // Relative file path.
        // The directories will be created automatically.
        "MyApp/my-image.png",

        // Mime type.
        Some("image/png") 
    )?;

    // Write the contents to the PNG image
    if let Err(e) = api.write(&uri, &[]) {
        // handle err
        api.remove_file(&uri).ok();
        return Err(e)
    }

    // Clear pending state.
    storage.set_pending(&uri, false)?;



    // Get any available volume other than the primary one.
    // e.g. SD card, USB drive
    let volume = storage.get_volumes()?
        .into_iter()
        .find(|v| !v.is_primary && !v.is_readonly);

    let uri = storage.create_new_file(
         volume.as_ref().map(|v| &v.id),
         PublicGeneralPurposeDir::Documents,
         "MyApp/2025-9-14/data.txt",
         Some("text/plain")
    )?;

    let mut file: std::fs::File = api.open_file_writable(&uri)?;

    Ok(())
}
```

### 3. Private Storage
File storage intended for the app’s use only.

```rust
use tauri_plugin_android_fs::{AndroidFsExt, PrivateDir, Result};

fn example(app: tauri::AppHandle<impl tauri::Runtime>) -> Result<()> {
    let storage = app.android_fs().private_storage();

    // Get the absolute path.
    // Apps can fully manage entries within those directories with 'std::fs'.
    let cache_dir_path: std::path::PathBuf = storage.resolve_path(PrivateDir::Cache)?;
    let data_dir_path: std::path::PathBuf = storage.resolve_path(PrivateDir::Data)?;

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