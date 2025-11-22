use serde::{Deserialize, Serialize};
use crate::*;


#[tauri::command]
pub async fn get_name<R: tauri::Runtime>(
    uri: AfsUriOrFsPath,
    app: tauri::AppHandle<R>
) -> Result<String> {

    let uri = uri.into();
    let api = app.android_fs_async();
    api.get_name(&uri).await
}

#[tauri::command]
pub async fn get_byte_length<R: tauri::Runtime>(
    uri: AfsUriOrFsPath,
    app: tauri::AppHandle<R>
) -> Result<u64> {

    let uri = uri.into();
    let api = app.android_fs_async();
    api.get_len(&uri).await
}

#[tauri::command]
pub async fn get_mime_type<R: tauri::Runtime>(
    uri: AfsUriOrFsPath,
    app: tauri::AppHandle<R>
) -> Result<String> {

    let uri = uri.into();
    let api = app.android_fs_async();
    api.get_mime_type(&uri).await
}

#[tauri::command]
pub async fn get_type<R: tauri::Runtime>(
    uri: AfsUriOrFsPath,
    app: tauri::AppHandle<R>
) -> Result<EntryType> {

    let uri = uri.into();
    let api = app.android_fs_async();
    api.get_type(&uri).await
}

#[tauri::command]
pub async fn get_metadata<R: tauri::Runtime>(
    uri: AfsUriOrFsPath,
    app: tauri::AppHandle<R>
) -> Result<EntryMetadata> {

    let uri = uri.into();
    let api = app.android_fs_async();
    let entry = api.get_info(&uri).await?;

    match api.get_info(&uri).await? {
        Entry::File { name, last_modified, len, mime_type, .. } => {
            let last_modified = convert_time_to_f64(last_modified)?;
            Ok(EntryMetadata::File { name, last_modified, len, mime_type })
        },
        Entry::Dir { name, last_modified, .. } => {
            let last_modified = convert_time_to_f64(last_modified)?;
            Ok(EntryMetadata::Dir { name, last_modified })
        },
    }
}

#[tauri::command]
pub fn get_fs_path(uri: AfsUriOrFsPath) -> Result<tauri_plugin_fs::FilePath> {
    Ok(uri.into())
}

#[tauri::command]
pub async fn get_thumbnail<R: tauri::Runtime>(
    uri: AfsUriOrFsPath,
    width: f64,
    height: f64,
    format: String,
    app: tauri::AppHandle<R>
) -> Result<tauri::ipc::Response> {

    let uri = uri.into();
    let format = convert_to_image_format(&format)?;
    let size = convert_to_size(width, height)?;
    let api = app.android_fs_async();
    
    let Some(bytes) = api.get_thumbnail(&uri, size, format).await? else {
        return Ok(tauri::ipc::Response::new(Vec::with_capacity(0)))
    };
    
    Ok(tauri::ipc::Response::new(bytes))
}

#[tauri::command]
pub async fn get_thumbnail_base64<R: tauri::Runtime>(
    uri: AfsUriOrFsPath,
    width: f64,
    height: f64,
    format: String,
    app: tauri::AppHandle<R>
) -> Result<Option<String>> {

    let uri = uri.into();
    let format = convert_to_image_format(&format)?;
    let size = convert_to_size(width, height)?;
    let api = app.android_fs_async();

    api.get_thumbnail_base64(&uri, size, format).await
}

#[tauri::command]
pub async fn get_thumbnail_data_url<R: tauri::Runtime>(
    uri: AfsUriOrFsPath,
    width: f64,
    height: f64,
    format: String,
    app: tauri::AppHandle<R>
) -> Result<Option<String>> {

    let uri = uri.into();
    let format = convert_to_image_format(&format)?;
    let size = convert_to_size(width, height)?;
    let api = app.android_fs_async();
    
    let Some(base64) = api.get_thumbnail_base64(&uri, size, format).await? else {
        return Ok(None)
    };
    
    let mime_type = format.mime_type();
    let prefix = format!("data:{mime_type};base64,");
    let mut data_url = String::with_capacity(prefix.len() + base64.len());
    data_url.push_str(&prefix);
    data_url.push_str(&base64);
    Ok(Some(data_url))
}

#[tauri::command]
pub async fn get_volumes<R: tauri::Runtime>(
    app: tauri::AppHandle<R>
) -> Result<Vec<impl Serialize>> {

    list_volumes(app).await
}

#[tauri::command]
pub async fn list_volumes<R: tauri::Runtime>(
    app: tauri::AppHandle<R>
) -> Result<Vec<impl Serialize>> {

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
        .collect::<Vec<_>>();

    Ok(volumes)
}

async fn create_new_public_file_inner<R: tauri::Runtime>(
    volume_id: Option<String>,
    base_dir: impl Into<PublicDir>,
    relative_path: String,
    mime_type: Option<String>,
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
        relative_path,
        mime_type.as_deref()
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

    create_new_public_file_inner(volume_id, base_dir, relative_path, mime_type, request_permission, app).await
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

    create_new_public_file_inner(volume_id, base_dir, relative_path, mime_type, request_permission, app).await
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

    create_new_public_file_inner(volume_id, base_dir, relative_path, mime_type, request_permission, app).await
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

    let mut base_dir: PublicDir = base_dir.into();
    let mut relative_path = relative_path;

    let ps = app.android_fs_async().public_storage();
    if base_dir == PublicAudioDir::Audiobooks.into() && !ps.is_audiobooks_dir_available()? {
        base_dir = PublicAudioDir::Music.into();
        relative_path = format!("Audiobooks/{}", relative_path.trim_start_matches('/'))
    }
    if base_dir == PublicAudioDir::Recordings.into() && !ps.is_recordings_dir_available()? {
        base_dir = PublicAudioDir::Music.into();
        relative_path = format!("Recordings/{}", relative_path.trim_start_matches('/'))
    }

    create_new_public_file_inner(volume_id, base_dir, relative_path, mime_type, request_permission, app).await
}

#[tauri::command]
pub async fn scan_public_file<R: tauri::Runtime>(
    uri: FileUri,
    app: tauri::AppHandle<R>,
) -> Result<()> {

    let uri: FileUri = uri.into();
    let api = app.android_fs_async();
    api.public_storage().scan(&uri).await
}

#[tauri::command]
pub async fn request_public_files_permission<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<bool> {

    let api = app.android_fs_async();
    api.public_storage().request_permission().await
}

#[tauri::command]
pub async fn has_public_files_permission<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<bool> {

    let api = app.android_fs_async();
    api.public_storage().has_permission().await
}

#[tauri::command]
pub async fn create_dir_all<R: tauri::Runtime>(
    parent_dir_uri: FileUri,
    relative_path: String,
    app: tauri::AppHandle<R>
) -> Result<FileUri> {

    let api = app.android_fs_async();
    api.create_dir_all(&parent_dir_uri, relative_path).await
}

#[tauri::command]
pub async fn create_new_file<R: tauri::Runtime>(
    parent_dir_uri: FileUri,
    relative_path: String,
    mime_type: Option<String>,
    app: tauri::AppHandle<R>,
) -> Result<FileUri> {

    let api = app.android_fs_async();
    api.create_new_file(&parent_dir_uri, relative_path, mime_type.as_deref()).await
}

#[tauri::command]
pub async fn copy_file<R: tauri::Runtime>(
    src_uri: AfsUriOrFsPath,
    dest_uri: AfsUriOrFsPath,
    app: tauri::AppHandle<R>
) -> Result<()> {

    let src_uri: FileUri = src_uri.into();
    let dest_uri: FileUri = dest_uri.into();

    if let Some(dest_path) = dest_uri.as_path() {
        if !std::fs::exists(dest_path)? {
            std::fs::File::create(dest_path)?;
        }
    }

    let api = app.android_fs_async();
    api.copy(&src_uri, &dest_uri).await
}

#[tauri::command]
pub async fn truncate_file<R: tauri::Runtime>(
    uri: AfsUriOrFsPath,
    app: tauri::AppHandle<R>
) -> Result<()> {

    let uri = uri.into();
    let api = app.android_fs_async();
    api.open_file_writable(&uri).await?;
    Ok(())
}

#[tauri::command]
pub async fn read_dir<R: tauri::Runtime>(
    uri: FileUri,
    app: tauri::AppHandle<R>
) -> Result<Vec<EntryMetadataWithUri>> {

    let api = app.android_fs_async();
    let entries = api.read_dir(&uri).await?;
    let mut buffer = Vec::new();

    for entry in entries {
        let entry = match entry {
            Entry::File { uri, name, last_modified, len, mime_type } => {
                let last_modified = convert_time_to_f64(last_modified)?;
                EntryMetadataWithUri::File { name, uri, len, mime_type, last_modified }
            },
            Entry::Dir { uri, name, last_modified } => {
                let last_modified = convert_time_to_f64(last_modified)?;
                EntryMetadataWithUri::Dir { name, uri, last_modified }
            },
        };
        buffer.push(entry);
    }

    Ok(buffer)
}

#[tauri::command]
pub async fn remove_file<R: tauri::Runtime>(
    uri: AfsUriOrFsPath,
    app: tauri::AppHandle<R>,
) -> Result<()> {

    let uri = uri.into();
    let api = app.android_fs_async();
    api.remove_file(&uri).await
}

#[tauri::command]
pub async fn remove_empty_dir<R: tauri::Runtime>(
    uri: AfsUriOrFsPath,
    app: tauri::AppHandle<R>,
) -> Result<()> {

    let uri = uri.into();
    let api = app.android_fs_async();
    api.remove_dir(&uri).await
}

#[tauri::command]
pub async fn remove_dir_all<R: tauri::Runtime>(
    uri: AfsUriOrFsPath,
    app: tauri::AppHandle<R>,
) -> Result<()> {

    let uri = uri.into();
    let api = app.android_fs_async();
    api.remove_dir_all(&uri).await
}

#[tauri::command]
pub async fn persist_uri_permission<R: tauri::Runtime>(
    uri: FileUri,
    app: tauri::AppHandle<R>,
) -> Result<()> {

    let uri = uri.into();
    let api = app.android_fs_async();
    api.take_persistable_uri_permission(&uri).await
}

#[tauri::command]
pub async fn check_persisted_uri_permission<R: tauri::Runtime>(
    uri: FileUri,
    app: tauri::AppHandle<R>,
    state: UriPermission
) -> Result<bool> {

    let api = app.android_fs_async();
    api.check_persisted_uri_permission(&uri, state).await
}

#[tauri::command]
pub async fn release_persisted_uri_permission<R: tauri::Runtime>(
    uri: FileUri,
    app: tauri::AppHandle<R>,
) -> Result<()> {

    let api = app.android_fs_async();
    api.release_persisted_uri_permission(&uri).await
}

#[tauri::command]
pub async fn release_all_persisted_uri_permissions<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<()> {

    let api = app.android_fs_async();
    api.release_all_persisted_uri_permissions().await
}

#[tauri::command]
pub async fn show_share_file_dialog<R: tauri::Runtime>(
    uris: Vec<FileUri>,
    app: tauri::AppHandle<R>,
) -> Result<()> {
    
    let api = app.android_fs_async();
    api.file_opener().share_files(uris.iter()).await
}

#[tauri::command]
pub async fn show_view_file_dialog<R: tauri::Runtime>(
    uri: FileUri,
    app: tauri::AppHandle<R>,
) -> Result<()> {
    
    let api = app.android_fs_async();
    api.file_opener().open_file(&uri).await
}

#[tauri::command]
pub async fn show_view_dir_dialog<R: tauri::Runtime>(
    uri: FileUri,
    app: tauri::AppHandle<R>,
) -> Result<()> {
    
    let api = app.android_fs_async();
    api.file_opener().open_dir(&uri).await
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

#[tauri::command]
pub async fn show_open_dir_picker<R: tauri::Runtime>(
    local_only: bool,
    app: tauri::AppHandle<R>
) -> Result<Option<FileUri>> {

    let api = app.android_fs_async();
    api.file_picker().pick_dir(None, local_only).await
}

#[tauri::command]
pub async fn show_save_file_picker<R: tauri::Runtime>(
    default_file_name: String,
    mime_type: Option<String>,
    local_only: bool,
    app: tauri::AppHandle<R>
) -> Result<Option<FileUri>> {

    let api = app.android_fs_async();
    api.file_picker().save_file(None, &default_file_name, mime_type.as_deref(), local_only).await
}

#[derive(Deserialize, Serialize, PartialEq, Eq, Clone, Copy)]
pub enum FilePickerType {
    FilePicker,
    Gallery
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum PublicImageOrGeneralPurposeDir {
    Image(PublicImageDir),
    GeneralPurpose(PublicGeneralPurposeDir),
}

impl From<PublicImageOrGeneralPurposeDir> for PublicDir {

    fn from(value: PublicImageOrGeneralPurposeDir) -> Self {
        match value {
            PublicImageOrGeneralPurposeDir::Image(d) => d.into(),
            PublicImageOrGeneralPurposeDir::GeneralPurpose(d) => d.into(),
        }
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum PublicVideoOrGeneralPurposeDir {
    Video(PublicVideoDir),
    GeneralPurpose(PublicGeneralPurposeDir),
}

impl From<PublicVideoOrGeneralPurposeDir> for PublicDir {

    fn from(value: PublicVideoOrGeneralPurposeDir) -> Self {
        match value {
            PublicVideoOrGeneralPurposeDir::Video(d) => d.into(),
            PublicVideoOrGeneralPurposeDir::GeneralPurpose(d) => d.into(),
        }
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum PublicAudioOrGeneralPurposeDir {
    Audio(PublicAudioDir),
    GeneralPurpose(PublicGeneralPurposeDir),
}

impl From<PublicAudioOrGeneralPurposeDir> for PublicDir {

    fn from(value: PublicAudioOrGeneralPurposeDir) -> Self {
        match value {
            PublicAudioOrGeneralPurposeDir::Audio(d) => d.into(),
            PublicAudioOrGeneralPurposeDir::GeneralPurpose(d) => d.into(),
        }
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum AfsUriOrFsPath {
    AfsUri(FileUri),
    FsPath(tauri_plugin_fs::FilePath),
}

impl From<AfsUriOrFsPath> for FileUri {

    fn from(value: AfsUriOrFsPath) -> Self {
        match value {
            AfsUriOrFsPath::AfsUri(uri) => uri,
            AfsUriOrFsPath::FsPath(path) => path.into(),
        }
    }
}

impl From<AfsUriOrFsPath> for tauri_plugin_fs::FilePath {

    fn from(value: AfsUriOrFsPath) -> Self {
        match value {
            AfsUriOrFsPath::AfsUri(uri) => uri.into(),
            AfsUriOrFsPath::FsPath(path) => path,
        }
    }
}

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum EntryMetadata {
    File {
        name: String,

        #[serde(rename = "lastModified")]
        last_modified: f64,

        #[serde(rename = "byteLength")]
        len: u64,

        #[serde(rename = "mimeType")]
        mime_type: String,
    },
    #[serde(rename_all = "camelCase")]
    Dir {
        name: String,

        #[serde(rename = "lastModified")]
        last_modified: f64,
    }
}

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum EntryMetadataWithUri {
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


fn convert_to_size(w: f64, h: f64) -> Result<Size> {
    if w < 0.0 || h < 0.0 {
        return Err(Error::with(format!("width and height must be non-negative: ({w}, {h})")))
    }
    if w > u32::MAX as f64 || h > u32::MAX as f64 {
        return Err(Error::with(format!("width or height too large: ({w}, {h})")))
    }

    Ok(Size { width: w as u32, height: h as u32 })
}

fn convert_to_image_format(format: &str) -> Result<ImageFormat> {
    match format.to_ascii_lowercase().as_str() {
        "jpeg" | "jpg" => Ok(ImageFormat::Jpeg),
        "webp" => Ok(ImageFormat::Webp),
        "png" => Ok(ImageFormat::Png),
        _ => Err(Error::with(format!("Unexpected image format: {format}")))
    }
}

fn convert_to_storage_volume_id(id: &str) -> Result<StorageVolumeId> {
    serde_json::from_str(id).map_err(Into::into)
}

fn convert_from_storage_volume_id(id: &StorageVolumeId) -> Result<String> {
    serde_json::to_string(id).map_err(Into::into)
}

fn convert_time_to_f64(time: std::time::SystemTime) -> Result<f64> {
    let duration = time
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or(std::time::Duration::ZERO);

    Ok(duration.as_millis() as f64)
}