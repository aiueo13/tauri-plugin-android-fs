use serde::{Deserialize, Serialize};
use crate::*;


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
    /// Until Android 10, this will always truncate existing contents.   
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