use sync_async::sync_async;
use crate::*;
use super::*;


/// ***Root API***  
/// 
/// # Examples
/// ```no_run
/// fn example(app: &tauri::AppHandle) {
///     use tauri_plugin_android_fs::AndroidFsExt as _;
/// 
///     let api = app.android_fs();
/// }
/// ```

#[sync_async]
pub struct AndroidFs<R: tauri::Runtime> {
    #[cfg(target_os = "android")]
    pub(crate) handle: tauri::plugin::PluginHandle<R>,

    #[cfg(not(target_os = "android"))]
    #[allow(unused)]
    pub(crate) handle: std::marker::PhantomData<fn() -> R>
}

#[cfg(target_os = "android")]
#[sync_async(
    use(if_sync) impls::SyncImpls as Impls;
    use(if_async) impls::AsyncImpls as Impls;
)]
impl<R: tauri::Runtime> AndroidFs<R> {
    
    #[always_sync]
    pub(crate) fn impls(&self) -> Impls<'_, R> {
        Impls { handle: &self.handle }
    }
}

#[sync_async(
    use(if_async) api_async::{FileOpener, FilePicker, AppStorage, PrivateStorage, PublicStorage};
    use(if_sync) api_sync::{FileOpener, FilePicker, AppStorage, PrivateStorage, PublicStorage};
)]
impl<R: tauri::Runtime> AndroidFs<R> {

    /// API of file storage that is available to other applications and users.
    #[always_sync]
    pub fn public_storage(&self) -> PublicStorage<'_, R> {
        PublicStorage { handle: &self.handle }
    }

    /// API of file storage intended for the app's use only.
    #[always_sync]
    pub fn private_storage(&self) -> PrivateStorage<'_, R> {
        PrivateStorage { handle: &self.handle }
    }

    /// API of file storage intended for the app's use.  
    #[always_sync]
    pub fn app_storage(&self) -> AppStorage<'_, R> {
        AppStorage { handle: &self.handle }
    }

    /// API of file/dir picker.
    #[always_sync]
    pub fn file_picker(&self) -> FilePicker<'_, R> {
        FilePicker { handle: &self.handle }
    }

    /// API of opening file/dir with other apps.
    #[always_sync]
    pub fn file_opener(&self) -> FileOpener<'_, R> {
        FileOpener { handle: &self.handle }
    }

    /// Get the file or directory name.  
    /// 
    /// # Args
    /// - ***uri*** :  
    /// Target URI.  
    /// Must be **readable**.
    /// 
    /// # Support
    /// All Android version.
    #[maybe_async]
    pub fn get_name(&self, uri: &FileUri) -> Result<String> {
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().get_entry_name(uri).await
        }
    }

    /// Queries the provider to get the MIME type.
    ///
    /// For file URIs via [`FileUri::from_path`], the MIME type is determined from the file extension.  
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
    /// All Android version.
    #[maybe_async]
    pub fn get_mime_type(&self, uri: &FileUri) -> Result<String> {
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
    /// For file URIs via [`FileUri::from_path`], the MIME type is determined from the file extension.  
    /// In most other cases, it uses the MIME type that was associated with the file when it was created.  
    /// If the MIME type is unknown or unset, it falls back to `"application/octet-stream"`.  
    /// 
    /// # Args
    /// - ***uri*** :  
    /// Target URI.  
    /// Must be **readable**.
    /// 
    /// # Support
    /// All Android version.
    #[maybe_async]
    pub fn get_type(&self, uri: &FileUri) -> Result<EntryType> {
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().get_entry_type(uri).await
        }
    }

    /// Gets the entry information.
    ///
    /// # Args
    /// - ***uri*** :  
    /// Target URI.  
    /// Must be **readable**.
    /// 
    /// # Support
    /// All Android version.
    #[maybe_async]
    pub fn get_info(&self, uri: &FileUri) -> Result<Entry> {
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
    /// All Android version.
    #[maybe_async]
    pub fn get_len(&self, uri: &FileUri) -> Result<u64> {
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().get_file_len(uri).await
        }
    }

    /// Queries the file system to get information about a file, directory.
    /// 
    /// # Args
    /// - ***uri*** :  
    /// Target URI.  
    /// Must be **readable**.
    /// 
    /// # Note
    /// This uses [`AndroidFs::open_file`] internally.
    /// 
    /// # Support
    /// All Android version.
    #[maybe_async]
    pub fn get_metadata(&self, uri: &FileUri) -> Result<std::fs::Metadata> {
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().get_entry_metadata(uri).await
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
    /// All Android version.
    #[maybe_async]
    pub fn open_file_readable(&self, uri: &FileUri) -> Result<std::fs::File> {
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
    /// All Android version.
    #[maybe_async]
    pub fn open_file_writable(
        &self, 
        uri: &FileUri, 
    ) -> Result<std::fs::File> {

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
    /// All Android version.
    #[maybe_async]
    pub fn open_file(&self, uri: &FileUri, mode: FileAccessMode) -> Result<std::fs::File> {
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
        uri: &FileUri, 
        candidate_modes: impl IntoIterator<Item = FileAccessMode>
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
    /// All Android version.
    #[maybe_async]
    pub fn read(&self, uri: &FileUri) -> Result<Vec<u8>> {
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
    /// All Android version.
    #[maybe_async]
    pub fn read_to_string(&self, uri: &FileUri) -> Result<String> {
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
    /// All Android version.
    #[maybe_async]
    pub fn write(&self, uri: &FileUri, contents: impl AsRef<[u8]>) -> Result<()> {
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
    /// All Android version.
    #[maybe_async]
    pub fn copy(&self, src: &FileUri, dest: &FileUri) -> Result<()> {
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
    /// The behaviour in the same name already exists depends on the file provider.  
    /// In the case of e.g. [`PublicStorage`], the suffix (e.g. `(1)`) is added to this name.  
    /// In the case of files hosted by other applications, errors may occur.  
    /// But at least, the existing file will not be overwritten.  
    /// The system may sanitize these strings as needed, so those strings may not be used as it is.
    /// 
    /// # Support
    /// All Android version.
    #[maybe_async]
    pub fn rename(&self, uri: &FileUri, new_name: impl AsRef<str>) -> Result<FileUri> {
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
    /// All Android version.
    #[maybe_async]
    pub fn remove_file(&self, uri: &FileUri) -> Result<()> {
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
    /// All Android version.
    #[maybe_async]
    pub fn remove_dir(&self, uri: &FileUri) -> Result<()> {
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
    /// All Android version.
    #[maybe_async]
    pub fn remove_dir_all(&self, uri: &FileUri) -> Result<()> {
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
    /// (e.g., the top directory selected by [`FilePicker::pick_dir`]) 
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
    /// All Android version.
    #[maybe_async]
    pub fn resolve_file_uri(
        &self, 
        dir: &FileUri, 
        relative_path: impl AsRef<std::path::Path>
    ) -> Result<FileUri> {

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
    /// (e.g., the top directory selected by [`FilePicker::pick_dir`]) 
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
    /// All Android version.
    #[maybe_async]
    pub fn resolve_dir_uri(
        &self,
        dir: &FileUri, 
        relative_path: impl AsRef<std::path::Path>
    ) -> Result<FileUri> {

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
        src: &FileUri,
        dest: &FileUri,
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
    /// Thumbnail availablty depends on the file provider.  
    /// In general, images and videos are available.  
    /// For file URIs via [`FileUri::from_path`], 
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
    /// All Android version.
    #[maybe_async]
    pub fn get_thumbnail(
        &self,
        uri: &FileUri,
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
    /// Thumbnail availablty depends on the file provider.  
    /// In general, images and videos are available.  
    /// For file URIs via [`FileUri::from_path`], 
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
    /// All Android version.
    #[maybe_async]
    pub fn get_thumbnail_base64(
        &self,
        uri: &FileUri,
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
    /// (e.g., the top directory selected by [`FilePicker::pick_dir`]) 
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
    /// All Android version.
    #[maybe_async]
    pub fn create_new_file(
        &self,
        dir: &FileUri, 
        relative_path: impl AsRef<std::path::Path>, 
        mime_type: Option<&str>
    ) -> Result<FileUri> {

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
    /// (e.g., the top directory selected by [`FilePicker::pick_dir`]) 
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
    /// All Android version.
    #[maybe_async]
    pub fn create_new_file_and_return_relative_path(
        &self,
        dir: &FileUri, 
        relative_path: impl AsRef<std::path::Path>, 
        mime_type: Option<&str>
    ) -> Result<(FileUri, std::path::PathBuf)> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().create_new_file_and_retrun_relative_path(dir, relative_path, mime_type).await
        }
    }

    /// Creates a directory and it's parents at the specified location if they are missing,
    /// then return the URI.  
    /// If it already exists, do nothing and just return the direcotry uri.
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
    /// All Android version.
    #[maybe_async]
    pub fn create_dir_all(
        &self,
        dir: &FileUri, 
        relative_path: impl AsRef<std::path::Path>, 
    ) -> Result<FileUri> {

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
    /// All Android version.
    #[maybe_async]
    pub fn create_dir_all_and_return_relative_path(
        &self,
        dir: &FileUri, 
        relative_path: impl AsRef<std::path::Path>, 
    ) -> Result<(FileUri, std::path::PathBuf)> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().create_dir_all_and_return_relative_path(dir, relative_path).await
        }
    }

    /// Returns the child files and directories of the specified directory.  
    /// The order of the entries is not guaranteed.  
    /// 
    /// The permissions and validity period of the returned URIs depend on the origin directory 
    /// (e.g., the top directory selected by [`FilePicker::pick_dir`])  
    /// 
    /// This retrieves all metadata including `uri`, `name`, `last_modified`, `len`, and `mime_type`. 
    /// If only specific information is needed, 
    /// using [`AndroidFs::read_dir_with_options`] will improve performance.
    /// 
    /// # Note
    /// The returned type is an iterator, but the file system call is not executed lazily.  
    /// Instead, all data is retrieved at once.  
    /// For directories containing thousands or even tens of thousands of entries,  
    /// this function may take several seconds to complete.  
    /// The returned iterator itself is low-cost, as it only performs lightweight data formatting.
    /// 
    /// # Args
    /// - ***uri*** :  
    /// Target directory URI.  
    /// Must be **readable**.
    /// 
    /// # Support
    /// All Android version.
    #[maybe_async]
    pub fn read_dir(&self, uri: &FileUri) -> Result<impl Iterator<Item = Entry>> {
        let entries = self.read_dir_with_options(uri, EntryOptions::ALL).await?
            .map(Entry::try_from)
            .filter_map(Result::ok);
        
        Ok(entries)
    }

    /// Returns the child files and directories of the specified directory.  
    /// The order of the entries is not guaranteed.  
    /// 
    /// The permissions and validity period of the returned URIs depend on the origin directory 
    /// (e.g., the top directory selected by [`FilePicker::pick_dir`])  
    /// 
    /// # Note
    /// The returned type is an iterator, but the file system call is not executed lazily.  
    /// Instead, all data is retrieved at once.  
    /// For directories containing thousands or even tens of thousands of entries,  
    /// this function may take several seconds to complete.  
    /// The returned iterator itself is low-cost, as it only performs lightweight data formatting.
    /// 
    /// # Args
    /// - ***uri*** :  
    /// Target directory URI.  
    /// Must be **readable**.
    /// 
    /// # Support
    /// All Android version.
    #[maybe_async]
    pub fn read_dir_with_options(
        &self, 
        uri: &FileUri, 
        options: EntryOptions
    ) -> Result<impl Iterator<Item = OptionalEntry>> {
        
        #[cfg(not(target_os = "android"))] {
            Err::<std::iter::Empty<_>, _>(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().read_dir_with_options(uri, options).await
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

    /// Builds the storage volume root URI.  
    /// 
    /// This should only be used as `initial_location` in the file picker, such as [`FilePicker::pick_files`]. 
    /// It must not be used for any other purpose.  
    /// 
    /// This is useful when selecting save location, 
    /// but when selecting existing entries, `initial_location` is often better with None.
    /// 
    /// # Args  
    /// - ***volume_id*** :  
    /// ID of the storage volume, such as internal storage, SD card, etc.  
    /// If `None` is provided, [`the primary storage volume`](AndroidFs::get_primary_volume) will be used.  
    /// 
    /// # Support
    /// All Android version.
    #[maybe_async]
    pub fn resolve_root_initial_location(&self, volume_id: Option<&StorageVolumeId>) -> Result<FileUri> {
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().resolve_root_initial_location(volume_id).await
        }
    }

    /// Get a MIME type from the extension.
    /// 
    /// # Support
    /// All Android version.
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
        dir: &FileUri, 
        relative_path: impl AsRef<std::path::Path>
    ) -> Result<FileUri> {

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
        dir: &FileUri, 
        relative_path: impl AsRef<std::path::Path>
    ) -> Result<FileUri> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().resolve_dir_uri(dir, relative_path, true).await
        }
    }



    
    #[deprecated = "use `FilePicker::persist_uri_permission` instead"]
    #[maybe_async]
    pub fn take_persistable_uri_permission(&self, uri: &FileUri) -> Result<()> {
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().persist_picker_uri_permission(uri).await
        }
    }

    #[deprecated = "use `FilePicker::check_persisted_uri_permission` instead"]
    #[maybe_async]
    pub fn check_persisted_uri_permission(
        &self, 
        uri: &FileUri, 
        permission: UriPermission
    ) -> Result<bool> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().check_persisted_picker_uri_permission(uri, permission).await
        }
    }

    #[deprecated = "use `FilePicker::get_all_persisted_uri_permissions` instead"]
    #[maybe_async]
    pub fn get_all_persisted_uri_permissions(&self) -> Result<impl Iterator<Item = PersistedUriPermissionState>> {
        #[cfg(not(target_os = "android"))] {
            Err::<std::iter::Empty<_>, _>(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().get_all_persisted_picker_uri_permissions().await
        }
    }

    #[deprecated = "use `FilePicker::release_persisted_uri_permission` instead"]
    #[maybe_async]
    pub fn release_persisted_uri_permission(&self, uri: &FileUri) -> Result<()> {
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().release_persisted_picker_uri_permission(uri).await?;
            Ok(())
        }
    }

    #[deprecated = "use `FilePicker::release_all_persisted_uri_permissions` instead"]
    #[maybe_async]
    pub fn release_all_persisted_uri_permissions(&self) -> Result<()> {
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().release_all_persisted_picker_uri_permissions().await
        }
    }
}