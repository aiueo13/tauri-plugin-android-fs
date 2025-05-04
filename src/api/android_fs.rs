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
    /// Note that files provided by third-party apps may not support [`FileAccessMode::WriteAppend`]. (ex: Files on GoogleDrive)  
    ///
    /// # Note
    /// This method uses a FileDescriptor internally. 
    /// However, if the target file does not physically exist on the device, such as cloud-based files, 
    /// the write operation using a FileDescriptor may not be reflected properly.
    /// In such cases, consider using [AndroidFs::write_via_kotlin], 
    /// which writes using a standard method, 
    /// or [AndroidFs::write], which automatically falls back to that approach when necessary.
    /// If you specifically need to write using std::fs::File not entire contents, see [AndroidFs::write_via_kotlin_in] or [AndroidFs::copy_via_kotlin].  
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
                return Ok(())
            }
    
            let mut file = self.open_file(uri, FileAccessMode::WriteTruncate)?;
            file.write_all(contents.as_ref())?;
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
    /// 
    /// This copy process is done on Kotlin side, not on Rust.  
    /// Large files in GB units are also supported.  
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

    /// Build the URI of a file or dir located at a relative path from the specified directory.  
    /// 
    /// The permissions and validity period of the returned URIs depend on the origin directory 
    /// (e.g., the top directory selected by [`AndroidFs::show_open_dir_dialog`]) 
    /// 
    /// Note this does not perform a validity check for args and returned uri. 
    /// Even if **dir** of args refers to a file, err not occur.  
    /// If need, check its presence using [`AndroidFs::get_mime_type`] and etc.
    /// 
    /// 
    /// # Performance
    /// This operation is relatively fast 
    /// because it only involves concatenating and encoding strings on Rust side.
    /// 
    /// # Support
    /// All.
    pub fn resolve_uri(&self, dir: &FileUri, relative_path: impl AsRef<str>) -> crate::Result<FileUri> {
        on_android!({
            let base_dir = &dir.uri;
            let relative_path = relative_path.as_ref().trim_matches('/');
            let relative_path = encode_document_id(relative_path);

            Ok(FileUri {
                document_top_tree_uri: dir.document_top_tree_uri.clone(),
                uri: format!("{base_dir}%2F{relative_path}")
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
                DecodeOption::Png => (1.0, "Png"),
                DecodeOption::Jpeg => (0.75, "Jpeg"),
                DecodeOption::Webp => (0.7, "Webp"),
                DecodeOption::JpegWith { quality } => (quality, "Jpeg"),
                DecodeOption::WebpWith { quality } => (quality, "Webp"),
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
    /// (e.g., the top directory selected by [`AndroidFs::show_open_dir_dialog`]) 
    ///  
    /// # Args  
    /// - ***dir*** :  
    /// The URI of the base directory.  
    /// This needs to be **read-write**.
    ///  
    /// - ***relative_path*** :  
    /// The file path relative to the base directory.  
    /// If a file with the same name already exists, a sequential number will be appended to ensure uniqueness.  
    /// Any missing subdirectories in the specified path will be created automatically.  
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

    /// Returns the child files and directories of the specified directory.  
    /// The order of the entries is not guaranteed.  
    /// 
    /// The permissions and validity period of the returned URIs depend on the origin directory 
    /// (e.g., the top directory selected by [`AndroidFs::show_open_dir_dialog`])  
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

    /// Opens a system file picker and returns a **read-write** URIs.  
    /// If no file is selected or the user cancels, an empty vec is returned.  
    /// 
    /// By default, returned URI is valid until the app is terminated. 
    /// If you want to persist it across app restarts, use [`AndroidFs::take_persistable_uri_permission`].
    /// 
    /// This provides a standardized file explorer-style interface, 
    /// and also allows file selection from part of third-party apps or cloud storage.
    ///
    /// Removing the returned files is also supported in most cases, 
    /// but note that files provided by third-party apps may not be removable.  
    ///  
    /// # Args  
    /// - ***initial_location*** :  
    /// Indicate the initial location of dialog.  
    /// There is no need to use this if there is no special reason.  
    /// System will do its best to launch the dialog in the specified entry 
    /// if it's a directory, or the directory that contains the specified file if not.  
    /// If this is missing or failed to resolve the desired initial location, the initial location is system specific.  
    /// This must be a URI taken from following :   
    ///     - [`AndroidFs::resolve_initial_location`]
    ///     - [`AndroidFs::show_open_file_dialog`]
    ///     - [`AndroidFs::show_save_file_dialog`]
    ///     - [`AndroidFs::show_manage_dir_dialog`]
    ///     - [`AndroidFs::read_dir`] (with `AndroidFs::show_manage_dir_dialog`)
    /// 
    /// - ***mime_types*** :  
    /// The MIME types of the file to be selected.  
    /// However, there is no guarantee that the returned file will match the specified types.  
    /// If left empty, all file types will be available (equivalent to `["*/*"]`).  
    ///  
    /// - ***multiple*** :  
    /// Indicates whether multiple file selection is allowed.  
    /// 
    /// # Support
    /// All.
    /// 
    /// # References
    /// <https://developer.android.com/reference/android/content/Intent#ACTION_OPEN_DOCUMENT>
    pub fn show_open_file_dialog(
        &self,
        initial_location: Option<&FileUri>,
        mime_types: &[&str],
        multiple: bool,
    ) -> crate::Result<Vec<FileUri>> {

        on_android!({
            impl_se!(struct Req<'a> { 
                mime_types: &'a [&'a str],
                multiple: bool,
                initial_location: Option<&'a FileUri>
            });
            impl_de!(struct Res { uris: Vec<FileUri> });
    
            let _guard = self.intent_lock.lock();
            self.api
                .run_mobile_plugin::<Res>("showOpenFileDialog", Req { mime_types, multiple, initial_location })
                .map(|v| v.uris)
                .map_err(Into::into)
        })
    }

    /// Opens a file picker and returns a **readonly** URIs.  
    /// If no file is selected or the user cancels, an empty vec is returned.  
    ///  
    /// Returned URI is valid until the app is terminated. Can not persist it.
    /// 
    /// This works differently depending on the model and version.  
    /// But recent devices often have the similar behaviour as [`AndroidFs::show_open_visual_media_dialog`] or [`AndroidFs::show_open_file_dialog`].  
    /// Use this, if you want your app to simply read/import data.
    /// 
    /// # Args  
    /// - ***mime_types*** :  
    /// The MIME types of the file to be selected.  
    /// However, there is no guarantee that the returned file will match the specified types.  
    /// If left empty, all file types will be available (equivalent to `["*/*"]`).  
    ///  
    /// - ***multiple*** :  
    /// Indicates whether multiple file selection is allowed.  
    /// 
    /// # Support
    /// All.
    /// 
    /// # References
    /// <https://developer.android.com/reference/android/content/Intent#ACTION_GET_CONTENT>
    pub fn show_open_content_dialog(
        &self,
        mime_types: &[&str],
        multiple: bool
    ) -> crate::Result<Vec<FileUri>> {

        on_android!({
            impl_se!(struct Req<'a> { mime_types: &'a [&'a str], multiple: bool });
            impl_de!(struct Res { uris: Vec<FileUri> });

            let _guard = self.intent_lock.lock();
            self.api
                .run_mobile_plugin::<Res>("showOpenContentDialog", Req { mime_types, multiple })
                .map(|v| v.uris)
                .map_err(Into::into)
        })
    }

    /// Opens a media picker and returns a **readonly** URIs.  
    /// If no file is selected or the user cancels, an empty vec is returned.  
    ///  
    /// By default, returned URI is valid until the app is terminated. 
    /// If you want to persist it across app restarts, use [`AndroidFs::take_persistable_uri_permission`].
    ///  
    /// This media picker provides a browsable interface that presents the user with their media library, 
    /// sorted by date from newest to oldest. 
    /// 
    /// # Args  
    /// - ***target*** :  
    /// The media type of the file to be selected.  
    /// Images or videos, or both.  
    ///  
    /// - ***multiple*** :  
    /// Indicates whether multiple file selection is allowed.  
    ///  
    /// # Note
    /// The file obtained from this function cannot retrieve the correct file name using [`AndroidFs::get_name`].  
    /// Instead, it will be assigned a sequential number, such as `1000091523.png`. 
    /// And this is marked intended behavior, not a bug.
    /// - <https://issuetracker.google.com/issues/268079113>  
    ///  
    /// # Support
    /// This feature is available on devices that meet the following criteria:  
    /// - Running Android 11 (API level 30) or higher  
    /// - Receive changes to Modular System Components through Google System Updates  
    ///  
    /// Availability on a given device can be verified by calling [`AndroidFs::is_visual_media_dialog_available`].  
    /// If not supported, this function behaves the same as [`AndroidFs::show_open_file_dialog`].  
    /// 
    /// # References
    /// <https://developer.android.com/training/data-storage/shared/photopicker>
    pub fn show_open_visual_media_dialog(
        &self,
        target: VisualMediaTarget,
        multiple: bool,
    ) -> crate::Result<Vec<FileUri>> {

        on_android!({
            impl_se!(struct Req { multiple: bool, target: VisualMediaTarget });
            impl_de!(struct Res { uris: Vec<FileUri> });
    
            let _guard = self.intent_lock.lock();
            self.api
                .run_mobile_plugin::<Res>("showOpenVisualMediaDialog", Req { multiple, target })
                .map(|v| v.uris)
                .map_err(Into::into)
        })
    }

    /// Opens a system directory picker, allowing the creation of a new directory or the selection of an existing one, 
    /// and returns a **read-write** directory URI. 
    /// App can fully manage entries within the returned directory.  
    /// If no directory is selected or the user cancels, `None` is returned. 
    /// 
    /// By default, returned URI is valid until the app is terminated. 
    /// If you want to persist it across app restarts, use [`AndroidFs::take_persistable_uri_permission`].
    /// 
    /// This provides a standardized file explorer-style interface.
    /// 
    /// # Args  
    /// - ***initial_location*** :  
    /// Indicate the initial location of dialog.    
    /// There is no need to use this if there is no special reason.  
    /// System will do its best to launch the dialog in the specified entry 
    /// if it's a directory, or the directory that contains the specified file if not.  
    /// If this is missing or failed to resolve the desired initial location, the initial location is system specific.   
    /// This must be a URI taken from following :   
    ///     - [`AndroidFs::resolve_initial_location`]
    ///     - [`AndroidFs::show_open_file_dialog`]
    ///     - [`AndroidFs::show_save_file_dialog`]
    ///     - [`AndroidFs::show_manage_dir_dialog`]
    ///     - [`AndroidFs::read_dir`] (with `AndroidFs::show_manage_dir_dialog`)
    /// 
    /// # Support
    /// All.
    /// 
    /// # References
    /// <https://developer.android.com/reference/android/content/Intent#ACTION_OPEN_DOCUMENT_TREE>
    pub fn show_manage_dir_dialog(
        &self,
        initial_location: Option<&FileUri>,
    ) -> crate::Result<Option<FileUri>> {

        on_android!({
            impl_se!(struct Req<'a> { initial_location: Option<&'a FileUri> });
            impl_de!(struct Res { uri: Option<FileUri> });

            let _guard = self.intent_lock.lock();
            self.api
                .run_mobile_plugin::<Res>("showManageDirDialog", Req { initial_location })
                .map(|v| v.uri)
                .map_err(Into::into)
        })
    }

    /// Please use [`AndroidFs::show_manage_dir_dialog`] instead.
    #[deprecated = "Confusing name. Please use show_manage_dir_dialog instead."]
    #[warn(deprecated)]
    pub fn show_open_dir_dialog(&self) -> crate::Result<Option<FileUri>> {
        on_android!({
            self.show_manage_dir_dialog(None)
        })
    }

    /// Opens a dialog to save a file and returns a **writeonly** URI.  
    /// The returned file may be a newly created file with no content,
    /// or it may be an existing file with the requested MIME type.  
    /// If the user cancels, `None` is returned. 
    /// 
    /// By default, returned URI is valid until the app is terminated. 
    /// If you want to persist it across app restarts, use [`AndroidFs::take_persistable_uri_permission`].
    /// 
    /// This provides a standardized file explorer-style interface, 
    /// and also allows file selection from part of third-party apps or cloud storage.
    /// 
    /// Removing and reading the returned files is also supported in most cases, 
    /// but note that files provided by third-party apps may not.  
    ///  
    /// # Args  
    /// - ***initial_location*** :  
    /// Indicate the initial location of dialog.  
    /// There is no need to use this if there is no special reason.  
    /// System will do its best to launch the dialog in the specified entry 
    /// if it's a directory, or the directory that contains the specified file if not.  
    /// If this is missing or failed to resolve the desired initial location, the initial location is system specific.  
    /// This must be a URI taken from following :   
    ///     - [`AndroidFs::resolve_initial_location`]
    ///     - [`AndroidFs::show_open_file_dialog`]
    ///     - [`AndroidFs::show_save_file_dialog`]
    ///     - [`AndroidFs::show_manage_dir_dialog`]
    ///     - [`AndroidFs::read_dir`] (with `AndroidFs::show_manage_dir_dialog`)
    /// 
    /// - ***initial_file_name*** :  
    /// An initial file name, but the user may change this value before creating the file.  
    /// 
    /// - ***mime_type*** :  
    /// The MIME type of the file to be saved.  
    /// If this is None, MIME type is inferred from the extension of ***initial_file_name*** (not file name by user input)
    /// and if that fails, `application/octet-stream` is used.  
    ///  
    /// # Support
    /// All.
    /// 
    /// # References
    /// <https://developer.android.com/reference/android/content/Intent#ACTION_CREATE_DOCUMENT>
    pub fn show_save_file_dialog(
        &self,
        initial_location: Option<&FileUri>,
        initial_file_name: impl AsRef<str>,
        mime_type: Option<&str>,
    ) -> crate::Result<Option<FileUri>> {

        on_android!({
            impl_se!(struct Req<'a> {
                initial_file_name: &'a str, 
                mime_type: Option<&'a str>, 
                initial_location: Option<&'a FileUri> 
            });
            impl_de!(struct Res { uri: Option<FileUri> });
    
            let initial_file_name = initial_file_name.as_ref();
        
            let _guard = self.intent_lock.lock();
            self.api
                .run_mobile_plugin::<Res>("showSaveFileDialog", Req { initial_file_name, mime_type, initial_location })
                .map(|v| v.uri)
                .map_err(Into::into)
        })
    }

    /// Create an **restricted** URI for the specified directory.  
    /// This should only be used as `initial_location` in the dialog. 
    /// It must not be used for any other purpose.  
    /// 
    /// This is useful when selecting (creating) new files and folders, 
    /// but when selecting existing entries, `initial_location` is often better with None.
    /// 
    /// Note this is an informal method and is not guaranteed to work reliably.
    /// But this URI does not cause the dialog to error.  
    /// So please use this with the mindset that it’s better than doing nothing.  
    /// 
    /// # Examples
    /// ```
    /// use tauri_plugin_android_fs::{AndroidFs, AndroidFsExt, InitialLocation, PublicGeneralPurposeDir, PublicImageDir};
    ///
    /// fn sample(app: tauri::AppHandle) {
    ///     let api = app.android_fs();
    ///
    ///     // Get URI of the top directory
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
    ///     let _ = api.show_save_file_dialog(Some(&initial_location), "", None);
    ///     let _ = api.show_open_file_dialog(Some(&initial_location), &[], true);
    ///     let _ = api.show_manage_dir_dialog(Some(&initial_location));
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
            const TOP_DIR: &str = "content://com.android.externalstorage.documents/document/primary%3A";

            let uri = match dir.into() {
                InitialLocation::TopPublicDir => TOP_DIR.into(),
                InitialLocation::PublicDir(dir) => format!("{TOP_DIR}{dir}"),
                InitialLocation::DirInPublicDir { base_dir, relative_path } => {
                    let relative_path = relative_path.trim_matches('/');

                    if relative_path.is_empty() {
                        format!("{TOP_DIR}{base_dir}")
                    }
                    else {
                        if create_dirs {
                            let _ = self.public_storage()
                                .create_file_in_public_dir(base_dir, format!("{relative_path}/tmp"), Some("application/octet-stream"))
                                .and_then(|u| self.remove_file(&u));
                        }
            
                        let sub_dirs = encode_document_id(relative_path);
                        format!("{TOP_DIR}{base_dir}%2F{sub_dirs}")
                    }
                }
            };

            Ok(FileUri { uri, document_top_tree_uri: None })
        })
    }

    /// Opens a dialog for sharing file to other apps.  
    /// 
    /// An error will occur if there is no app that can handle the request. 
    /// Please use [`AndroidFs::can_share_file`] to confirm.
    /// 
    /// # Args
    /// - **uri** :  
    /// Target file uri to share.  
    /// This needs to be **readable**.  
    /// This given from [`PrivateStorage`] or [`AndroidFs::show_open_visual_media_dialog`] ***cannot*** be used.
    /// 
    /// # Support
    /// All.
    pub fn show_share_file_dialog(&self, uri: &FileUri) -> crate::Result<()> {
        on_android!({
            impl_se!(struct Req<'a> { uri: &'a FileUri });
            impl_de!(struct Res;);

            self.api
                .run_mobile_plugin::<Res>("shareFile", Req { uri })
                .map(|_| ())
                .map_err(Into::into)
        })
    }

    /// Opens a dialog for viewing file on other apps.  
    /// This performs the general "open file" action.
    /// 
    /// An error will occur if there is no app that can handle the request. 
    /// Please use [`AndroidFs::can_view_file`] to confirm.
    /// 
    /// # Args
    /// - **uri** :  
    /// Target file uri to view.  
    /// This needs to be **readable**.  
    /// This given from [`PrivateStorage`] or [`AndroidFs::show_open_visual_media_dialog`] ***cannot*** be used.
    /// 
    /// # Support
    /// All.
    pub fn show_view_file_dialog(&self, uri: &FileUri) -> crate::Result<()> {
        on_android!({
            impl_se!(struct Req<'a> { uri: &'a FileUri });
            impl_de!(struct Res;);
    
            self.api
                .run_mobile_plugin::<Res>("viewFile", Req { uri })
                .map(|_| ())
                .map_err(Into::into)
        })
    }

    /// Determines whether the specified file can be used with [`AndroidFs::show_share_file_dialog`].
    /// # Args
    /// - **uri** :  
    /// Target file uri.  
    /// This needs to be **readable**.
    /// 
    /// # Support
    /// All.
    pub fn can_share_file(&self, uri: &FileUri) -> crate::Result<bool> {
        on_android!({
            impl_se!(struct Req<'a> { uri: &'a FileUri });
            impl_de!(struct Res { value: bool });

            self.api
                .run_mobile_plugin::<Res>("canShareFile", Req { uri })
                .map(|v| v.value)
                .map_err(Into::into)
        })
    }

    /// Determines whether the specified file can be used with [`AndroidFs::show_view_file_dialog`].
    /// 
    /// # Args
    /// - **uri** :  
    /// Target file uri.  
    /// This needs to be **readable**.
    /// 
    /// # Support
    /// All.
    pub fn can_view_file(&self, uri: &FileUri) -> crate::Result<bool> {
        on_android!({
            impl_se!(struct Req<'a> { uri: &'a FileUri });
            impl_de!(struct Res { value: bool });

            self.api
                .run_mobile_plugin::<Res>("canViewFile", Req { uri })
                .map(|v| v.value)
                .map_err(Into::into)
        })
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
    ///     - [`AndroidFs::show_open_file_dialog`]
    ///     - [`AndroidFs::show_open_visual_media_dialog`]
    ///     - [`AndroidFs::show_save_file_dialog`]
    ///     - [`AndroidFs::show_manage_dir_dialog`]  
    ///     - [`AndroidFs::read_dir`] :  
    ///         If this, the permissions of the origin directory URI is persisted, not a entry iteself. 
    ///         Because the permissions and validity period of the entry URIs depend on the origin directory.
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

    /// Verify whether [`AndroidFs::show_open_visual_media_dialog`] is available on a given device.
    /// 
    /// # Support
    /// All.
    pub fn is_visual_media_dialog_available(&self) -> crate::Result<bool> {
        on_android!({
            impl_de!(struct Res { value: bool });

            self.api
                .run_mobile_plugin::<Res>("isVisualMediaDialogAvailable", "")
                .map(|v| v.value)
                .map_err(Into::into)
        })
    }

    /// File storage intended for the app’s use only.
    pub fn private_storage(&self) -> PrivateStorage<'_, R> {
        PrivateStorage(self)
    }

    /// File storage that is available to other applications and users.
    pub fn public_storage(&self) -> PublicStorage<'_, R> {
        PublicStorage(self)
    }
}