use crate::*;
use std::sync::Arc;


/// Custom file provider.
pub trait CustomFileProvider: Send + Sync + 'static {
    
    /// Opens the custom file represented by the specified logical path.
    ///
    /// Returns an error if the specified mode is not supported.
    /// See the documentation for each [`FileAccessMode`] variant
    /// for the definition and the behaviors.
    /// In particular,
    /// if [`FileAccessMode::WriteTruncate`] or [`FileAccessMode::ReadWriteTruncate`] is supported,
    /// the implementation must handle truncation if such modes are provided.
    /// Similarly, if [`FileAccessMode::WriteAppend`] is supported,
    /// the implementation must handle appending if such a mode is provided.
    /// [`FileAccessMode::Write`] does not prescribe whether the file should be truncated.
    /// However, for consistency with common conventions,
    /// it is strongly recommended to truncate the file and treat it the same as [`FileAccessMode::WriteTruncate`].
    /// 
    /// # Invocation Context
    /// In this plugin, this function is executed on the caller's Java thread or the Android Binder thread.
    /// To perform asynchronous operations or require a Tauri's Tokio runtime context,
    /// use [`tauri::async_runtime::block_on`](https://docs.rs/tauri/latest/tauri/async_runtime/fn.block_on.html)
    /// inside the implementation.
    fn open(&self, path: String, mode: FileAccessMode) -> std::io::Result<CustomFile>;

    /// Removes the custom file represented by the specified logical path.
    ///
    /// If this Custom File Provider does not support the operation,
    /// return an error with [`std::io::ErrorKind::Unsupported`].
    ///
    /// If it is supported but the specified Custom File does not exist,
    /// return an error with [`std::io::ErrorKind::NotFound`].
    /// If the operation fails, return the resulting error.
    ///
    /// # Invocation Context
    /// In this plugin, this function is executed on the caller's Java thread or the Android Binder thread.
    /// To perform asynchronous operations or require a Tauri's Tokio runtime context,
    /// use [`tauri::async_runtime::block_on`](https://docs.rs/tauri/latest/tauri/async_runtime/fn.block_on.html)
    /// inside the implementation.
    fn remove(&self, path: String) -> std::io::Result<()> {
        let _ = path;
        Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "unsupported operation"))
    }

    /// Returns the name of the custom file represented by the specified logical path.
    ///
    /// If the name is unknown, returns `Ok(None)`.
    /// In this case, falls back to last path segment in the logical path.
    ///
    /// If the specified Custom File does not exist,
    /// returns an error with [`std::io::ErrorKind::NotFound`].
    ///
    /// # Invocation Context
    /// In this plugin, this function is executed on the caller's Java thread or the Android Binder thread.
    /// To perform asynchronous operations or require a Tauri's Tokio runtime context,
    /// use [`tauri::async_runtime::block_on`](https://docs.rs/tauri/latest/tauri/async_runtime/fn.block_on.html)
    /// inside the implementation.
    fn get_name(&self, path: String) -> std::io::Result<Option<String>>;

    /// Returns the size in bytes of the custom file represented by the specified logical path.
    ///
    /// If the size is unknown, returns `Ok(None)`.
    ///
    /// If the specified Custom File does not exist,
    /// returns an error with [`std::io::ErrorKind::NotFound`].
    ///
    /// # Invocation Context
    /// In this plugin, this function is executed on the caller's Java thread or the Android Binder thread.
    /// To perform asynchronous operations or require a Tauri's Tokio runtime context,
    /// use [`tauri::async_runtime::block_on`](https://docs.rs/tauri/latest/tauri/async_runtime/fn.block_on.html)
    /// inside the implementation.
    fn get_len(&self, path: String) -> std::io::Result<Option<u64>>;

    /// Returns the MIME type of the custom file represented by the specified logical path.
    ///
    /// If the MIME type is unknown, returns `Ok(None)`.
    /// In this case, fall back to `"application/octet-stream"`.
    ///
    /// If the specified Custom File does not exist,
    /// returns an error with [`std::io::ErrorKind::NotFound`].
    ///
    /// # Invocation Context
    /// In this plugin, this function is executed on the caller's Java thread or the Android Binder thread.
    /// To perform asynchronous operations or require a Tauri's Tokio runtime context,
    /// use [`tauri::async_runtime::block_on`](https://docs.rs/tauri/latest/tauri/async_runtime/fn.block_on.html)
    /// inside the implementation.
    fn get_mime_type(&self, path: String) -> std::io::Result<Option<String>>;

    /// Returns the last modification time of the custom file represented by the specified logical path.
    ///
    /// If the last modification time is unknown, returns `Ok(None)`.
    ///
    /// If the specified Custom File does not exist,
    /// returns an error with [`std::io::ErrorKind::NotFound`].
    ///
    /// # Invocation Context
    /// In this plugin, this function is executed on the caller's Java thread or the Android Binder thread.
    /// To perform asynchronous operations or require a Tauri's Tokio runtime context,
    /// use [`tauri::async_runtime::block_on`](https://docs.rs/tauri/latest/tauri/async_runtime/fn.block_on.html)
    /// inside the implementation.
    fn get_last_modified(&self, path: String) -> std::io::Result<Option<std::time::SystemTime>>;
}

/// A handle to a custom file.
pub struct CustomFile {
    pub(crate) repr: CustomFileRepr,
}

pub(crate) enum CustomFileRepr {
    Custom(Box<dyn CustomFileCallback>),
    Raw(std::fs::File),
}

impl CustomFile {

    fn new_custom(callback: impl CustomFileCallback) -> Self {
        Self { repr: CustomFileRepr::Custom(Box::new(callback))}
    }

    fn new_raw(file: std::fs::File) -> Self {
        Self { repr: CustomFileRepr::Raw(file)}
    }

    /// Creates a new custom file backed by the given [`std::fs::File`].
    /// 
    /// # Invocation Context
    /// In this plugin, the file descriptor is passed directly to the caller.
    pub fn from_file(file: std::fs::File) -> Self {
        Self::new_raw(file)
    }

    /// Creates a new custom file backed by the given raw file descriptor.
    ///
    /// Ownership of the descriptor is transferred to the returned custom file,
    /// and the descriptor is closed when the returned custom file is dropped.
    /// 
    /// # Invocation Context
    /// In this plugin, the file descriptor is passed directly to the caller.
    ///
    /// # Safety
    /// `fd` must be an owned, open file descriptor.
    #[cfg(unix)]
    pub unsafe fn from_raw_fd(fd: std::os::fd::RawFd) -> Self {
        use std::os::fd::FromRawFd;
        Self::new_raw(std::fs::File::from_raw_fd(fd))
    }

    /// Creates a new custom file backed by the given [`CustomFileCallback`].
    /// 
    /// # Invocation Context
    /// In this plugin, it is passed to the caller as an [Android Proxy File](https://developer.android.com/reference/android/os/storage/StorageManager#openProxyFileDescriptor(int,%20android.os.ProxyFileDescriptorCallback,%20android.os.Handler))
    /// which is available for Android 8 (API level 26) or higher
    /// If unavailable, an error is returned to the caller when the file is opened.
    pub fn from_custom_callback(callback: impl CustomFileCallback) -> Self {
        Self::new_custom(callback)
    }

    /// Creates a new read-only custom file backed by the given [`ReadableCustomFileCallback`].
    /// 
    /// # Invocation Context
    /// In this plugin, it is passed to the caller as an [Android Proxy File](https://developer.android.com/reference/android/os/storage/StorageManager#openProxyFileDescriptor(int,%20android.os.ProxyFileDescriptorCallback,%20android.os.Handler))
    /// which is available for Android 8 (API level 26) or higher
    /// If unavailable, an error is returned to the caller when the file is opened.
    pub fn from_custom_readonly_callback(callback: impl ReadableCustomFileCallback) -> Self {
        Self::from_custom_callback(custom_callback_impl::ReadableImpl(callback))
    }

    /// Creates a new write-only custom file backed by the given [`WritableCustomFileCallback`].
    /// 
    /// # Invocation Context
    /// In this plugin, it is passed to the caller as an [Android Proxy File](https://developer.android.com/reference/android/os/storage/StorageManager#openProxyFileDescriptor(int,%20android.os.ProxyFileDescriptorCallback,%20android.os.Handler))
    /// which is available for Android 8 (API level 26) or higher
    /// If unavailable, an error is returned to the caller when the file is opened.
    pub fn from_custom_writeonly_callback(callback: impl WritableCustomFileCallback) -> Self {
        Self::from_custom_callback(custom_callback_impl::WritableImpl(callback))
    }

    /// Creates a new read-only custom file backed by the implementation of [`std::io::Read`] and [`std::io::Seek`].
    /// 
    /// # Invocation Context
    /// In this plugin, it is passed to the caller as an [Android Proxy File](https://developer.android.com/reference/android/os/storage/StorageManager#openProxyFileDescriptor(int,%20android.os.ProxyFileDescriptorCallback,%20android.os.Handler))
    /// which is available for Android 8 (API level 26) or higher
    /// If unavailable, an error is returned to the caller when the file is opened.
    pub fn from_read_seek(callback: impl std::io::Read + std::io::Seek + Send + 'static) -> Self {
        Self::new_custom(custom_callback_impl::ReadSeekImpl(callback))
    }

    /// Creates a new write-only custom file backed by the implementation of [`std::io::Write`] and [`std::io::Seek`].
    /// 
    /// # Invocation Context
    /// In this plugin, it is passed to the caller as an [Android Proxy File](https://developer.android.com/reference/android/os/storage/StorageManager#openProxyFileDescriptor(int,%20android.os.ProxyFileDescriptorCallback,%20android.os.Handler))
    /// which is available for Android 8 (API level 26) or higher
    /// If unavailable, an error is returned to the caller when the file is opened.
    pub fn from_write_seek(callback: impl std::io::Write + std::io::Seek + Send + 'static) -> Self {
        Self::new_custom(custom_callback_impl::WriteSeekImpl(callback))
    }

    /// Creates a new custom file backed by the implementation of [`std::io::Read`], [`std::io::Write`] and [`std::io::Seek`].
    /// 
    /// # Invocation Context
    /// In this plugin, it is passed to the caller as an [Android Proxy File](https://developer.android.com/reference/android/os/storage/StorageManager#openProxyFileDescriptor(int,%20android.os.ProxyFileDescriptorCallback,%20android.os.Handler))
    /// which is available for Android 8 (API level 26) or higher
    /// If unavailable, an error is returned to the caller when the file is opened.
    pub fn from_read_write_seek(callback: impl std::io::Read + std::io::Write + std::io::Seek + Send + 'static) -> Self {
        Self::new_custom(custom_callback_impl::ReadWriteSeekImpl(callback))
    }

    /// Creates a new read-only custom file backed by the bytes in memory.
    /// 
    /// # Invocation Context
    /// In this plugin, it is passed to the caller as an [Android Proxy File](https://developer.android.com/reference/android/os/storage/StorageManager#openProxyFileDescriptor(int,%20android.os.ProxyFileDescriptorCallback,%20android.os.Handler))
    /// which is available for Android 8 (API level 26) or higher
    /// If unavailable, an error is returned to the caller when the file is opened.
    pub fn from_bytes(content: impl AsRef<[u8]> + Send + 'static) -> Self {
        Self::new_custom(custom_callback_impl::BytesImpl(content))
    }

    /// Creates a new read-only custom file backed by the bytes in memory.
    /// 
    /// # Invocation Context
    /// In this plugin, it is passed to the caller as an [Android Proxy File](https://developer.android.com/reference/android/os/storage/StorageManager#openProxyFileDescriptor(int,%20android.os.ProxyFileDescriptorCallback,%20android.os.Handler))
    /// which is available for Android 8 (API level 26) or higher
    /// If unavailable, an error is returned to the caller when the file is opened.
    pub fn from_arc_bytes(content: Arc<impl AsRef<[u8]> + Send + Sync + 'static>) -> Self {
        Self::new_custom(custom_callback_impl::ArcBytesImpl(content))
    }
}

pub trait CustomFileCallback: Send + 'static {

    /// Returns the size in bytes.
    ///
    /// # Invocation Context
    /// In this plugin, this function is executed on a dedicated Java thread for performing blocking operations.
    /// To perform asynchronous operations or require a Tauri's Tokio runtime context,
    /// use [`tauri::async_runtime::block_on`](https://docs.rs/tauri/latest/tauri/async_runtime/fn.block_on.html)
    /// inside the implementation.
    /// 
    /// See: [ProxyFileDescriptorCallback.onGetSize](https://developer.android.com/reference/android/os/ProxyFileDescriptorCallback#onGetSize())
    fn len(&mut self) -> std::io::Result<u64>;

    /// Ensures that all written data is committed to persistent storage.
    /// 
    /// # Invocation Context
    /// In this plugin, this function is executed on a dedicated Java thread for performing blocking operations.
    /// To perform asynchronous operations or require a Tauri's Tokio runtime context,
    /// use [`tauri::async_runtime::block_on`](https://docs.rs/tauri/latest/tauri/async_runtime/fn.block_on.html)
    /// inside the implementation.
    /// 
    /// See: [ProxyFileDescriptorCallback.onFsync](https://developer.android.com/reference/android/os/ProxyFileDescriptorCallback#onFsync())
    fn fsync(&mut self) -> std::io::Result<()>;

    /// Reads bytes starting from a given offset
    /// and returns the number of bytes read.
    ///
    /// The offset is relative to the start of the file.
    ///
    /// Returns `Ok(0)` if the end of the data has been reached
    /// or if the offset is out of range.
    ///
    /// # Invocation Context
    /// In this plugin, this function is executed on a dedicated Java thread for performing blocking operations.
    /// To perform asynchronous operations or require a Tauri's Tokio runtime context,
    /// use [`tauri::async_runtime::block_on`](https://docs.rs/tauri/latest/tauri/async_runtime/fn.block_on.html)
    /// inside the implementation.
    /// 
    /// If this function does not fill the given buffer, it will be called repeatedly
    /// with the offset advanced accordingly, until the entire buffer is filled,
    /// EOF is reached, or an error occurs.
    /// 
    /// See: [ProxyFileDescriptorCallback.onRead](https://developer.android.com/reference/android/os/ProxyFileDescriptorCallback#onRead())
    fn read_at(&mut self, buf: &mut [u8], offset: u64) -> std::io::Result<usize>;

    /// Writes bytes at a given offset
    /// and returns the number of bytes written.
    ///
    /// The offset is relative to the start of the file.
    ///
    /// # Invocation Context
    /// In this plugin, this function is executed on a dedicated Java thread for performing blocking operations.
    /// To perform asynchronous operations or require a Tauri's Tokio runtime context,
    /// use [`tauri::async_runtime::block_on`](https://docs.rs/tauri/latest/tauri/async_runtime/fn.block_on.html)
    /// inside the implementation.
    /// 
    /// If this function does not write all of the given data, 
    /// it will be called repeatedly with the offset advanced accordingly, 
    /// until all of the data has been written or an error occurs.
    /// If nothing is written and 0 is returned, it is treated as an error.
    /// 
    /// See: [ProxyFileDescriptorCallback.onWrite](https://developer.android.com/reference/android/os/ProxyFileDescriptorCallback#onWrite())
    fn write_at(&mut self, data: &[u8], offset: u64) -> std::io::Result<usize>;
}

pub trait ReadableCustomFileCallback: Send + 'static {

    /// Returns the size in bytes.
    ///
    /// # Invocation Context
    /// In this plugin, this function is executed on a dedicated Java thread for performing blocking operations.
    /// To perform asynchronous operations or require a Tauri's Tokio runtime context,
    /// use [`tauri::async_runtime::block_on`](https://docs.rs/tauri/latest/tauri/async_runtime/fn.block_on.html)
    /// inside the implementation.
    /// 
    /// See: [ProxyFileDescriptorCallback.onGetSize](https://developer.android.com/reference/android/os/ProxyFileDescriptorCallback#onGetSize())
    fn len(&mut self) -> std::io::Result<u64>;

    /// Reads bytes starting from a given offset
    /// and returns the number of bytes read.
    ///
    /// The offset is relative to the start of the file.
    ///
    /// Returns `Ok(0)` if the end of the data has been reached
    /// or if the offset is out of range.
    ///
    /// # Invocation Context
    /// In this plugin, this function is executed on a dedicated Java thread for performing blocking operations.
    /// To perform asynchronous operations or require a Tauri's Tokio runtime context,
    /// use [`tauri::async_runtime::block_on`](https://docs.rs/tauri/latest/tauri/async_runtime/fn.block_on.html)
    /// inside the implementation.
    /// 
    /// If this function does not fill the given buffer, it will be called repeatedly
    /// with the offset advanced accordingly, until the entire buffer is filled,
    /// EOF is reached, or an error occurs.
    /// 
    /// See: [ProxyFileDescriptorCallback.onRead](https://developer.android.com/reference/android/os/ProxyFileDescriptorCallback#onRead())
    fn read_at(&mut self, buf: &mut [u8], offset: u64) -> std::io::Result<usize>;
}

pub trait WritableCustomFileCallback: Send + 'static {

    /// Returns the size in bytes.
    ///
    /// # Invocation Context
    /// In this plugin, this function is executed on a dedicated Java thread for performing blocking operations.
    /// To perform asynchronous operations or require a Tauri's Tokio runtime context,
    /// use [`tauri::async_runtime::block_on`](https://docs.rs/tauri/latest/tauri/async_runtime/fn.block_on.html)
    /// inside the implementation.
    /// 
    /// See: [ProxyFileDescriptorCallback.onGetSize](https://developer.android.com/reference/android/os/ProxyFileDescriptorCallback#onGetSize())
    fn len(&mut self) -> std::io::Result<u64>;

    /// Ensures that all written data is committed to persistent storage.
    /// 
    /// # Invocation Context
    /// In this plugin, this function is executed on a dedicated Java thread for performing blocking operations.
    /// To perform asynchronous operations or require a Tauri's Tokio runtime context,
    /// use [`tauri::async_runtime::block_on`](https://docs.rs/tauri/latest/tauri/async_runtime/fn.block_on.html)
    /// inside the implementation.
    /// 
    /// See: [ProxyFileDescriptorCallback.onFsync](https://developer.android.com/reference/android/os/ProxyFileDescriptorCallback#onFsync())
    fn fsync(&mut self) -> std::io::Result<()>;

    /// Writes bytes at a given offset
    /// and returns the number of bytes written.
    ///
    /// The offset is relative to the start of the file.
    ///
    /// # Invocation Context
    /// In this plugin, this function is executed on a dedicated Java thread for performing blocking operations.
    /// To perform asynchronous operations or require a Tauri's Tokio runtime context,
    /// use [`tauri::async_runtime::block_on`](https://docs.rs/tauri/latest/tauri/async_runtime/fn.block_on.html)
    /// inside the implementation.
    /// 
    /// If this function does not write all of the given data, 
    /// it will be called repeatedly with the offset advanced accordingly, 
    /// until all of the data has been written or an error occurs.
    /// If nothing is written and 0 is returned, it is treated as an error.
    /// 
    /// See: [ProxyFileDescriptorCallback.onWrite](https://developer.android.com/reference/android/os/ProxyFileDescriptorCallback#onWrite())
    fn write_at(&mut self, data: &[u8], offset: u64) -> std::io::Result<usize>;
}


impl<T: CustomFileCallback> ReadableCustomFileCallback for T {

    fn len(&mut self) -> std::io::Result<u64> {
        CustomFileCallback::len(self)
    }

    fn read_at(&mut self, buf: &mut [u8], offset: u64) -> std::io::Result<usize> {
        CustomFileCallback::read_at(self, buf, offset)
    }
}

impl<T: CustomFileCallback> WritableCustomFileCallback for T {

    fn len(&mut self) -> std::io::Result<u64> {
        CustomFileCallback::len(self)
    }

    fn fsync(&mut self) -> std::io::Result<()> {
        CustomFileCallback::fsync(self)
    }

    fn write_at(&mut self, data: &[u8], offset: u64) -> std::io::Result<usize> {
        CustomFileCallback::write_at(self, data, offset)
    }
}

/// One implementation of [`CustomFileProvider`] with [`std::fs`].
/// 
/// This is useful when you want to specify a file
/// by its absolute path and treat it directly as a custom file.
pub struct StdFileProvider<R: tauri::Runtime> {
    resolve_path: Arc<dyn Fn(String) -> Option<tauri::path::SafePathBuf> + Send + Sync + 'static>,
    get_mime_type: Arc<dyn Fn(std::path::PathBuf, String) -> std::io::Result<Option<String>> + Send + Sync + 'static>,
    _app: tauri::AppHandle<R>,
}

impl<R: tauri::Runtime> StdFileProvider<R> {

    /// Creates a new file provider.
    ///
    /// Specify the absolute file path as the logical path in [`AndroidFs::build_custom_file_uri`].
    ///
    /// `check_path` can be used to control which paths are allowed to be accessed.
    /// Paths that are not absolute paths, as well as paths containing components such as `../`,
    /// are rejected before `check_path` is called to prevent path traversal.
    ///
    /// The MIME type is inferred from the file extension.
    /// If the inference fails, it defaults to `application/octet-stream`.
    /// If an explicit MIME type needs to be specified for each path,
    /// use [`StdFileProvider::new_custom`].
    ///
    /// [`AndroidFs::build_custom_file_uri`]: crate::api::api_async::AndroidFs::build_custom_file_uri
    pub fn new(
        app: tauri::AppHandle<R>,
        check_path: impl Fn(&std::path::Path) -> bool + Send + Sync + 'static,
    ) -> Self {

        let resolve_path = Arc::new(move |path: String| {
            let path = std::path::PathBuf::from(path);
            if path.file_name().is_none() {
                return None;
            }
            if !path.is_absolute() {
                return None;
            }
            let Ok(path) = tauri::path::SafePathBuf::new(path) else {
                return None;
            };

            if check_path(path.as_ref()) {
                Some(path)
            } 
            else {
                None
            }
        });

        let get_mime_type = {
            let app = app.clone();
            Arc::new(move |resolved_path: std::path::PathBuf, _path: String| {
                if let Some(extension) = resolved_path.extension() {
                    app.android_fs()
                        .get_mime_type_from_extension(&extension.to_string_lossy())
                        .map_err(|_| std::io::Error::other("anything wrong"))
                } 
                else {
                    Ok(None)
                }
            })
        };

        Self {
            resolve_path,
            get_mime_type,
            _app: app,
        }
    }

    /// Creates a new file provider.
    ///
    /// `resolve_path` can be used to parse path and control which paths are allowed to be accessed.
    /// Paths that are not absolute paths, as well as paths containing components such as ../,
    /// are rejected after `resolve_path` is called to prevent path traversal.
    /// The logical path specified in [`AndroidFs::build_custom_file_uri`] is passed to this as it.
    ///
    /// `get_mime_type` is called only after the logical path has been successfully resolved by `resolve_path`, with path traversal prevented.
    /// The first argument of this closure is the resolved path,
    /// and the second argument is the original logical path before being resolved.
    ///
    /// [`AndroidFs::build_custom_file_uri`]: crate::api::api_async::AndroidFs::build_custom_file_uri
    pub fn new_custom(
        app: tauri::AppHandle<R>,
        resolve_path: impl Fn(String) -> Option<std::path::PathBuf> + Send + Sync + 'static,
        get_mime_type: impl Fn(std::path::PathBuf, String) -> std::io::Result<Option<String>> + Send + Sync + 'static,
    ) -> Self {

        let resolve_path = move |path| {
            let Some(path) = (resolve_path)(path) else {
                return None;
            };
            if path.file_name().is_none() {
                return None;
            }
            if !path.is_absolute() {
                return None;
            }
            let Ok(path) = tauri::path::SafePathBuf::new(path) else {
                return None;
            };

            Some(path)
        };

        Self {
            resolve_path: Arc::new(resolve_path),
            get_mime_type: Arc::new(get_mime_type),
            _app: app,
        }
    }

    fn resolve_path(&self, path: String) -> std::io::Result<tauri::path::SafePathBuf> {
        (self.resolve_path)(path).ok_or_else(|| std::io::Error::new(std::io::ErrorKind::PermissionDenied, "path denied"))
    }
}

impl<R: tauri::Runtime> CustomFileProvider for StdFileProvider<R> {

    fn open(&self, path: String, mode: FileAccessMode) -> std::io::Result<CustomFile> {
        let path = self.resolve_path(path)?;
        let path = path.as_ref();

        let mut options = std::fs::OpenOptions::new();

        // FileProvider の実装と同じように、w は wt と同じで切り捨てるように扱う
        // <https://android.googlesource.com/platform/frameworks/support/%2B/refs/heads/androidx-main/core/core/src/main/java/androidx/core/content/FileProvider.java?utm_source=chatgpt.com#555>
        match mode {
            FileAccessMode::Read => options.read(true),
            FileAccessMode::Write |
            FileAccessMode::WriteTruncate => options.write(true).truncate(true).create(true),
            FileAccessMode::WriteAppend => options.write(true).append(true).create(true),
            FileAccessMode::ReadWrite => options.read(true).write(true).create(true),
            FileAccessMode::ReadWriteTruncate => options.read(true).write(true).truncate(true).create(true)
        };

        let file = options.open(path)?;
        Ok(CustomFile::from_file(file))
    }

    fn remove(&self, path: String) -> std::io::Result<()> {
        let path = self.resolve_path(path)?;
        let path = path.as_ref();
        std::fs::remove_file(path)
    }

    fn get_len(&self, path: String) -> std::io::Result<Option<u64>> {
        let path = self.resolve_path(path)?;
        let path = path.as_ref();
        std::fs::metadata(path).map(|m| Some(m.len()))
    }

    fn get_mime_type(&self, path: String) -> std::io::Result<Option<String>> {
        let resolved_path = self.resolve_path(path.clone())?;
        let resolved_path = resolved_path.as_ref().to_path_buf();
        (self.get_mime_type)(resolved_path, path)
    }

    fn get_name(&self, path: String) -> std::io::Result<Option<String>> {
        let path = self.resolve_path(path)?;
        let path = path.as_ref();
        path.file_name()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "not found"))
            .map(|p| Some(p.to_string_lossy().into_owned()))
    }

    fn get_last_modified(&self, path: String) -> std::io::Result<Option<std::time::SystemTime>> {
        let path = self.resolve_path(path)?;
        let path = path.as_ref();
        std::fs::metadata(&path).and_then(|m| Some(m.modified()).transpose())
    }
}

mod custom_callback_impl {
    use super::*;


    pub struct ReadableImpl<T>(pub T);

    impl<T: ReadableCustomFileCallback> CustomFileCallback for ReadableImpl<T> {

        fn len(&mut self) -> std::io::Result<u64> {
            ReadableCustomFileCallback::len(&mut self.0)
        }

        fn read_at(&mut self, buf: &mut [u8], offset: u64) -> std::io::Result<usize> {
            ReadableCustomFileCallback::read_at(&mut self.0, buf, offset)
        }

        fn fsync(&mut self) -> std::io::Result<()> {
            Ok(())
        }

        fn write_at(&mut self, _data: &[u8], _offset: u64) -> std::io::Result<usize> {
            Err(std::io::Error::from_raw_os_error(9 /* EBADF */))
        }
    }


    pub struct WritableImpl<T>(pub T);

    impl<T: WritableCustomFileCallback> CustomFileCallback for WritableImpl<T> {

        fn len(&mut self) -> std::io::Result<u64> {
            WritableCustomFileCallback::len(&mut self.0)
        }

        fn write_at(&mut self, data: &[u8], offset: u64) -> std::io::Result<usize> {
            WritableCustomFileCallback::write_at(&mut self.0, data, offset)
        }

        fn fsync(&mut self) -> std::io::Result<()> {
            WritableCustomFileCallback::fsync(&mut self.0)
        }

        fn read_at(&mut self, _buf: &mut [u8], _offset: u64) -> std::io::Result<usize> {
            Err(std::io::Error::from_raw_os_error(9 /* EBADF */))
        }
    }


    pub struct ReadSeekImpl<T>(pub T);

    impl<T: std::io::Read + std::io::Seek + Send + 'static> CustomFileCallback for ReadSeekImpl<T> {
        
        fn len(&mut self) -> std::io::Result<u64> {
            with_retry_if_io_interrupted(|| self.0.seek(std::io::SeekFrom::End(0)))
        }
    
        fn read_at(&mut self, buf: &mut [u8], offset: u64) -> std::io::Result<usize> {
            with_retry_if_io_interrupted(|| self.0.seek(std::io::SeekFrom::Start(offset)))?;
            with_retry_if_io_interrupted(|| self.0.read(buf))
        }

        fn fsync(&mut self) -> std::io::Result<()> {
            Ok(())
        }

        fn write_at(&mut self, _data: &[u8], _offset: u64) -> std::io::Result<usize> {
            Err(std::io::Error::from_raw_os_error(9 /* EBADF */))
        }
    }

    pub struct WriteSeekImpl<T>(pub T);

    impl<T: std::io::Write + std::io::Seek + Send + 'static> CustomFileCallback for WriteSeekImpl<T> {
        
        fn len(&mut self) -> std::io::Result<u64> {
            with_retry_if_io_interrupted(|| self.0.seek(std::io::SeekFrom::End(0)))
        }

        fn fsync(&mut self) -> std::io::Result<()> {
            with_retry_if_io_interrupted(|| self.0.flush())
        }

        fn write_at(&mut self, data: &[u8], offset: u64) -> std::io::Result<usize> {
            with_retry_if_io_interrupted(|| self.0.seek(std::io::SeekFrom::Start(offset)))?;
            with_retry_if_io_interrupted(|| self.0.write(data))
        }

        fn read_at(&mut self, _buf: &mut [u8], _offset: u64) -> std::io::Result<usize> {
            Err(std::io::Error::from_raw_os_error(9 /* EBADF */))
        }
    }


    pub struct ReadWriteSeekImpl<T>(pub T);

    impl<T: std::io::Read + std::io::Write + std::io::Seek + Send + 'static> CustomFileCallback for ReadWriteSeekImpl<T> {
        
        fn len(&mut self) -> std::io::Result<u64> {
            with_retry_if_io_interrupted(|| self.0.seek(std::io::SeekFrom::End(0)))
        }

        fn fsync(&mut self) -> std::io::Result<()> {
            with_retry_if_io_interrupted(|| self.0.flush())
        }

        fn write_at(&mut self, data: &[u8], offset: u64) -> std::io::Result<usize> {
            with_retry_if_io_interrupted(|| self.0.seek(std::io::SeekFrom::Start(offset)))?;
            with_retry_if_io_interrupted(|| self.0.write(data))
        }

        fn read_at(&mut self, buf: &mut [u8], offset: u64) -> std::io::Result<usize> {
            with_retry_if_io_interrupted(|| self.0.seek(std::io::SeekFrom::Start(offset)))?;
            with_retry_if_io_interrupted(|| self.0.read(buf))
        }
    }


    pub struct BytesImpl<T>(pub T);

    impl<T: AsRef<[u8]> + Send + 'static> CustomFileCallback for BytesImpl<T> {
        
        fn len(&mut self) -> std::io::Result<u64> {
            Ok(self.0.as_ref().len() as u64)
        }
    
        fn read_at(&mut self, buf: &mut [u8], offset: u64) -> std::io::Result<usize> {
            let Ok(offset) = usize::try_from(offset) else {
                return Ok(0)
            };

            let content = self.0.as_ref();
            if content.len() <= offset {
                return Ok(0);
            }

            let nread = buf.len().min(content.len() - offset);
            buf[..nread].copy_from_slice(&content[offset..offset + nread]);
            Ok(nread)
        }

        fn fsync(&mut self) -> std::io::Result<()> {
            Ok(())
        }

        fn write_at(&mut self, _data: &[u8], _offset: u64) -> std::io::Result<usize> {
            Err(std::io::Error::from_raw_os_error(9 /* EBADF */))
        }
    }


    pub struct ArcBytesImpl<T>(pub Arc<T>);

    impl<T: AsRef<[u8]> + Send + Sync + 'static> CustomFileCallback for ArcBytesImpl<T> {
        
        fn len(&mut self) -> std::io::Result<u64> {
            Ok(self.0.as_ref().as_ref().len() as u64)
        }
    
        fn read_at(&mut self, buf: &mut [u8], offset: u64) -> std::io::Result<usize> {
            let Ok(offset) = usize::try_from(offset) else {
                return Ok(0)
            };

            let content = self.0.as_ref().as_ref();
            if content.len() <= offset {
                return Ok(0);
            }

            let nread = buf.len().min(content.len() - offset);
            buf[..nread].copy_from_slice(&content[offset..offset + nread]);
            Ok(nread)
        }

        fn fsync(&mut self) -> std::io::Result<()> {
            Ok(())
        }

        fn write_at(&mut self, _data: &[u8], _offset: u64) -> std::io::Result<usize> {
            Err(std::io::Error::from_raw_os_error(9 /* EBADF */))
        }
    }


    fn with_retry_if_io_interrupted<F, T>(mut operation: F) -> std::io::Result<T>
    where
        F: FnMut() -> std::io::Result<T>,
    {
        loop {
            match operation() {
                Ok(value) => return Ok(value),
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
                Err(e) => return Err(e),
            }
        }
    }
}