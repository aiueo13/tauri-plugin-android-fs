Note: **I’m using a translation tool, so there may be some inappropriate expressions.**

# Overview

The Android file system is strict and complex. This plugin was created to provide practical file operations. You don’t need to set special permissions or configurations. 

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

There are three main ways to manipulate files:

### 1. Dialog

Opens the file/folder picker to read and write user-selected entries.

```rust
use tauri_plugin_android_fs::{AndroidFsExt, FileAccessMode, ImageFormat, Size};

fn file_picker_example(app: tauri::AppHandle) -> tauri_plugin_android_fs::Result<()> {
    let api = app.android_fs();
    
    // pick files to read and write
    let selected_files = api.file_picker().pick_files(
        None, // Initial location
        &["*/*"], // Target MIME types
        true, // Allow multiple files
    )?;

    if selected_files.is_empty() {
        // Handle cancel
    }
    else {
        for uri in selected_files {
            // This is FilePath::Url(..)
            // Not FilePath::Path(..)
            let file_path: tauri_plugin_fs::FilePath = uri.clone().into();

            let file_type = api.get_mime_type(&uri)?.unwrap(); // If file, this returns no None.
            let file_name = api.get_name(&uri)?;
            let file_thumbnail = api.get_thumbnail(
                &uri, 
                Size { width: 200, height: 200}, 
                ImageFormat::Jpeg
            )?;

            {
                // Handle readonly file.
                let file: std::fs::File = api.open_file(&uri, FileAccessMode::Read)?;
            }

            {
                // Handle writeonly file. 
                // This truncate existing contents.
                let file: std::fs::File = api.open_file(&uri, FileAccessMode::WriteTruncate)?;
            }
        }
    }
    Ok(())
}
```
```rust
use tauri_plugin_android_fs::{AndroidFsExt, Entry};

fn dir_picker_example(app: tauri::AppHandle) -> tauri_plugin_android_fs::Result<()> {
    let api = app.android_fs();

    // Pick folder to read and write
    let selected_folder = api.file_picker().pick_dir(
        None, // Initial location
    )?;

    if let Some(dir_uri) = selected_folder {
        for entry in api.read_dir(&dir_uri)? {
            match entry {
                Entry::File { name, uri, last_modified, len, mime_type, .. } => {
                    // Handle file
                },
                Entry::Dir { name, uri, last_modified, .. } => {
                    // Handle folder
                },
            }
        }
    } 
    else {
        // Handle cancel
    }
    
    Ok(())
}
```
```rust
use tauri_plugin_android_fs::{AndroidFsExt, FileUri, InitialLocation, PersistableAccessMode, PrivateDir};

/// Opens a dialog to save the file,
/// then write contents to the selected file.
/// 
/// return Ok(false) when canceled by user.  
/// return Ok(true) when success.
fn save_file_with_file_saver(
    app: tauri::AppHandle,
    file_name: &str,
    mime_type: &str,
    contents: &[u8],
) -> tauri_plugin_android_fs::Result<bool> {

    let api = app.android_fs();

    // Pick file to write
    let file_uri = api.file_picker().save_file(
        None, // Initial location
        file_name, // Initial file name
        Some(mime_type), // MIME type
    )?;

    let Some(file_uri) = file_uri else {
        return Ok(false)
    };

    // Write contents
    if let Err(e) = api.write(&file_uri, contents) {
        // Handle err
        let _ = api.remove_file(&file_uri);
        return Err(e)
    }
    
    Ok(true)
}

/// Open a dialog to select a directory, 
/// and create a new file at the relative_path position from it,
/// then write contents.  
/// If a folder has been selected in the past, use it without opening a dialog.
/// 
/// return Ok(false) when canceled by user.  
/// return Ok(true) when success.  
fn save_file_with_dir_picker(
    app: tauri::AppHandle, 
    relative_path: &str,
    mime_type: &str,
    contents: &[u8],
) -> tauri_plugin_android_fs::Result<bool> {

    const DEST_DIR_URI_DATA_RELATIVE_PATH: &str = "01JQMFWVH65YNCWM31V3DZG6GR";
    let api = app.android_fs();

    // Retrieve previously retrieved dest dir uri, if exists.
    let dest_dir_uri = api
        .private_storage()
        .read_to_string(PrivateDir::Data, DEST_DIR_URI_DATA_RELATIVE_PATH)
        .and_then(|u| FileUri::from_str(&u))
        .ok();

    // Check permission, if exists.
    let dest_dir_uri = match dest_dir_uri {
        Some(dest_dir_uri) => {
            if api.check_persisted_uri_permission(&dest_dir_uri, PersistableAccessMode::ReadAndWrite)? {
                Some(dest_dir_uri)
            }
            else {
                None
            }
        },
        None => None
    };
    
    // If there is no valid dest dir, select a new one
    let dest_dir_uri = match dest_dir_uri {
        Some(dest_dir_uri) => dest_dir_uri,
        None => {
            // Get initial location for folder picker.
            // But this returned uri might be ignored by it.
            let initial_location = api.resolve_initial_location(
                InitialLocation::TopPublicDir,
                false
            )?;

            // Show folder picker
            let uri = api.file_picker().pick_dir(
                Some(&initial_location)
            )?;

            let Some(uri) = uri else {
                // Canceled by user
                return Ok(false)
            };

            // Store uri
            api.private_storage().write(
                PrivateDir::Data, 
                DEST_DIR_URI_DATA_RELATIVE_PATH, 
                uri.to_string()?.as_bytes()
            )?;

            // Persist uri permission across app restarts
            api.take_persistable_uri_permission(&uri)?;

            uri
        },
    };
    
    // Create a new empty file in dest folder
    let new_file_uri = api.create_file(
        &dest_dir_uri, 
        relative_path, 
        Some(mime_type)
    )?;

    // Write contents
    if let Err(e) = api.write(&new_file_uri, contents) {
        // Handle err
        let _ = api.remove_file(&new_file_uri);
        return Err(e)
    }
    // or
    // let mut file: std::fs::File = api.open_file(&new_file_uri, FileAccessMode::WriteTruncate)?;
    
    Ok(true)
}
```

### 2. Public Storage
File storage that is available to other applications and users.
This is for Android 10 (API level 29) or higher.  

```rust
use tauri_plugin_android_fs::{AndroidFsExt, PublicGeneralPurposeDir, PublicImageDir};

fn example(app: tauri::AppHandle) -> tauri_plugin_android_fs::Result<()> {
    let api = app.android_fs();
    let storage = api.public_storage();
    let contents = &[];

    // Create a new empty PNG image file
    //
    // This path is represented as follows:
    //   ~/Pictures/{app_name}/my-image.png
    //   $HOME/Pictures/{app_name}/my-image.png
    //   /storage/emulated/0/Pictures/{app_name}/my-image.png
    let uri = storage.create_file_in_app_dir(
         PublicImageDir::Pictures, // Base directory
         "my-image.png", // Relative file path
         Some("image/png") // Mime type
    )?;

    // Write the contents to the PNG image
    if let Err(e) = api.write(&uri, contents) {
        // handle err
        let _ = api.remove_file(&uri);
        return Err(e)
    }


    // Create a new empty text file.
    // All subdirectories are created automatically.
    //
    // This path is represented as follows:
    //   ~/Documents/{app_name}/2025-3-2/data.txt
    //   $HOME/Documents/{app_name}/2025-3-2/data.txt
    //   /storage/emulated/0/Documents/{app_name}/2025-3-2/data.txt
    let uri = storage.create_file_in_app_dir(
         PublicGeneralPurposeDir::Documents, // Base directory
         "2025-3-2/data.txt", // Relative file path
         Some("text/plain") // Mime type
    )?;

    // Write the contents to the text file
    if let Err(e) = api.write(&uri, contents) {
        // Handle err
        let _ = api.remove_file(&uri);
        return Err(e)
    }

    Ok(())
}
```

### 3. Private Storage
File storage intended for the app’s use only.

```rust
use tauri_plugin_android_fs::{AndroidFsExt, PrivateDir};

fn example(app: tauri::AppHandle) -> tauri_plugin_android_fs::Result<()> {
    let storage = app.android_fs().private_storage();
    let contents = &[];

    // Get the absolute path.
    // Apps can fully manage entries within this directory.
    let _cache_dir_path: std::path::PathBuf = storage.resolve_path(PrivateDir::Cache)?;
    let _data_dir_path: std::path::PathBuf = storage.resolve_path(PrivateDir::Data)?;


    // Write the contents.
    // This is wrapper of above resolve_path and std::fs
    storage.write(
        PrivateDir::Data, // Base directory
        "config/data1", // Relative file path
        contents
    )?;

    // Read the contents.
    // This is wrapper of above resolve_path and std::fs
    let contents = storage.read(
        PrivateDir::Data, // Base directory
        "config/data1" // Relative file path
    )?;

    Ok(())
}
```

# Link
- [Changelog](https://github.com/aiueo13/tauri-plugin-android-fs/blob/main/CHANGES.md)

# License
MIT OR Apache-2.0