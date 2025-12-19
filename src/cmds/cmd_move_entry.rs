use crate::*;

#[tauri::command]
pub async fn move_entry<R: tauri::Runtime>(
    uri: FileUri,
    dest_dir: FileUri,
    new_name: Option<String>,
    app: tauri::AppHandle<R>,
) -> Result<FileUri> {

    #[cfg(not(target_os = "android"))] {
        Err(Error::NOT_ANDROID)
    }
    #[cfg(target_os = "android")] {
        uri.require_content_scheme()?;

        let api = app.android_fs_async();
        api.move_entry(&uri, &dest_dir, new_name.as_deref()).await
    }
}
