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
    pub uri: String,
    pub document_top_tree_uri: Option<String>,
}

#[allow(unused)]
impl FileUri {

    /// Same as `serde_json::to_string(...)`
    pub fn to_json_string(&self) -> Result<String> {
        serde_json::to_string(self).map_err(Into::into)
    }

    /// Same as `serde_json::from_str(...)`
    pub fn from_json_str(json: impl AsRef<str>) -> Result<Self> {
        serde_json::from_str(json.as_ref()).map_err(Into::into)
    }

    pub fn from_uri(uri: impl Into<String>) -> Self {
        FileUri {
            uri: uri.into(),
            document_top_tree_uri: None 
        }
    }

    /// Constructs a URI from the absolute path of a file or directory.   
    /// 
    /// This must be an absolute path that does not contain `./` or `../`.
    /// Even if the path is invalid, it will not cause an error or panic; an invalid URI will be returned.   
    /// 
    /// # Note
    /// There are a few points to note regarding this.
    /// - This URI cannot be passed to functions of [`FileOpener`](crate::api::api_async::FileOpener) for sending to other apps.
    /// - Operations using this URI may fall back to [`std::fs`] instead of Kotlin API.
    pub fn from_path(path: impl AsRef<std::path::Path>) -> Self {
        Self {
            uri: path_to_android_file_uri(path), 
            document_top_tree_uri: None
        }
    }

    /// If this URI is an Android file-scheme URI, for example,
    /// via [`FileUri::from_path`], its absolute path will be retrieved.
    pub fn to_path(&self) -> Option<std::path::PathBuf> {
        if self.is_file_uri() {
            return Some(android_file_uri_to_path(&self.uri))
        }
        None
    }

    /// Indicates whether this is `file://` URI.
    pub fn is_file_uri(&self) -> bool {
        self.uri.starts_with("file://")
    }

    /// Indicates whether this is `content://` URI.
    pub fn is_content_uri(&self) -> bool {
        self.uri.starts_with("content://")
    }

    pub(crate) fn require_content_uri(&self) -> Result<()> {
        if self.is_content_uri() {
            Ok(())
        }
        else {
            Err(Error::invalid_uri_scheme(&self.uri))
        }
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
            tauri_plugin_fs::FilePath::Url(url) => Self::from_uri(url),
            tauri_plugin_fs::FilePath::Path(path) => Self::from_path(path),
        }
    }
}

impl From<FileUri> for tauri_plugin_fs::FilePath {

    fn from(value: FileUri) -> Self {
        type NeverErr<T> = std::result::Result::<T, std::convert::Infallible>;
        NeverErr::unwrap(value.uri.parse())
    }
}


fn android_file_uri_to_path(uri: impl AsRef<str>) -> std::path::PathBuf {
    let uri = uri.as_ref();
    let path_part = uri.strip_prefix("file://").unwrap_or(uri);
    let decoded = percent_encoding::percent_decode_str(path_part)
        .decode_utf8_lossy();

    std::path::PathBuf::from(decoded.as_ref())
}

fn path_to_android_file_uri(path: impl AsRef<std::path::Path>) -> String {
    let encoded = path
        .as_ref()
        .to_string_lossy()
        .split('/')
        .map(|s| encode_android_uri_component(s))
        .collect::<Vec<_>>()
        .join("/");

    format!("file://{}", encoded)
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_android_safe_characters() {
        let path = Path::new("/sdcard/test_file-name!.~'()*.txt");
        let uri = path_to_android_file_uri(path);
        
        assert_eq!(uri, "file:///sdcard/test_file-name!.~'()*.txt");
        assert_eq!(android_file_uri_to_path(&uri), path);
    }

    #[test]
    fn test_spaces_and_unsafe_chars() {
        let path = Path::new("/sdcard/My Documents/file @#$%.txt");
        let uri = path_to_android_file_uri(path);
        
        assert_eq!(uri, "file:///sdcard/My%20Documents/file%20%40%23%24%25.txt");
        assert_eq!(android_file_uri_to_path(&uri), path);
    }

    #[test]
    fn test_unicode_characters() {
        let path = Path::new("/sdcard/ダウンロード");
        let uri = path_to_android_file_uri(path);
        
        assert_eq!(uri, "file:///sdcard/%E3%83%80%E3%82%A6%E3%83%B3%E3%83%AD%E3%83%BC%E3%83%89");
        assert_eq!(android_file_uri_to_path(&uri), path);
    }
}