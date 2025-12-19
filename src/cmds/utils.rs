use tauri::Manager as _;
use crate::*;
use super::*;


#[cfg(target_os = "android")]
pub fn convert_to_size(w: f64, h: f64) -> Result<Size> {
    if w <= 0.0 || h <= 0.0 {
        return Err(Error::with(format!("non-positive width or height: ({w}, {h})")));
    }
    if w > u32::MAX as f64 || h > u32::MAX as f64 {
        return Err(Error::with(format!("too large width or height: ({w}, {h})")));
    }

    Ok(Size {
        width: w as u32,
        height: h as u32,
    })
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