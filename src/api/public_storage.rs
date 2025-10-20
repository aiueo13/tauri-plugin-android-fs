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
    /// # Note
    /// The volume represents the logical view of a storage volume for an individual user:
    /// each user may have a different view for the same physical volume.
    /// In other words, it provides a separate area for each user in a multi-user environment.
    /// 
    /// # Support
    /// Android 10 (API level 29) or higher.  
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
    /// Android 10 (API level 29) or higher.   
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
    /// The created file has the following features:  
    /// - It is registered with the appropriate MediaStore as needed.  
    /// - The app can fully manage it until the app is uninstalled.  
    /// - It is **not** removed when the app itself is uninstalled.  
    /// 
    /// # Args
    /// - ***volume_id*** :  
    /// ID of the storage volume, such as internal storage, SD card, etc.  
    /// Usually, you don't need to specify this unless there is a special reason.  
    /// If `None` is provided, [`the primary storage volume`](PublicStorage::get_primary_volume) will be used.  
    /// 
    /// - ***base_dir*** :  
    /// The base directory.  
    /// When using [`PublicImageDir`], use only image MIME types for ***mime_type***, which is discussed below.; using other types may cause errors.
    /// Similarly, use only the corresponding media types for [`PublicVideoDir`] and [`PublicAudioDir`].
    /// Only [`PublicGeneralPurposeDir`] supports all MIME types. 
    /// 
    /// - ***relative_path*** :  
    /// The file path relative to the base directory.  
    /// To avoid cluttering files, it is helpful to place the app name directory at the top level.   
    /// Any missing subdirectories in the specified path will be created automatically.  
    /// If a file with the same name already exists, 
    /// the system append a sequential number to ensure uniqueness.  
    /// If no extension is present, 
    /// the system may infer one from ***mime_type*** and may append it to the file name. 
    /// But this append-extension operation depends on the model and version.  
    /// The system may sanitize these strings as needed, so those strings may not be used as it is.
    ///  
    /// - ***mime_type*** :  
    /// The MIME type of the file to be created.  
    /// If this is None, MIME type is inferred from the extension of ***relative_path***
    /// and if that fails, `application/octet-stream` is used.  
    /// 
    /// # Support
    /// Android 10 (API level 29) or higher.  
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
            self.impls().create_new_file_in_public_storage(volume_id, base_dir, relative_path, mime_type, false).await
        }
    }

    /// Creates a new empty file in the specified public directory of the storage volume.  
    /// This returns a **persistent read-write** URI.
    ///
    /// The created file has the following features:  
    /// - Marked as pending and will not be visible to other apps until [`PublicStorage::set_pending(..., false)`](PublicStorage::set_pending) is called.
    /// - It is registered with the appropriate MediaStore as needed.  
    /// - The app can fully manage it until the app is uninstalled.  
    /// - It is **not** removed when the app itself is uninstalled.  
    /// 
    /// # Args
    /// - ***volume_id*** :  
    /// ID of the storage volume, such as internal storage, SD card, etc.  
    /// Usually, you don't need to specify this unless there is a special reason.  
    /// If `None` is provided, [`the primary storage volume`](PublicStorage::get_primary_volume) will be used.  
    /// 
    /// - ***base_dir*** :  
    /// The base directory.  
    /// When using [`PublicImageDir`], use only image MIME types for ***mime_type***, which is discussed below.; using other types may cause errors.
    /// Similarly, use only the corresponding media types for [`PublicVideoDir`] and [`PublicAudioDir`].
    /// Only [`PublicGeneralPurposeDir`] supports all MIME types. 
    /// 
    /// - ***relative_path*** :  
    /// The file path relative to the base directory.  
    /// To avoid cluttering files, it is helpful to place the app name directory at the top level.   
    /// Any missing subdirectories in the specified path will be created automatically.  
    /// If a file with the same name already exists, 
    /// the system append a sequential number to ensure uniqueness.  
    /// If no extension is present, 
    /// the system may infer one from ***mime_type*** and may append it to the file name. 
    /// But this append-extension operation depends on the model and version.  
    /// The system may sanitize these strings as needed, so those strings may not be used as it is.
    ///  
    /// - ***mime_type*** :  
    /// The MIME type of the file to be created.  
    /// If this is None, MIME type is inferred from the extension of ***relative_path***
    /// and if that fails, `application/octet-stream` is used.  
    /// 
    /// # Support
    /// Android 10 (API level 29) or higher.  
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
            self.impls().create_new_file_in_public_storage(volume_id, base_dir, relative_path, mime_type, true).await
        }
    }

    /// Recursively create a directory and all of its parent components if they are missing.  
    /// If it already exists, do nothing.
    /// 
    /// [`PublicStorage::create_new_file`] does this automatically, so there is no need to use it together.
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
    /// The system may sanitize these strings as needed, so those strings may not be used as it is.
    ///  
    /// # Support
    /// Android 10 (API level 29) or higher.  
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

    /// Specifies whether the specified file on PublicStorage is marked as pending.   
    /// When set to `true`, the app has exclusive access to the file, and it becomes invisible to other apps.
    /// 
    /// If it remains `true` for more than seven days, 
    /// the system will automatically delete the file.
    /// 
    /// # Args
    /// - ***uri*** :  
    /// Target file URI on PublicStorage.
    /// This must be **read-writable**.
    /// 
    /// # Support
    /// Android 10 (API level 29) or higher.  
    /// 
    /// # References
    /// <https://developer.android.com/reference/android/provider/MediaStore.MediaColumns#IS_PENDING>
    /// <https://developer.android.com/training/data-storage/shared/media?hl=en#toggle-pending-status>
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
    /// **Please avoid using this whenever possible.**    
    /// Use it only in cases that cannot be handled by [`PublicStorage::create_new_file`] or [`PrivateStorage::resolve_path`], 
    /// such as when you need to pass the absolute path of a user-accessible file as an argument to any database library, debug logger, and etc.  
    ///
    /// Since **Android 11** (not Android 10),
    /// you can create files and folders under this directory and read or write **only** them.  
    /// If not, you can do nothing with this path.
    /// 
    /// When using [`PublicImageDir`], use only image type for file name extension, 
    /// using other type extension or none may cause errors.
    /// Similarly, use only the corresponding extesions for [`PublicVideoDir`] and [`PublicAudioDir`].
    /// Only [`PublicGeneralPurposeDir`] supports all extensions and no extension. 
    /// 
    /// # Note
    /// Filesystem access via this path may be heavily impacted by emulation overhead.
    /// And those files will not be registered in MediaStore. 
    /// It might eventually be registered over time, but this should not be expected.
    /// As a result, it may not appear in gallery apps or photo picker tools.
    /// 
    /// You cannot access files created by other apps. 
    /// Additionally, if the app is uninstalled, 
    /// you will no longer be able to access the files you created, 
    /// even if the app is reinstalled.  
    /// Android tends to restrict public file access using paths, so this may stop working in the future.
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
    /// Android 10 (API level 29) or higher.  
    /// 
    /// Note :  
    /// - [`PublicAudioDir::Audiobooks`] is not available on Android 9 (API level 28) and lower.
    /// Availability on a given device can be verified by calling [`PublicStorage::is_audiobooks_dir_available`].  
    /// - [`PublicAudioDir::Recordings`] is not available on Android 11 (API level 30) and lower.
    /// Availability on a given device can be verified by calling [`PublicStorage::is_recordings_dir_available`].  
    /// - Others dirs are available in all Android versions.
    #[maybe_async]
    pub fn resolve_path(
        &self,
        volume_id: Option<&StorageVolumeId>,
        base_dir: impl Into<PublicDir>,
    ) -> Result<std::path::PathBuf> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().resolve_path(volume_id, base_dir).await
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
            self.impls().resolve_public_storage_initial_location(volume_id, base_dir, relative_path, create_dir_all).await
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
            self.impls().resolve_public_storage_initial_location_top(volume_id).await
        }
    }

    /// Verify whether the basic functions of PublicStorage 
    /// (such as [`PublicStorage::create_new_file`]) can be performed.
    /// 
    /// If on Android 9 (API level 28) and lower, this returns false.  
    /// If on Android 10 (API level 29) or higher, this returns true.  
    /// 
    /// # Support
    /// All Android version.
    #[always_sync]
    pub fn is_available(&self) -> Result<bool> {
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            Ok(api_level::ANDROID_10 <= self.impls().api_level()?)
        }
    }

    /// Verify whether [`PublicAudioDir::Audiobooks`] is available on a given device.   
    /// 
    /// If on Android 9 (API level 28) and lower, this returns false.  
    /// If on Android 10 (API level 29) or higher, this returns true.  
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
    /// If on Android 11 (API level 30) and lower, this returns false.  
    /// If on Android 12 (API level 31) or higher, this returns true.  
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
}