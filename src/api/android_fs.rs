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
    use(if_async) api_async::{FileOpener, FilePicker, PrivateStorage, PublicStorage, WritableStream};
    use(if_sync) api_sync::{FileOpener, FilePicker, PrivateStorage, PublicStorage, WritableStream};
)]
impl<R: tauri::Runtime> AndroidFs<R> {

    /// API of file storage intended for the app's use only.
    #[always_sync]
    pub fn private_storage(&self) -> PrivateStorage<'_, R> {
        PrivateStorage { handle: &self.handle }
    }

    /// API of file storage that is available to other applications and users.
    #[always_sync]
    pub fn public_storage(&self) -> PublicStorage<'_, R> {
        PublicStorage { handle: &self.handle }
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
    /// # Note
    /// For file provider of some cloud storage, 
    /// writing by file descriptor like std::fs may not correctoly notify and reflect changes. 
    /// If you need to write to such files, use [`AndroidFs::open_writable_stream`].
    /// It will fall back to Kotlin API as needed.
    /// And you can check by [`AndroidFs::need_write_via_kotlin`].
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
    /// If the target is a file on cloud storage or otherwise not physically present on the device,
    /// the file provider may downloads the entire contents, and then opens it. 
    /// As a result, this processing may take longer than with regular local files.
    /// And files might be a pair of pipe or socket for streaming data.
    /// 
    /// When writing to a file with this function,
    /// pay attention to the following points:
    /// 
    /// 1. **File reflection**:  
    /// For file provider of some cloud storage, 
    /// writing by file descriptor like std::fs may not correctoly notify and reflect changes. 
    /// If you need to write to such files, use [`AndroidFs::open_writable_stream`].
    /// It will fall back to Kotlin API as needed.
    /// And you can check by [`AndroidFs::need_write_via_kotlin`].
    /// 
    /// 2. **File mode restrictions**:  
    /// Files provided by third-party apps may not support writable modes other than
    /// [`FileAccessMode::Write`]. However, [`FileAccessMode::Write`] does not guarantee
    /// that existing contents will always be truncated.  
    /// As a result, if the new contents are shorter than the original, the file may
    /// become corrupted. To avoid this, consider using
    /// [`AndroidFs::open_file_writable`] or [`AndroidFs::open_writable_stream`], which
    /// ensure that existing contents are truncated and also automatically apply the
    /// maximum possible fallbacks.  
    /// <https://issuetracker.google.com/issues/180526528>
    /// 
    /// # Args
    /// - ***uri*** :  
    /// Target file URI.  
    /// This must have corresponding permissions (read, write, or both) for the specified ***mode***.
    /// 
    /// - ***mode*** :  
    /// Indicates how the file is opened and the permissions granted. 
    /// The only ones that can be expected to work in all cases are [`FileAccessMode::Write`] and [`FileAccessMode::Read`].
    /// Because files hosted by third-party apps may not support others.
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

    /// Opens a stream for writing to the specified file.  
    /// This truncates the existing contents.  
    /// 
    /// # Usage
    /// [`WritableStream`] implements [`std::io::Write`], so it can be used for writing.  
    /// As with [`std::fs::File`], wrap it with [`std::io::BufWriter`] if buffering is needed.  
    ///
    /// After writing, call [`WritableStream::reflect`].  
    /// 
    /// # Note
    /// The behavior depends on [`AndroidFs::need_write_via_kotlin`].  
    /// If it is `false`, this behaves like [`AndroidFs::open_file_writable`].  
    /// If it is `true`, this behaves like [`AndroidFs::open_writable_stream_via_kotlin`].  
    /// 
    /// # Args
    /// - ***uri*** :  
    /// Target file URI.  
    /// This need to be **writable**.
    /// 
    /// # Support
    /// All Android version.
    #[maybe_async]
    pub fn open_writable_stream(
        &self,
        uri: &FileUri
    ) -> Result<WritableStream<R>> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            let impls = self.impls().create_writable_stream_auto(uri).await?;
            Ok(WritableStream { impls })
        }
    }

    /// Opens a writable stream to the specified file.  
    /// This truncates the existing contents.  
    /// 
    /// This function always writes content via the Kotlin API.
    /// But this takes several times longer compared.  
    /// [`AndroidFs::open_writable_stream`] automatically falls back to this function depending on [`AndroidFs::need_write_via_kotlin`].  
    /// 
    /// # Usage
    /// [`WritableStream`] implements [`std::io::Write`], so it can be used for writing.  
    /// As with [`std::fs::File`], wrap it with [`std::io::BufWriter`] if buffering is needed.  
    ///
    /// After writing, call [`WritableStream::reflect`].
    /// 
    /// # Args
    /// - ***uri*** :  
    /// Target file URI.  
    /// This need to be **writable**.
    /// 
    /// # Support
    /// All Android version.
    #[maybe_async]
    pub fn open_writable_stream_via_kotlin(
        &self,
        uri: &FileUri
    ) -> Result<WritableStream<R>> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            let impls = self.impls().create_writable_stream_via_kotlin(uri).await?;
            Ok(WritableStream { impls })
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
    /// # Note
    /// The behavior depends on [`AndroidFs::need_write_via_kotlin`].  
    /// If it is `false`, this uses [`std::fs::File`].  
    /// If it is `true`, this uses [`AndroidFs::write_via_kotlin`].  
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
            self.impls().write_file_auto(uri, contents).await
        }
    }

    /// Writes a slice as the entire contents of a file.  
    /// This function will entirely replace its contents if it does exist.    
    /// 
    /// This function always writes content via the Kotlin API.
    /// But this takes several times longer compared.   
    /// [`AndroidFs::write`] automatically falls back to this function depending on [`AndroidFs::need_write_via_kotlin`].  
    /// 
    /// # Support
    /// All Android version.
    #[maybe_async]
    pub fn write_via_kotlin(
        &self, 
        uri: &FileUri,
        contents: impl AsRef<[u8]>
    ) -> Result<()> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().write_file_via_kotlin(uri, contents).await
        }
    }

    /// Copies the contents of the source file to the destination.  
    /// If the destination already has contents, they are truncated before writing the source contents.  
    /// 
    /// # Note
    /// The behavior depends on [`AndroidFs::need_write_via_kotlin`].  
    /// If it is `false`, this uses [`std::io::copy`] with [`std::fs::File`].  
    /// If it is `true`, this uses [`AndroidFs::copy_via_kotlin`].  
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

    /// Copies the contents of src file to dest.  
    /// If dest already has contents, it is truncated before write src contents.  
    /// 
    /// This function always writes content via the Kotlin API.  
    /// [`AndroidFs::copy`] automatically falls back to this function depending on [`AndroidFs::need_write_via_kotlin`].   
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
    /// - ***buffer_size***:  
    /// The size of the buffer, in bytes, to use during the copy process on Kotlin.  
    /// If `None`, [`DEFAULT_BUFFER_SIZE`](https://kotlinlang.org/api/core/kotlin-stdlib/kotlin.io/-d-e-f-a-u-l-t_-b-u-f-f-e-r_-s-i-z-e.html) is used. 
    /// At least, when I checked, it was 8 KB.  
    /// If zero, this causes error.
    /// 
    /// # Support
    /// All Android version.
    #[maybe_async]
    pub fn copy_via_kotlin(
        &self, 
        src: &FileUri, 
        dest: &FileUri,
        buffer_size: Option<u32>,
    ) -> Result<()> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().copy_file_via_kotlin(src, dest, buffer_size).await
        }
    }

    /// Determines whether the file must be written via the Kotlin API rather than through a file descriptor.
    /// 
    /// In the case of a file that physically exists on the device, this will always return false.
    /// This is intended for special cases, such as some cloud storage.
    /// 
    /// # Support
    /// All Android version.
    #[maybe_async]
    pub fn need_write_via_kotlin(&self, uri: &FileUri) -> Result<bool> {
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().need_write_file_via_kotlin(uri).await
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
            self.impls().resolve_file_uri(dir, relative_path).await
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
            self.impls().resolve_dir_uri(dir, relative_path).await
        }
    }

    /// See [`AndroidFs::get_thumbnail_to`] for descriptions.  
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
    /// but never more than double the requested size. 
    /// In any case, the aspect ratio is maintained.
    /// 
    /// - ***format*** :  
    /// Thumbnail image format.  
    /// [`ImageFormat::Jpeg`] is recommended. 
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
            self.impls().get_file_thumbnail_in_memory(uri, preferred_size, format).await
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

    /// Recursively create a directory and all of its parent components if they are missing,
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

    /// Take persistent permission to access the file, directory and its descendants.  
    /// This is a prolongation of an already acquired permission, not the acquisition of a new one.  
    /// 
    /// This works by just calling, without displaying any confirmation to the user.
    /// 
    /// Note that [there is a limit to the total number of URI that can be made persistent by this function.](https://stackoverflow.com/questions/71099575/should-i-release-persistableuripermission-when-a-new-storage-location-is-chosen/71100621#71100621)  
    /// Therefore, it is recommended to relinquish the unnecessary persisted URI by [`AndroidFs::release_persisted_uri_permission`] or [`AndroidFs::release_all_persisted_uri_permissions`].  
    /// Persisted permissions may be relinquished by other apps, user, or by moving/removing entries.
    /// So check by [`AndroidFs::check_persisted_uri_permission`].  
    /// And you can retrieve the list of persisted uris using [`AndroidFs::get_all_persisted_uri_permissions`].
    /// 
    /// # Args
    /// - **uri** :  
    /// URI of the target file or directory.   
    /// This must be a URI taken from following :  
    ///     - [`FilePicker::pick_files`]  
    ///     - [`FilePicker::pick_file`]  
    ///     - [`FilePicker::pick_visual_medias`]  
    ///     - [`FilePicker::pick_visual_media`]  
    ///     - [`FilePicker::pick_dir`]  
    ///     - [`FilePicker::save_file`]  
    ///     - [`AndroidFs::resolve_file_uri`], [`AndroidFs::resolve_dir_uri`], [`AndroidFs::read_dir`], [`AndroidFs::create_new_file`], [`AndroidFs::create_dir_all`] :  
    ///     If use URI from thoese fucntions, the permissions of the origin directory URI is persisted, not a entry iteself by this function. 
    ///     Because the permissions and validity period of the descendant entry URIs depend on the origin directory.   
    /// 
    /// # Support
    /// All Android version. 
    #[maybe_async]
    pub fn take_persistable_uri_permission(&self, uri: &FileUri) -> Result<()> {
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().take_persistable_uri_permission(uri).await
        }
    }

    /// Check a persisted URI permission grant by [`AndroidFs::take_persistable_uri_permission`].  
    /// Returns false if there are only non-persistent permissions or no permissions.
    /// 
    /// # Args
    /// - **uri** :  
    /// URI of the target file or directory.  
    /// This must be a URI taken from following :  
    ///     - [`FilePicker::pick_files`]  
    ///     - [`FilePicker::pick_file`]  
    ///     - [`FilePicker::pick_visual_medias`]  
    ///     - [`FilePicker::pick_visual_media`]  
    ///     - [`FilePicker::pick_dir`]  
    ///     - [`FilePicker::save_file`]  
    ///     - [`AndroidFs::resolve_file_uri`], [`AndroidFs::resolve_dir_uri`], [`AndroidFs::read_dir`], [`AndroidFs::create_new_file`], [`AndroidFs::create_dir_all`] :  
    ///     If use URI from thoese fucntions, the permissions of the origin directory URI is checked, not a entry iteself by this function. 
    ///     Because the permissions and validity period of the descendant entry URIs depend on the origin directory.   
    /// 
    /// - **mode** :  
    /// The mode of permission you want to check.  
    /// 
    /// # Support
    /// All Android version.
    #[maybe_async]
    pub fn check_persisted_uri_permission(
        &self, 
        uri: &FileUri, 
        mode: PersistableAccessMode
    ) -> Result<bool> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().check_persisted_uri_permission(uri, mode).await
        }
    }

    /// Return list of all persisted URIs that have been persisted by [`AndroidFs::take_persistable_uri_permission`] and currently valid.   
    /// 
    /// # Support
    /// All Android version.
    #[maybe_async]
    pub fn get_all_persisted_uri_permissions(&self) -> Result<impl Iterator<Item = PersistedUriPermission>> {
        #[cfg(not(target_os = "android"))] {
            Err::<std::iter::Empty<_>, _>(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().get_all_persisted_uri_permissions().await
        }
    }

    /// Relinquish a persisted URI permission grant by [`AndroidFs::take_persistable_uri_permission`].   
    /// 
    /// # Args
    /// - ***uri*** :  
    /// URI of the target file or directory.  
    ///
    /// # Support
    /// All Android version.
    #[maybe_async]
    pub fn release_persisted_uri_permission(&self, uri: &FileUri) -> Result<()> {
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().release_persisted_uri_permission(uri).await
        }
    }

    /// Relinquish a all persisted uri permission grants by [`AndroidFs::take_persistable_uri_permission`].  
    /// 
    /// # Support
    /// All Android version.
    #[maybe_async]
    pub fn release_all_persisted_uri_permissions(&self) -> Result<()> {
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().release_all_persisted_uri_permissions().await
        }
    }

    /// See [`PublicStorage::get_volumes`] or [`PrivateStorage::get_volumes`] for details.
    /// 
    /// The difference is that this does not perform any filtering.
    /// You can it by [`StorageVolume { is_available_for_public_storage, is_available_for_private_storage, .. } `](StorageVolume).
    #[maybe_async]
    pub fn get_volumes(&self) -> Result<Vec<StorageVolume>> {
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().get_available_storage_volumes().await
        }
    }

    /// See [`PublicStorage::get_primary_volume`] or [`PrivateStorage::get_primary_volume`] for details.
    /// 
    /// The difference is that this does not perform any filtering.
    /// You can it by [`StorageVolume { is_available_for_public_storage, is_available_for_private_storage, .. } `](StorageVolume).
    #[maybe_async]
    pub fn get_primary_volume(&self) -> Result<Option<StorageVolume>> {
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().get_primary_storage_volume_if_available().await
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


    #[deprecated = "Use resolve_file_uri instead"]
    #[maybe_async]
    pub fn try_resolve_file_uri(
        &self,
        dir: &FileUri, 
        relative_path: impl AsRef<std::path::Path>
    ) -> Result<FileUri> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().resolve_file_uri(dir, relative_path).await
        }
    }

    #[deprecated = "Use resolve_dir_uri instead"]
    #[maybe_async]
    pub fn try_resolve_dir_uri(
        &self,
        dir: &FileUri, 
        relative_path: impl AsRef<std::path::Path>
    ) -> Result<FileUri> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().resolve_dir_uri(dir, relative_path).await
        }
    }

    #[deprecated = "This may not return the correct uri. Use resolve_file_uri or resolve_dir_uri instead"]
    #[maybe_async]
    pub fn resolve_uri_unvalidated(
        &self, 
        dir: &FileUri, 
        relative_path: impl AsRef<std::path::Path>
    ) -> Result<FileUri> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            #[allow(deprecated)]
            self.impls()._resolve_uri_legacy(dir, relative_path).await
        }
    }

    #[deprecated = "This may not return the correct uri. Use resolve_file_uri or resolve_dir_uri instead"]
    #[maybe_async]
    pub fn resolve_uri(
        &self, 
        dir: &FileUri, 
        relative_path: impl AsRef<std::path::Path>
    ) -> Result<FileUri> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            #[allow(deprecated)]
            self.impls()._resolve_uri_legacy(dir, relative_path).await
        }
    }
}