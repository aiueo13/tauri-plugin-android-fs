fn main() {}


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

    // Save jpeg image to new file.
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
    api.public_storage()
        .set_pending(&uri, false).await?;

    // Register with Gallery
    api.public_storage()
        .scan_file(&uri).await?;

    Ok(())
}