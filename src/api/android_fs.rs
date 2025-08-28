use std::io::{Read as _, Write as _};
use crate::*;


/// ***Root API***  
/// 
/// # Examples
/// ```
/// fn example(app: &tauri::AppHandle) {
///     use tauri_plugin_android_fs::AndroidFsExt as _;
/// 
///     let api = app.android_fs();
/// }
/// ```
pub struct AndroidFs<R: tauri::Runtime> {
    #[cfg(target_os = "android")]
    pub(crate) app: tauri::AppHandle<R>, 

    #[cfg(target_os = "android")]
    pub(crate) api: tauri::plugin::PluginHandle<R>, 

    #[cfg(target_os = "android")]
    pub(crate) intent_lock: std::sync::Mutex<()>,

    #[cfg(not(target_os = "android"))]
    _marker: std::marker::PhantomData<fn() -> R>
}

impl<R: tauri::Runtime> AndroidFs<R> {

    pub(crate) fn new<C: serde::de::DeserializeOwned>(
        app: tauri::AppHandle<R>,
        api: tauri::plugin::PluginApi<R, C>,
    ) -> crate::Result<Self> {

        #[cfg(target_os = "android")] {
            Ok(Self {
                api: api.register_android_plugin("com.plugin.android_fs", "AndroidFsPlugin")?, 
                app,
                intent_lock: std::sync::Mutex::new(())
            })
        }
        
        #[cfg(not(target_os = "android"))] {
            Ok(Self { _marker: Default::default() })
        }
    }
}

impl<R: tauri::Runtime> AndroidFs<R> {

    /// Verify whether this plugin is available.  
    /// 
    /// On Android, this returns true.  
    /// On other platforms, this returns false.  
    pub fn is_available(&self) -> bool {
        cfg!(target_os = "android")
    }

    /// API of file storage intended for the app's use only.
    pub fn private_storage(&self) -> PrivateStorage<'_, R> {
        PrivateStorage(self)
    }

    /// API of file storage that is available to other applications and users.
    pub fn public_storage(&self) -> PublicStorage<'_, R> {
        PublicStorage(self)
    }

    /// API of file/dir picker.
    pub fn file_picker(&self) -> FilePicker<'_, R> {
        FilePicker(self)
    }

    /// API of sharing files with other apps.
    pub fn file_sender(&self) -> FileSender<'_, R> {
        FileSender(self)
    }

    /// Get the file or directory name.  
    /// 
    /// # Args
    /// - ***uri*** :  
    /// Target URI.  
    /// This needs to be **readable**.
    /// 
    /// # Support
    /// All.
    pub fn get_name(&self, uri: &FileUri) -> crate::Result<String> {
        on_android!({
            impl_se!(struct Req<'a> { uri: &'a FileUri });
            impl_de!(struct Res { name: String });

            self.api
                .run_mobile_plugin::<Res>("getName", Req { uri })
                .map(|v| v.name)
                .map_err(Into::into)
        })
    }

    /// Query the provider to get mime type.  
    /// If the directory, this returns `None`.  
    /// If the file, this returns no `None`.  
    /// If the file type is unknown or unset, this returns `Some("application/octet-stream")`.  
    ///
    /// In the case of files in [`PrivateStorage`], this is determined from the extension.
    /// 
    /// # Args
    /// - ***uri*** :  
    /// Target URI.  
    /// This needs to be **readable**.
    /// 
    /// # Support
    /// All.
    pub fn get_mime_type(&self, uri: &FileUri) -> crate::Result<Option<String>> {
        on_android!({
            impl_se!(struct Req<'a> { uri: &'a FileUri });
            impl_de!(struct Res { value: Option<String> });

            self.api
                .run_mobile_plugin::<Res>("getMimeType", Req { uri })
                .map(|v| v.value)
                .map_err(Into::into)
        })
    }

    /// Queries the file system to get information about a file, directory.
    /// 
    /// # Args
    /// - ***uri*** :  
    /// Target URI.  
    /// This needs to be **readable**.
    /// 
    /// # Note
    /// This uses [`AndroidFs::open_file`] internally.
    /// 
    /// # Support
    /// All.
    pub fn get_metadata(&self, uri: &FileUri) -> crate::Result<std::fs::Metadata> {
        on_android!({
            let file = self.open_file(uri, FileAccessMode::Read)?;
            Ok(file.metadata()?)
        })
    }

    /// Open a file in the specified mode.
    /// 
    /// # Args
    /// - ***uri*** :  
    /// Target file URI.  
    /// This must have corresponding permissions (read, write, or both) for the specified **mode**.
    /// 
    /// - ***mode*** :  
    /// Indicates how the file is opened and the permissions granted.  
    /// Note that files hosted by third-party apps may not support following:
    ///     - [`FileAccessMode::ReadWrite`]
    ///     - [`FileAccessMode::ReadWriteTruncate`]  
    ///     - [`FileAccessMode::WriteAppend`]  
    /// (ex: Files on GoogleDrive)  
    ///
    /// # Note
    /// This method uses a FileDescriptor internally. 
    /// However, if the target file does not physically exist on the device, such as cloud-based files, 
    /// the write operation using a FileDescriptor may not be reflected properly.
    /// In such cases, consider using [AndroidFs::write_via_kotlin], 
    /// which writes using a standard method, 
    /// or [AndroidFs::write], which automatically falls back to that approach when necessary.
    /// If you specifically need to write using stream not entire contents, see [AndroidFs::write_via_kotlin_in] or [AndroidFs::copy_via_kotlin] with temporary file.  
    /// 
    /// It seems that the issue does not occur on all cloud storage platforms. At least, files on Google Drive have issues, 
    /// but files on Dropbox can be written to correctly using a FileDescriptor.
    /// If you encounter issues with cloud storage other than Google Drive, please let me know on [Github](https://github.com/aiueo13/tauri-plugin-android-fs/issues/new). 
    /// This information will be used in [AndroidFs::need_write_via_kotlin] used by `AndroidFs::write`.  
    /// 
    /// There are no problems with file reading.
    /// 
    /// # Support
    /// All.
    pub fn open_file(&self, uri: &FileUri, mode: FileAccessMode) -> crate::Result<std::fs::File> {
        on_android!({
            impl_se!(struct Req<'a> { uri: &'a FileUri, mode: &'a str });
            impl_de!(struct Res { fd: std::os::fd::RawFd });
    
            let mode = match mode {
                FileAccessMode::Read => "r",
                FileAccessMode::Write => "w",
                FileAccessMode::WriteTruncate => "wt",
                FileAccessMode::WriteAppend => "wa",
                FileAccessMode::ReadWriteTruncate => "rwt",
                FileAccessMode::ReadWrite => "rw",
            };

            self.api
                .run_mobile_plugin::<Res>("getFileDescriptor", Req { uri, mode })
                .map(|v| {
                    use std::os::fd::FromRawFd;
                    unsafe { std::fs::File::from_raw_fd(v.fd) }
                })
                .map_err(Into::into)
        })
    }

    /// Reads the entire contents of a file into a bytes vector.  
    /// 
    /// If you need to operate the file, use [`AndroidFs::open_file`] instead.  
    /// 
    /// # Args
    /// - ***uri*** :  
    /// Target file URI.    
    /// This needs to be **readable**.
    /// 
    /// # Support
    /// All.
    pub fn read(&self, uri: &FileUri) -> crate::Result<Vec<u8>> {
        on_android!({
            let mut file = self.open_file(uri, FileAccessMode::Read)?;
            let mut buf = file.metadata().ok()
                .map(|m| m.len() as usize)
                .map(Vec::with_capacity)
                .unwrap_or_else(Vec::new);

            file.read_to_end(&mut buf)?;
            Ok(buf)
        })
    }

    /// Reads the entire contents of a file into a string.  
    /// 
    /// If you need to operate the file, use [`AndroidFs::open_file`] instead.  
    /// 
    /// # Args
    /// - ***uri*** :  
    /// Target file URI.  
    /// This needs to be **readable**.
    /// 
    /// # Support
    /// All.
    pub fn read_to_string(&self, uri: &FileUri) -> crate::Result<String> {
        on_android!({
            let mut file = self.open_file(uri, FileAccessMode::Read)?;
            let mut buf = file.metadata().ok()
                .map(|m| m.len() as usize)
                .map(String::with_capacity)
                .unwrap_or_else(String::new);
    
            file.read_to_string(&mut buf)?;
            Ok(buf)
        })
    }

    /// Writes a slice as the entire contents of a file.  
    /// This function will entirely replace its contents if it does exist.    
    /// 
    /// If you want to operate the file, use [`AndroidFs::open_file`] instead.  
    /// 
    /// # Args
    /// - ***uri*** :  
    /// Target file URI.  
    /// This needs to be **writable**.
    /// 
    /// # Support
    /// All.
    pub fn write(&self, uri: &FileUri, contents: impl AsRef<[u8]>) -> crate::Result<()> {
        on_android!({
            if self.need_write_via_kotlin(uri)? {
                self.write_via_kotlin(uri, contents)?;
            }
            else {
                let mut file = self.open_file(uri, FileAccessMode::WriteTruncate)?;
                file.write_all(contents.as_ref())?;
            }
            Ok(())
        })
    }

    /// Writes a slice as the entire contents of a file.  
    /// This function will entirely replace its contents if it does exist.    
    /// 
    /// Differences from `std::fs::File::write_all` is the process is done on Kotlin side.  
    /// See [`AndroidFs::open_file`] for why this function exists.
    /// 
    /// If [`AndroidFs::write`] is used, it automatically fall back to this by [`AndroidFs::need_write_via_kotlin`], 
    /// so there should be few opportunities to use this.
    /// 
    /// If you want to write using `std::fs::File`, not entire contents, use [`AndroidFs::write_via_kotlin_in`].
    /// 
    /// # Inner process
    /// The contents is written to a temporary file by Rust side 
    /// and then copied to the specified file on Kotlin side by [`AndroidFs::copy_via_kotlin`].  
    /// 
    /// # Support
    /// All.
    pub fn write_via_kotlin(
        &self, 
        uri: &FileUri,
        contents: impl AsRef<[u8]>
    ) -> crate::Result<()> {

        on_android!({
            self.write_via_kotlin_in(uri, |file| file.write_all(contents.as_ref()))
        })
    }

    /// See [`AndroidFs::write_via_kotlin`] for information.  
    /// Use this if you want to write using `std::fs::File`, not entire contents.
    /// 
    /// If you want to retain the file outside the closure, 
    /// you can perform the same operation using [`AndroidFs::copy_via_kotlin`] and [`PrivateStorage`]. 
    /// For details, please refer to the internal implementation of this function.
    /// 
    /// # Args
    /// - ***uri*** :  
    /// Target file URI to write.
    /// 
    /// - **contetns_writer** :  
    /// A closure that accepts a mutable reference to a `std::fs::File`
    /// and performs the actual write operations. Note that this represents a temporary file.
    pub fn write_via_kotlin_in<T>(
        &self, 
        uri: &FileUri,
        contents_writer: impl FnOnce(&mut std::fs::File) -> std::io::Result<T>
    ) -> crate::Result<T> {

        on_android!({
            let tmp_file_path = {
                use std::sync::atomic::{AtomicUsize, Ordering};

                static COUNTER: AtomicUsize = AtomicUsize::new(0);
                let id = COUNTER.fetch_add(1, Ordering::Relaxed);

                self.private_storage().resolve_path_with(
                    PrivateDir::Cache,
                    format!("{TMP_DIR_RELATIVE_PATH}/write_via_kotlin_in {id}")
                )?
            };

            if let Some(parent) = tmp_file_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }

            let result = {
                let ref mut file = std::fs::File::create(&tmp_file_path)?;
                contents_writer(file)
            };

            let result = result
                .map_err(crate::Error::from)
                .and_then(|t| self.copy_via_kotlin(&(&tmp_file_path).into(), uri).map(|_| t));

            let _ = std::fs::remove_file(&tmp_file_path);

            result
        })
    }

    /// Determines if the file needs to be written via Kotlin side instead of Rust side.  
    /// Currently, this returns true only if the file is on GoogleDrive.  
    /// 
    /// # Support
    /// All.
    pub fn need_write_via_kotlin(&self, uri: &FileUri) -> crate::Result<bool> {
        on_android!({
            Ok(uri.uri.starts_with("content://com.google.android.apps.docs.storage"))
        })
    }

    /// Copies the contents of src file to dest.  
    /// If dest already has contents, it is truncated before write src contents.  
    /// 
    /// This copy process is done on Kotlin side, not on Rust.  
    /// Large files in GB units are also supported.  
    /// Note that [`AndroidFs::copy`] and [`std::io::copy`] are faster.  
    ///  
    /// See [`AndroidFs::write_via_kotlin`] for why this function exists.
    /// 
    /// # Args
    /// - ***src*** :  
    /// The URI of source file.   
    /// This needs to be **readable**.
    /// 
    /// - ***dest*** :  
    /// The URI of destination file.  
    /// This needs to be **writable**.
    /// 
    /// # Support
    /// All.
    pub fn copy_via_kotlin(&self, src: &FileUri, dest: &FileUri) -> crate::Result<()> {
        on_android!({
            impl_se!(struct Req<'a> { src: &'a FileUri, dest: &'a FileUri });
            impl_de!(struct Res;);

            self.api
                .run_mobile_plugin::<Res>("copyFile", Req { src, dest })
                .map(|_| ())
                .map_err(Into::into)
        })
    }

    /// Copies the contents of src file to dest.  
    /// If dest already has contents, it is truncated before write src contents.  
    /// 
    /// # Args
    /// - ***src*** :  
    /// The URI of source file.   
    /// This needs to be **readable**.
    /// 
    /// - ***dest*** :  
    /// The URI of destination file.  
    /// This needs to be **writable**.
    /// 
    /// # Support
    /// All.
    pub fn copy(&self, src: &FileUri, dest: &FileUri) -> crate::Result<()> {
        on_android!({
            let src = &mut self.open_file(src, FileAccessMode::Read)?;
            let dest = &mut self.open_file(dest, FileAccessMode::WriteTruncate)?;
            std::io::copy(src, dest)?;
            Ok(())
        })
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
    /// 
    /// # Support
    /// All.
    pub fn rename(&self, uri: &FileUri, new_name: impl AsRef<str>) -> crate::Result<FileUri> {
        on_android!({
            impl_se!(struct Req<'a> { uri: &'a FileUri, new_name: &'a str });

            let new_name = new_name.as_ref();

            self.api
                .run_mobile_plugin::<FileUri>("rename", Req { uri, new_name })
                .map_err(Into::into)
        })
    }

    /// Remove the file.
    /// 
    /// # Args
    /// - ***uri*** :  
    /// Target file URI.  
    /// This needs to be **writable**, at least. But even if it is, 
    /// removing may not be possible in some cases. 
    /// For details, refer to the documentation of the function that provided the URI.  
    /// If not file, an error will occur.
    /// 
    /// # Support
    /// All.
    pub fn remove_file(&self, uri: &FileUri) -> crate::Result<()> {
        on_android!({
            impl_se!(struct Req<'a> { uri: &'a FileUri });
            impl_de!(struct Res;);
    
            self.api
                .run_mobile_plugin::<Res>("deleteFile", Req { uri })
                .map(|_| ())
                .map_err(Into::into)
        })
    }

    /// Remove the **empty** directory.
    /// 
    /// # Args
    /// - ***uri*** :  
    /// Target directory URI.  
    /// This needs to be **writable**.  
    /// If not empty directory, an error will occur.
    /// 
    /// # Support
    /// All.
    pub fn remove_dir(&self, uri: &FileUri) -> crate::Result<()> {
        on_android!({
            impl_se!(struct Req<'a> { uri: &'a FileUri });
            impl_de!(struct Res;);
        
            self.api
                .run_mobile_plugin::<Res>("deleteEmptyDir", Req { uri })
                .map(|_| ())
                .map_err(Into::into)
        })
    }

    /// Removes a directory and all its contents. Use carefully!
    /// 
    /// # Args
    /// - ***uri*** :  
    /// Target directory URI.  
    /// This needs to be **writable**.  
    /// If not directory, an error will occur.
    /// 
    /// # Support
    /// All.
    pub fn remove_dir_all(&self, uri: &FileUri) -> crate::Result<()> {
        on_android!({
            impl_se!(struct Req<'a> { uri: &'a FileUri });
            impl_de!(struct Res;);
        
            self.api
                .run_mobile_plugin::<Res>("deleteDirAll", Req { uri })
                .map(|_| ())
                .map_err(Into::into)
        })
    }

    /// Build a URI of an **existing** file located at the relative path from the specified directory.   
    /// Error occurs, if the file does not exist.  
    /// 
    /// The permissions and validity period of the returned URI depend on the origin directory 
    /// (e.g., the top directory selected by [`FilePicker::pick_dir`]) 
    /// 
    /// # Support
    /// All.
    pub fn try_resolve_file_uri(&self, dir: &FileUri, relative_path: impl AsRef<str>) -> crate::Result<FileUri> {
        on_android!({
            let uri = self.resolve_uri(dir, relative_path)?;            
            if self.get_mime_type(&uri)?.is_none() {
                return Err(crate::Error { msg: format!("This is a directory, not a file: {uri:?}").into() })
            }
            Ok(uri)
        })
    }

    /// Build a URI of an **existing** directory located at the relative path from the specified directory.   
    /// Error occurs, if the directory does not exist.  
    /// 
    /// The permissions and validity period of the returned URI depend on the origin directory 
    /// (e.g., the top directory selected by [`FilePicker::pick_dir`]) 
    /// 
    /// # Support
    /// All.
    pub fn try_resolve_dir_uri(&self, dir: &FileUri, relative_path: impl AsRef<str>) -> crate::Result<FileUri> {
        on_android!({
            let uri = self.resolve_uri(dir, relative_path)?;
            if self.get_mime_type(&uri)?.is_some() {
                return Err(crate::Error { msg: format!("This is a file, not a directory: {uri:?}").into() })
            }
            Ok(uri)
        })
    }

    /// Build a URI of an entry located at the relative path from the specified directory.   
    /// 
    /// This function does not perform checks on the arguments or the returned URI.  
    /// Even if the dir argument refers to a file, no error occurs (and no panic either).
    /// Instead, it simply returns an invalid URI that will cause errors if used with other functions.  
    /// 
    /// If you need check, consider using [`AndroidFs::try_resolve_file_uri`] or [`AndroidFs::try_resolve_dir_uri`] instead. 
    /// Or use this with [`AndroidFs::get_mime_type`].
    /// 
    /// The permissions and validity period of the returned URI depend on the origin directory 
    /// (e.g., the top directory selected by [`FilePicker::pick_dir`]) 
    /// 
    /// # Performance
    /// This operation is relatively fast 
    /// because it does not call Kotlin API and only involves operating strings on Rust side.
    /// 
    /// # Support
    /// All.
    pub fn resolve_uri(&self, dir: &FileUri, relative_path: impl AsRef<str>) -> crate::Result<FileUri> {
        on_android!({
            let base_dir = &dir.uri;
            let relative_path = relative_path.as_ref().trim_matches('/');

            if relative_path.is_empty() {
                return Ok(dir.clone())
            }

            Ok(FileUri {
                document_top_tree_uri: dir.document_top_tree_uri.clone(),
                uri: format!("{base_dir}%2F{}", encode_document_id(relative_path))
            })
        })
    }

    /// See [`AndroidFs::get_thumbnail_to`] for descriptions.  
    /// 
    /// If thumbnail does not wrote to dest, return false.
    pub fn get_thumbnail_to(
        &self, 
        src: &FileUri,
        dest: &FileUri,
        preferred_size: Size,
        format: ImageFormat,
    ) -> crate::Result<bool> {

        on_android!({
            impl_se!(struct Req<'a> {
                src: &'a FileUri, 
                dest: &'a FileUri,
                format: &'a str,
                quality: u8,
                width: u32,
                height: u32,
            });
            impl_de!(struct Res { value: bool });

            let (quality, format) = match format {
                ImageFormat::Png => (1.0, "Png"),
                ImageFormat::Jpeg => (0.75, "Jpeg"),
                ImageFormat::Webp => (0.7, "Webp"),
                ImageFormat::JpegWith { quality } => (quality, "Jpeg"),
                ImageFormat::WebpWith { quality } => (quality, "Webp"),
            };
            let quality = (quality * 100.0).clamp(0.0, 100.0) as u8;
            let Size { width, height } = preferred_size;
        
            self.api
                .run_mobile_plugin::<Res>("getThumbnail", Req { src, dest, format, quality, width, height })
                .map(|v| v.value)
                .map_err(Into::into)
        })
    }

    /// Query the provider to get a file thumbnail.  
    /// If thumbnail does not exist it, return None.
    /// 
    /// Note this does not cache. Please do it in your part if need.  
    /// 
    /// # Args
    /// - ***uri*** :  
    /// Targe file uri.  
    /// Thumbnail availablty depends on the file provider.  
    /// In general, images and videos are available.  
    /// For files in [`PrivateStorage`], 
    /// the file type must match the filename extension.  
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
    /// All.
    pub fn get_thumbnail(
        &self,
        uri: &FileUri,
        preferred_size: Size,
        format: ImageFormat,
    ) -> crate::Result<Option<Vec<u8>>> {

        on_android!({
            let tmp_file_path = {
                use std::sync::atomic::{AtomicUsize, Ordering};

                static COUNTER: AtomicUsize = AtomicUsize::new(0);
                let id = COUNTER.fetch_add(1, Ordering::Relaxed);

                self.private_storage().resolve_path_with(
                    PrivateDir::Cache,
                    format!("{TMP_DIR_RELATIVE_PATH}/get_thumbnail {id}")
                )?
            };

            if let Some(parent) = tmp_file_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }

            std::fs::File::create(&tmp_file_path)?;

            let result = self.get_thumbnail_to(uri, &(&tmp_file_path).into(), preferred_size, format)
                .and_then(|ok| {
                    if (ok) {
                        std::fs::read(&tmp_file_path)
                            .map(Some)
                            .map_err(Into::into)
                    }
                    else {
                        Ok(None)
                    }
                });

            let _ = std::fs::remove_file(&tmp_file_path);

            result
        })
    }

    /// Creates a new empty file in the specified location and returns a URI.   
    /// 
    /// The permissions and validity period of the returned URIs depend on the origin directory 
    /// (e.g., the top directory selected by [`FilePicker::pick_dir`]) 
    ///  
    /// Please note that this has a different meaning from `std::fs::create` that open the file in write mod.
    /// If you need it, use [`AndroidFs::open_file`] with [`FileAccessMode::WriteTruncate`].
    /// 
    /// # Args  
    /// - ***dir*** :  
    /// The URI of the base directory.  
    /// This needs to be **read-write**.
    ///  
    /// - ***relative_path*** :  
    /// The file path relative to the base directory.  
    /// Any missing subdirectories in the specified path will be created automatically.  
    /// If a file with the same name already exists, 
    /// the system append a sequential number to ensure uniqueness.  
    /// If no extension is present, 
    /// the system may infer one from ***mime_type*** and may append it to the file name. 
    /// But this append-extension operation depends on the model and version.  
    ///  
    /// - ***mime_type*** :  
    /// The MIME type of the file to be created.  
    /// If this is None, MIME type is inferred from the extension of ***relative_path***
    /// and if that fails, `application/octet-stream` is used.  
    ///  
    /// # Support
    /// All.
    pub fn create_file(
        &self,
        dir: &FileUri, 
        relative_path: impl AsRef<str>, 
        mime_type: Option<&str>
    ) -> crate::Result<FileUri> {

        on_android!({
            impl_se!(struct Req<'a> { dir: &'a FileUri, mime_type: Option<&'a str>, relative_path: &'a str });
        
            let relative_path = relative_path.as_ref();

            self.api
                .run_mobile_plugin::<FileUri>("createFile", Req { dir, mime_type, relative_path })
                .map_err(Into::into)
        })
    }

    /// Recursively create a directory and all of its parent components if they are missing,
    /// then return the URI.  
    /// If it already exists, do nothing and just return the direcotry uri.
    /// 
    /// [`AndroidFs::create_file`] does this automatically, so there is no need to use it together.
    /// 
    /// # Args  
    /// - ***dir*** :  
    /// The URI of the base directory.  
    /// This needs to be **read-write**.
    ///  
    /// - ***relative_path*** :  
    /// The directory path relative to the base directory.    
    ///  
    /// # Support
    /// All.
    pub fn create_dir_all(
        &self,
        dir: &FileUri, 
        relative_path: impl AsRef<str>, 
    ) -> Result<FileUri> {

        on_android!({
            let relative_path = relative_path.as_ref().trim_matches('/');
            if relative_path.is_empty() {
                return Ok(dir.clone())
            }

            // TODO:
            // create_file経由ではなく folder作成専用のkotlin apiを作成し呼び出すようにする
            let tmp_file_uri = self.create_file(
                dir, 
                format!("{relative_path}/TMP-01K3CGCKYSAQ1GHF8JW5FGD4RW"), 
                Some("application/octet-stream")
            )?;
            let _ = self.remove_file(&tmp_file_uri);
            let uri = self.resolve_uri(dir, relative_path)?;

            Ok(uri)
        })
    }

    /// Returns the child files and directories of the specified directory.  
    /// The order of the entries is not guaranteed.  
    /// 
    /// The permissions and validity period of the returned URIs depend on the origin directory 
    /// (e.g., the top directory selected by [`FilePicker::pick_dir`])  
    /// 
    /// # Args
    /// - ***uri*** :  
    /// Target directory URI.  
    /// This needs to be **readable**.
    ///  
    /// # Note  
    /// The returned type is an iterator because of the data formatting and the file system call is not executed lazily. 
    /// Thus, for directories with thousands or tens of thousands of elements, it may take several seconds.  
    /// 
    /// # Support
    /// All.
    pub fn read_dir(&self, uri: &FileUri) -> crate::Result<impl Iterator<Item = Entry>> {
        on_android!(std::iter::Empty::<_>, {
            impl_se!(struct Req<'a> { uri: &'a FileUri });
            impl_de!(struct Obj { name: String, uri: FileUri, last_modified: i64, byte_size: i64, mime_type: Option<String> });
            impl_de!(struct Res { entries: Vec<Obj> });
    
            self.api
                .run_mobile_plugin::<Res>("readDir", Req { uri })
                .map(|v| v.entries.into_iter())
                .map(|v| v.map(|v| match v.mime_type {
                    Some(mime_type) => Entry::File {
                        name: v.name,
                        last_modified: std::time::UNIX_EPOCH + std::time::Duration::from_millis(v.last_modified as u64),
                        len: v.byte_size as u64,
                        mime_type,
                        uri: v.uri,
                    },
                    None => Entry::Dir {
                        name: v.name,
                        last_modified: std::time::UNIX_EPOCH + std::time::Duration::from_millis(v.last_modified as u64),
                        uri: v.uri,
                    }
                }))
                .map_err(Into::into)
        })
    }

    /// Use [`FilePicker::pick_files`] instead.
    #[deprecated = "Use FilePicker::pick_files instead"]
    pub fn show_open_file_dialog(
        &self,
        initial_location: Option<&FileUri>,
        mime_types: &[&str],
        multiple: bool,
    ) -> crate::Result<Vec<FileUri>> {

        self.file_picker().pick_files(initial_location, mime_types, multiple)
    }

    /// Use [`FilePicker::pick_contents`] instead.
    #[deprecated = "Use FilePicker::pick_contents instead"]
    pub fn show_open_content_dialog(
        &self,
        mime_types: &[&str],
        multiple: bool
    ) -> crate::Result<Vec<FileUri>> {

        self.file_picker().pick_contents(mime_types, multiple)
    }

    /// Use [`FilePicker::pick_visual_medias`] instead.
    #[deprecated = "Use FilePicker::pick_visual_medias instead"]
    pub fn show_open_visual_media_dialog(
        &self,
        target: VisualMediaTarget,
        multiple: bool,
    ) -> crate::Result<Vec<FileUri>> {

        self.file_picker().pick_visual_medias(target, multiple)
    }

    /// Use [`FilePicker::pick_dir`] instead.
    #[deprecated = "Use FilePicker::pick_dir instead"]
    pub fn show_manage_dir_dialog(
        &self,
        initial_location: Option<&FileUri>,
    ) -> crate::Result<Option<FileUri>> {

        self.file_picker().pick_dir(initial_location)
    }

    /// Use [`FilePicker::pick_dir`] instead.
    #[deprecated = "Use FilePicker::pick_dir instead."]
    pub fn show_open_dir_dialog(&self) -> crate::Result<Option<FileUri>> {
        self.file_picker().pick_dir(None)
    }


    /// Use [`FilePicker::save_file`] instead.
    #[deprecated = "Use FilePicker::save_file instead."]
    pub fn show_save_file_dialog(
        &self,
        initial_location: Option<&FileUri>,
        initial_file_name: impl AsRef<str>,
        mime_type: Option<&str>,
    ) -> crate::Result<Option<FileUri>> {
        
        self.file_picker().save_file(initial_location, initial_file_name, mime_type)
    }

    /// Create an **restricted** URI for the specified directory.  
    /// This should only be used as `initial_location` in the file picker. 
    /// It must not be used for any other purpose.  
    /// 
    /// This is useful when selecting (creating) new files and folders, 
    /// but when selecting existing entries, `initial_location` is often better with None.
    /// 
    /// Note this is an informal method and is not guaranteed to work reliably.
    /// But this URI does not cause the dialog to error.  
    /// So please use this with the mindset that it's better than doing nothing.  
    /// 
    /// # Examples
    /// ```rust
    ///  use tauri_plugin_android_fs::{AndroidFsExt, InitialLocation, PublicGeneralPurposeDir, PublicImageDir};
    ///
    /// fn sample(app: tauri::AppHandle) {
    ///     let api = app.android_fs();
    ///
    ///     // Get URI of the top public directory in primary volume
    ///     let initial_location = api.resolve_initial_location(
    ///         InitialLocation::TopPublicDir,
    ///         false,
    ///     ).expect("Should be on Android");
    ///
    ///     // Get URI of ~/Pictures/
    ///     let initial_location = api.resolve_initial_location(
    ///         PublicImageDir::Pictures,
    ///         false
    ///     ).expect("Should be on Android");
    ///
    ///     // Get URI of ~/Documents/sub_dir1/sub_dir2/
    ///     let initial_location = api.resolve_initial_location(
    ///         InitialLocation::DirInPublicDir {
    ///             base_dir: PublicGeneralPurposeDir::Documents.into(),
    ///             relative_path: "sub_dir1/sub_dir2"
    ///         },
    ///         true // Create dirs of 'sub_dir1' and 'sub_dir2', if not exists
    ///     ).expect("Should be on Android");
    ///
    ///     // Open dialog with initial_location
    ///     let _ = api.file_picker().save_file(Some(&initial_location), "", None);
    ///     let _ = api.file_picker().pick_file(Some(&initial_location), &[]);
    ///     let _ = api.file_picker().pick_dir(Some(&initial_location));
    /// }
    /// ```
    /// 
    /// # Support
    /// All.
    pub fn resolve_initial_location<'a>(
        &self,
        dir: impl Into<InitialLocation<'a>>,
        create_dirs: bool
    ) -> crate::Result<FileUri> {

        on_android!({
            const TOP_DIR: &str = "content://com.android.externalstorage.documents/document/primary";

            let uri = match dir.into() {
                InitialLocation::TopPublicDir => format!("{TOP_DIR}%3A"),
                InitialLocation::PublicDir(dir) => format!("{TOP_DIR}%3A{dir}"),
                InitialLocation::DirInPublicDir { base_dir, relative_path } => {
                    let relative_path = relative_path.trim_matches('/');

                    if relative_path.is_empty() {
                        format!("{TOP_DIR}%3A{base_dir}")
                    }
                    else {
                        if create_dirs {
                            let _ = self.public_storage().create_dir_all(base_dir, relative_path);
                        }
                        let sub_dirs = encode_document_id(relative_path);
                        format!("{TOP_DIR}%3A{base_dir}%2F{sub_dirs}")
                    }
                },
                InitialLocation::DirInPublicAppDir { base_dir, relative_path } => {
                    let relative_path = &format!(
                        "{}/{}", 
                        self.public_storage().app_dir_name()?,
                        relative_path.trim_matches('/'),
                    );
                  
                    return self.resolve_initial_location(
                        InitialLocation::DirInPublicDir { base_dir, relative_path }, 
                        create_dirs
                    )
                }
            };

            Ok(FileUri { uri, document_top_tree_uri: None })
        })
    }

    /// Use [`FileSender::share_file`] instead
    #[deprecated = "Use FileSender::share_file instead."]
    pub fn show_share_file_dialog(&self, uri: &FileUri) -> crate::Result<()> {
        self.file_sender().share_file(uri)
    }
    
    /// Use [`FileSender::open_file`] instead
    #[deprecated = "Use FileSender::open_file instead."]
    pub fn show_view_file_dialog(&self, uri: &FileUri) -> crate::Result<()> {
        self.file_sender().open_file(uri)
    }

    /// Use [`FileSender::can_share_file`] instead
    #[deprecated = "Use FileSender::can_share_file instead"]
    pub fn can_share_file(&self, uri: &FileUri) -> crate::Result<bool> {
        #[allow(deprecated)]
        self.file_sender().can_share_file(uri)
    }

    /// Use [`FileSender::can_open_file`] instead
    #[deprecated = "Use FileSender::can_open_file instead"]
    pub fn can_view_file(&self, uri: &FileUri) -> crate::Result<bool> {
        #[allow(deprecated)]
        self.file_sender().can_open_file(uri)
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
    /// URI of the target file or directory. This must be a URI taken from following :  
    ///     - [`FilePicker::pick_files`]  
    ///     - [`FilePicker::pick_file`]  
    ///     - [`FilePicker::pick_visual_medias`]  
    ///     - [`FilePicker::pick_visual_media`]  
    ///     - [`FilePicker::pick_dir`]  
    ///     - [`FilePicker::save_file`]  
    ///     - [`AndroidFs::try_resolve_file_uri`], [`AndroidFs::try_resolve_dir_uri`], [`AndroidFs::resolve_uri`], [`AndroidFs::read_dir`], [`AndroidFs::create_file`], [`AndroidFs::create_dir_all`] :  
    ///     If use URI from thoese fucntions, the permissions of the origin directory URI is persisted, not a entry iteself by this function. 
    ///     Because the permissions and validity period of the descendant entry URIs depend on the origin directory.   
    /// 
    /// # Support
    /// All. 
    pub fn take_persistable_uri_permission(&self, uri: &FileUri) -> crate::Result<()> {
        on_android!({
            impl_se!(struct Req<'a> { uri: &'a FileUri });
            impl_de!(struct Res;);

            self.api
                .run_mobile_plugin::<Res>("takePersistableUriPermission", Req { uri })
                .map(|_| ())
                .map_err(Into::into)
        })
    }

    /// Check a persisted URI permission grant by [`AndroidFs::take_persistable_uri_permission`].  
    /// Returns false if there are only non-persistent permissions or no permissions.
    /// 
    /// # Args
    /// - **uri** :  
    /// URI of the target file or directory.  
    /// If this is via [`AndroidFs::read_dir`], the permissions of the origin directory URI is checked, not a entry iteself. 
    /// Because the permissions and validity period of the entry URIs depend on the origin directory.
    ///
    /// - **mode** :  
    /// The mode of permission you want to check.  
    /// 
    /// # Support
    /// All.
    pub fn check_persisted_uri_permission(&self, uri: &FileUri, mode: PersistableAccessMode) -> crate::Result<bool> {
        on_android!({
            impl_se!(struct Req<'a> { uri: &'a FileUri, mode: PersistableAccessMode });
            impl_de!(struct Res { value: bool });

            self.api
                .run_mobile_plugin::<Res>("checkPersistedUriPermission", Req { uri, mode })
                .map(|v| v.value)
                .map_err(Into::into)
        })
    }

    /// Return list of all persisted URIs that have been persisted by [`AndroidFs::take_persistable_uri_permission`] and currently valid.   
    /// 
    /// # Support
    /// All.
    pub fn get_all_persisted_uri_permissions(&self) -> crate::Result<impl Iterator<Item = PersistedUriPermission>> {
        on_android!(std::iter::Empty::<_>, {
            impl_de!(struct Obj { uri: FileUri, r: bool, w: bool, d: bool });
            impl_de!(struct Res { items: Vec<Obj> });
    
            self.api
                .run_mobile_plugin::<Res>("getAllPersistedUriPermissions", "")
                .map(|v| v.items.into_iter())
                .map(|v| v.map(|v| {
                    let (uri, can_read, can_write) = (v.uri, v.r, v.w);
                    match v.d {
                        true => PersistedUriPermission::Dir { uri, can_read, can_write },
                        false => PersistedUriPermission::File { uri, can_read, can_write }
                    }
                }))
                .map_err(Into::into)
        })
    }

    /// Relinquish a persisted URI permission grant by [`AndroidFs::take_persistable_uri_permission`].   
    /// 
    /// # Args
    /// - ***uri*** :  
    /// URI of the target file or directory.  
    ///
    /// # Support
    /// All.
    pub fn release_persisted_uri_permission(&self, uri: &FileUri) -> crate::Result<()> {
        on_android!({
            impl_se!(struct Req<'a> { uri: &'a FileUri });
            impl_de!(struct Res;);

            self.api
                .run_mobile_plugin::<Res>("releasePersistedUriPermission", Req { uri })
                .map(|_| ())
                .map_err(Into::into)
        })
    }

    /// Relinquish a all persisted uri permission grants by [`AndroidFs::take_persistable_uri_permission`].  
    /// 
    /// # Support
    /// All.
    pub fn release_all_persisted_uri_permissions(&self) -> crate::Result<()> {
        on_android!({
            impl_de!(struct Res);

            self.api
                .run_mobile_plugin::<Res>("releaseAllPersistedUriPermissions", "")
                .map(|_| ())
                .map_err(Into::into)
        })
    }

    /// Use [`FilePicker::is_visual_media_picker_available`] instead.
    #[deprecated = "Use FilePicker::is_visual_media_picker_available instead"]
    pub fn is_visual_media_dialog_available(&self) -> crate::Result<bool> {
        self.file_picker().is_visual_media_picker_available()
    }
}