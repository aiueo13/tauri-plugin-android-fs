use sync_async::sync_async;
use crate::*;
use super::*;


/// API of file storage intended for the app's use.  
/// 
/// # Examples
/// ```no_run
/// fn example(app: &tauri::AppHandle) {
///     use tauri_plugin_android_fs::AndroidFsExt as _;
/// 
///     let api = app.android_fs();
///     let outside_private_storage = api.private_storage().outside();
/// }
/// ```
#[sync_async]
pub struct AppStorage<'a, R: tauri::Runtime> {
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
impl<'a, R: tauri::Runtime> AppStorage<'a, R> {
    
    #[always_sync]
    fn impls(&self) -> Impls<'_, R> {
        Impls { handle: &self.handle }
    }
}

#[sync_async(
    use(if_async) api_async::{AndroidFs, FileOpener, FilePicker, PublicStorage, PrivateStorage};
    use(if_sync) api_sync::{AndroidFs, FileOpener, FilePicker, PublicStorage, PrivateStorage};
)]
impl<'a, R: tauri::Runtime> AppStorage<'a, R> {

    /// Gets a list of currently available storage volumes (internal storage, SD card, USB drive, etc.).
    /// Be aware of TOCTOU.
    /// 
    /// Since read-only SD cards and similar cases may be included, 
    /// please use [`StorageVolume { is_readonly, .. }`](StorageVolume) for filtering as needed.
    /// 
    /// This function returns only storage volume that is considered stable by system. 
    /// It includes device’s built-in storage and physical media slots under protective covers,
    /// but does not include storage volume considered temporary, 
    /// such as USB flash drives connected to handheld devices.
    /// 
    /// This typically includes [`primary storage volume`](AppStorage::get_primary_volume),
    /// but it may occasionally be absent if primary torage volume is inaccessible 
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
    /// All Android version.
    #[maybe_async]
    pub fn get_volumes(&self) -> Result<Vec<StorageVolume>> {
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().get_available_storage_volumes_for_app_storage().await
        }
    }

    /// Gets a primary storage volume.  
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
            self.impls().get_primary_storage_volume_if_available_for_app_storage().await
        }
    }

    /// Gets the absolute path of the app directory on the specified storage volume.  
    /// App can fully manage entries within this directory.  
    /// 
    /// This function does **not** create any directories; it only constructs the path.
    ///    
    /// These entries will be deleted when the app is uninstalled 
    /// and may also be deleted at the user’s initialising request.   
    /// 
    /// Since storage volume id and returned paths can change,
	/// only relative paths should be stored.
    /// 
    /// # Note
    /// ### About AppDir::Data and AppDir::Cache
    /// Since these locations may contain files created by other Tauri plugins or webview systems, 
    /// it is recommended to add a subdirectory with a unique name.
    ///
    /// If you are unsure between this function and [`PrivateStorage::resolve_path`] with [`PrivateDir::Data`] or [`PrivateDir::Cache`],
    /// you don’t need to use this one.  
    /// The difference from it is that these files may be accessed by user or other apps that have permissions,
    /// and it cannot always be available since removable storage can be ejected.  
    /// 
    /// One advantage of using this is that it allows storing large app-specific data/cache on SD cards or other supplementary storage, 
    /// which can be useful on older devices with limited built-in storage capacity. 
    /// However on modern devices, the built-in storage capacity is relatively large,
    /// and there is little advantage in using this.  
    /// 
    /// By using [`StorageVolume { is_emulated, .. }`](StorageVolume), 
    /// you can determine whether this belongs to the same storage volume as [`PrivateStorage::resolve_path`]. 
    /// If it is `true`, there is no advantage in using this instead of [`PrivateStorage::resolve_path`]. 
    /// It only reduces security.
    /// 
    /// ### About AppDir::PublicMedia
    /// This is a location for storing media files shared with other apps nad user on older versions of Android. 
    /// For Android 11 (API level 30) or higher, 
    /// it has been marked as deprecated. 
    /// It still works, but you should consider migrating to [`PublicStorage`].
    /// 
    /// This is a location that is unfamiliar to the user, 
    /// but calling [`AppStorage::scan_public_media_by_path`] will make it 
    /// displayed in a more user-friendly way in gallery apps and file managers.  
    /// 
    /// For file in this directory, do not use operations such as rename or remove that rely on paths 
    /// (including URIs obtained via [`FileUri::from_path`] with this paths), 
    /// as they may break consistency with the MediaStore on old version.
    /// Instead, use the URI obtained through [`AppStorage::scan_public_media_by_path`] together with methods 
    /// such as [`AndroidFs::rename`] or [`AndroidFs::remove_file`].
    /// 
    /// # Args
    /// - ***volume_id*** :  
    /// ID of the storage volume, such as internal storage, SD card, etc.  
    /// If `None` is provided, [`the primary storage volume`](AppStorage::get_primary_volume) will be used.  
    /// 
    /// # Support
    /// All Android version. 
    #[maybe_async]
    pub fn resolve_path(
        &self, 
        volume_id: Option<&StorageVolumeId>,
        dir: AppDir
    ) -> Result<std::path::PathBuf> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().resolve_dir_path_in_app_storage(volume_id, dir).await
        }
    }

    /// See [`AppStorage::resolve_path`] and [`FileUri::from_path`].
    #[maybe_async]
    pub fn resolve_uri(
        &self, 
        volume_id: Option<&StorageVolumeId>,
        dir: AppDir,
        relative_path: impl AsRef<std::path::Path>
    ) -> Result<FileUri> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            let mut path = self.resolve_path(volume_id, dir).await?;
            path.push(relative_path.as_ref());
            Ok(path.into())
        }
    }

    /// Scans the specified file in MediaStore and returns it's URI if success.   
    /// By doing this, the file will be visible in the Gallery and etc.
    ///
    /// # Args
    /// - ***uri*** :  
    /// Absolute path of the target file.   
    /// This must be a path obtained from [`AppStorage::resolve_path`] with [`AppDir::PublicMedia`]
    /// and it's descendants path.
    /// 
    /// - ***mime_type*** :  
    /// The MIME type of the file.  
    /// If `None`, the MIME type will be inferred from the extension of the path.  
    /// If that also fails, `application/octet-stream` will be used.
    /// 
    /// # Support
    /// All Android version.
    #[maybe_async]
    pub fn scan_public_media_by_path(
        &self,
        path: impl AsRef<std::path::Path>,
        mime_type: Option<&str>,
    ) -> Result<FileUri> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().scan_file_to_media_store_by_path(path, mime_type).await
        }
    }

    /// Gets the absolute path of the specified file.
    /// 
    /// # Args
    /// - ***uri*** :   
    /// Target file URI.
    /// This must be a URI obtained from [`AppStorage::scan_public_media_by_path`].
    /// 
    /// # Support
    /// All Android version.
    #[maybe_async]
    pub fn get_public_media_path(
        &self,
        uri: &FileUri,
    ) -> Result<std::path::PathBuf> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().get_public_media_file_path_in_app_storage(uri).await
        }
    }
}