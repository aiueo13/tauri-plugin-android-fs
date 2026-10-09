use super::*;
use crate::*;
use sync_async::sync_async;
use tauri::Manager;

/// ***Root API***
///
/// # Examples
/// ```no_run
/// use tauri_plugin_android_fs::AndroidFsExt;
///
/// async fn example(app: &tauri::AppHandle) {
///     let api = app.android_fs();
///     let api_async = app.android_fs_async();
/// }
/// ```
#[sync_async]
pub struct AndroidFs<R: tauri::Runtime> {

    #[cfg(target_os = "android")]
    pub(crate) handle: tauri::plugin::PluginHandle<R>,

    #[cfg(not(target_os = "android"))]
    #[allow(unused)]
    pub(crate) handle: std::marker::PhantomData<fn() -> R>,
}

#[cfg(target_os = "android")]
#[sync_async(
    use(if_sync) impls::SyncImpls as Impls;
    use(if_async) impls::AsyncImpls as Impls;
)]
impl<R: tauri::Runtime> AndroidFs<R> {

    #[always_sync]
    pub(crate) fn impls(&self) -> Impls<'_, R> {
        Impls {
            handle: &self.handle,
        }
    }
}

#[sync_async(
    use(if_async) api_async::{Opener, Picker, AppStorage, PrivateStorage, PublicStorage, Notifications, ProgressNotificationGuard};
    use(if_sync) api_sync::{Opener, Picker, AppStorage, PrivateStorage, PublicStorage, Notifications, ProgressNotificationGuard};
)]
impl<R: tauri::Runtime> AndroidFs<R> {

    /// API of file storage that is available to other applications and users.
    #[always_sync]
    pub fn public_storage(&self) -> PublicStorage<'_, R> {
        PublicStorage { handle: &self.handle, }
    }

    /// API of file storage intended for the app's use only.
    #[always_sync]
    pub fn private_storage(&self) -> PrivateStorage<'_, R> {
        PrivateStorage { handle: &self.handle, }
    }

    /// API of file storage intended for the app's use.
    #[always_sync]
    pub fn app_storage(&self) -> AppStorage<'_, R> {
        AppStorage { handle: &self.handle, }
    }

    /// API of file/dir picker.
    #[always_sync]
    pub fn picker(&self) -> Picker<'_, R> {
        Picker { handle: &self.handle, }
    }

    /// API of opening file/dir with other apps.
    #[always_sync]
    pub fn opener(&self) -> Opener<'_, R> {
        Opener { handle: &self.handle, }
    }

    /// API of notifications.
    #[always_sync]
    pub fn notifications(&self) -> Notifications<'_, R> {
        Notifications { handle: &self.handle, }
    }

    /// Get the file or directory name.
    ///
    /// # Args
    /// - ***uri*** :
    /// Target URI.
    /// Must be **readable**.
    ///
    /// # Support
    /// All Android versions supported by Tauri.
    #[maybe_async]
    pub fn get_name(&self, uri: &FsUri) -> Result<String> {
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().get_entry_name(uri).await
        }
    }

    /// Gets the file or directory name,
    /// or falls back to the URI's last path segment (percent-decoded).
    #[maybe_async]
    pub fn get_name_or_last_path_segment(&self, uri: &FsUri) -> String {
        #[cfg(target_os = "android")] {
            if let Ok(name) = self.impls().get_entry_name(uri).await {
                return name;
            }
        }

        let uri = percent_encoding::percent_decode_str(&uri.uri).decode_utf8_lossy();

        uri.rsplit_once("/")
            .map(|(_, l)| l)
            .unwrap_or(&uri)
            .to_string()
    }

    /// Queries the provider to get the MIME type.
    ///
    /// For file URIs via [`FsUri::from_path`], the MIME type is determined from the file extension.
    /// In most other cases, it uses the MIME type that was associated with the file when it was created.
    /// If the MIME type is unknown or unset, it falls back to `"application/octet-stream"`.
    ///
    /// If the target is a directory, an error will occur.
    /// To check whether the target is a file or a directory, use [`AndroidFs::get_type`].
    ///
    /// # Args
    /// - ***uri*** :
    /// Target file URI.
    /// Must be **readable**.
    ///
    /// # Support
    /// All Android versions supported by Tauri.
    #[maybe_async]
    pub fn get_mime_type(&self, uri: &FsUri) -> Result<String> {
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().get_file_mime_type(uri).await
        }
    }

    /// Gets the entry type.
    ///
    /// If the target is a directory, returns [`EntryType::Dir`].
    ///
    /// If the target is a file, returns [`EntryType::File { mime_type }`](EntryType::File).
    /// For file URIs via [`FsUri::from_path`], the MIME type is determined from the file extension.
    /// In most other cases, it uses the MIME type that was associated with the file when it was created.
    /// If the MIME type is unknown or unset, it falls back to `"application/octet-stream"`.
    ///
    /// # Args
    /// - ***uri*** :
    /// Target URI.
    /// Must be **readable**.
    ///
    /// # Support
    /// All Android versions supported by Tauri.
    #[maybe_async]
    pub fn get_type(&self, uri: &FsUri) -> Result<EntryType> {
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().get_entry_type(uri).await
        }
    }

    /// Gets the entry metadata.
    ///
    /// # Args
    /// - ***uri*** :
    /// Target URI.
    /// Must be **readable**.
    ///
    /// # Support
    /// All Android versions supported by Tauri.
    #[maybe_async]
    pub fn get_metadata(&self, uri: &FsUri) -> Result<Entry> {
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().get_entry_info(uri).await
        }
    }

    /// Gets the file length in bytes.
    ///
    /// # Args
    /// - ***uri*** :
    /// Target URI.
    /// Must be **readable**.
    ///
    /// # Support
    /// All Android versions supported by Tauri.
    #[maybe_async]
    pub fn get_len(&self, uri: &FsUri) -> Result<u64> {
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().get_file_len(uri).await
        }
    }

    /// Open the file in **readable** mode.
    ///
    /// # Note
    /// If the target is a file on cloud storage or otherwise not physically present on the device,
    /// the file provider may downloads the entire contents, and then opens it.
    /// As a result, this processing may take longer than with regular local files.
    /// And files might be a pair of pipe or socket for streaming data.
    ///
    /// # Args
    /// - ***uri*** :
    /// Target file URI.
    /// This need to be **readable**.
    ///
    /// # Support
    /// All Android versions supported by Tauri.
    #[maybe_async]
    pub fn open_file_readable(&self, uri: &FsUri) -> Result<std::fs::File> {
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().open_file_readable(uri).await
        }
    }

    /// Open the file in **writable** mode.
    /// This truncates the existing contents.
    ///
    /// # Args
    /// - ***uri*** :
    /// Target file URI.
    /// This need to be **writable**.
    ///
    /// # Support
    /// All Android versions supported by Tauri.
    #[maybe_async]
    pub fn open_file_writable(&self, uri: &FsUri) -> Result<std::fs::File> {
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().open_file_writable(uri).await
        }
    }

    /// Open the file in the specified mode.
    ///
    /// # Note
    /// 1. **Delay**:
    /// If the target is a file on cloud storage or otherwise not physically present on the device,
    /// the file provider may downloads the entire contents, and then opens it.
    /// As a result, this processing may take longer than with regular local files.
    /// And files might be a pair of pipe or socket for streaming data.
    ///
    /// 2. **File mode restrictions**:
    /// Files provided by third-party apps may not support modes other than
    /// [`FileAccessMode::Write`] or [`FileAccessMode::Read`].
    /// However, [`FileAccessMode::Write`] does not guarantee
    /// that existing contents will always be truncated.
    /// As a result, if the new contents are shorter than the original, the file may
    /// become corrupted. To avoid this, consider using
    /// [`AndroidFs::open_file_writable`], which
    /// ensure that existing contents are truncated and also automatically apply the
    /// maximum possible fallbacks.
    /// - <https://issuetracker.google.com/issues/180526528>
    ///
    /// # Args
    /// - ***uri*** :
    /// Target file URI.
    /// This must have corresponding permissions (read, write, or both) for the specified ***mode***.
    ///
    /// - ***mode*** :
    /// Indicates how the file is opened and the permissions granted.
    ///
    /// # Support
    /// All Android versions supported by Tauri.
    #[maybe_async]
    pub fn open_file(&self, uri: &FsUri, mode: FileAccessMode) -> Result<std::fs::File> {
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().open_file(uri, mode).await
        }
    }

    /// For detailed documentation and notes, see [`AndroidFs::open_file`].
    ///
    /// The modes specified in ***candidate_modes*** are tried in order.
    /// If the file can be opened, this returns the file along with the mode used.
    /// If all attempts fail, an error is returned.
    #[maybe_async]
    pub fn open_file_with_fallback(
        &self,
        uri: &FsUri,
        candidate_modes: impl IntoIterator<Item = FileAccessMode>,
    ) -> Result<(std::fs::File, FileAccessMode)> {
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().open_file_with_fallback(uri, candidate_modes).await
        }
    }

    /// Reads the entire contents of a file into a bytes vector.
    ///
    /// # Args
    /// - ***uri*** :
    /// Target file URI.
    /// Must be **readable**.
    ///
    /// # Support
    /// All Android versions supported by Tauri.
    #[maybe_async]
    pub fn read(&self, uri: &FsUri) -> Result<Vec<u8>> {
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().read_file(uri).await
        }
    }

    /// Reads the entire contents of a file into a string.
    ///
    /// # Args
    /// - ***uri*** :
    /// Target file URI.
    /// Must be **readable**.
    ///
    /// # Support
    /// All Android versions supported by Tauri.
    #[maybe_async]
    pub fn read_to_string(&self, uri: &FsUri) -> Result<String> {
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().read_file_to_string(uri).await
        }
    }

    /// Writes a slice as the entire contents of a file.
    /// This function will entirely replace its contents if it does exist.
    ///
    /// # Args
    /// - ***uri*** :
    /// Target file URI.
    /// Must be **writable**.
    ///
    /// # Support
    /// All Android versions supported by Tauri.
    #[maybe_async]
    pub fn write(&self, uri: &FsUri, contents: impl AsRef<[u8]>) -> Result<()> {
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().write_file(uri, contents).await
        }
    }

    /// Copies the contents of the source file to the destination.
    /// If the destination already has contents, they are truncated before writing the source contents.
    ///
    /// # Args
    /// - ***src*** :
    /// The URI of source file.
    /// Must be **readable**.
    ///
    /// - ***dest*** :
    /// The URI of destination file.
    /// Must be **writable**.
    ///
    /// # Support
    /// All Android versions supported by Tauri.
    #[maybe_async]
    pub fn copy(&self, src: &FsUri, dest: &FsUri) -> Result<()> {
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().copy_file(src, dest).await
        }
    }

    /// Renames a file or directory to a new name, and return new URI.
    /// Even if the names conflict, the existing file will not be overwritten.
    ///
    /// Note that when files or folders (and their descendants) are renamed, their URIs will change, and any previously granted permissions will be lost.
    /// In other words, this function returns a new URI without any permissions.
    /// However, for files created in PublicStorage, the URI remains unchanged even after such operations, and all permissions are retained.
    /// In this, this function returns the same URI as original URI.
    ///
    /// # Args
    /// - ***uri*** :
    /// URI of target entry.
    ///
    /// - ***new_name*** :
    /// New name of target entry.
    /// This include extension if use.
    /// The behavior in the same name already exists depends on the file provider.
    /// In the case of e.g. [`PublicStorage`], the suffix (e.g. `(1)`) is added to this name.
    /// In the case of files hosted by other applications, errors may occur.
    /// But at least, the existing file will not be overwritten.
    /// The system may sanitize these strings as needed, so those strings may not be used as it is.
    ///
    /// # Support
    /// All Android versions supported by Tauri.
    #[maybe_async]
    pub fn rename(&self, uri: &FsUri, new_name: impl AsRef<str>) -> Result<FsUri> {
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().rename_entry(uri, new_name).await
        }
    }

    /// Remove the file.
    ///
    /// # Args
    /// - ***uri*** :
    /// Target file URI.
    /// Must be **read-writable**.
    /// If not file, an error will occur.
    ///
    /// # Support
    /// All Android versions supported by Tauri.
    #[maybe_async]
    pub fn remove_file(&self, uri: &FsUri) -> Result<()> {
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().remove_file(uri).await
        }
    }

    /// Remove the **empty** directory.
    ///
    /// # Args
    /// - ***uri*** :
    /// Target directory URI.
    /// Must be **read-writable**.
    /// If not empty directory, an error will occur.
    ///
    /// # Support
    /// All Android versions supported by Tauri.
    #[maybe_async]
    pub fn remove_dir(&self, uri: &FsUri) -> Result<()> {
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().remove_dir_if_empty(uri).await
        }
    }

    /// Removes a directory and all its contents. Use carefully!
    ///
    /// # Args
    /// - ***uri*** :
    /// Target directory URI.
    /// Must be **read-writable**.
    /// If not directory, an error will occur.
    ///
    /// # Support
    /// All Android versions supported by Tauri.
    #[maybe_async]
    pub fn remove_dir_all(&self, uri: &FsUri) -> Result<()> {
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().remove_dir_all(uri).await
        }
    }

    /// Build a URI of an **existing** file located at the relative path from the specified directory.
    /// Error occurs, if the file does not exist.
    ///
    /// The permissions and validity period of the returned URI depend on the origin directory
    /// (e.g., the top directory selected by [`Picker::pick_dir`])
    ///
    /// # Note
    /// For [`AndroidFs::create_new_file`] and etc, the system may sanitize path strings as needed, so those strings may not be used as it is.
    /// However, this function does not perform any sanitization, so the same ***relative_path*** may still fail.
    /// So consider using [`AndroidFs::create_new_file_and_return_relative_path`].
    ///
    /// # Args
    /// - ***uri*** :
    /// Base directory URI.
    /// Must be **readable**.
    ///
    /// - ***relative_path*** :
    /// Relative path from base directory.
    ///
    /// # Support
    /// All Android versions supported by Tauri.
    #[maybe_async]
    pub fn resolve_file_uri(
        &self,
        dir: &FsUri,
        relative_path: impl AsRef<std::path::Path>,
    ) -> Result<FsUri> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().resolve_file_uri(dir, relative_path, false).await
        }
    }

    /// Build a URI of an **existing** directory located at the relative path from the specified directory.
    /// Error occurs, if the directory does not exist.
    ///
    /// The permissions and validity period of the returned URI depend on the origin directory
    /// (e.g., the top directory selected by [`Picker::pick_dir`])
    ///
    /// # Note
    /// For [`AndroidFs::create_dir_all`] and etc, the system may sanitize path strings as needed, so those strings may not be used as it is.
    /// However, this function does not perform any sanitization, so the same ***relative_path*** may still fail.
    /// So consider using [`AndroidFs::create_dir_all_and_return_relative_path`].
    ///
    /// # Args
    /// - ***uri*** :
    /// Base directory URI.
    /// Must be **readable**.
    ///
    /// - ***relative_path*** :
    /// Relative path from base directory.
    ///
    /// # Support
    /// All Android versions supported by Tauri.
    #[maybe_async]
    pub fn resolve_dir_uri(
        &self,
        dir: &FsUri,
        relative_path: impl AsRef<std::path::Path>,
    ) -> Result<FsUri> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().resolve_dir_uri(dir, relative_path, false).await
        }
    }

    /// See [`AndroidFs::get_thumbnail`] for descriptions.
    ///
    /// If thumbnail does not wrote to dest, return false.
    #[maybe_async]
    pub fn get_thumbnail_to(
        &self,
        src: &FsUri,
        dest: &FsUri,
        preferred_size: Size,
        format: ImageFormat,
    ) -> Result<bool> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().get_file_thumbnail_to_file(src, dest, preferred_size, format).await
        }
    }

    /// Get a file thumbnail.
    /// If thumbnail does not exist it, return None.
    ///
    /// Note this does not cache. Please do it in your part if need.
    ///
    /// # Args
    /// - ***uri*** :
    /// Targe file uri.
    /// Thumbnail availability depends on the file provider.
    /// In general, images and videos are available.
    /// For file URIs via [`FsUri::from_path`],
    /// the file type must match the filename extension.
    /// In this case, the type is determined by the extension and generate thumbnails.
    /// Otherwise, thumbnails are provided through MediaStore, file provider, and etc.
    ///
    /// - ***preferred_size*** :
    /// Optimal thumbnail size desired.
    /// This may return a thumbnail of a different size,
    /// but never more than about double the requested size.
    /// In any case, the aspect ratio is maintained.
    ///
    /// - ***format*** :
    /// Thumbnail image format.
    /// If you’re not sure which one to use, [`ImageFormat::Jpeg`] is recommended.
    /// If you need transparency, use others.
    ///
    /// # Support
    /// All Android versions supported by Tauri.
    #[maybe_async]
    pub fn get_thumbnail(
        &self,
        uri: &FsUri,
        preferred_size: Size,
        format: ImageFormat,
    ) -> Result<Option<Vec<u8>>> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().get_file_thumbnail(uri, preferred_size, format).await
        }
    }

    /// Get a file thumbnail that encoded to base64 string.
    /// If thumbnail does not exist it, return None.
    ///
    /// Note this does not cache. Please do it in your part if need.
    ///
    /// # Inner
    /// This uses Kotlin's [`android.util.Base64.encodeToString(.., android.util.Base64.NO_WRAP)`](https://developer.android.com/reference/android/util/Base64#encodeToString(byte[],%20int)) internally.
    /// It is the same as [`base64::engine::general_purpose::STANDARD`](https://docs.rs/base64/0.22.1/base64/engine/general_purpose/constant.STANDARD.html) in `base64` crate.
    ///
    /// # Args
    /// - ***uri*** :
    /// Targe file uri.
    /// Thumbnail availability depends on the file provider.
    /// In general, images and videos are available.
    /// For file URIs via [`FsUri::from_path`],
    /// the file type must match the filename extension.
    /// In this case, the type is determined by the extension and generate thumbnails.
    /// Otherwise, thumbnails are provided through MediaStore, file provider, and etc.
    ///
    /// - ***preferred_size*** :
    /// Optimal thumbnail size desired.
    /// This may return a thumbnail of a different size,
    /// but never more than about double the requested size.
    /// In any case, the aspect ratio is maintained.
    ///
    /// - ***format*** :
    /// Thumbnail image format.
    /// If you’re not sure which one to use, [`ImageFormat::Jpeg`] is recommended.
    /// If you need transparency, use others.
    ///
    /// # Support
    /// All Android versions supported by Tauri.
    #[maybe_async]
    pub fn get_thumbnail_base64(
        &self,
        uri: &FsUri,
        preferred_size: Size,
        format: ImageFormat,
    ) -> Result<Option<String>> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().get_file_thumbnail_base64(uri, preferred_size, format).await
        }
    }

    /// Creates a new empty file in the specified location and returns a URI.
    ///
    /// The permissions and validity period of the returned URIs depend on the origin directory
    /// (e.g., the top directory selected by [`Picker::pick_dir`])
    ///
    /// # Args
    /// - ***dir*** :
    /// The URI of the base directory.
    /// Must be **read-write**.
    ///
    /// - ***relative_path*** :
    /// The file path relative to the base directory.
    /// Any missing parent directories will be created automatically.
    /// If a file with the same name already exists, a sequential number may be appended to ensure uniqueness.
    /// If the file has no extension, one may be inferred from ***mime_type*** and appended to the file name.
    /// Strings may also be sanitized as needed, so they may not be used exactly as provided.
    /// Note those operation may vary depending on the file provider.
    ///
    /// - ***mime_type*** :
    /// The MIME type of the file to be created.
    /// If this is None, MIME type is inferred from the extension of ***relative_path***
    /// and if that fails, `application/octet-stream` is used.
    ///
    /// # Support
    /// All Android versions supported by Tauri.
    #[maybe_async]
    pub fn create_new_file(
        &self,
        dir: &FsUri,
        relative_path: impl AsRef<std::path::Path>,
        mime_type: Option<&str>,
    ) -> Result<FsUri> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().create_new_file(dir, relative_path, mime_type).await
        }
    }

    /// Creates a new empty file in the specified location and returns a URI and relative path.
    ///
    /// The returned relative path may be sanitized and have a suffix appended to the file name,
    /// so it may differ from the input relative path.
    /// And it is a logical path within the file provider and
    /// available for [`AndroidFs::resolve_file_uri`].
    ///
    /// The permissions and validity period of the returned URIs depend on the origin directory
    /// (e.g., the top directory selected by [`Picker::pick_dir`])
    ///
    /// # Args
    /// - ***dir*** :
    /// The URI of the base directory.
    /// Must be **read-write**.
    ///
    /// - ***relative_path*** :
    /// The file path relative to the base directory.
    /// Any missing parent directories will be created automatically.
    /// If a file with the same name already exists, a sequential number may be appended to ensure uniqueness.
    /// If the file has no extension, one may be inferred from ***mime_type*** and appended to the file name.
    /// Strings may also be sanitized as needed, so they may not be used exactly as provided.
    /// Note those operation may vary depending on the file provider.
    ///
    /// - ***mime_type*** :
    /// The MIME type of the file to be created.
    /// If this is None, MIME type is inferred from the extension of ***relative_path***
    /// and if that fails, `application/octet-stream` is used.
    ///
    /// # Support
    /// All Android versions supported by Tauri.
    #[maybe_async]
    pub fn create_new_file_and_return_relative_path(
        &self,
        dir: &FsUri,
        relative_path: impl AsRef<std::path::Path>,
        mime_type: Option<&str>,
    ) -> Result<(FsUri, std::path::PathBuf)> {
        
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().create_new_file_and_return_relative_path(dir, relative_path, mime_type).await
        }
    }

    /// Creates a directory and it's parents at the specified location if they are missing,
    /// then return the URI.
    /// If it already exists, do nothing and just return the directory uri.
    ///
    /// [`AndroidFs::create_new_file`] does this automatically, so there is no need to use it together.
    ///
    /// # Args
    /// - ***dir*** :
    /// The URI of the base directory.
    /// Must be **read-write**.
    ///
    /// - ***relative_path*** :
    /// The directory path relative to the base directory.
    /// Any missing parent directories will be created automatically.
    /// Strings may also be sanitized as needed, so they may not be used exactly as provided.
    /// Note this sanitization may vary depending on the file provider.
    ///
    /// # Support
    /// All Android versions supported by Tauri.
    #[maybe_async]
    pub fn create_dir_all(
        &self,
        dir: &FsUri,
        relative_path: impl AsRef<std::path::Path>,
    ) -> Result<FsUri> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().create_dir_all(dir, relative_path).await
        }
    }

    /// Recursively create a directory and all of its parent components if they are missing,
    /// then return the URI and relative path.
    ///
    /// The returned relative path may be sanitized,
    /// so it may differ from the input relative path.
    /// And it is a logical path within the file provider and
    /// available for [`AndroidFs::resolve_dir_uri`].
    ///
    /// [`AndroidFs::create_new_file`] does this automatically, so there is no need to use it together.
    ///
    /// # Args
    /// - ***dir*** :
    /// The URI of the base directory.
    /// Must be **read-write**.
    ///
    /// - ***relative_path*** :
    /// The directory path relative to the base directory.
    /// Any missing parent directories will be created automatically.
    /// Strings may also be sanitized as needed, so they may not be used exactly as provided.
    /// Note this sanitization may vary depending on the file provider.
    ///
    /// # Support
    /// All Android versions supported by Tauri.
    #[maybe_async]
    pub fn create_dir_all_and_return_relative_path(
        &self,
        dir: &FsUri,
        relative_path: impl AsRef<std::path::Path>,
    ) -> Result<(FsUri, std::path::PathBuf)> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().create_dir_all_and_return_relative_path(dir, relative_path).await
        }
    }

    /// # Support
    /// All Android versions supported by Tauri.
    #[maybe_async]
    pub fn create_new_dir(
        &self,
        dir: &FsUri,
        relative_path: impl AsRef<std::path::Path>,
    ) -> Result<FsUri> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().create_new_dir(dir, relative_path).await
        }
    }

    /// # Support
    /// All Android versions supported by Tauri.
    #[maybe_async]
    pub fn create_new_dir_and_return_relative_path(
        &self,
        dir: &FsUri,
        relative_path: impl AsRef<std::path::Path>,
    ) -> Result<(FsUri, std::path::PathBuf)> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().create_new_dir_and_return_relative_path(dir, relative_path).await
        }
    }

    /// Returns the child files and directories of the specified directory.
    /// The order of the entries depends on the file provider.
    ///
    /// The permissions and validity period of the returned URIs depend on the origin directory
    /// (e.g., the top directory selected by [`Picker::pick_dir`])
    ///
    /// # Args
    /// - ***uri*** :
    /// Target directory URI.
    /// Must be **readable**.
    ///
    /// # Support
    /// All Android versions supported by Tauri.
    #[maybe_async]
    pub fn read_dir(&self, uri: &FsUri) -> Result<Vec<Entry>> {
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls()
                .read_dir(uri, EntryOptions::ALL, ..).await?
                .map(Entry::try_from)
                .collect::<Result<_>>()
        }
    }

    /// Returns the child files and directories of the specified directory.
    /// The order of the entries depends on the file provider.
    ///
    /// The permissions and validity period of the returned URIs depend on the origin directory
    /// (e.g., the top directory selected by [`Picker::pick_dir`])
    ///
    /// # Args
    /// - ***uri*** :
    /// Target directory URI.
    /// Must be **readable**.
    ///
    /// # Support
    /// All Android versions supported by Tauri.
    #[maybe_async]
    pub fn read_dir_with_range(
        &self,
        uri: &FsUri,
        range: impl std::ops::RangeBounds<u64>,
    ) -> Result<Vec<Entry>> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls()
                .read_dir(uri, EntryOptions::ALL, range).await?
                .map(Entry::try_from)
                .collect::<Result<_>>()
        }
    }

    /// Returns the child files and directories of the specified directory.
    /// The order of the entries depends on the file provider.
    ///
    /// The permissions and validity period of the returned URIs depend on the origin directory
    /// (e.g., the top directory selected by [`Picker::pick_dir`])
    ///
    /// # Args
    /// - ***uri*** :
    /// Target directory URI.
    /// Must be **readable**.
    ///
    /// # Support
    /// All Android versions supported by Tauri.
    #[maybe_async]
    pub fn read_dir_with_options(
        &self,
        uri: &FsUri,
        options: EntryOptions,
    ) -> Result<Vec<OptionalEntry>> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls()
                .read_dir(uri, options, ..).await
                .map(|i| i.collect())
        }
    }

    /// Returns the child files and directories of the specified directory.
    /// The order of the entries depends on the file provider.
    ///
    /// The permissions and validity period of the returned URIs depend on the origin directory
    /// (e.g., the top directory selected by [`Picker::pick_dir`])
    ///
    /// # Args
    /// - ***uri*** :
    /// Target directory URI.
    /// Must be **readable**.
    ///
    /// # Support
    /// All Android versions supported by Tauri.
    #[maybe_async]
    pub fn read_dir_with_options_and_range(
        &self,
        uri: &FsUri,
        options: EntryOptions,
        range: impl std::ops::RangeBounds<u64>,
    ) -> Result<Vec<OptionalEntry>> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls()
                .read_dir(uri, options, range).await
                .map(|i| i.collect())
        }
    }

    /// Constructs a URI from a provider name and a logical path that uniquely identifies an arbitrary custom file within the provider.
    ///
    /// The returned URI remains valid until the current application process terminates,
    /// after which it can no longer be accessed.
    /// The URI is an Android Content URI
    /// and can be shared with other applications using [`Opener`], for example.
    ///
    /// # Args
    /// - **provider_name** :
    /// The identifier of the custom file provider registered by [`Builder::register_custom_file_provider`].
    /// If it does not exist, this function returns an error.
    ///
    /// - **path** :
    /// The path passed to the [`CustomFileProvider`] specified
    /// by [`Builder::register_custom_file_provider`] to identify the requested custom file.
    /// The path can be any string in any format,
    /// but excessively long strings may cause issues when the resulting URI is handled by Android.
    /// The path will be percent-encoded and then included as part of the URI.
    /// Therefore, do not include any information that should not be disclosed if the URI may be shared with other applications or otherwise exposed.
    ///
    /// # Support
    /// All Android versions supported by Tauri.
    #[always_sync]
    pub fn build_custom_file_uri(
        &self,
        provider_name: impl AsRef<str>,
        path: impl AsRef<str>,
    ) -> Result<FsUri> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().build_custom_file_uri(provider_name, path)
        }
    }

    /// Registers a readonly custom file and returns a URI for accessing it.
    ///
    /// **Note**: Enable `custom_file_provider` feature to use.
    /// 
    /// The registered custom file remains valid until the current application process terminates or
    /// [`AndroidFs::unregister_simple_custom_file`] is called. 
    /// Call [`AndroidFs::unregister_simple_custom_file`] to release the associated resources
    /// when the custom file is no longer needed.
    /// 
    /// This is a simplified version of [Builder::register_custom_file_provider]
    /// and [AndroidFs::build_custom_file_uri].
    /// See those methods for details.
    /// 
    /// # Args
    /// - **name** :
    /// Name of the custom file.
    /// This can be duplicated. 
    /// Even if a simple custom file with the same name already exists, 
    /// they are treated as separate custom files.
    ///
    /// # Support
    /// This API is available on all Android versions supported by Tauri. 
    /// However, some [`CustomFile`] variants are only available on Android 8.0 (API level 26) or higher. 
    /// See the documentation for `CustomFile::from_*` for details.
    #[always_sync]
    pub fn register_simple_readonly_custom_file(
        &self,
        name: String,
        mime_type: Option<String>,
        len: Option<u64>,
        last_modified: Option<std::time::SystemTime>,
        open_custom_file: impl Fn() -> std::io::Result<CustomFile> + Send + Sync + 'static,
    ) -> Result<FsUri> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            let simple_custom_files: SimpleCustomFilesState<'_, R> = self.handle.app().state();
            simple_custom_files.add(name, mime_type, len, last_modified, open_custom_file)
        }
    }

    /// Unregisters a custom file previously registered with
    /// [`AndroidFs::register_simple_readonly_custom_file`].
    ///
    /// After this method returns, the URI can no longer be used to access the custom file.
    /// Any resources associated with the registered custom file are released.
    ///
    /// # Support
    /// All Android versions supported by Tauri.
    #[always_sync]
    pub fn unregister_simple_custom_file(&self, uri: &FsUri) -> Result<()> {
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            let Ok(id) = take_simple_custom_file_id(uri) else {
                return Ok(())
            };

            let simple_custom_files: SimpleCustomFilesState<'_, R> = self.handle.app().state();
            simple_custom_files.close(id);
            Ok(())
        }
    }

    /// Constructs a URI for a file at the specified absolute path.
    ///
    /// The returned URI remains valid until the current application process terminates,
    /// after which it can no longer be accessed.
    /// The URI is an Android Content URI
    /// and can be shared with other applications using [`Opener`], for example.
    ///
    /// # Setup
    /// This function only constructs a URI.
    /// To actually access the file using the returned URI, follow the steps below.
    ///
    /// ### 1. Enable file provider feature
    /// Enable `file_provider` feature.
    ///
    /// ### 2. Configuration
    /// Set the configuration to allow files to be loaded,
    /// and configure the scope.
    ///
    /// `src-tauri/tauri.conf.json`
    /// ```json
    /// {
    ///   "plugins": {
    ///     "android-fs": {
    ///       "fileProvider": {
    ///         "enable": true,
    ///         "scope": {
    ///           "allow": ["$APPDATA/my-data/shared/**/*"],
    ///         }
    ///       }
    ///     }
    ///   }
    /// }
    /// ```
    ///
    /// NOTE:
    /// Ensure that `serde_json` is present in your Rust dependencies.
    /// It is included by default in Tauri project templates, but if it has been removed, add it back.
    /// If it is missing, the project will fail to build.
    ///
    /// # Support
    /// All Android versions supported by Tauri.
    #[always_sync]
    pub fn get_uri_for_file_path(
        &self,
        path: impl AsRef<std::path::Path>,
        mime_type: Option<&str>,
    ) -> Result<FsUri> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().get_uri_for_file_path(path, mime_type)
        }
    }

    /// See [`AppStorage::get_volumes`] or [`PublicStorage::get_volumes`] for details.
    ///
    /// The difference is that this does not perform any filtering.
    /// You can it by [`StorageVolume { is_available_for_app_storage, is_available_for_public_storage, .. } `](StorageVolume).
    #[maybe_async]
    pub fn get_volumes(&self) -> Result<Vec<StorageVolume>> {
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().get_available_storage_volumes().await
        }
    }

    /// See [`AppStorage::get_primary_volume`] or [`PublicStorage::get_primary_volume`] for details.
    ///
    /// The difference is that this does not perform any filtering.
    /// You can it by [`StorageVolume { is_available_for_app_storage, is_available_for_public_storage, .. } `](StorageVolume).
    #[maybe_async]
    pub fn get_primary_volume(&self) -> Result<Option<StorageVolume>> {
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().get_primary_storage_volume_if_available().await
        }
    }

    /// Get a MIME type from the extension.
    ///
    /// # Support
    /// All Android versions supported by Tauri.
    #[maybe_async]
    pub fn get_mime_type_from_extension(&self, ext: impl AsRef<str>) -> Result<Option<String>> {
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().get_mime_type_from_extension(ext).await
        }
    }

    /// Verify whether this plugin is available.
    ///
    /// On Android, this returns true.
    /// On other platforms, this returns false.
    #[always_sync]
    pub fn is_available(&self) -> bool {
        cfg!(target_os = "android")
    }

    /// Get the api level of this Android device.
    ///
    /// The correspondence table between API levels and Android versions can be found following.
    /// <https://developer.android.com/guide/topics/manifest/uses-sdk-element#api-level-table>
    ///
    /// If you want the constant value of the API level from an Android version, there is the [`api_level`] module.
    ///
    /// # Table
    /// | Android version  | API Level |
    /// |------------------|-----------|
    /// | 16.0             | 36        |
    /// | 15.0             | 35        |
    /// | 14.0             | 34        |
    /// | 13.0             | 33        |
    /// | 12L              | 32        |
    /// | 12.0             | 31        |
    /// | 11.0             | 30        |
    /// | 10.0             | 29        |
    /// | 9.0              | 28        |
    /// | 8.1              | 27        |
    /// | 8.0              | 26        |
    /// | 7.1 - 7.1.2      | 25        |
    /// | 7.0              | 24        |
    ///
    /// Tauri does not support Android versions below 7.
    #[always_sync]
    pub fn api_level(&self) -> Result<i32> {
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().api_level()
        }
    }

    /// Gets the application id of this Android app.
    #[always_sync]
    pub fn app_id(&self) -> Result<&'static str> {
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            Ok(&self.impls().consts()?.app_id)
        }
    }

    /// See [`AndroidFs::resolve_file_uri`] for details.
    ///
    /// The difference is that this may skip checking whether the target exists and is a file.
    /// As a result, in many cases it avoids the delay (from a few to several tens of milliseconds) caused by calling a Kotlin-side function.
    ///
    /// Note that, depending on the situation,
    /// the Kotlin-side function may be called or a check may be performed,
    /// which could result in an error or a delay.
    #[maybe_async]
    pub fn _resolve_file_uri(
        &self,
        dir: &FsUri,
        relative_path: impl AsRef<std::path::Path>,
    ) -> Result<FsUri> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().resolve_file_uri(dir, relative_path, true).await
        }
    }

    /// See [`AndroidFs::resolve_dir_uri`] for details.
    ///
    /// The difference is that this may skip checking whether the target exists and is a directory.
    /// As a result, in many cases it avoids the delay (from a few to several tens of milliseconds) caused by calling a Kotlin-side function.
    ///
    /// Note that, depending on the situation,
    /// the Kotlin-side function may be called or a check may be performed,
    /// which could result in an error or a delay.
    #[maybe_async]
    pub fn _resolve_dir_uri(
        &self,
        dir: &FsUri,
        relative_path: impl AsRef<std::path::Path>,
    ) -> Result<FsUri> {
        
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().resolve_dir_uri(dir, relative_path, true).await
        }
    }
}