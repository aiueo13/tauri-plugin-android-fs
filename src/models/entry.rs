use serde::{Deserialize, Serialize};
use crate::*;


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

    /// If a file, this is no None.  
    /// If a directory, this is None.  
    pub fn file_mime_type(&self) -> Option<&str> {
        match self {
            EntryType::File { mime_type } => Some(&mime_type),
            EntryType::Dir => None,
        }
    }

    /// If a file, this is no None.  
    /// If a directory, this is None.  
    pub fn into_file_mime_type(self) -> Option<String> {
        match self {
            EntryType::File { mime_type } => Some(mime_type),
            EntryType::Dir => None,
        }
    }

    /// If a file, this is no Err.  
    /// If a directory, this is Err.  
    pub fn file_mime_type_or_err(&self) -> Result<&str> {
        self.file_mime_type().ok_or_else(|| Error::with("This is not a file"))
    }

    /// If a file, this is no Err.  
    /// If a directory, this is Err.  
    pub fn into_file_mime_type_or_err(self) -> Result<String> {
        self.into_file_mime_type().ok_or_else(|| Error::with("This is not a file"))
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

impl Entry {

    pub fn is_file(&self) -> bool {
        matches!(self, Self::File { .. })
    }

    pub fn is_dir(&self) -> bool {
        matches!(self, Self::Dir { .. })
    }

    pub fn uri(&self) -> &FileUri {
        match self {
            Entry::File { uri, .. } => uri,
            Entry::Dir { uri, .. } => uri,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Entry::File { name, .. } => name,
            Entry::Dir { name, .. } => name,
        }
    }

    pub fn last_modified(&self) -> std::time::SystemTime {
        match self {
            Entry::File { last_modified, .. } => *last_modified,
            Entry::Dir { last_modified, .. } => *last_modified,
        }
    }

    /// If file, this is no None.  
    /// If directory, this is None.  
    pub fn file_mime_type(&self) -> Option<&str> {
        match self {
            Entry::File { mime_type, .. } => Some(mime_type),
            Entry::Dir { .. } => None,
        }
    }

    /// If a file, this is no None.  
    /// If a directory, this is None.  
    pub fn file_len(&self) -> Option<u64> {
        match self {
            Entry::File { len, .. } => Some(*len),
            Entry::Dir { .. } => None,
        }
    }

    /// If a file, this is no Err.  
    /// If a directory, this is Err.  
    pub fn file_mime_type_or_err(&self) -> Result<&str> {
        self.file_mime_type().ok_or_else(|| Error::with("This is not a file"))
    }

    /// If a file, this is no Err.  
    /// If a directory, this is Err.  
    pub fn file_len_or_err(&self) -> Result<u64> {
        self.file_len().ok_or_else(|| Error::with("This is not a file"))
    }
 }

#[derive(Debug, Clone, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum OptionalEntry {

    #[non_exhaustive]
    File {
        /// If `EntryOptions { uri, .. }` is set to `true`, 
        /// this will never be `None`.
        uri: Option<FileUri>,

        /// If `EntryOptions { name, .. }` is set to `true`, 
        /// this will never be `None`.
        name: Option<String>,

        /// If `EntryOptions { last_modified, .. }` is set to `true`, 
        /// this will never be `None`.
        last_modified: Option<std::time::SystemTime>,

        /// If `EntryOptions { len, .. }` is set to `true`, 
        /// this will never be `None`.
        len: Option<u64>,

        /// If `EntryOptions { mime_type, .. }` is set to `true`, 
        /// this will never be `None`.
        mime_type: Option<String>,
    },

    #[non_exhaustive]
    Dir {
        /// If `EntryOptions { uri, .. }` is set to `true`, 
        /// this will never be `None`.
        uri: Option<FileUri>,

        /// If `EntryOptions { name, .. }` is set to `true`, 
        /// this will never be `None`.
        name: Option<String>,

        /// If `EntryOptions { last_modified, .. }` is set to `true`, 
        /// this will never be `None`.
        last_modified: Option<std::time::SystemTime>,
    }
}

impl OptionalEntry {

    pub fn is_file(&self) -> bool {
        matches!(self, Self::File { .. })
    }

    pub fn is_dir(&self) -> bool {
        matches!(self, Self::Dir { .. })
    }

    /// If `EntryOptions { uri, .. }` is set to `true`, 
    /// this will never be `None`.
    pub fn into_uri(self) -> Option<FileUri> {
        match self {
            Self::File { uri, .. } => uri,
            Self::Dir { uri, .. } => uri,
        }
    }
    
    /// If `EntryOptions { uri, .. }` is set to `true`, 
    /// this will never be error.
    pub fn into_uri_or_err(self) -> Result<FileUri> {
        self.into_uri().ok_or_else(|| Error::missing_value("uri"))
    }

    /// If `EntryOptions { uri, .. }` is set to `true`, 
    /// this will never be error.
    pub fn uri_or_err(&self) -> Result<&FileUri> {
        self.uri().ok_or_else(|| Error::missing_value("uri"))
    }

    /// If `EntryOptions { name, .. }` is set to `true`, 
    /// this will never be error.
    pub fn name_or_err(&self) -> Result<&str> {
        self.name().ok_or_else(|| Error::missing_value("name"))
    }

    /// If `EntryOptions { last_modified, .. }` is set to `true`, 
    /// this will never be error.
    pub fn last_modified_or_err(&self) -> Result<std::time::SystemTime> {
        self.last_modified().ok_or_else(|| Error::missing_value("last_modified"))
    }

    /// If a file and `EntryOptions { mime_type, .. }` is set to `true`, 
    /// this will never be error.
    pub fn file_mime_type_or_err(&self) -> Result<&str> {
        self.file_mime_type().ok_or_else(|| Error::with("Not file or missing value: mime_type"))
    }

    /// If a file and `EntryOptions { len, .. }` is set to `true`, 
    /// this will never be error.
    pub fn file_len_or_err(&self) -> Result<u64> {
        self.file_len().ok_or_else(|| Error::with("Not file or missing value: len"))
    }

    /// If `EntryOptions { uri, .. }` is set to `true`, 
    /// this will never be `None`.
    pub fn uri(&self) -> Option<&FileUri> {
        match self {
            Self::File { uri, .. } => uri.as_ref(),
            Self::Dir { uri, .. } => uri.as_ref(),
        }
    }

    /// If `EntryOptions { name, .. }` is set to `true`, 
    /// this will never be `None`.
    pub fn name(&self) -> Option<&str> {
        match self {
            Self::File { name, .. } => name.as_deref(),
            Self::Dir { name, .. } => name.as_deref(),
        }
    }

    /// If `EntryOptions { last_modified, .. }` is set to `true`, 
    /// this will never be `None`.
    pub fn last_modified(&self) -> Option<std::time::SystemTime> {
        match self {
            Self::File { last_modified, .. } => *last_modified,
            Self::Dir { last_modified, .. } => *last_modified,
        }
    }

    /// If a file and `EntryOptions { mime_type, .. }` is set to `true`, 
    /// this will never be `None`.
    pub fn file_mime_type(&self) -> Option<&str> {
        match self {
            Self::File { mime_type, .. } => mime_type.as_deref(),
            Self::Dir { .. } => None,
        }
    }

    /// If a file and `EntryOptions { len, .. }` is set to `true`, 
    /// this will never be `None`.
    pub fn file_len(&self) -> Option<u64> {
        match self {
            Self::File { len, .. } => *len,
            Self::Dir { .. } => None,
        }
    }
 }

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EntryOptions {
    pub uri: bool,
    pub name: bool,
    pub last_modified: bool,
    pub len: bool,
    pub mime_type: bool,
}

impl EntryOptions {

    pub const ALL: EntryOptions = EntryOptions {
        uri: true,
        name: true,
        last_modified: true,
        len: true,
        mime_type: true
    };

    pub const NONE: EntryOptions = EntryOptions {
        uri: false,
        name: false,
        last_modified: false,
        len: false,
        mime_type: false
    };

    pub const URI_ONLY: EntryOptions = EntryOptions {
        uri: true,
        ..Self::NONE
    };

    pub const URI_AND_NAME: EntryOptions = EntryOptions {
        uri: true,
        name: true,
        ..Self::NONE
    };
}

impl TryFrom<OptionalEntry> for Entry {
    type Error = crate::Error;

    fn try_from(value: OptionalEntry) -> std::result::Result<Self, Self::Error> {
        Ok(match value {
            OptionalEntry::File { uri, name, last_modified, len, mime_type } => Entry::File {
                uri: uri.ok_or_else(|| Error::missing_value("uri"))?,
                name: name.ok_or_else(|| Error::missing_value("name"))?,
                last_modified: last_modified.ok_or_else(|| Error::missing_value("last_modified"))?,
                len: len.ok_or_else(|| Error::missing_value("len"))?,
                mime_type: mime_type.ok_or_else(|| Error::missing_value("mime_type"))?,
            },
            OptionalEntry::Dir { uri, name, last_modified } => Entry::Dir {
                uri: uri.ok_or_else(|| Error::missing_value("uri"))?,
                name: name.ok_or_else(|| Error::missing_value("name"))?,
                last_modified: last_modified.ok_or_else(|| Error::missing_value("last_modified"))?,
            },
        })
    }
}