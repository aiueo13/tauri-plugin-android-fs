use tauri::Manager as _;
use crate::*;
use super::*;


#[cfg(target_os = "android")]
pub async fn resolve_mime_type<'a, R: tauri::Runtime>(
    mime_type: Option<&'a str>,
    path: impl AsRef<str>,
    app: &tauri::AppHandle<R>,
) -> Result<std::borrow::Cow<'a, str>> {

    if let Some(mime_type) = mime_type {
        return Ok(mime_type.into())
    }

    let path = path.as_ref();
    let file_name = path.rsplit_once('/')
        .map(|(_, file_name)| file_name)
        .unwrap_or(path);

    if let Some((_, ext)) = file_name.rsplit_once('.') {
        let api = app.android_fs_async();
        if let Some(mime_type) = api.get_mime_type_from_extension(ext).await? {
            return Ok(mime_type.into())
        }
    }
    
    Ok("application/octet-stream".into())
}

#[cfg(target_os = "android")]
pub async fn resolve_picker_initial_location<R: tauri::Runtime>(
    initial_location: PickerInitialLocation,
    app: &tauri::AppHandle<R>,
) -> Result<FileUri> {

    let api = app.android_fs_async();
    let map_volume_id = |id: Option<&str>| -> Result<Option<StorageVolumeId>> {
        match id {
            Some(v) => Ok(Some(convert_to_storage_volume_id(v)?)),
            None => Ok(None),
        }
    };

    match initial_location {
        PickerInitialLocation::Any { uri } => {
            Ok(uri)
        },
        PickerInitialLocation::VolumeTop { volume_id } => {
            api.resolve_root_initial_location(
                map_volume_id(volume_id.as_deref())?.as_ref()
            ).await
        },
        PickerInitialLocation::PublicDir { base_dir, relative_path, volume_id } => {
            api.public_storage().resolve_initial_location(
                map_volume_id(volume_id.as_deref())?.as_ref(), 
                base_dir, 
                relative_path.as_deref().unwrap_or(""), 
                true,
            ).await
        },
    }
}

#[cfg(target_os = "android")]
pub fn convert_to_thumbnail_preferred_size(w: f64, h: f64) -> Result<Size> {
    if !w.is_finite() || !h.is_finite() {
        return Err(Error::with("non-finite width or height"));
    }
    if w <= 0.0 || h <= 0.0 {
        return Err(Error::with(format!("non-positive width or height: ({w}, {h})")));
    }

    const MAX: u32 = 1000;

    let width = u32::clamp(w.round() as u32, 1, MAX);
    let height = u32::clamp(h.round() as u32, 1, MAX);

    Ok(Size { width, height })
}

#[cfg(target_os = "android")]
pub fn convert_to_image_format(format: &str) -> Result<ImageFormat> {
    match format.to_ascii_lowercase().as_str() {
        "jpeg" | "jpg" => Ok(ImageFormat::Jpeg),
        "webp" => Ok(ImageFormat::Webp),
        "png" => Ok(ImageFormat::Png),
        _ => Err(Error::with(format!("unexpected image format: {format}")))
    }
}

#[cfg(target_os = "android")]
pub fn convert_to_storage_volume_id(id: &str) -> Result<StorageVolumeId> {
    serde_json::from_str(id).map_err(Into::into)
}

#[cfg(target_os = "android")]
pub fn convert_from_storage_volume_id(id: &StorageVolumeId) -> Result<String> {
    serde_json::to_string(id).map_err(Into::into)
}

#[cfg(target_os = "android")]
pub fn convert_time_to_f64_millis(time: std::time::SystemTime) -> Result<f64> {
    let duration = time
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or(std::time::Duration::ZERO);

    Ok(duration.as_millis() as f64)
}

#[cfg(target_os = "android")]
pub fn convert_rid_to_bytes(rid: tauri::ResourceId) -> Vec<u8> {
    rid.to_be_bytes().to_vec()
}

/// DataURL/Base64 to bytes
#[cfg(target_os = "android")]
pub fn convert_to_bytes(data: &str) -> Result<Vec<u8>> {
    let b64 = match data.starts_with("data:") {
        // data URL
        true => {
            let comma = data
                .find(',')
                .ok_or_else(|| Error::with("invalid data URL: missing comma"))?;

            let (_, b64) = data.split_at(comma + 1);
            b64
        },
        // base64 encoded string
        false => data,
    };

    use base64::engine::Engine;
    let bytes = base64::engine::general_purpose::STANDARD.decode(b64)?;
    Ok(bytes)
}

#[cfg(target_os = "android")]
pub struct PluginResource<T> {
    resource: std::sync::Arc<T>
}

#[cfg(target_os = "android")]
impl<T> PluginResource<T> {

    pub fn new(resource: T) -> Self {
        Self { resource: std::sync::Arc::new(resource) }
    }

    pub fn get(&self) -> std::sync::Arc<T> {
        std::sync::Arc::clone(&self.resource)
    }
}

#[cfg(target_os = "android")]
impl<T: Sync + Send + 'static> tauri::Resource for PluginResource<T> {}

#[derive(Deserialize)]
#[serde(tag = "type")]
#[cfg_attr(not(target_os = "android"), allow(unused))]
pub enum WriteFileEvent {
    Open {
        uri: AfsUriOrFsPath
    },
    Write {
        id: tauri::ResourceId,
        data: String,
    },
    Close {
        id: tauri::ResourceId,
    },
    WriteOnce {
        uri: AfsUriOrFsPath,
        data: String
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
#[cfg_attr(not(target_os = "android"), allow(unused))]
pub enum WriteFileEventOutput {
    Open(tauri::ResourceId),
    Write(()),
    Close(()),
    WriteOnce(()),
}

#[derive(Deserialize)]
#[serde(tag = "type")]
#[cfg_attr(not(target_os = "android"), allow(unused))]
pub enum OpenWriteFileStreamEvent {
    Open {
        uri: AfsUriOrFsPath
    },
    Write {
        id: tauri::ResourceId,
        data: String,
    },
    Close {
        id: tauri::ResourceId,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
#[cfg_attr(not(target_os = "android"), allow(unused))]
pub enum OpenWriteFileStreamEventOutput {
    Open(tauri::ResourceId),
    Write(()),
    Close(()),
}

#[derive(Deserialize)]
#[serde(tag = "type")]
#[cfg_attr(not(target_os = "android"), allow(unused))]
pub enum OpenReadFileStreamEvent {
    Open {
        uri: AfsUriOrFsPath
    },
    Read {
        id: tauri::ResourceId,
        len: u32
    },
    Close {
        id: tauri::ResourceId,
    },
}

#[derive(Deserialize)]
#[serde(tag = "type")]
#[cfg_attr(not(target_os = "android"), allow(unused))]
pub enum OpenReadTextFileLinesStreamEvent {
    Open {
        uri: AfsUriOrFsPath,
    },
    Read {
        id: tauri::ResourceId,
        len: u32,
        fatal: bool,

        #[serde(rename = "maxLineByteLength")]
        max_line_byte_length: u64
    },
    Close {
        id: tauri::ResourceId,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PickerInitialLocation {
    Any {
        uri: FileUri,
    },
    VolumeTop {
        #[serde(rename = "volumeId")]
        volume_id: Option<String>,
    },
    PublicDir {
        #[serde(rename = "baseDir")]
        base_dir: PublicDir,

        #[serde(rename = "relativePath")]
        relative_path: Option<String>,

        #[serde(rename = "volumeId")]
        volume_id: Option<String>,
    },
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

// Based on code from tauri-plugin-fs crate
//
// Source:
// - https://github.com/tauri-apps/plugins-workspace/blob/3d0d2e041bbad9766aebecaeba291a28d8d7bf5c/plugins/fs/src/commands.rs#L1090
// - Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// - Licensed under the MIT License or the Apache 2.0 License
#[cfg(target_os = "android")]
pub fn validate_path_permission<R: tauri::Runtime>(
    path: impl AsRef<std::path::Path>,
    app: &tauri::AppHandle<R>,
    cmd_scope: &tauri::ipc::CommandScope<Scope>,
    global_scope: &tauri::ipc::GlobalScope<Scope>,
) -> Result<()> {

    let path = path.as_ref();
    let require_literal_leading_dot = true;

    let scope = tauri::scope::fs::Scope::new(
        app,
        &tauri::utils::config::FsScope::Scope {
            allow: global_scope
                .allows()
                .iter()
                .filter_map(|e| e.path.clone())
                .chain(cmd_scope.allows().iter().filter_map(|e| e.path.clone()))
                .collect(),

            deny: global_scope
                .denies()
                .iter()
                .filter_map(|e| e.path.clone())
                .chain(cmd_scope.denies().iter().filter_map(|e| e.path.clone()))
                .collect(),

            require_literal_leading_dot: Some(require_literal_leading_dot),
        },
    )?;

    if !is_forbidden(&scope, &path, require_literal_leading_dot) && scope.is_allowed(&path) {
        return Ok(())
    }
    
    if cfg!(debug_assertions) {
        Err(Error::with(format!(
            "forbidden path: {}, maybe it is not allowed on the scope configuration in your capability file",
            path.display()
        )))
    }
    else {
        Err(Error::with(format!(
            "forbidden path: {}", 
            path.display()
        )))
    }
}

// Based on code from tauri-plugin-fs crate
//
// Source:
// - https://github.com/tauri-apps/plugins-workspace/blob/3d0d2e041bbad9766aebecaeba291a28d8d7bf5c/plugins/fs/src/commands.rs#L1151
// - Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// - Licensed under the MIT License or the Apache 2.0 License
#[cfg(target_os = "android")]
fn is_forbidden<P: AsRef<std::path::Path>>(
    scope: &tauri::fs::Scope,
    path: P,
    require_literal_leading_dot: bool,
) -> bool {

    let path = path.as_ref();
    let path = if path.is_symlink() {
        match std::fs::read_link(path) {
            Ok(p) => p,
            Err(_) => return false,
        }
    } else {
        path.to_path_buf()
    };
    let path = if !path.exists() {
        crate::Result::Ok(path)
    } else {
        std::fs::canonicalize(path).map_err(Into::into)
    };

    if let Ok(path) = path {
        let path: std::path::PathBuf = path.components().collect();
        scope.forbidden_patterns().iter().any(|p| {
            p.matches_path_with(
                &path,
                glob::MatchOptions {
                    // this is needed so `/dir/*` doesn't match files within subdirectories such as `/dir/subdir/file.txt`
                    // see: <https://github.com/tauri-apps/tauri/security/advisories/GHSA-6mv3-wm7j-h4w5>
                    require_literal_separator: true,
                    require_literal_leading_dot,
                    ..Default::default()
                },
            )
        })
    } else {
        false
    }
}

// Based on code from tauri-plugin-fs crate
//
// Source:
// - https://github.com/tauri-apps/plugins-workspace/blob/3d0d2e041bbad9766aebecaeba291a28d8d7bf5c/plugins/fs/src/lib.rs#L347
// - Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// - Licensed under the MIT License or the Apache 2.0 License
impl tauri::ipc::ScopeObject for Scope {
    type Error = Error;

    fn deserialize<R: tauri::Runtime>(
        app: &tauri::AppHandle<R>,
        raw: tauri::utils::acl::Value
    ) -> Result<Self> {
        
        let path = serde_json::from_value(raw.into()).map(|raw| match raw {
            ScopeSchema::Value(path) => path,
            ScopeSchema::Object { path } => path,
        })?;

        match app.path().parse(path) {
            Ok(path) => Ok(Self { path: Some(path) }),
            Err(err) => Err(err.into()),
        }
    }
}