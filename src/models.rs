use serde::{Deserialize, Serialize};

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

    /// Only files/directories under the directory obtained by AndroidFs::show_picker_for_manage_dir will own this.
    pub document_top_tree_uri: Option<String>,
}

impl FileUri {

    pub fn to_string(&self) -> crate::Result<String> {
        serde_json::to_string(self).map_err(Into::into)
    }

    pub fn from_str(s: &str) -> crate::Result<Self> {
        serde_json::from_str(s).map_err(Into::into)
    }
}

impl From<&std::path::PathBuf> for FileUri {

    fn from(value: &std::path::PathBuf) -> Self {
        Self { uri: format!("file://{}", value.to_string_lossy()), document_top_tree_uri: None }
    }
}

impl From<std::path::PathBuf> for FileUri {

    fn from(ref value: std::path::PathBuf) -> Self {
        value.into()
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
        let result: std::result::Result<_, std::convert::Infallible> = value.uri.parse();

        // This will not cause panic. 
        // Because result err is infallible.
        result.unwrap()
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
    /// **This may or may not truncate existing contents.**  
    /// So please use [`FileAccessMode::WriteTruncate`] or [`FileAccessMode::WriteAppend`] instead.  
    ///
    /// FileDescriptor mode: "w"
    #[deprecated(note = "This may or may not truncate existing contents. So please use WriteTruncate or WriteAppend instead.")]
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

/// Filters for VisualMediaPicker.
#[derive(Debug, Clone, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub enum VisualMediaTarget {

    /// Allow only images to be selected.  
    ImageOnly,

    /// Allow only videos to be selected.  
    VideoOnly,

    /// Allow only images and videos to be selected.  
    ImageAndVideo,
}

/// The application specific directory.  
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub enum PrivateDir {

    /// The application specific persistent-data directory.  
    /// 
    /// The system prevents other apps and user from accessing these locations. 
    /// In cases where the device is rooted or the user has special permissions, the user may be able to access this.   
    ///  
    /// These files will be deleted when the app is uninstalled and may also be deleted at the user’s request.  
    /// 
    /// ex: `/data/user/0/{app-package-name}/files`
    Data,

    /// The application specific cache directory.  
    /// 
    /// The system prevents other apps and user from accessing these locations. 
    /// In cases where the device is rooted or the user has special permissions, the user may be able to access this.   
    /// 
    /// These files will be deleted when the app is uninstalled and may also be deleted at the user’s request. 
    /// In addition, the system will automatically delete files in this directory as disk space is needed elsewhere on the device.  
    /// 
    /// ex: `/data/user/0/{app-package-name}/cache`
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
    /// ex: `~/Pictures`
    Pictures,

    /// The traditional location for pictures and videos when mounting the device as a camera.  
    /// 
    /// ex: `~/DCIM`
    DCIM,
}

/// Directory in which to place videos that are available to other applications and users.  
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub enum PublicVideoDir {

	/// Standard directory in which to place movies that are available to the user.  
	/// 
	/// ex: `~/Movies`
	Movies,

	/// The traditional location for pictures and videos when mounting the device as a camera.  
	/// 
	/// ex: `~/DCIM`
	DCIM,
}

/// Directory in which to place audios that are available to other applications and users.  
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub enum PublicAudioDir {

    /// Standard directory in which to place movies that are available to the user.  
    /// 
    /// ex: `~/Music`
    Music,

    /// Standard directory in which to place any audio files that should be in the list of alarms that the user can select (not as regular music).  
    /// 
    /// ex: `~/Alarms`
    Alarms,

    /// Standard directory in which to place any audio files that should be in the list of audiobooks that the user can select (not as regular music).  
    /// 
    /// This is not available on Android 9 (API level 28) and lower.  
    /// 
    /// ex: `~/Audiobooks`  
    Audiobooks,

    /// Standard directory in which to place any audio files that should be in the list of notifications that the user can select (not as regular music).  
    /// 
    /// ex: `~/Notifications`
    Notifications,

    /// Standard directory in which to place any audio files that should be in the list of podcasts that the user can select (not as regular music).  
    /// 
    /// ex: `~/Podcasts`
    Podcasts,

    /// Standard directory in which to place any audio files that should be in the list of ringtones that the user can select (not as regular music).  
    /// 
    /// ex: `~/Ringtones`
    Ringtones,

    /// Standard directory in which to place any audio files that should be in the list of voice recordings recorded by voice recorder apps that the user can select (not as regular music).   
    /// 
    /// This is not available on Android 11 (API level 30) and lower.  
    /// 
    /// ex: `~/Recordings`
    Recordings,
}

/// Directory in which to place files that are available to other applications and users.  
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub enum PublicGeneralPurposeDir {

    /// Standard directory in which to place documents that have been created by the user.  
    /// 
    /// ex: `~/Documents`
    Documents,

    /// Standard directory in which to place files that have been downloaded by the user.  
    /// 
    /// ex: `~/Download`  
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

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub enum InitialLocation<'a> {

    TopPublicDir,

    PublicDir(PublicDir),
    
    DirInPublicDir {
        base_dir: PublicDir,
        relative_path: &'a str,
    },

    DirInPublicAppDir {
        base_dir: PublicDir,
        relative_path: &'a str,
    }
}

impl<T: Into<PublicDir>> From<T> for InitialLocation<'_> {
    fn from(value: T) -> Self {
        InitialLocation::PublicDir(value.into())
    }
}