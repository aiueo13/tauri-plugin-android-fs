use serde::{Deserialize, Serialize};
use crate::*;


/// Access mode.
/// 
/// # Serialization
/// Serialized by `serde` as the following TypeScript type:
///
/// ```ts
/// // NOTE: New variants may be added in the future
/// type FileAccessMode = "Read" | "Write" | "WriteTruncate" | "WriteAppend" | "ReadWrite" | "ReadWriteTruncate";
/// ```
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Deserialize, Serialize)]
pub enum FileAccessMode {

    /// Opens the file in read-only mode.
    /// 
    /// FileDescriptor mode: "r"
    Read,

    /// Opens the file in write-only mode.  
    /// 
    /// Note that this may or may not truncate existing contents.   
    /// If the new file is smaller than the old one, **this may cause the file to become corrupted**.
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
            mode => Err(Error::with(format!("Illegal mode: {mode}")))
        }
    }
}

/// Uri permission
/// 
/// # Serialization
/// Serialized by `serde` as the following TypeScript type:
///
/// ```ts
/// type UriPermission = "Read" | "Write" | "ReadAndWrite" | "ReadOrWrite";
/// ```
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Deserialize, Serialize)]
pub enum UriPermission {

    /// Read access.
    Read,

    /// Write access.
    Write,

    /// Read-write access.
    ReadAndWrite,

    /// Read or write access.
    ReadOrWrite,
}

/// Persisted uri permission state
/// 
/// # Serialization
/// Serialized by `serde` as the following TypeScript type:
///
/// ```ts
/// type PersistedUriPermissionState = {
///     type: "Dir" | "File", 
///     uri: FsUri, 
///     canRead: boolean, 
///     canWrite: boolean,
/// };
/// 
/// // See `tauri_plugin_android_fs::FsUri` for details
/// type FsUri = unknown;
/// ```
#[derive(Debug, Clone, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum PersistedUriPermissionState {
    File {
        uri: FsUri,

        #[serde(rename = "canRead")]
        can_read: bool,

        #[serde(rename = "canWrite")]
        can_write: bool,
    },
    Dir {
        uri: FsUri,

        #[serde(rename = "canRead")]
        can_read: bool,

        #[serde(rename = "canWrite")]
        can_write: bool,
    }
}

impl PersistedUriPermissionState {

    pub fn uri(&self) -> &FsUri {
        match self {
            PersistedUriPermissionState::File { uri, .. } => uri,
            PersistedUriPermissionState::Dir { uri, .. } => uri,
        }
    }

    pub fn into_uri(self) -> FsUri {
        match self {
            PersistedUriPermissionState::File { uri, .. } => uri,
            PersistedUriPermissionState::Dir { uri, .. } => uri,
        }
    }

    pub fn can_read(&self) -> bool {
        match self {
            PersistedUriPermissionState::File { can_read, .. } => *can_read,
            PersistedUriPermissionState::Dir { can_read, .. } => *can_read,
        }
    }

    pub fn can_write(&self) -> bool {
        match self {
            PersistedUriPermissionState::File { can_write, .. } => *can_write,
            PersistedUriPermissionState::Dir { can_write, .. } => *can_write,
        }
    }

    pub fn is_file(&self) -> bool {
        matches!(self, PersistedUriPermissionState::File { .. })
    }

    pub fn is_dir(&self) -> bool {
        matches!(self, PersistedUriPermissionState::Dir { .. })
    }
}