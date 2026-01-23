mod utils;
mod scope;

use scope::*;
use utils::*;
use serde::{Deserialize, Serialize};
use crate::*;


#[tauri::command]
pub async fn get_android_api_level<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<i32> {

    #[cfg(not(target_os = "android"))] {
        Err(Error::NOT_ANDROID)
    }
    #[cfg(target_os = "android")] {
        let api = app.android_fs_async();
        api.api_level()
    }
}

#[tauri::command]
pub async fn get_name<R: tauri::Runtime>(
    uri: AfsUriOrFsPath,
    app: tauri::AppHandle<R>,
    cmd_scope: tauri::ipc::CommandScope<Scope>,
    global_scope: tauri::ipc::GlobalScope<Scope>,
) -> Result<String> {

    #[cfg(not(target_os = "android"))] {
        Err(Error::NOT_ANDROID)
    }
    #[cfg(target_os = "android")] {
        let uri = FileUri::from(uri);
        if let Some(path) = uri.as_path() {
            validate_path_permission(path, &app, &cmd_scope, &global_scope)?;
        }

        let api = app.android_fs_async();
        api.get_name(&uri).await
    }
}

#[tauri::command]
pub async fn get_byte_length<R: tauri::Runtime>(
    uri: AfsUriOrFsPath,
    app: tauri::AppHandle<R>,
    cmd_scope: tauri::ipc::CommandScope<Scope>,
    global_scope: tauri::ipc::GlobalScope<Scope>,
) -> Result<u64> {

    #[cfg(not(target_os = "android"))] {
        Err(Error::NOT_ANDROID)
    }
    #[cfg(target_os = "android")] {
        let uri = FileUri::from(uri);
        if let Some(path) = uri.as_path() {
            validate_path_permission(path, &app, &cmd_scope, &global_scope)?;
        }

        let api = app.android_fs_async();
        api.get_len(&uri).await
    }
}

#[tauri::command]
pub async fn get_mime_type<R: tauri::Runtime>(
    uri: AfsUriOrFsPath,
    app: tauri::AppHandle<R>,
    cmd_scope: tauri::ipc::CommandScope<Scope>,
    global_scope: tauri::ipc::GlobalScope<Scope>,
) -> Result<String> {

    #[cfg(not(target_os = "android"))] {
        Err(Error::NOT_ANDROID)
    }
    #[cfg(target_os = "android")] {
        let uri = FileUri::from(uri);
        if let Some(path) = uri.as_path() {
            validate_path_permission(path, &app, &cmd_scope, &global_scope)?;
        }

        let api = app.android_fs_async();
        api.get_mime_type(&uri).await
    }
}

#[tauri::command]
pub async fn get_type<R: tauri::Runtime>(
    uri: AfsUriOrFsPath,
    app: tauri::AppHandle<R>,
    cmd_scope: tauri::ipc::CommandScope<Scope>,
    global_scope: tauri::ipc::GlobalScope<Scope>,
) -> Result<EntryType> {

    #[cfg(not(target_os = "android"))] {
        Err(Error::NOT_ANDROID)
    }
    #[cfg(target_os = "android")] {
        let uri = FileUri::from(uri);
        if let Some(path) = uri.as_path() {
            validate_path_permission(path, &app, &cmd_scope, &global_scope)?;
        }

        let api = app.android_fs_async();
        api.get_type(&uri).await
    }
}

#[tauri::command]
pub async fn get_metadata<R: tauri::Runtime>(
    uri: AfsUriOrFsPath,
    app: tauri::AppHandle<R>,
    cmd_scope: tauri::ipc::CommandScope<Scope>,
    global_scope: tauri::ipc::GlobalScope<Scope>,
) -> Result<impl Serialize> {

    #[cfg(not(target_os = "android"))] {
        Result::<String>::Err(Error::NOT_ANDROID)
    }
    #[cfg(target_os = "android")] {
        #[derive(Serialize)]
        #[serde(tag = "type")]
        enum EntryMetadata {
            File {
                name: String,

                #[serde(rename = "lastModified")]
                last_modified: f64,

                #[serde(rename = "byteLength")]
                len: u64,

                #[serde(rename = "mimeType")]
                mime_type: String,
            },
            Dir {
                name: String,

                #[serde(rename = "lastModified")]
                last_modified: f64,
            }
        }

        let uri = FileUri::from(uri);
        if let Some(path) = uri.as_path() {
            validate_path_permission(path, &app, &cmd_scope, &global_scope)?;
        }

        let api = app.android_fs_async();
        let entry = api.get_info(&uri).await?;

        match api.get_info(&uri).await? {
            Entry::File { name, last_modified, len, mime_type, .. } => {
                let last_modified = convert_time_to_f64_millis(last_modified)?;
                Ok(EntryMetadata::File { name, last_modified, len, mime_type })
            },
            Entry::Dir { name, last_modified, .. } => {
                let last_modified = convert_time_to_f64_millis(last_modified)?;
                Ok(EntryMetadata::Dir { name, last_modified })
            },
        }
    }
}

#[tauri::command]
pub fn get_fs_path(uri: AfsUriOrFsPath) -> Result<tauri_plugin_fs::FilePath> {
    #[cfg(not(target_os = "android"))] {
        Err(Error::NOT_ANDROID)
    }
    #[cfg(target_os = "android")] {
        Ok(uri.into())
    }
}

#[tauri::command]
pub async fn get_thumbnail<R: tauri::Runtime>(
    uri: AfsUriOrFsPath,
    width: f64,
    height: f64,
    format: String,
    app: tauri::AppHandle<R>,
    cmd_scope: tauri::ipc::CommandScope<Scope>,
    global_scope: tauri::ipc::GlobalScope<Scope>,
) -> Result<tauri::ipc::Response> {

    #[cfg(not(target_os = "android"))] {
        Err(Error::NOT_ANDROID)
    }
    #[cfg(target_os = "android")] {
        let uri = FileUri::from(uri);
        if let Some(path) = uri.as_path() {
            validate_path_permission(path, &app, &cmd_scope, &global_scope)?;
        }

        let format = convert_to_image_format(&format)?;
        let size = convert_to_thumbnail_preferred_size(width, height)?;
        let api = app.android_fs_async();

        let Some(bytes) = api.get_thumbnail(&uri, size, format).await? else {
            return Ok(tauri::ipc::Response::new(Vec::with_capacity(0)))
        };
    
        Ok(tauri::ipc::Response::new(bytes))
    }
}

#[tauri::command]
pub async fn get_thumbnail_base64<R: tauri::Runtime>(
    uri: AfsUriOrFsPath,
    width: f64,
    height: f64,
    format: String,
    app: tauri::AppHandle<R>,
    cmd_scope: tauri::ipc::CommandScope<Scope>,
    global_scope: tauri::ipc::GlobalScope<Scope>,
) -> Result<Option<String>> {

    #[cfg(not(target_os = "android"))] {
        Err(Error::NOT_ANDROID)
    }
    #[cfg(target_os = "android")] {
        let uri = FileUri::from(uri);
        if let Some(path) = uri.as_path() {
            validate_path_permission(path, &app, &cmd_scope, &global_scope)?;
        }

        let format = convert_to_image_format(&format)?;
        let size = convert_to_thumbnail_preferred_size(width, height)?;
        let api = app.android_fs_async();

        api.get_thumbnail_base64(&uri, size, format).await
    }
}

#[tauri::command]
pub async fn get_thumbnail_data_url<R: tauri::Runtime>(
    uri: AfsUriOrFsPath,
    width: f64,
    height: f64,
    format: String,
    app: tauri::AppHandle<R>,
    cmd_scope: tauri::ipc::CommandScope<Scope>,
    global_scope: tauri::ipc::GlobalScope<Scope>,
) -> Result<Option<String>> {

    #[cfg(not(target_os = "android"))] {
        Err(Error::NOT_ANDROID)
    }
    #[cfg(target_os = "android")] {
        let uri = FileUri::from(uri);
        if let Some(path) = uri.as_path() {
            validate_path_permission(path, &app, &cmd_scope, &global_scope)?;
        }

        let format = convert_to_image_format(&format)?;
        let size = convert_to_thumbnail_preferred_size(width, height)?;
        let api = app.android_fs_async();
    
        let Some(data) = api.get_thumbnail_base64(&uri, size, format).await? else {
            return Ok(None)
        };
    
        let mime_type = format.mime_type();
        let prefix = format!("data:{mime_type};base64,");
        let mut data_url = String::with_capacity(prefix.len() + data.len());
        data_url.push_str(&prefix);
        data_url.push_str(&data);
        Ok(Some(data_url))
    }
}

#[tauri::command]
pub async fn list_volumes<R: tauri::Runtime>(
    app: tauri::AppHandle<R>
) -> Result<Vec<impl Serialize>> {

    #[cfg(not(target_os = "android"))] {
        Result::<Vec<String>>::Err(Error::NOT_ANDROID)
    }
    #[cfg(target_os = "android")] {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct StorageVolumeInfo {
            description: String,
            is_primary: bool,
            is_removable: bool,
            is_stable: bool,
            is_emulated: bool,
            id: String,
        }

        let api = app.android_fs_async();
        let volumes = api.get_volumes().await?
            .into_iter()
            .filter(|v| v.is_available_for_public_storage)
            .filter(|v| !v.is_readonly)
            .filter_map(|v| convert_from_storage_volume_id(&v.id).map(|id| (v, id)).ok())
            .map(|(v, id)| StorageVolumeInfo {
                description: v.description, 
                is_primary: v.is_primary, 
                is_removable: v.is_removable, 
                is_stable: v.is_stable, 
                is_emulated: v.is_emulated, 
                id
            })
        .   collect::<Vec<_>>();

        Ok(volumes)
    }
}

#[cfg(target_os = "android")]
async fn create_new_public_file_inner<R: tauri::Runtime>(
    volume_id: Option<String>,
    base_dir: impl Into<PublicDir>,
    relative_path: String,
    mime_type: Option<&str>,
    request_permission: bool,
    app: tauri::AppHandle<R>,
) -> Result<FileUri> {

    let volume_id = match volume_id {
        Some(volume_id) => Some(convert_to_storage_volume_id(&volume_id)?),
        None => None,
    };
    let api = app.android_fs_async();

    if request_permission {
        api.public_storage().request_permission().await?;
    }
    
    api.public_storage().create_new_file(
        volume_id.as_ref(),
        base_dir,
        relative_path.trim_start_matches('/'),
        mime_type
    ).await
}

#[tauri::command]
pub async fn create_new_public_file<R: tauri::Runtime>(
    volume_id: Option<String>,
    base_dir: PublicGeneralPurposeDir,
    relative_path: String,
    mime_type: Option<String>,
    request_permission: bool,
    app: tauri::AppHandle<R>,
) -> Result<FileUri> {

    #[cfg(not(target_os = "android"))] {
        Err(Error::NOT_ANDROID)
    }
    #[cfg(target_os = "android")] {
        create_new_public_file_inner(
            volume_id,
            base_dir,
            relative_path,
            mime_type.as_deref(),
            request_permission,
            app
        ).await
    }
}

#[tauri::command]
pub async fn create_new_public_image_file<R: tauri::Runtime>(
    volume_id: Option<String>,
    base_dir: PublicImageOrGeneralPurposeDir,
    relative_path: String,
    mime_type: Option<String>,
    request_permission: bool,
    app: tauri::AppHandle<R>,
) -> Result<FileUri> {

    #[cfg(not(target_os = "android"))] {
        Err(Error::NOT_ANDROID)
    }
    #[cfg(target_os = "android")] {
        let mime_type = resolve_mime_type(mime_type.as_deref(), &relative_path, &app).await?;

        if !mime_type.starts_with("image/") {
            return Err(Error::with(format!("invalid image type: {mime_type}")))
        }

        create_new_public_file_inner(
            volume_id,
            base_dir,
            relative_path,
            Some(mime_type.as_ref()),
            request_permission,
            app
        ).await
    }
}

#[tauri::command]
pub async fn create_new_public_video_file<R: tauri::Runtime>(
    volume_id: Option<String>,
    base_dir: PublicVideoOrGeneralPurposeDir,
    relative_path: String,
    mime_type: Option<String>,
    request_permission: bool,
    app: tauri::AppHandle<R>,
) -> Result<FileUri> {

    #[cfg(not(target_os = "android"))] {
        Err(Error::NOT_ANDROID)
    }
    #[cfg(target_os = "android")] {
        let mime_type = resolve_mime_type(mime_type.as_deref(), &relative_path, &app).await?;

        if !mime_type.starts_with("video/") {
            return Err(Error::with(format!("invalid video type: {mime_type}")))
        }

        create_new_public_file_inner(
            volume_id,
            base_dir,
            relative_path,
            Some(mime_type.as_ref()),
            request_permission,
            app
        ).await
    }
}

#[tauri::command]
pub async fn create_new_public_audio_file<R: tauri::Runtime>(
    volume_id: Option<String>,
    base_dir: PublicAudioOrGeneralPurposeDir,
    relative_path: String,
    mime_type: Option<String>,
    request_permission: bool,
    app: tauri::AppHandle<R>,
) -> Result<FileUri> {

    #[cfg(not(target_os = "android"))] {
        Err(Error::NOT_ANDROID)
    }
    #[cfg(target_os = "android")] {
        let mime_type = resolve_mime_type(mime_type.as_deref(), &relative_path, &app).await?;

        if !mime_type.starts_with("audio/") {
            return Err(Error::with(format!("invalid audio type: {mime_type}")))
        }

        let mut base_dir: PublicDir = base_dir.into();
        let mut relative_path = relative_path.trim_start_matches('/').to_string();

        let api = app.android_fs_async();
        let ps = api.public_storage();
        if base_dir == PublicAudioDir::Audiobooks.into() && !ps.is_audiobooks_dir_available()? {
            base_dir = PublicAudioDir::Music.into();
            relative_path = format!("Audiobooks/{}", relative_path)
        }
        if base_dir == PublicAudioDir::Recordings.into() && !ps.is_recordings_dir_available()? {
            base_dir = PublicAudioDir::Music.into();
            relative_path = format!("Recordings/{}", relative_path)
        }

        create_new_public_file_inner(
            volume_id, 
            base_dir, 
            relative_path, 
            Some(mime_type.as_ref()), 
            request_permission, 
            app
        ).await
    }
}

#[tauri::command]
pub async fn scan_public_file<R: tauri::Runtime>(
    uri: FileUri,
    app: tauri::AppHandle<R>,
) -> Result<()> {

    #[cfg(not(target_os = "android"))] {
        Err(Error::NOT_ANDROID)
    }
    #[cfg(target_os = "android")] {
        uri.require_content_scheme()?;

        let api = app.android_fs_async();
        api.public_storage().scan(&uri).await?;
        Ok(())
    }
}

#[tauri::command]
pub async fn request_public_files_permission<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<bool> {

    #[cfg(not(target_os = "android"))] {
        Err(Error::NOT_ANDROID)
    }
    #[cfg(target_os = "android")] {
        let api = app.android_fs_async();
        api.public_storage().request_permission().await
    }
}

#[tauri::command]
pub async fn has_public_files_permission<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<bool> {

    #[cfg(not(target_os = "android"))] {
        Err(Error::NOT_ANDROID)
    }
    #[cfg(target_os = "android")] {
        let api = app.android_fs_async();
        api.public_storage().has_permission().await
    }
}

#[tauri::command]
pub async fn create_dir_all<R: tauri::Runtime>(
    base_dir_uri: FileUri,
    relative_path: String,
    app: tauri::AppHandle<R>
) -> Result<FileUri> {

    #[cfg(not(target_os = "android"))] {
        Err(Error::NOT_ANDROID)
    }
    #[cfg(target_os = "android")] {
        base_dir_uri.require_content_scheme()?;

        let api = app.android_fs_async();
        api.create_dir_all(&base_dir_uri, relative_path).await
    }
}

#[tauri::command]
pub async fn create_new_file<R: tauri::Runtime>(
    base_dir_uri: FileUri,
    relative_path: String,
    mime_type: Option<String>,
    app: tauri::AppHandle<R>,
) -> Result<FileUri> {

    #[cfg(not(target_os = "android"))] {
        Err(Error::NOT_ANDROID)
    }
    #[cfg(target_os = "android")] {
        base_dir_uri.require_content_scheme()?;

        let api = app.android_fs_async();
        api.create_new_file(&base_dir_uri, relative_path, mime_type.as_deref()).await
    }
}

#[tauri::command]
pub async fn copy_file<R: tauri::Runtime>(
    src_uri: AfsUriOrFsPath,
    dest_uri: AfsUriOrFsPath,
    app: tauri::AppHandle<R>,
    cmd_scope: tauri::ipc::CommandScope<Scope>,
    global_scope: tauri::ipc::GlobalScope<Scope>,
) -> Result<()> {

    #[cfg(not(target_os = "android"))] {
        Err(Error::NOT_ANDROID)
    }
    #[cfg(target_os = "android")] {
        let src_uri: FileUri = src_uri.into();
        let dest_uri: FileUri = dest_uri.into();
        let api = app.android_fs_async();

        if let Some(src_path) = src_uri.as_path() {
            validate_path_permission(src_path, &app, &cmd_scope, &global_scope)?;
        }
        if let Some(dest_path) = dest_uri.as_path() {
            validate_path_permission(dest_path, &app, &cmd_scope, &global_scope)?;

            if !std::fs::exists(dest_path)? {
                std::fs::File::create(dest_path)?;
            }
        }

        api.copy(&src_uri, &dest_uri).await
    }
}

#[tauri::command]
pub async fn truncate_file<R: tauri::Runtime>(
    uri: FileUri,
    app: tauri::AppHandle<R>
) -> Result<()> {

    #[cfg(not(target_os = "android"))] {
        Err(Error::NOT_ANDROID)
    }
    #[cfg(target_os = "android")] {
        uri.require_content_scheme()?;

        let api = app.android_fs_async();
        api.open_file_writable(&uri).await?;
        Ok(())
    }
}

#[tauri::command]
pub async fn read_dir<R: tauri::Runtime>(
    uri: FileUri,
    app: tauri::AppHandle<R>
) -> Result<Vec<impl Serialize>> {

    #[cfg(not(target_os = "android"))] {
        Result::<Vec<String>>::Err(Error::NOT_ANDROID)
    }
    #[cfg(target_os = "android")] {
        #[derive(Serialize)]
        #[serde(tag = "type")]
        enum EntryMetadataWithUri {
            File {
                name: String,
                uri: FileUri,

                #[serde(rename = "lastModified")]
                last_modified: f64,

                #[serde(rename = "byteLength")]
                len: u64,

                #[serde(rename = "mimeType")]
                mime_type: String,
            },
            Dir {
                name: String,
                uri: FileUri,

                #[serde(rename = "lastModified")]
                last_modified: f64,
            }
        }

        uri.require_content_scheme()?;

        let api = app.android_fs_async();
        let entries = api.read_dir(&uri).await?;
        let mut buffer = Vec::new();

        for entry in entries {
            let entry = match entry {
                Entry::File { uri, name, last_modified, len, mime_type } => {
                    let last_modified = convert_time_to_f64_millis(last_modified)?;
                    EntryMetadataWithUri::File { name, uri, len, mime_type, last_modified }
                },
                Entry::Dir { uri, name, last_modified } => {
                    let last_modified = convert_time_to_f64_millis(last_modified)?;
                    EntryMetadataWithUri::Dir { name, uri, last_modified }
                },
            };
            buffer.push(entry);
        }

        Ok(buffer)
    }
}

#[tauri::command]
pub async fn rename_file<R: tauri::Runtime>(
    uri: FileUri,
    name: String,
    app: tauri::AppHandle<R>,
) -> Result<FileUri> {

    #[cfg(not(target_os = "android"))] {
        Err(Error::NOT_ANDROID)
    }
    #[cfg(target_os = "android")] {
        uri.require_content_scheme()?;

        let api = app.android_fs_async();

        if !api.get_type(&uri).await?.is_file() {
            return Err(Error::with("not a file: {uri:?}"))
        }

        api.rename(&uri, name).await
    }
}

#[tauri::command]
pub async fn rename_dir<R: tauri::Runtime>(
    uri: FileUri,
    name: String,
    app: tauri::AppHandle<R>,
) -> Result<FileUri> {

    #[cfg(not(target_os = "android"))] {
        Err(Error::NOT_ANDROID)
    }
    #[cfg(target_os = "android")] {
        uri.require_content_scheme()?;

        let api = app.android_fs_async();

        if !api.get_type(&uri).await?.is_dir() {
            return Err(Error::with("not a directory: {uri:?}"))
        }

        api.rename(&uri, name).await
    }
}

#[tauri::command]
pub async fn remove_file<R: tauri::Runtime>(
    uri: FileUri,
    app: tauri::AppHandle<R>,
) -> Result<()> {

    #[cfg(not(target_os = "android"))] {
        Err(Error::NOT_ANDROID)
    }
    #[cfg(target_os = "android")] {
        uri.require_content_scheme()?;

        let api = app.android_fs_async();
        api.remove_file(&uri).await?;
        Ok(())
    }
}

#[tauri::command]
pub async fn remove_empty_dir<R: tauri::Runtime>(
    uri: FileUri,
    app: tauri::AppHandle<R>,
) -> Result<()> {

    #[cfg(not(target_os = "android"))] {
        Err(Error::NOT_ANDROID)
    }
    #[cfg(target_os = "android")] {
        uri.require_content_scheme()?;

        let api = app.android_fs_async();
        api.remove_dir(&uri).await?;
        Ok(())
    }
}

#[tauri::command]
pub async fn remove_dir_all<R: tauri::Runtime>(
    uri: FileUri,
    app: tauri::AppHandle<R>,
) -> Result<()> {

    #[cfg(not(target_os = "android"))] {
        Err(Error::NOT_ANDROID)
    }
    #[cfg(target_os = "android")] {
        uri.require_content_scheme()?;

        let api = app.android_fs_async();
        api.remove_dir_all(&uri).await?;
        Ok(())
    }
}

#[tauri::command]
pub async fn persist_uri_permission<R: tauri::Runtime>(
    uri: FileUri,
    app: tauri::AppHandle<R>,
) -> Result<()> {

    #[cfg(not(target_os = "android"))] {
        Err(Error::NOT_ANDROID)
    }
    #[cfg(target_os = "android")] {
        uri.require_content_scheme()?;

        let api = app.android_fs_async();
        api.file_picker().persist_uri_permission(&uri).await?;
        Ok(())
    }
}

#[tauri::command]
pub async fn check_persisted_uri_permission<R: tauri::Runtime>(
    uri: FileUri,
    app: tauri::AppHandle<R>,
    state: UriPermission
) -> Result<bool> {

    #[cfg(not(target_os = "android"))] {
        Err(Error::NOT_ANDROID)
    }
    #[cfg(target_os = "android")] {
        uri.require_content_scheme()?;

        let api = app.android_fs_async();
        api.file_picker().check_persisted_uri_permission(&uri, state).await
    }
}

#[tauri::command]
pub async fn release_persisted_uri_permission<R: tauri::Runtime>(
    uri: FileUri,
    app: tauri::AppHandle<R>,
) -> Result<()> {

    #[cfg(not(target_os = "android"))] {
        Err(Error::NOT_ANDROID)
    }
    #[cfg(target_os = "android")] {
        uri.require_content_scheme()?;

        let api = app.android_fs_async();
        api.file_picker().release_persisted_uri_permission(&uri).await?;
        Ok(())
    }
}

#[tauri::command]
pub async fn release_all_persisted_uri_permissions<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<()> {

    #[cfg(not(target_os = "android"))] {
        Err(Error::NOT_ANDROID)
    }
    #[cfg(target_os = "android")] {
        let api = app.android_fs_async();
        api.file_picker().release_all_persisted_uri_permissions().await?;
        Ok(())
    }
}

#[tauri::command]
pub async fn check_picker_uri_permission<R: tauri::Runtime>(
    uri: FileUri,
    app: tauri::AppHandle<R>,
    state: UriPermission
) -> Result<bool> {

    #[cfg(not(target_os = "android"))] {
        Err(Error::NOT_ANDROID)
    }
    #[cfg(target_os = "android")] {
        uri.require_content_scheme()?;

        let api = app.android_fs_async();
        api.file_picker().check_uri_permission(&uri, state).await
    }
}

#[tauri::command]
pub async fn persist_picker_uri_permission<R: tauri::Runtime>(
    uri: FileUri,
    app: tauri::AppHandle<R>,
) -> Result<()> {

    #[cfg(not(target_os = "android"))] {
        Err(Error::NOT_ANDROID)
    }
    #[cfg(target_os = "android")] {
        uri.require_content_scheme()?;

        let api = app.android_fs_async();
        api.file_picker().persist_uri_permission(&uri).await?;
        Ok(())
    }
}

#[tauri::command]
pub async fn check_persisted_picker_uri_permission<R: tauri::Runtime>(
    uri: FileUri,
    app: tauri::AppHandle<R>,
    state: UriPermission
) -> Result<bool> {

    #[cfg(not(target_os = "android"))] {
        Err(Error::NOT_ANDROID)
    }
    #[cfg(target_os = "android")] {
        uri.require_content_scheme()?;

        let api = app.android_fs_async();
        api.file_picker().check_persisted_uri_permission(&uri, state).await
    }
}

#[tauri::command]
pub async fn release_persisted_picker_uri_permission<R: tauri::Runtime>(
    uri: FileUri,
    app: tauri::AppHandle<R>,
) -> Result<bool> {

    #[cfg(not(target_os = "android"))] {
        Err(Error::NOT_ANDROID)
    }
    #[cfg(target_os = "android")] {
        uri.require_content_scheme()?;

        let api = app.android_fs_async();
        api.file_picker().release_persisted_uri_permission(&uri).await
    }
}

#[tauri::command]
pub async fn release_all_persisted_picker_uri_permissions<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<()> {

    #[cfg(not(target_os = "android"))] {
        Err(Error::NOT_ANDROID)
    }
    #[cfg(target_os = "android")] {
        let api = app.android_fs_async();
        api.file_picker().release_all_persisted_uri_permissions().await?;
        Ok(())
    }
}

#[tauri::command]
pub async fn show_share_file_dialog<R: tauri::Runtime>(
    uris: Vec<FileUri>,
    app: tauri::AppHandle<R>,
) -> Result<()> {
    
    #[cfg(not(target_os = "android"))] {
        Err(Error::NOT_ANDROID)
    }
    #[cfg(target_os = "android")] {
        for uri in &uris {
            uri.require_content_scheme()?;
        }

        let api = app.android_fs_async();
        api.file_opener().share_files(uris.iter()).await?;
        Ok(())
    }
}

#[tauri::command]
pub async fn show_view_file_dialog<R: tauri::Runtime>(
    uri: FileUri,
    app: tauri::AppHandle<R>,
) -> Result<()> {
    
    #[cfg(not(target_os = "android"))] {
        Err(Error::NOT_ANDROID)
    }
    #[cfg(target_os = "android")] {
        uri.require_content_scheme()?;

        let api = app.android_fs_async();
        api.file_opener().open_file(&uri).await?;
        Ok(())
    }
}

#[tauri::command]
pub async fn show_view_dir_dialog<R: tauri::Runtime>(
    uri: FileUri,
    app: tauri::AppHandle<R>,
) -> Result<()> {
    
    #[cfg(not(target_os = "android"))] {
        Err(Error::NOT_ANDROID)
    }
    #[cfg(target_os = "android")] {
        uri.require_content_scheme()?;

        let api = app.android_fs_async();
        api.file_opener().open_dir(&uri).await?;
        Ok(())
    }
}

#[tauri::command]
pub async fn show_open_file_picker<R: tauri::Runtime>(
    picker_type: Option<FilePickerType>,
    multiple: bool,
    mime_types: Vec<String>,
    need_write_permission: bool,
    local_only: bool,
    app: tauri::AppHandle<R>
) -> Result<Vec<FileUri>> {

    #[cfg(not(target_os = "android"))] {
        Err(Error::NOT_ANDROID)
    }
    #[cfg(target_os = "android")] {
        let target_is_single = mime_types.len() == 1;
        let (
            target_is_only_image,
            target_is_only_video,
            target_is_only_image_or_video,
            target_include_all_image,
            target_include_all_video,
        ) = match mime_types.len() {
            0 => (false, false, false, true, true),
            _ => (
                mime_types.iter().all(|s| s.starts_with("image/")),
                mime_types.iter().all(|s| s.starts_with("video/")),
                mime_types.iter().all(|s| s.starts_with("image/") || s.starts_with("video/")),
                mime_types.iter().any(|s| s == "*/*" || s == "image/*"),
                mime_types.iter().any(|s| s == "*/*" || s == "video/*"),
            )
        };

        let picker_type = match picker_type {
            _ if need_write_permission => FilePickerType::FilePicker,
            Some(picker_type) => picker_type,
            _ if target_is_single && target_is_only_image_or_video => FilePickerType::Gallery,
            _ if target_is_only_image && target_include_all_image => FilePickerType::Gallery,
            _ if target_is_only_video && target_include_all_video => FilePickerType::Gallery,
            _ if target_is_only_image_or_video && target_include_all_image && target_include_all_video => FilePickerType::Gallery,
            _ => FilePickerType::FilePicker,
        };

        let api = app.android_fs_async();

        match picker_type {
            FilePickerType::FilePicker => {
                let mime_types = mime_types.iter().map(|s| s.as_str()).collect::<Vec<_>>();

                if multiple {
                    api.file_picker().pick_files(None, &mime_types, local_only).await
                }
                else {
                    let file = api.file_picker().pick_file(None, &mime_types, local_only).await?;
                    let files = file.map(|f| vec![f]).unwrap_or_else(|| Vec::new());
                    Ok(files)
                }
            },
            FilePickerType::Gallery => {
                let target;
                if target_is_single && target_is_only_image_or_video {
                    target = VisualMediaTarget::ImageOrVideo { mime_type: &mime_types[0] }
                }
                else if target_is_only_image {
                    target = VisualMediaTarget::ImageOnly;
                }
                else if target_is_only_video {
                    target = VisualMediaTarget::VideoOnly;
                }
                else {
                    target = VisualMediaTarget::ImageAndVideo
                }

                if multiple {
                    api.file_picker().pick_visual_medias(target, local_only).await
                }
                else {
                    let file = api.file_picker().pick_visual_media(target, local_only).await?;
                    let files = file.map(|f| vec![f]).unwrap_or_else(|| Vec::new());
                    Ok(files)
                }
            },
        }
    }
}

#[tauri::command]
pub async fn show_open_dir_picker<R: tauri::Runtime>(
    local_only: bool,
    app: tauri::AppHandle<R>
) -> Result<Option<FileUri>> {

    #[cfg(not(target_os = "android"))] {
        Err(Error::NOT_ANDROID)
    }
    #[cfg(target_os = "android")] {
        let api = app.android_fs_async();
        api.file_picker().pick_dir(None, local_only).await
    }
}

#[tauri::command]
pub async fn show_save_file_picker<R: tauri::Runtime>(
    default_file_name: String,
    mime_type: Option<String>,
    local_only: bool,
    app: tauri::AppHandle<R>
) -> Result<Option<FileUri>> {

    #[cfg(not(target_os = "android"))] {
        Err(Error::NOT_ANDROID)
    }
    #[cfg(target_os = "android")] {
        let api = app.android_fs_async();
        api.file_picker().save_file(None, &default_file_name, mime_type.as_deref(), local_only).await
    }
}