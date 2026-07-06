use crate::*;
use serde::{Deserialize, Serialize};

#[deprecated = "Use `tauri_plugin_android_fs::FsUri` instead."]
pub type FileUri = FsUri;

/// URI for a file or directory.
///
/// # Note
/// Serialized by `serde` as:
///
/// ```ts
/// type AndroidFsUri = {
///     uri: string,
///     documentTopTreeUri: string | null
/// }
/// ```
#[derive(Debug, Clone, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FsUri {

    /// URI for a file or directory.
    ///
    /// # Note
    /// This is a URI with either the `content` or `file` scheme.
    pub uri: String,

    /// Document tree URI of the root directory from which this entry originates.
    ///
    /// # Note
    /// This field is set for directories obtained via Directory Picker
    /// and for entries derived from those directories.
    pub document_top_tree_uri: Option<String>,
}

impl FsUri {

    /// Same as `serde_json::to_string()`
    pub fn to_json_string(&self) -> Result<String> {
        serde_json::to_string(self).map_err(Into::into)
    }

    /// Same as `serde_json::from_str()`
    pub fn from_json_str(json: impl AsRef<str>) -> Result<Self> {
        serde_json::from_str(json.as_ref()).map_err(Into::into)
    }

    pub fn from_uri(uri: impl Into<String>) -> Self {
        FsUri {
            uri: uri.into(),
            document_top_tree_uri: None,
        }
    }

    /// Constructs a URI from the absolute path of a file or directory.
    ///
    /// # Note
    /// The path must be absolute and must not contain `./` or `../`.
    /// Even if the path is invalid, this function will not return an error or panic;
    /// instead, it returns an invalid URI.
    ///
    /// Note the following:
    /// - This URI cannot be used with [`Opener`](crate::api::api_async::Opener) to open files in other apps.
    /// - Operations using this URI may fall back to [`std::fs`] instead of the Kotlin API.
    pub fn from_path(path: impl AsRef<std::path::Path>) -> Self {
        Self {
            uri: path_to_android_file_uri(path),
            document_top_tree_uri: None,
        }
    }

    /// Returns the path if this URI uses the `file` scheme;
    /// otherwise, returns `None`.
    pub fn to_path(&self) -> Option<std::path::PathBuf> {
        if self.is_file_scheme() {
            return Some(android_file_uri_to_path(&self.uri));
        }
        None
    }

    /// Returns `true` if this URI uses the `file` scheme.
    pub fn is_file_scheme(&self) -> bool {
        self.uri.starts_with("file://")
    }

    /// Returns `true` if this URI uses the `content` scheme.
    pub fn is_content_scheme(&self) -> bool {
        self.uri.starts_with("content://")
    }
}

impl From<&std::path::Path> for FsUri {
    fn from(path: &std::path::Path) -> Self {
        Self::from_path(path)
    }
}

impl From<&std::path::PathBuf> for FsUri {
    fn from(path: &std::path::PathBuf) -> Self {
        Self::from_path(path)
    }
}

impl From<std::path::PathBuf> for FsUri {
    fn from(path: std::path::PathBuf) -> Self {
        Self::from_path(path)
    }
}

impl From<tauri_plugin_fs::FilePath> for FsUri {
    fn from(value: tauri_plugin_fs::FilePath) -> Self {
        match value {
            tauri_plugin_fs::FilePath::Url(url) => Self::from_uri(url),
            tauri_plugin_fs::FilePath::Path(path) => Self::from_path(path),
        }
    }
}

impl From<FsUri> for tauri_plugin_fs::FilePath {
    fn from(value: FsUri) -> Self {
        type NeverErr<T> = std::result::Result<T, std::convert::Infallible>;
        NeverErr::unwrap(value.uri.parse())
    }
}

fn android_file_uri_to_path(uri: impl AsRef<str>) -> std::path::PathBuf {
    let uri = uri.as_ref();
    let path_part = uri.strip_prefix("file://").unwrap_or(uri);
    let decoded = percent_encoding::percent_decode_str(path_part).decode_utf8_lossy();

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

        assert_eq!(
            uri,
            "file:///sdcard/%E3%83%80%E3%82%A6%E3%83%B3%E3%83%AD%E3%83%BC%E3%83%89"
        );
        assert_eq!(android_file_uri_to_path(&uri), path);
    }
}
