use sync_async::sync_async;
use crate::*;
use super::*;


/// API of file storage that is available to other applications and users.  
/// 
/// # Examples
/// ```no_run
/// fn example(app: &tauri::AppHandle) {
///     use tauri_plugin_android_fs::AndroidFsExt as _;
/// 
///     let api = app.android_fs();
///     let public_storage = api.public_storage();
/// }
/// ```
#[sync_async]
pub struct PublicStorage<'a, R: tauri::Runtime> {
    #[cfg(target_os = "android")]
    pub(crate) handle: &'a tauri::plugin::PluginHandle<R>,

    #[cfg(not(target_os = "android"))]
    #[allow(unused)]
    pub(crate) handle: &'a std::marker::PhantomData<fn() -> R>,
}

#[cfg(target_os = "android")]
#[sync_async(
    use(if_sync) impls::SyncImpls as Impls;
    use(if_async) impls::AsyncImpls as Impls;
)]
impl<'a, R: tauri::Runtime> PublicStorage<'a, R> {
    
    #[always_sync]
    fn impls(&self) -> Impls<'_, R> {
        Impls { handle: &self.handle }
    }
}

#[sync_async(
    use(if_async) api_async::{AndroidFs, FileOpener, FilePicker, PrivateStorage, WritableStream};
    use(if_sync) api_sync::{AndroidFs, FileOpener, FilePicker, PrivateStorage, WritableStream};
)]
impl<'a, R: tauri::Runtime> PublicStorage<'a, R> {

    /// Requests file access permission from the user if needed.
    ///
    /// When this function returns `true`,
    /// the app is allowed to create files in `PublicStorage` and read/write the files it creates. 
    /// Access to files created by other apps is not guaranteed.
    /// Additionally, after the app is uninstalled and reinstalled, 
    /// previously created files may become inaccessible
    ///
    /// # Version behavior
    /// ### Android 10 or higher
    /// Requests no permission.   
    /// This function always returns `true`.
    ///
    /// ### Android 9 or lower
    /// Requests [`WRITE_EXTERNAL_STORAGE`](https://developer.android.com/reference/android/Manifest.permission#WRITE_EXTERNAL_STORAGE) and [`READ_EXTERNAL_STORAGE`](https://developer.android.com/reference/android/Manifest.permission#READ_EXTERNAL_STORAGE) permissions.   
    /// To request the permissions, you must declare it in `AndroidManifest.xml`.
    /// By enabling the `legacy_storage_permission` feature,
    /// the permissions will be declared automatically only for Android 9 or lower.
    ///
    /// # Support
    /// All Android versions
    #[maybe_async]
    pub fn request_permission(&self) -> Result<bool> {
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().request_storage_permission_for_public_storage().await
        }
    }

    /// Indicates whether the app has file access permission.
    ///
    /// When this function returns `true`,
    /// the app is allowed to create files in `PublicStorage` and read/write the files it creates. 
    /// Access to files created by other apps is not guaranteed.
    /// Additionally, after the app is uninstalled and reinstalled, 
    /// previously created files may become inaccessible
    ///
    /// # Version behavior
    /// ### Android 10 or higher
    /// Always returns `true`.
    ///
    /// ### Android 9 or lower
    /// Returns `true` if the app has been granted [`WRITE_EXTERNAL_STORAGE`](https://developer.android.com/reference/android/Manifest.permission#WRITE_EXTERNAL_STORAGE) and [`READ_EXTERNAL_STORAGE`](https://developer.android.com/reference/android/Manifest.permission#READ_EXTERNAL_STORAGE) permissions.    
    /// See [`PublicStorage::request_permission`] for requesting the permissions.
    ///
    /// # Support
    /// All Android versions.
    #[maybe_async]
    pub fn has_permission(&self) -> Result<bool> {
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().has_storage_permission_for_public_storage().await
        }
    }

    /// Gets a list of currently available storage volumes (internal storage, SD card, USB drive, etc.).
    /// Be aware of TOCTOU.
    /// 
    /// Since read-only SD cards and similar cases may be included, 
    /// please use [`StorageVolume { is_readonly, .. }`](StorageVolume) for filtering as needed.
    /// 
    /// This typically includes [`primary storage volume`](PublicStorage::get_primary_volume),
    /// but it may occasionally be absent if the primary volume is inaccessible 
    /// (e.g., mounted on a computer, removed, or another issue).
    ///
    /// Primary storage volume is always listed first, if included. 
    /// But the order of the others is not guaranteed.  
    /// 
    /// # Version behavior
    /// For Android 9 (API level 28) or lower, 
    /// this does not include any storage volumes other than the primary one. 
    /// 
    /// # Note
    /// The volume represents the logical view of a storage volume for an individual user:
    /// each user may have a different view for the same physical volume.
    /// In other words, it provides a separate area for each user in a multi-user environment.
    /// 
    /// # Support
    /// All Android version.  
    #[maybe_async]
    pub fn get_volumes(&self) -> Result<Vec<StorageVolume>> {
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().get_available_storage_volumes_for_public_storage().await
        }
    }

    /// Gets a primary storage volume.  
    /// This is the most common and recommended storage volume for placing files that can be accessed by other apps or user.
    /// In many cases, it is device's built-in storage.  
    /// 
    /// A device always has one (and one only) primary storage volume.  
    /// 
    /// Primary volume may not currently be accessible 
    /// if it has been mounted by the user on their computer, 
    /// has been removed from the device, or some other problem has happened. 
    /// If so, this returns `None`.
    /// 
    /// # Note
    /// The volume represents the logical view of a storage volume for an individual user:
    /// each user may have a different view for the same physical volume.
    /// In other words, it provides a separate area for each user in a multi-user environment.
    /// 
    /// # Support
    /// All Android version.
    #[maybe_async]
    pub fn get_primary_volume(&self) -> Result<Option<StorageVolume>> {
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().get_primary_storage_volume_if_available_for_public_storage().await
        }
    }

    /// Creates a new empty file in the specified public directory of the storage volume.  
    /// This returns a **persistent read-write** URI.
    ///
    /// The app can read/write it until the app is uninstalled. 
    /// And it is **not** removed when the app itself is uninstalled.  
    /// 
    /// # Version behavior
    /// ### Android 10 or higher. 
    /// No permission is required.  
    /// Files are automatically registered in the appropriate MediaStore as needed. 
    /// Scanning is triggered when the file descriptor is closed
    /// or as part of the [`pending`](PublicStorage::set_pending) lifecycle.
    /// 
    /// ### Android 9 or lower
    /// [`WRITE_EXTERNAL_STORAGE`](https://developer.android.com/reference/android/Manifest.permission#WRITE_EXTERNAL_STORAGE) and [`READ_EXTERNAL_STORAGE`](https://developer.android.com/reference/android/Manifest.permission#READ_EXTERNAL_STORAGE) permissions are required.    
    /// This needs two steps: 
    /// 
    /// 1. Declare :  
    ///     By enabling the `legacy_storage_permission` feature,  
    ///     you can declare the permissions only for Android 9 or lower automatically at build time.  
    ///
    /// 2. Runtime request :  
    ///     By calling [`PublicStorage::request_permission`],
    ///     you can request the permissions from the user at runtime.  
    ///
    /// After writing content to the file, call [`PublicStorage::scan_file`].  
    /// Until then, the file may not appear in the gallery or other apps.
    /// 
    /// # Args
    /// - ***volume_id*** :  
    /// The ID of the storage volume, such as internal storage or an SD card.  
    /// Usually, you don't need to specify this unless there is a special reason.  
    /// If `None` is provided, [`the primary storage volume`](PublicStorage::get_primary_volume) will be used.
    ///
    /// - ***base_dir*** :  
    /// The base directory for the file.  
    /// When using [`PublicImageDir`], only image MIME types should be used for ***mime_type***; using other types may cause errors.  
    /// Similarly, [`PublicVideoDir`] and [`PublicAudioDir`] should only be used with their respective media types.  
    /// Only [`PublicGeneralPurposeDir`] supports all MIME types.
    ///
    /// - ***relative_path*** :  
    /// The file path relative to the base directory.  
    /// To keep files organized, it is recommended to place your app's name directory at the top level.  
    /// Any missing parent directories will be created automatically.  
    /// If a file with the same name already exists, a sequential number is appended to ensure uniqueness.  
    /// If the file has no extension, one may be inferred from ***mime_type*** and appended to the file name.  
    /// Strings may also be sanitized as needed, so they may not be used exactly as provided.
    /// Note that append-exntesion and sanitize-path operation may vary depending on the device model and Android version.  
    ///
    /// - ***mime_type*** :  
    /// The MIME type of the file to be created.  
    /// If `None`, the MIME type will be inferred from the extension of ***relative_path***.  
    /// If that also fails, `application/octet-stream` will be used.
    /// 
    /// # Support
    /// All Android version.
    ///
    /// Note :  
    /// - [`PublicAudioDir::Audiobooks`] is not available on Android 9 (API level 28) and lower.
    /// Availability on a given device can be verified by calling [`PublicStorage::is_audiobooks_dir_available`].  
    /// - [`PublicAudioDir::Recordings`] is not available on Android 11 (API level 30) and lower.
    /// Availability on a given device can be verified by calling [`PublicStorage::is_recordings_dir_available`].  
    /// - Others dirs are available in all Android versions.
    #[maybe_async]
    pub fn create_new_file(
        &self,
        volume_id: Option<&StorageVolumeId>,
        base_dir: impl Into<PublicDir>,
        relative_path: impl AsRef<std::path::Path>, 
        mime_type: Option<&str>
    ) -> Result<FileUri> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().create_new_file_in_public_storage(
                volume_id, 
                base_dir, 
                relative_path, 
                mime_type, 
                false
            ).await
        }
    }

    /// Creates a new empty file in the specified public directory of the storage volume.  
    /// This returns a **persistent read-write** URI.
    ///
    /// The app can read/write it until the app is uninstalled. 
    /// And it is **not** removed when the app itself is uninstalled.  
    /// 
    /// # Version behavior
    /// ### Android 10 or higher
    /// No permission is required.  
    /// Files are automatically registered in the appropriate MediaStore as needed. 
    /// Scanning is triggered when the file descriptor is closed
    /// or as part of the [`pending`](PublicStorage::set_pending) lifecycle.
    /// 
    /// Diffrences from [`PublicStorage::create_new_file`] are that
    /// files are marked as pending and will not be visible to other apps until 
    /// [`PublicStorage::set_pending(..., false)`](PublicStorage::set_pending) is called. 
    ///
    /// ### Android 9 or lower
    /// This behavior is equal to [`PublicStorage::create_new_file`]. 
    /// So `pending` is ignored.  
    /// 
    /// [`WRITE_EXTERNAL_STORAGE`](https://developer.android.com/reference/android/Manifest.permission#WRITE_EXTERNAL_STORAGE) and [`READ_EXTERNAL_STORAGE`](https://developer.android.com/reference/android/Manifest.permission#READ_EXTERNAL_STORAGE) permissions are required.    
    /// This needs two steps: 
    /// 
    /// 1. Declare :  
    ///     By enabling the `legacy_storage_permission` feature,  
    ///     you can declare the permissions only for Android 9 or lower automatically at build time.  
    ///
    /// 2. Runtime request :  
    ///     By calling [`PublicStorage::request_permission`],
    ///     you can request the permissions from the user at runtime.  
    ///
    /// After writing content to the file, call [`PublicStorage::scan_file`].  
    /// Until then, the file may not appear in the gallery or other apps.
    /// 
    /// # Args
    /// - ***volume_id*** :  
    /// The ID of the storage volume, such as internal storage or an SD card.  
    /// Usually, you don't need to specify this unless there is a special reason.  
    /// If `None` is provided, [`the primary storage volume`](PublicStorage::get_primary_volume) will be used.
    ///
    /// - ***base_dir*** :  
    /// The base directory for the file.  
    /// When using [`PublicImageDir`], only image MIME types should be used for ***mime_type***; using other types may cause errors.  
    /// Similarly, [`PublicVideoDir`] and [`PublicAudioDir`] should only be used with their respective media types.  
    /// Only [`PublicGeneralPurposeDir`] supports all MIME types.
    ///
    /// - ***relative_path*** :  
    /// The file path relative to the base directory.  
    /// To keep files organized, it is recommended to place your app's name directory at the top level.  
    /// Any missing parent directories will be created automatically.  
    /// If a file with the same name already exists, a sequential number is appended to ensure uniqueness.  
    /// If the file has no extension, one may be inferred from ***mime_type*** and appended to the file name.  
    /// Strings may also be sanitized as needed, so they may not be used exactly as provided.
    /// Note that append-exntesion and sanitize-path operation may vary depending on the device model and Android version.  
    ///
    /// - ***mime_type*** :  
    /// The MIME type of the file to be created.  
    /// If `None`, the MIME type will be inferred from the extension of ***relative_path***.  
    /// If that also fails, `application/octet-stream` will be used.
    /// 
    /// # Support
    /// All Android version.
    ///
    /// Note :  
    /// - [`PublicAudioDir::Audiobooks`] is not available on Android 9 (API level 28) and lower.
    /// Availability on a given device can be verified by calling [`PublicStorage::is_audiobooks_dir_available`].  
    /// - [`PublicAudioDir::Recordings`] is not available on Android 11 (API level 30) and lower.
    /// Availability on a given device can be verified by calling [`PublicStorage::is_recordings_dir_available`].  
    /// - Others dirs are available in all Android versions.
    #[maybe_async]
    pub fn create_new_file_with_pending(
        &self,
        volume_id: Option<&StorageVolumeId>,
        base_dir: impl Into<PublicDir>,
        relative_path: impl AsRef<std::path::Path>, 
        mime_type: Option<&str>
    ) -> Result<FileUri> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().create_new_file_in_public_storage(
                volume_id, 
                base_dir, 
                relative_path, 
                mime_type, 
                true
            ).await
        }
    }

    /// Recursively create a directory and all of its parent components if they are missing.  
    /// If it already exists, do nothing.
    /// 
    /// [`PublicStorage::create_new_file`] and [`PublicStorage::create_new_file_with_pending`]
    /// do this automatically, so there is no need to use it together.
    /// 
    /// # Version behavior
    /// ### Android 10 or higher
    /// No permission is required.  
    /// 
    /// ### Android 9 or lower
    /// [`WRITE_EXTERNAL_STORAGE`](https://developer.android.com/reference/android/Manifest.permission#WRITE_EXTERNAL_STORAGE) and [`READ_EXTERNAL_STORAGE`](https://developer.android.com/reference/android/Manifest.permission#READ_EXTERNAL_STORAGE) permissions are required.    
    /// This needs two steps: 
    /// 
    /// 1. Declare :  
    ///     By enabling the `legacy_storage_permission` feature,  
    ///     you can declare the permissions only for Android 9 or lower automatically at build time.  
    ///
    /// 2. Runtime request :  
    ///     By calling [`PublicStorage::request_permission`],
    ///     you can request the permissions from the user at runtime.  
    ///
    /// # Args  
    /// - ***volume_id*** :  
    /// ID of the storage volume, such as internal storage, SD card, etc.  
    /// If `None` is provided, [`the primary storage volume`](PublicStorage::get_primary_volume) will be used.  
    /// 
    /// - ***base_dir*** :  
    /// The base directory.  
    ///  
    /// - ***relative_path*** :  
    /// The directory path relative to the base directory.    
    /// Strings may also be sanitized as needed, so they may not be used exactly as provided.
    /// Note that sanitize-path operation may vary depending on the device model and Android version.  
    ///
    /// # Support
    /// All Android Version.
    ///
    /// Note :  
    /// - [`PublicAudioDir::Audiobooks`] is not available on Android 9 (API level 28) and lower.
    /// Availability on a given device can be verified by calling [`PublicStorage::is_audiobooks_dir_available`].  
    /// - [`PublicAudioDir::Recordings`] is not available on Android 11 (API level 30) and lower.
    /// Availability on a given device can be verified by calling [`PublicStorage::is_recordings_dir_available`].  
    /// - Others dirs are available in all Android versions.
    #[maybe_async]
    pub fn create_dir_all(
        &self,
        volume_id: Option<&StorageVolumeId>,
        base_dir: impl Into<PublicDir>,
        relative_path: impl AsRef<std::path::Path>, 
    ) -> Result<()> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().create_dir_all_in_public_storage(volume_id, base_dir, relative_path).await
        }
    }

    /// Writes contents to the new file in the specified public directory of the storage volume.  
    /// This returns a **persistent read-write** URI.
    ///
    /// The app can read/write it until the app is uninstalled. 
    /// And it is **not** removed when the app itself is uninstalled.  
    /// 
    /// # Version behavior
    /// ### Android 10 or higher. 
    /// No permission is required.  
    /// 
    /// ### Android 9 or lower
    /// [`WRITE_EXTERNAL_STORAGE`](https://developer.android.com/reference/android/Manifest.permission#WRITE_EXTERNAL_STORAGE) and [`READ_EXTERNAL_STORAGE`](https://developer.android.com/reference/android/Manifest.permission#READ_EXTERNAL_STORAGE) permissions are required.    
    /// This needs two steps: 
    /// 
    /// 1. Declare :  
    ///     By enabling the `legacy_storage_permission` feature,  
    ///     you can declare the permissions only for Android 9 or lower automatically at build time.  
    ///
    /// 2. Runtime request :  
    ///     By calling [`PublicStorage::request_permission`],
    ///     you can request the permissions from the user at runtime.  
    ///
    /// # Args
    /// - ***volume_id*** :  
    /// The ID of the storage volume, such as internal storage or an SD card.  
    /// Usually, you don't need to specify this unless there is a special reason.  
    /// If `None` is provided, [`the primary storage volume`](PublicStorage::get_primary_volume) will be used.
    ///
    /// - ***base_dir*** :  
    /// The base directory for the file.  
    /// When using [`PublicImageDir`], only image MIME types should be used for ***mime_type***; using other types may cause errors.  
    /// Similarly, [`PublicVideoDir`] and [`PublicAudioDir`] should only be used with their respective media types.  
    /// Only [`PublicGeneralPurposeDir`] supports all MIME types.
    ///
    /// - ***relative_path*** :  
    /// The file path relative to the base directory.  
    /// To keep files organized, it is recommended to place your app's name directory at the top level.  
    /// Any missing parent directories will be created automatically.  
    /// If a file with the same name already exists, a sequential number is appended to ensure uniqueness.  
    /// If the file has no extension, one may be inferred from ***mime_type*** and appended to the file name.  
    /// Strings may also be sanitized as needed, so they may not be used exactly as provided.
    /// Note that append-exntesion and sanitize-path operation may vary depending on the device model and Android version.  
    ///
    /// - ***mime_type*** :  
    /// The MIME type of the file to be created.  
    /// If `None`, the MIME type will be inferred from the extension of ***relative_path***.  
    /// If that also fails, `application/octet-stream` will be used.
    /// 
    /// - ***contents*** :  
    /// Contents.
    /// 
    /// # Support
    /// All Android version.
    ///
    /// Note :  
    /// - [`PublicAudioDir::Audiobooks`] is not available on Android 9 (API level 28) and lower.
    /// Availability on a given device can be verified by calling [`PublicStorage::is_audiobooks_dir_available`].  
    /// - [`PublicAudioDir::Recordings`] is not available on Android 11 (API level 30) and lower.
    /// Availability on a given device can be verified by calling [`PublicStorage::is_recordings_dir_available`].  
    /// - Others dirs are available in all Android versions.
    #[maybe_async]
    pub fn write_new(
        &self,
        volume_id: Option<&StorageVolumeId>,
        base_dir: impl Into<PublicDir>,
        relative_path: impl AsRef<std::path::Path>,
        mime_type: Option<&str>,
        contents: impl AsRef<[u8]>
    ) -> Result<FileUri> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().write_new_file_in_public_storage(volume_id, base_dir, relative_path, mime_type, contents).await
        }
    }

    /// Scans the specified file.   
    /// This ensures that the file will be visible in the Gallery and etc.
    ///
    /// You don’t need to call this after [`PublicStorage::write_new`].   
    /// 
    /// # Version behavior
    /// ### Android 10 or higher
    /// This function does nothing, 
    /// because files are automatically registered in the appropriate MediaStore as needed. 
    /// Scanning is triggered when the file descriptor is closed
    /// or as part of the [`pending`](PublicStorage::set_pending) lifecycle.
    ///
    /// ### Android 9 or lower
    /// Requests the specified file to be scanned by MediaStore.  
    /// This function returns when the scan request has been initiated.   
    /// 
    /// # Args
    /// - **uri** :  
    /// The target file URI.
    /// This must be a URI obtained from one of the following:  
    ///     - [`PublicStorage::write_new`]
    ///     - [`PublicStorage::create_new_file`]
    ///     - [`PublicStorage::create_new_file_with_pending`]
    ///
    /// # Support
    /// All Android versions.
    #[maybe_async]
    pub fn scan_file(
        &self, 
        uri: &FileUri,
    ) -> Result<()> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().scan_file_in_public_storage(uri).await
        }
    }

    /// Specifies whether the specified file on PublicStorage is marked as pending.   
    /// When set to `true`, the app has exclusive access to the file, and it becomes invisible to other apps.
    /// 
    /// If it remains `true` for more than seven days, 
    /// the system will automatically delete the file.
    /// 
    /// # Version behavior
    /// This is available for Android 10 or higher.  
    /// On Android 9 or lower, this does nothing. 
    /// 
    /// # Args
    /// - ***uri*** :  
    /// Target file URI on PublicStorage.
    /// This must be **read-writable**.
    /// 
    /// # Support
    /// All Android version.
    /// 
    /// # References
    /// - <https://developer.android.com/reference/android/provider/MediaStore.MediaColumns#IS_PENDING>
    /// - <https://developer.android.com/training/data-storage/shared/media?hl=en#toggle-pending-status>
    #[maybe_async]
    pub fn set_pending(&self, uri: &FileUri, is_pending: bool) -> Result<()> {
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().set_file_pending_in_public_storage(uri, is_pending).await
        }
    }

    /// Retrieves the absolute path for a specified public directory within the given storage volume.   
    /// This function does **not** create any directories; it only constructs the path.
    /// 
    /// Please **avoid using this whenever possible.**  
    /// Use it only in cases that cannot be handled by [`PublicStorage::create_new_file`] or [`PrivateStorage::resolve_path`],  
    /// such as when you need to pass the absolute path of a user-accessible file as an argument to debug logger, and etc.
    /// This should not be used in production.
    /// 
    /// # Version behavior
    /// ### Android 11 or higher
    /// You can create files and folders under this directory and read/write the files it creates.  
    /// You cannot access files created by other apps. 
    /// Additionally, if the app is uninstalled, 
    /// you will no longer be able to access the files you created, 
    /// even if the app is reinstalled.  
    /// 
    /// When using [`PublicImageDir`], use only image type for file name extension, 
    /// using other type extension or none may cause errors.
    /// Similarly, use only the corresponding extesions for [`PublicVideoDir`] and [`PublicAudioDir`].
    /// Only [`PublicGeneralPurposeDir`] supports all extensions and no extension. 
    /// 
    /// ### Android 10
    /// You can do nothing with this path.
    /// 
    /// ### Android 9 or lower
    /// You can create/read/write files and folders under this directory.  
    /// 
    /// [`WRITE_EXTERNAL_STORAGE`](https://developer.android.com/reference/android/Manifest.permission#WRITE_EXTERNAL_STORAGE) and [`READ_EXTERNAL_STORAGE`](https://developer.android.com/reference/android/Manifest.permission#READ_EXTERNAL_STORAGE) permissions are required.    
    /// This needs two steps: 
    /// 
    /// 1. Declare :  
    ///     By enabling the `legacy_storage_permission` feature,  
    ///     you can declare the permissions only for Android 9 or lower automatically at build time.  
    ///
    /// 2. Runtime request :  
    ///     By calling [`PublicStorage::request_permission`],
    ///     you can request the permissions from the user at runtime.  
    ///
    /// # Note
    /// Filesystem access via this path may be heavily impacted by emulation overhead.
    /// And those files will not be registered in MediaStore. 
    /// It might eventually be registered over time, but this should not be expected.
    /// As a result, it may not appear in gallery apps or photo picker tools.
    /// 
    /// # Args
    /// - ***volume_id*** :  
    /// ID of the storage volume, such as internal storage, SD card, etc.  
    /// If `None` is provided, [`the primary storage volume`](PublicStorage::get_primary_volume) will be used.  
    /// 
    /// - ***base_dir*** :  
    /// The base directory.  
    ///  
    /// # Support
    /// All Android version.
    /// 
    /// Note :  
    /// - [`PublicAudioDir::Audiobooks`] is not available on Android 9 (API level 28) and lower.
    /// Availability on a given device can be verified by calling [`PublicStorage::is_audiobooks_dir_available`].  
    /// - [`PublicAudioDir::Recordings`] is not available on Android 11 (API level 30) and lower.
    /// Availability on a given device can be verified by calling [`PublicStorage::is_recordings_dir_available`].  
    /// - Others dirs are available in all Android versions.
    #[maybe_async]
    #[deprecated]
    pub fn resolve_path(
        &self,
        volume_id: Option<&StorageVolumeId>,
        base_dir: impl Into<PublicDir>,
    ) -> Result<std::path::PathBuf> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().resolve_path_in_public_storage(volume_id, base_dir).await
        }
    }

    /// Create the specified directory URI that has **no permissions**.  
    /// 
    /// This should only be used as `initial_location` in the file picker. 
    /// It must not be used for any other purpose.  
    /// 
    /// This is useful when selecting save location, 
    /// but when selecting existing entries, `initial_location` is often better with None.
    /// 
    /// # Args  
    /// - ***volume_id*** :  
    /// ID of the storage volume, such as internal storage, SD card, etc.  
    /// If `None` is provided, [`the primary storage volume`](PublicStorage::get_primary_volume) will be used.  
    /// 
    /// - ***base_dir*** :  
    /// The base directory.  
    ///  
    /// - ***relative_path*** :  
    /// The directory path relative to the base directory.    
    ///  
    /// # Support
    /// All Android version.
    ///
    /// Note :  
    /// - [`PublicAudioDir::Audiobooks`] is not available on Android 9 (API level 28) and lower.
    /// Availability on a given device can be verified by calling [`PublicStorage::is_audiobooks_dir_available`].  
    /// - [`PublicAudioDir::Recordings`] is not available on Android 11 (API level 30) and lower.
    /// Availability on a given device can be verified by calling [`PublicStorage::is_recordings_dir_available`].  
    /// - Others dirs are available in all Android versions.
    #[maybe_async]
    pub fn resolve_initial_location(
        &self,
        volume_id: Option<&StorageVolumeId>,
        base_dir: impl Into<PublicDir>,
        relative_path: impl AsRef<std::path::Path>,
        create_dir_all: bool
    ) -> Result<FileUri> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().resolve_initial_location_in_public_storage(volume_id, base_dir, relative_path, create_dir_all).await
        }
    }

    /// Create the specified directory URI that has **no permissions**.  
    /// 
    /// This should only be used as `initial_location` in the file picker. 
    /// It must not be used for any other purpose.  
    /// 
    /// This is useful when selecting save location, 
    /// but when selecting existing entries, `initial_location` is often better with None.
    /// 
    /// # Args  
    /// - ***volume_id*** :  
    /// ID of the storage volume, such as internal storage, SD card, etc.  
    /// If `None` is provided, [`the primary storage volume`](PublicStorage::get_primary_volume) will be used.  
    /// 
    /// # Support
    /// All Android version.
    ///
    /// Note :  
    /// - [`PublicAudioDir::Audiobooks`] is not available on Android 9 (API level 28) and lower.
    /// Availability on a given device can be verified by calling [`PublicStorage::is_audiobooks_dir_available`].  
    /// - [`PublicAudioDir::Recordings`] is not available on Android 11 (API level 30) and lower.
    /// Availability on a given device can be verified by calling [`PublicStorage::is_recordings_dir_available`].  
    /// - Others dirs are available in all Android versions.
    #[maybe_async]
    pub fn resolve_initial_location_top(
        &self,
        volume_id: Option<&StorageVolumeId>
    ) -> Result<FileUri> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().resolve_initial_location_top_in_public_storage(volume_id).await
        }
    }

    /// Verify whether [`PublicAudioDir::Audiobooks`] is available on a given device.   
    /// 
    /// If on Android 10 (API level 29) or higher, this returns true.  
    /// If on Android 9 (API level 28) and lower, this returns false.  
    /// 
    /// # Support
    /// All Android version.
    #[always_sync]
    pub fn is_audiobooks_dir_available(&self) -> Result<bool> {
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            Ok(self.impls().consts()?.env_dir_audiobooks.is_some())
        }
    }

    /// Verify whether [`PublicAudioDir::Recordings`] is available on a given device.   
    /// 
    /// If on Android 12 (API level 31) or higher, this returns true.  
    /// If on Android 11 (API level 30) and lower, this returns false.  
    /// 
    /// # Support
    /// All Android version.
    #[always_sync]
    pub fn is_recordings_dir_available(&self) -> Result<bool> {
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            Ok(self.impls().consts()?.env_dir_recordings.is_some())
        }
    }



    /// ~~Verify whether the basic functions of PublicStorage (such as [`PublicStorage::create_new_file`]) can be performed.~~
    /// 
    /// If on Android 9 (API level 28) and lower, this returns false.  
    /// If on Android 10 (API level 29) or higher, this returns true.  
    /// 
    /// # Support
    /// All Android version.
    #[deprecated]
    #[always_sync]
    pub fn is_available(&self) -> Result<bool> {
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            Ok(api_level::ANDROID_10 <= self.impls().api_level()?)
        }
    }
}