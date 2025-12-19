use crate::*;

#[tauri::command]
pub async fn rename<R: tauri::Runtime>(
    uri: FileUri,
    new_name: String,
    app: tauri::AppHandle<R>,
) -> Result<FileUri> {

    #[cfg(not(target_os = "android"))] {
        Err(Error::NOT_ANDROID)
    }
    #[cfg(target_os = "android")] {
        uri.require_content_scheme()?;

        let api = app.android_fs_async();
        api.rename(&uri, new_name).await
    }
}
