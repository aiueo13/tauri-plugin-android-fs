use serde::{Deserialize, Serialize};
use crate::{Error, Result};

/// Path to represent a file or directory.
/// 
/// # Note
/// For compatibility, an interconversion to [`tauri_plugin_fs::FilePath`] is implemented, such as follwing.  
/// This is lossy and also not guaranteed to work properly with other plugins.  
/// However, reading and writing files by official [`tauri_plugin_fs`] etc. should work well.  
/// ```no_run
/// use tauri_plugin_android_fs::FileUri;
/// use tauri_plugin_fs::FilePath;
/// 
/// let uri: FileUri = unimplemented!();
/// let path: FilePath = uri.into();
/// let uri: FileUri = path.into();
/// ```
/// 
/// # Typescript type
/// ```typescript
/// type FileUri = {
///     uri: string, // This can use as path for official tauri_plugin_fs
///     documentTopTreeUri: string | null
/// }
/// ```
#[derive(Debug, Clone, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileUri {
    /// `file://` or `content://` URI of file or directory.
    pub uri: String,

    /// Only files/directories under the directory obtained by `FilePicker::pick_dir` will own this.
    pub document_top_tree_uri: Option<String>,
}

impl FileUri {

    pub fn to_string(&self) -> crate::Result<String> {
        serde_json::to_string(self).map_err(Into::into)
    }

    pub fn from_str(s: &str) -> crate::Result<Self> {
        serde_json::from_str(s).map_err(Into::into)
    }

    pub fn from_path(path: impl AsRef<std::path::Path>) -> Self {
        Self { uri: format!("file://{}", path.as_ref().to_string_lossy()), document_top_tree_uri: None }
    }

    #[allow(unused)]
    pub(crate) fn as_path(&self) -> Option<&std::path::Path> {
        if self.uri.starts_with("file://") {
            return Some(std::path::Path::new(self.uri.trim_start_matches("file://")))
        }
        None
    }
}

impl From<&std::path::Path> for FileUri {

    fn from(path: &std::path::Path) -> Self {
        Self::from_path(path)
    }
}

impl From<&std::path::PathBuf> for FileUri {

    fn from(path: &std::path::PathBuf) -> Self {
        Self::from_path(path)
    }
}

impl From<std::path::PathBuf> for FileUri {

    fn from(path: std::path::PathBuf) -> Self {
        Self::from_path(path)
    }
}

impl From<tauri_plugin_fs::FilePath> for FileUri {

    fn from(value: tauri_plugin_fs::FilePath) -> Self {
        match value {
            tauri_plugin_fs::FilePath::Url(url) => Self { uri: url.to_string(), document_top_tree_uri: None },
            tauri_plugin_fs::FilePath::Path(path_buf) => path_buf.into(),
        }
    }
}

impl From<FileUri> for tauri_plugin_fs::FilePath {

    fn from(value: FileUri) -> Self {
        type NeverErr<T> = std::result::Result::<T, std::convert::Infallible>;
        NeverErr::unwrap(value.uri.parse())
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageVolume {

    /// A user-visible description of the volume.  
    /// This can be determined by the manufacturer and is often localized according to the user’s language.
    ///
    /// e.g.
    /// - `Internal shared storage`
    /// - `SDCARD`
    /// - `SD card`
    /// - `Virtual SD card`
    pub description: String,

    /// Indicates whether this is primary storage volume. 
    /// A device always has one (and one only) primary storage volume. 
    pub is_primary: bool,

    /// Indicates whether this is physically removable.
    /// If `false`, this is device's built-in storage.
    pub is_removable: bool,

    /// Indicates whether thit is stable part of the device.
    /// 
    /// For example, a device’s built-in storage and physical media slots under protective covers are considered stable, 
    /// while USB flash drives connected to handheld devices are not.
    pub is_stable: bool,

    /// Indicates whether this is backed by private user data partition, 
    /// either internal storage or [adopted storage](https://source.android.com/docs/core/storage/adoptable).
    pub is_emulated: bool,

    /// Indicates whether this is readonly storage volume.
    ///
    /// e.g. SD card with readonly mode.
    /// 
    /// # Remark
    /// As far as I understand, this should never be `true` 
    /// when either `is_primary` or `is_emulated` is true, 
    /// or when `is_removable` is false, 
    /// but it might not be the case due to any issues or rare cases.
    pub is_readonly: bool,

    pub id: StorageVolumeId
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageVolumeId {
    pub(crate) top_directory_path: std::path::PathBuf,
    pub(crate) media_store_context: Option<StorageVolumeMediaStoreContext>,
    pub(crate) private_data_dir_path: Option<std::path::PathBuf>,
    pub(crate) private_cache_dir_path: Option<std::path::PathBuf>,
    pub(crate) uuid: Option<String>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StorageVolumeMediaStoreContext {
    pub(crate) volume_name: String,
    pub(crate) images_content_uri: String,
    pub(crate) videos_content_uri: String,
    pub(crate) audios_content_uri: String,
    pub(crate) files_content_uri: String,
}

#[allow(unused)]
impl StorageVolumeId {

    pub(crate) fn private_dir_path(&self, dir: OutsidePrivateDir) -> Option<&std::path::PathBuf> {
        match dir {
            OutsidePrivateDir::Data => self.private_data_dir_path.as_ref(),
            OutsidePrivateDir::Cache => self.private_cache_dir_path.as_ref(),
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Entry {

    #[non_exhaustive]
    File {
        uri: FileUri,
        name: String,
        last_modified: std::time::SystemTime,
        len: u64,
        mime_type: String,
    },

    #[non_exhaustive]
    Dir {
        uri: FileUri,
        name: String,
        last_modified: std::time::SystemTime,
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum EntryType {
    File {
        mime_type: String
    },
    Dir,
}

impl EntryType {

    pub fn is_file(&self) -> bool {
        matches!(self, Self::File { .. })
    }

    pub fn is_dir(&self) -> bool {
        matches!(self, Self::Dir)
    }

    /// If file, this is no None.  
    /// If directory, this is None.  
    pub fn mime_type(&self) -> Option<&str> {
        match self {
            EntryType::File { mime_type } => Some(&mime_type),
            EntryType::Dir => None,
        }
    }

    /// If file, this is no None.  
    /// If directory, this is None.  
    pub fn into_mime_type(self) -> Option<String> {
        match self {
            EntryType::File { mime_type } => Some(mime_type),
            EntryType::Dir => None,
        }
    }
}

/// Access mode
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Deserialize, Serialize)]
pub enum PersistableAccessMode {

    /// Read access.
    Read,

    /// Write access.
    Write,

    /// Read-write access.
    ReadAndWrite,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Deserialize, Serialize)]
pub enum PersistedUriPermission {
    File {
        uri: FileUri,
        can_read: bool,
        can_write: bool,
    },
    Dir {
        uri: FileUri,
        can_read: bool,
        can_write: bool,
    }
}

impl PersistedUriPermission {

    pub fn uri(&self) -> &FileUri {
        match self {
            PersistedUriPermission::File { uri, .. } => uri,
            PersistedUriPermission::Dir { uri, .. } => uri,
        }
    }

    pub fn into_uri(self) -> FileUri {
        match self {
            PersistedUriPermission::File { uri, .. } => uri,
            PersistedUriPermission::Dir { uri, .. } => uri,
        }
    }

    pub fn can_read(&self) -> bool {
        match self {
            PersistedUriPermission::File { can_read, .. } => *can_read,
            PersistedUriPermission::Dir { can_read, .. } => *can_read,
        }
    }

    pub fn can_write(&self) -> bool {
        match self {
            PersistedUriPermission::File { can_write, .. } => *can_write,
            PersistedUriPermission::Dir { can_write, .. } => *can_write,
        }
    }

    pub fn is_file(&self) -> bool {
        matches!(self, PersistedUriPermission::File { .. })
    }

    pub fn is_dir(&self) -> bool {
        matches!(self, PersistedUriPermission::Dir { .. })
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Deserialize, Serialize)]
pub struct Size {
    pub width: u32,
    pub height: u32
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[non_exhaustive]
pub enum ImageFormat {

    /// - Loss less
    /// - Support transparency
    Png,

    /// - Lossy
    /// - Unsupport transparency
    Jpeg,

    /// - Lossy (**Not loss less**)
    /// - Support transparency
    Webp,

    /// - Lossy
    /// - Unsupport transparency
    JpegWith {

        /// Range is `0.0 ~ 1.0`  
        /// 0.0 means compress for the smallest size.  
        /// 1.0 means compress for max visual quality.  
        quality: f32
    },

    /// - Lossy
    /// - Support transparency
    WebpWith {
        
        /// Range is `0.0 ~ 1.0`  
        /// 0.0 means compress for the smallest size.  
        /// 1.0 means compress for max visual quality.  
        quality: f32
    }
}

/// Access mode
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub enum FileAccessMode {

    /// Opens the file in read-only mode.
    /// 
    /// FileDescriptor mode: "r"
    Read,

    /// Opens the file in write-only mode.  
    /// 
    /// Since Android 10, this may or may not truncate existing contents. 
    /// If the new file is smaller than the old one, **this may cause the file to become corrupted**.
    /// <https://issuetracker.google.com/issues/180526528>
    /// 
    /// The reason this is marked as deprecated is because of that behavior, 
    /// and it is not scheduled to be removed in the future. 
    /// 
    /// FileDescriptor mode: "w"
    #[deprecated(note = "This may or may not truncate existing contents. If the new file is smaller than the old one, this may cause the file to become corrupted.")]
    Write,

    /// Opens the file in write-only mode.
    /// The existing content is truncated (deleted), and new data is written from the beginning.
    ///
    /// FileDescriptor mode: "wt"
    WriteTruncate,

    /// Opens the file in write-only mode.
    /// The existing content is preserved, and new data is appended to the end of the file.
    /// 
    /// FileDescriptor mode: "wa"
    WriteAppend,

    /// Opens the file in read-write mode.  
    /// 
    /// FileDescriptor mode: "rw"
    ReadWrite,

    /// Opens the file in read-write mode.
    /// The existing content is truncated (deleted), and new data is written from the beginning.
    ///
    /// FileDescriptor mode: "rwt"
    ReadWriteTruncate,
}

#[allow(unused)]
#[allow(deprecated)]
impl FileAccessMode {
 
    pub(crate) fn to_mode(&self) -> &'static str {
        match self {
            FileAccessMode::Read => "r",
            FileAccessMode::Write => "w",
            FileAccessMode::WriteTruncate => "wt",
            FileAccessMode::WriteAppend => "wa",
            FileAccessMode::ReadWriteTruncate => "rwt",
            FileAccessMode::ReadWrite => "rw",
        }
    }

    pub(crate) fn from_mode(mode: &str) -> Result<Self> {
        match mode {
            "r" => Ok(Self::Read),
            "w" => Ok(Self::Write),
            "wt" => Ok(Self::WriteTruncate),
            "wa" => Ok(Self::WriteAppend),
            "rwt" => Ok(Self::ReadWriteTruncate),
            "rw" => Ok(Self::ReadWrite),
            mode => Err(Error { msg: format!("Illegal mode: {mode}").into() })
        }
    }
}

/// Filters for VisualMediaPicker.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub enum VisualMediaTarget<'a> {

    /// Allow only images to be selected.  
    ImageOnly,

    /// Allow only videos to be selected.  
    VideoOnly,

    /// Allow only images and videos to be selected.  
    ImageAndVideo,

    /// Allow only images or videos of specified single Mime type to be selected.  
    ImageOrVideo {
        mime_type: &'a str
    }
}

/// The application specific directory.  
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub enum PrivateDir {

    /// The application specific persistent-data directory.  
    /// 
    /// Files stored in this directory are included in [Android Auto Backup](https://developer.android.com/identity/data/autobackup).  
    /// 
    /// The system prevents other apps and user from accessing these locations. 
    /// In cases where the device is rooted or the user has special permissions, the user may be able to access this.   
    ///  
    /// These files will be deleted when the app is uninstalled and may also be deleted at the user’s request.  
    /// 
    /// e.g. `/data/user/0/{app-package-name}/files`
    /// 
    /// <https://developer.android.com/reference/android/content/Context#getFilesDir()>
    Data,

    /// The application specific cache directory.  
    /// 
    /// Files stored in this directory are **not** included in [Android Auto Backup](https://developer.android.com/identity/data/autobackup).  
    /// 
    /// The system prevents other apps and user from accessing these locations. 
    /// In cases where the device is rooted or the user has special permissions, the user may be able to access this.   
    /// 
    /// These files will be deleted when the app is uninstalled and may also be deleted at the user’s request. 
    /// In addition, the system will automatically delete files in this directory as disk space is needed elsewhere on the device.  
    /// 
    /// e.g. `/data/user/0/{app-package-name}/cache`
    /// 
    /// <https://developer.android.com/reference/android/content/Context#getCacheDir()>
    Cache,

    /// The application specific persistent-data directory.  
    /// 
    /// This is similar to [`PrivateDir::Data`].
    /// But files stored in this directory are **not** included in [Android Auto Backup](https://developer.android.com/identity/data/autobackup).  
    /// 
    /// The system prevents other apps and user from accessing these locations. 
    /// In cases where the device is rooted or the user has special permissions, the user may be able to access this.   
    ///  
    /// These files will be deleted when the app is uninstalled and may also be deleted at the user’s request.  
    /// 
    /// e.g. `/data/user/0/{app-package-name}/no_backup`
    /// 
    /// <https://developer.android.com/reference/android/content/Context#getNoBackupFilesDir()>
    NoBackupData,
}

/// The application specific directory.  
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub enum OutsidePrivateDir {

    /// The application specific persistent-data directory.  
    /// 
    /// These files will be deleted when the app is uninstalled and may also be deleted at the user’s request.  
    Data,
    
    /// The application specific cache directory.  
    /// 
    /// These files will be deleted when the app is uninstalled and may also be deleted at the user’s request. 
    Cache,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub enum PublicDir {
    
    #[serde(untagged)]
    Image(PublicImageDir),

    #[serde(untagged)]
    Video(PublicVideoDir),

    #[serde(untagged)]
    Audio(PublicAudioDir),

    #[serde(untagged)]
    GeneralPurpose(PublicGeneralPurposeDir),
}

/// Directory in which to place images that are available to other applications and users.  
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub enum PublicImageDir {

    /// Standard directory in which to place pictures that are available to the user.  
    /// 
    /// e.g. `~/Pictures`
    Pictures,

    /// The traditional location for pictures and videos when mounting the device as a camera.  
    /// 
    /// e.g. `~/DCIM`
    DCIM,
}

/// Directory in which to place videos that are available to other applications and users.  
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub enum PublicVideoDir {

	/// Standard directory in which to place movies that are available to the user.  
	/// 
	/// e.g. `~/Movies`
	Movies,

	/// The traditional location for pictures and videos when mounting the device as a camera.  
	/// 
	/// e.g. `~/DCIM`
	DCIM,
}

/// Directory in which to place audios that are available to other applications and users.  
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub enum PublicAudioDir {

    /// Standard directory in which to place movies that are available to the user.  
    /// 
    /// e.g. `~/Music`
    Music,

    /// Standard directory in which to place any audio files that should be in the list of alarms that the user can select (not as regular music).  
    /// 
    /// e.g. `~/Alarms`
    Alarms,

    /// Standard directory in which to place any audio files that should be in the list of audiobooks that the user can select (not as regular music).  
    /// 
    /// This is not available on Android 9 (API level 28) and lower.  
    /// 
    /// e.g. `~/Audiobooks`  
    Audiobooks,

    /// Standard directory in which to place any audio files that should be in the list of notifications that the user can select (not as regular music).  
    /// 
    /// e.g. `~/Notifications`
    Notifications,

    /// Standard directory in which to place any audio files that should be in the list of podcasts that the user can select (not as regular music).  
    /// 
    /// e.g. `~/Podcasts`
    Podcasts,

    /// Standard directory in which to place any audio files that should be in the list of ringtones that the user can select (not as regular music).  
    /// 
    /// e.g. `~/Ringtones`
    Ringtones,

    /// Standard directory in which to place any audio files that should be in the list of voice recordings recorded by voice recorder apps that the user can select (not as regular music).   
    /// 
    /// This is not available on Android 11 (API level 30) and lower.  
    /// 
    /// e.g. `~/Recordings`
    Recordings,
}

/// Directory in which to place files that are available to other applications and users.  
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub enum PublicGeneralPurposeDir {

    /// Standard directory in which to place documents that have been created by the user.  
    /// 
    /// e.g. `~/Documents`
    Documents,

    /// Standard directory in which to place files that have been downloaded by the user.  
    /// 
    /// e.g. `~/Download`  
    ///
    /// This is not the plural "Downloads", but the singular "Download".
    /// <https://developer.android.com/reference/android/os/Environment#DIRECTORY_DOWNLOADS>
    Download,
}

impl std::fmt::Display for PublicImageDir {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PublicImageDir::Pictures => write!(f, "Pictures"),
            PublicImageDir::DCIM => write!(f, "DCIM"),
        }
    }
}

impl std::fmt::Display for PublicVideoDir {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PublicVideoDir::Movies => write!(f, "Movies"),
            PublicVideoDir::DCIM => write!(f, "DCIM"),
        }
    }
}

impl std::fmt::Display for PublicAudioDir {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PublicAudioDir::Music => write!(f, "Music"),
            PublicAudioDir::Alarms => write!(f, "Alarms"),
            PublicAudioDir::Audiobooks => write!(f, "Audiobooks"),
            PublicAudioDir::Notifications => write!(f, "Notifications"),
            PublicAudioDir::Podcasts => write!(f, "Podcasts"),
            PublicAudioDir::Ringtones => write!(f, "Ringtones"),
            PublicAudioDir::Recordings => write!(f, "Recordings"),
        }
    }
}

impl std::fmt::Display for PublicGeneralPurposeDir {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PublicGeneralPurposeDir::Documents => write!(f, "Documents"),
            PublicGeneralPurposeDir::Download => write!(f, "Download"),
        }
    }
}

impl std::fmt::Display for PublicDir {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PublicDir::Image(p) => p.fmt(f),
            PublicDir::Video(p) => p.fmt(f),
            PublicDir::Audio(p) => p.fmt(f),
            PublicDir::GeneralPurpose(p) => p.fmt(f),
        }
    }
}

macro_rules! impl_into_pubdir {
    ($target: ident, $wrapper: ident) => {
        impl From<$target> for PublicDir {
            fn from(value: $target) -> Self {
                Self::$wrapper(value)
            }
        }
    };
}
impl_into_pubdir!(PublicImageDir, Image);
impl_into_pubdir!(PublicVideoDir, Video);
impl_into_pubdir!(PublicAudioDir, Audio);
impl_into_pubdir!(PublicGeneralPurposeDir, GeneralPurpose);