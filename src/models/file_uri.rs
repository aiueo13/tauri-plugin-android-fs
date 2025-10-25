use serde::{Deserialize, Serialize};
use crate::*;


/// Path to represent a file or directory.
/// 
/// # Note
/// For compatibility, an interconversion to [`tauri_plugin_fs::FilePath`] is implemented, such as follwing.  
/// This is lossy and also not guaranteed to work properly with other plugins.  
/// However, reading and writing files by official [`tauri_plugin_fs`] etc. should work well.  
/// ```ignore
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

#[allow(unused)]
impl FileUri {

    /// This is same as [`FileUri::to_json_string`]
    #[deprecated = "Confusing name. Use FileUri::to_json_string instead"]
    pub fn to_string(&self) -> Result<String> {
        serde_json::to_string(self).map_err(Into::into)
    }

    /// This is same as [`FileUri::from_json_str`]
    #[deprecated = "Confusing name. Use FileUri::from_json_str instead"]
    pub fn from_str(s: &str) -> Result<Self> {
        serde_json::from_str(s).map_err(Into::into)
    }

    pub fn to_json_string(&self) -> Result<String> {
        serde_json::to_string(self).map_err(Into::into)
    }

    pub fn from_json_str(json: impl AsRef<str>) -> Result<Self> {
        serde_json::from_str(json.as_ref()).map_err(Into::into)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(self).map_err(Into::into)
    }

    pub fn from_bytes(bytes: impl AsRef<[u8]>) -> Result<Self> {
        serde_json::from_slice(bytes.as_ref()).map_err(Into::into)
    }

    pub fn from_path(path: impl AsRef<std::path::Path>) -> Self {
        Self { uri: format!("file://{}", path.as_ref().to_string_lossy()), document_top_tree_uri: None }
    }

    pub(crate) fn as_path(&self) -> Option<&std::path::Path> {
        if self.uri.starts_with("file://") {
            return Some(std::path::Path::new(self.uri.trim_start_matches("file://")))
        }
        None
    }

    pub(crate) fn is_content_scheme(&self) -> bool {
        self.uri.starts_with("content://")
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

#[cfg(feature = "tauri-plugin-fs")]
impl From<tauri_plugin_fs::FilePath> for FileUri {

    fn from(value: tauri_plugin_fs::FilePath) -> Self {
        match value {
            tauri_plugin_fs::FilePath::Url(url) => Self { uri: url.to_string(), document_top_tree_uri: None },
            tauri_plugin_fs::FilePath::Path(path_buf) => path_buf.into(),
        }
    }
}

#[cfg(feature = "tauri-plugin-fs")]
impl From<FileUri> for tauri_plugin_fs::FilePath {

    fn from(value: FileUri) -> Self {
        type NeverErr<T> = std::result::Result::<T, std::convert::Infallible>;
        NeverErr::unwrap(value.uri.parse())
    }
}