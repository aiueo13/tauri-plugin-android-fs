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