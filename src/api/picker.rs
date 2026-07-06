use sync_async::sync_async;
use crate::*;
use super::*;


/// API of File/Directory Picker.
/// 
/// # Examples
/// ```no_run
/// use tauri_plugin_android_fs::{AndroidFsExt, Entry, ImageFormat, PublicImageDir, Result, Size};
///
/// async fn file_picker_example(app: &tauri::AppHandle<impl tauri::Runtime>) -> Result<()> {
///     let api = app.android_fs_async();
///
///     let selected_files = api
///         .picker()
///         .pick_files(
///             None, // Initial location
///             &["*/*"], // Target MIME types
///             false, // If true, only local files can be selected
///         )
///         .await?;
///
///     if !selected_files.is_empty() {
///         for uri in selected_files {
///             let file: std::fs::File = api.open_file_readable(&uri).await?;
///             let file_path: tauri_plugin_fs::FilePath = uri.clone().into();
///
///             let file_type = api.get_mime_type(&uri).await?;
///             let file_name = api.get_name(&uri).await?;
///             let file_thumbnail = api.get_thumbnail(
///                 &uri,
///                 Size { width: 200, height: 200 },
///                 ImageFormat::Jpeg,
///             ).await?;
///         }
///     }
///     else {
///         // User cancelled the picker.
///     }
///
///     Ok(())
/// }
///
/// async fn file_saver_example(app: &tauri::AppHandle<impl tauri::Runtime>) -> Result<()> {
///     let api = app.android_fs_async();
///
///     // Initial directory when the file saver opens.
///     // Resolves to `~/Pictures/MyApp/2025-10-22/`.
///     let initial_location = api
///         .picker()
///         .resolve_public_storage_initial_location(
///             None, // Storage volume (e.g. internal storage or SD card). If `None`, uses the primary volume
///             PublicImageDir::Pictures, // Base directory
///             "MyApp/2025-10-22", // Relative path
///             true, // Creates missing directories
///         )
///         .await?;
///
///     let selected_file = api
///         .picker()
///         .save_file(
///             Some(&initial_location), // Initial location
///             "my-image.jpg", // Initial file name
///             Some("image/jpeg"), // MIME type
///             false, // If true, only local files can be selected
///         )
///         .await?;
///
///     if let Some(uri) = selected_file {
///         // Open the file for writing.
///         // 
///         // NOTE:
///         // Existing contents will be truncated.
///         let file: std::fs::File = api.open_file_writable(&uri).await?;
///     }
///     else {
///         // User cancelled the picker.
///     }
///
///     Ok(())
/// }
///
/// async fn dir_picker_example(app: &tauri::AppHandle<impl tauri::Runtime>) -> Result<()> {
///     let api = app.android_fs_async();
///
///     let selected = api
///         .picker()
///         .pick_dir(
///             None, // Initial location
///             false, // If true, restricts selection to directories on the local device
///         )
///         .await?;
///
///     if let Some(dir_uri) = selected {
///         // Persist access permission across app/device restarts.
///         api.picker().persist_uri_permission(&dir_uri).await?;
///
///         // Read the directory.
///         for entry in api.read_dir(&dir_uri).await? {
///             match entry {
///                 Entry::File { uri, name, .. } => {
///                     // Handle a file.
///                 }
///                 Entry::Dir { uri, name, .. } => {
///                     // Handle a directory.
///                 }
///             }
///         }
///
///         // Create a new file.
///         // Parent directories are created recursively if necessary.
///         let file_uri = api
///             .create_new_file(
///                 &dir_uri,
///                 "MyApp/2025-1021/file.txt",
///                 Some("text/plain"),
///             )
///             .await?;
///     }
///     else {
///         // User cancelled the picker.
///     }
///
///     Ok(())
/// }
/// ```
#[sync_async]
pub struct Picker<'a, R: tauri::Runtime> {
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
impl<'a, R: tauri::Runtime> Picker<'a, R> {
    
    #[always_sync]
    fn impls(&self) -> Impls<'_, R> {
        Impls { handle: &self.handle }
    }
}

#[sync_async(
    use(if_async) api_async::{AndroidFs, Opener, PrivateStorage, PublicStorage};
    use(if_sync) api_sync::{AndroidFs, Opener, PrivateStorage, PublicStorage};
)]
impl<'a, R: tauri::Runtime> Picker<'a, R> {

    /// Opens a system file picker and returns a **read-write** URIs.  
    /// If no file is selected or the user cancels, an empty vec is returned.  
    /// 
    /// By default, returned URI is valid until the app or device is terminated. 
    /// If you want to persist it across app or device restarts, use [`Picker::persist_uri_permission`].
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
    /// This URI works even without any permissions.  
    /// There is no need to use this if there is no special reason.  
    /// System will do its best to launch the dialog in the specified entry 
    /// if it's a directory, or the directory that contains the specified file if not.  
    /// If this is missing or failed to resolve the desired initial location, the initial location is system specific.  
    /// This must be a URI taken from following or it's derivative :   
    ///     - [`Picker::resolve_public_storage_initial_location`]
    ///     - [`Picker::resolve_initial_location`]
    ///     - [`Picker::pick_files`]
    ///     - [`Picker::pick_file`]
    ///     - [`Picker::pick_dir`]
    ///     - [`Picker::save_file`]
    /// 
    /// - ***mime_types*** :  
    /// The MIME types of the file to be selected.  
    /// However, there is no guarantee that the returned file will match the specified types.  
    /// If left empty, all file types will be available (equivalent to `["*/*"]`).  
    ///  
    /// - ***local_only*** :
    /// Indicates whether only entry located on the local device should be selectable, without requiring it to be downloaded from a remote service when opened.
    ///  
    /// # Support
    /// All Android versions supported by Tauri.
    /// 
    /// # References
    /// - <https://developer.android.com/reference/android/content/Intent#ACTION_OPEN_DOCUMENT>
    #[maybe_async]
    pub fn pick_files(
        &self,
        initial_location: Option<&FsUri>,
        mime_types: &[&str],
        local_only: bool,
    ) -> Result<Vec<FsUri>> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().show_pick_file_dialog(initial_location, mime_types, true, local_only).await
        }
    }

    /// Opens a system file picker and returns a **read-write** URI.  
    /// If no file is selected or the user cancels, None is returned.  
    /// 
    /// By default, returned URI is valid until the app or device is terminated. 
    /// If you want to persist it across app or device restarts, use [`Picker::persist_uri_permission`].
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
    /// This URI works even without any permissions.  
    /// There is no need to use this if there is no special reason.  
    /// System will do its best to launch the dialog in the specified entry 
    /// if it's a directory, or the directory that contains the specified file if not.  
    /// If this is missing or failed to resolve the desired initial location, the initial location is system specific.  
    /// This must be a URI taken from following or it's derivative :   
    ///     - [`Picker::resolve_public_storage_initial_location`]
    ///     - [`Picker::resolve_initial_location`]
    ///     - [`Picker::pick_files`]
    ///     - [`Picker::pick_file`]
    ///     - [`Picker::pick_dir`]
    ///     - [`Picker::save_file`]
    /// 
    /// - ***mime_types*** :  
    /// The MIME types of the file to be selected.  
    /// However, there is no guarantee that the returned file will match the specified types.  
    /// If left empty, all file types will be available (equivalent to `["*/*"]`).  
    ///  
    /// - ***local_only*** :
    /// Indicates whether only entry located on the local device should be selectable, without requiring it to be downloaded from a remote service when opened.
    ///  
    /// # Support
    /// All Android versions supported by Tauri.
    /// 
    /// # References
    /// - <https://developer.android.com/reference/android/content/Intent#ACTION_OPEN_DOCUMENT>
    #[maybe_async]
    pub fn pick_file(
        &self,
        initial_location: Option<&FsUri>,
        mime_types: &[&str],
        local_only: bool
    ) -> Result<Option<FsUri>> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().show_pick_file_dialog(initial_location, mime_types, false, local_only)
                .await
                .map(|mut i| i.pop())
        }
    }

    /// Opens a media picker and returns a **readonly** URIs.  
    /// If no file is selected or the user cancels, an empty vec is returned.  
    ///  
    /// By default, returned URI is valid until the app or device is terminated. 
    /// If you want to persist it across app or device restarts, use [`Picker::persist_uri_permission`].
    ///  
    /// This media picker provides a gallery, 
    /// sorted by date from newest to oldest. 
    /// 
    /// # Args  
    /// - ***target*** :  
    /// The media type of the file to be selected.  
    /// Images or videos, or both.  
    ///  
    /// - ***local_only*** :
    /// Indicates whether only entry located on the local device should be selectable, without requiring it to be downloaded from a remote service when opened.
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
    /// Availability on a given device can be verified by calling [`Picker::is_visual_media_picker_available`].  
    /// If not supported, this function behaves the same as [`Picker::pick_files`].  
    /// 
    /// # References
    /// - <https://developer.android.com/training/data-storage/shared/photopicker>
    #[maybe_async]
    pub fn pick_visual_medias(
        &self,
        target: VisualMediaTarget<'_>,
        local_only: bool
    ) -> Result<Vec<FsUri>> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().show_pick_visual_media_dialog(target, true, local_only).await
        }
    }

    /// Opens a media picker and returns a **readonly** URI.  
    /// If no file is selected or the user cancels, None is returned.  
    ///  
    /// By default, returned URI is valid until the app or device is terminated. 
    /// If you want to persist it across app or device restarts, use [`Picker::persist_uri_permission`].
    ///  
    /// This media picker provides a gallery, 
    /// sorted by date from newest to oldest. 
    /// 
    /// # Args  
    /// - ***target*** :  
    /// The media type of the file to be selected.  
    /// Images or videos, or both.  
    /// 
    /// - ***local_only*** :
    /// Indicates whether only entry located on the local device should be selectable, without requiring it to be downloaded from a remote service when opened.
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
    /// Availability on a given device can be verified by calling [`Picker::is_visual_media_picker_available`].  
    /// If not supported, this function behaves the same as [`Picker::pick_file`].  
    /// 
    /// # References
    /// - <https://developer.android.com/training/data-storage/shared/photopicker>
    #[maybe_async]
    pub fn pick_visual_media(
        &self,
        target: VisualMediaTarget<'_>,
        local_only: bool
    ) -> Result<Option<FsUri>> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().show_pick_visual_media_dialog(target, false, local_only)
                .await
                .map(|mut i| i.pop())
        }
    }

    /// Opens a file picker and returns a **readonly** URIs.  
    /// If no file is selected or the user cancels, an empty vec is returned.  
    ///  
    /// Returned URI is valid until the app or device is terminated. Can not persist it.
    /// 
    /// This works differently depending on the model and version.  
    /// Recent devices often have the similar behaviour as [`Picker::pick_visual_medias`] or [`Picker::pick_files`].  
    /// In older versions, third-party apps often handle request instead.
    /// 
    /// # Args  
    /// - ***mime_types*** :  
    /// The MIME types of the file to be selected.  
    /// However, there is no guarantee that the returned file will match the specified types.  
    /// If left empty, all file types will be available (equivalent to `["*/*"]`).  
    ///  
    /// # Support
    /// All Android versions supported by Tauri.
    /// 
    /// # References
    /// - <https://developer.android.com/reference/android/content/Intent#ACTION_GET_CONTENT>
    #[maybe_async]
    #[deprecated = "This may not support operations other than opening files."]
    pub fn pick_contents(
        &self,
        mime_types: &[&str],
    ) -> Result<Vec<FsUri>> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().show_pick_content_dialog(mime_types, true).await
        }
    }

    /// Opens a file picker and returns a **readonly** URI.  
    /// If no file is selected or the user cancels, None is returned.  
    ///  
    /// Returned URI is valid until the app or device is terminated. Can not persist it.
    /// 
    /// This works differently depending on the model and version.  
    /// Recent devices often have the similar behaviour as [`Picker::pick_visual_media`] or [`Picker::pick_file`].  
    /// In older versions, third-party apps often handle request instead.
    /// 
    /// # Args  
    /// - ***mime_types*** :  
    /// The MIME types of the file to be selected.  
    /// However, there is no guarantee that the returned file will match the specified types.  
    /// If left empty, all file types will be available (equivalent to `["*/*"]`).  
    ///  
    /// # Support
    /// All Android versions supported by Tauri.
    /// 
    /// # References
    /// - <https://developer.android.com/reference/android/content/Intent#ACTION_GET_CONTENT>
    #[maybe_async]
    #[deprecated = "This may not support operations other than opening files."]
    pub fn pick_content(
        &self,
        mime_types: &[&str],
    ) -> Result<Option<FsUri>> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().show_pick_content_dialog(mime_types, false)
                .await
                .map(|mut i| i.pop())
        }
    }

    /// Opens a system directory picker, allowing the creation of a new directory or the selection of an existing one, 
    /// and returns a **read-write** directory URI. 
    /// App can fully manage entries within the returned directory.  
    /// If no directory is selected or the user cancels, `None` is returned. 
    /// 
    /// By default, returned URI is valid until the app or device is terminated. 
    /// If you want to persist it across app or device restarts, use [`Picker::persist_uri_permission`].
    /// 
    /// This provides a standardized file explorer-style interface,
    /// and also allows directory selection from part of third-party apps or cloud storage.
    /// 
    /// # Args  
    /// - ***initial_location*** :  
    /// Indicate the initial location of dialog.    
    /// This URI works even without any permissions.  
    /// There is no need to use this if there is no special reason.  
    /// System will do its best to launch the dialog in the specified entry 
    /// if it's a directory, or the directory that contains the specified file if not.  
    /// If this is missing or failed to resolve the desired initial location, the initial location is system specific.   
    /// This must be a URI taken from following or it's derivative :   
    ///     - [`Picker::resolve_public_storage_initial_location`]
    ///     - [`Picker::resolve_initial_location`]
    ///     - [`Picker::pick_files`]
    ///     - [`Picker::pick_file`]
    ///     - [`Picker::pick_dir`]
    ///     - [`Picker::save_file`]
    /// 
    /// - ***local_only*** :
    /// Indicates whether only entry located on the local device should be selectable, without requiring it to be downloaded from a remote service when opened.
    ///  
    /// # Support
    /// All Android versions supported by Tauri.
    /// 
    /// # References
    /// - <https://developer.android.com/reference/android/content/Intent#ACTION_OPEN_DOCUMENT_TREE>
    #[maybe_async]
    pub fn pick_dir(
        &self,
        initial_location: Option<&FsUri>,
        local_only: bool
    ) -> Result<Option<FsUri>> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().show_pick_dir_dialog(initial_location, local_only).await
        }
    }

    /// Opens a system file saver and returns a **writeonly** URI.  
    /// The returned file may be a newly created file with no content,
    /// or it may be an existing file with the requested MIME type.  
    /// If the user cancels, `None` is returned. 
    /// 
    /// By default, returned URI is valid until the app or device is terminated. 
    /// If you want to persist it across app or device restarts, use [`Picker::persist_uri_permission`].
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
    /// This URI works even without any permissions.  
    /// There is no need to use this if there is no special reason.  
    /// System will do its best to launch the dialog in the specified entry 
    /// if it's a directory, or the directory that contains the specified file if not.  
    /// If this is missing or failed to resolve the desired initial location, the initial location is system specific.   
    /// This must be a URI taken from following or it's derivative :   
    ///     - [`Picker::resolve_public_storage_initial_location`]
    ///     - [`Picker::resolve_initial_location`]
    ///     - [`Picker::pick_files`]
    ///     - [`Picker::pick_file`]
    ///     - [`Picker::pick_dir`]
    ///     - [`Picker::save_file`]
    /// 
    /// - ***initial_file_name*** :  
    /// An initial file name.  
    /// The user may change this value before creating the file.  
    /// If no extension is present, 
    /// the system may infer one from ***mime_type*** and may append it to the file name. 
    /// But this append-extension operation depends on the model and version.
    /// 
    /// - ***mime_type*** :  
    /// The MIME type of the file to be saved.  
    /// If this is None, MIME type is inferred from the extension of ***initial_file_name*** (not file name by user input)
    /// and if that fails, `application/octet-stream` is used.  
    /// 
    /// - ***local_only*** :
    /// Indicates whether only entry located on the local device should be selectable.
    ///  
    /// # Support
    /// All Android versions supported by Tauri.
    /// 
    /// # References
    /// - <https://developer.android.com/reference/android/content/Intent#ACTION_CREATE_DOCUMENT>
    #[maybe_async]
    pub fn save_file(
        &self,
        initial_location: Option<&FsUri>,
        initial_file_name: impl AsRef<str>,
        mime_type: Option<&str>,
        local_only: bool
    ) -> Result<Option<FsUri>> {
        
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
           self.impls().show_save_file_dialog(initial_location, initial_file_name, mime_type, local_only).await 
        }
    }

    /// Builds the specified directory URI.  
    /// 
    /// This should only be used as `initial_location` in the file picker, such as [`Picker::pick_files`]. 
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
    /// - ***create_dir_all*** :  
    /// Creates directories if missing.  
    /// See [`PublicStorage::create_dir_all`].
    /// If error occurs, it will be ignored.
    ///  
    /// # Support
    /// All Android versions supported by Tauri.
    ///
    /// Note :  
    /// - [`PublicAudioDir::Audiobooks`] is not available on Android 9 (API level 28) and lower.
    /// Availability on a given device can be verified by calling [`PublicStorage::is_audiobooks_dir_available`].  
    /// - [`PublicAudioDir::Recordings`] is not available on Android 11 (API level 30) and lower.
    /// Availability on a given device can be verified by calling [`PublicStorage::is_recordings_dir_available`].  
    /// - Others dirs are available in all Android versions.
    #[maybe_async]
    pub fn resolve_public_storage_initial_location(
        &self,
        volume_id: Option<&StorageVolumeId>,
        base_dir: impl Into<PublicDir>,
        relative_path: impl AsRef<std::path::Path>,
        create_dir_all: bool
    ) -> Result<FsUri> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().resolve_initial_location_in_public_storage(volume_id, base_dir, relative_path, create_dir_all).await
        }
    }

    /// Builds the storage volume root URI.  
    /// 
    /// This should only be used as `initial_location` in the file picker, such as [`Picker::pick_files`]. 
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
    /// All Android versions supported by Tauri.
    #[maybe_async]
    pub fn resolve_initial_location(&self, volume_id: Option<&StorageVolumeId>) -> Result<FsUri> {
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().resolve_root_initial_location(volume_id).await
        }
    }

    /// Verify whether [`Picker::pick_visual_medias`] is available on a given device.
    /// 
    /// # Support
    /// All Android versions supported by Tauri.
    #[maybe_async]
    pub fn is_visual_media_picker_available(&self) -> Result<bool> {
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().is_visual_media_picker_available().await
        }
    }

    /// Check a URI permission granted by the file picker.  
    /// Returns false if there are no permissions.
    /// 
    /// # Args
    /// - **uri** :  
    /// URI of the target file or directory.  
    ///
    /// - **permission** :  
    /// The permission you want to check.  
    /// 
    /// 
    /// # Support
    /// All Android versions supported by Tauri.
    #[maybe_async]
    pub fn check_uri_permission(
        &self, 
        uri: &FsUri, 
        permission: UriPermission
    ) -> Result<bool> {
        
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().check_picker_uri_permission(uri, permission).await
        }
    }

    /// Take persistent permission to access the file, directory and its descendants.  
    /// This is a prolongation of an already acquired permission, not the acquisition of a new one.  
    /// 
    /// This works by just calling, without displaying any confirmation to the user.
    /// 
    /// Note that [there is a limit to the total number of URI that can be made persistent by this function.](https://stackoverflow.com/questions/71099575/should-i-release-persistableuripermission-when-a-new-storage-location-is-chosen/71100621#71100621)  
    /// Therefore, it is recommended to relinquish the unnecessary persisted URI by [`Picker::release_persisted_uri_permission`] or [`Picker::release_all_persisted_uri_permissions`].  
    /// Persisted permissions may be relinquished by other apps, user, or by moving/removing entries.
    /// So check by [`Picker::check_persisted_uri_permission`].  
    /// And you can retrieve the list of persisted uris using [`Picker::get_all_persisted_uri_permissions`].
    /// 
    /// # Args
    /// - **uri** :  
    /// URI of the target file or directory.   
    /// This must be a URI taken from following :  
    ///     - [`Picker::pick_files`]  
    ///     - [`Picker::pick_file`]  
    ///     - [`Picker::pick_visual_medias`]  
    ///     - [`Picker::pick_visual_media`]  
    ///     - [`Picker::pick_dir`]  
    ///     - [`Picker::save_file`]  
    ///     - [`AndroidFs::resolve_file_uri`], [`AndroidFs::resolve_dir_uri`], [`AndroidFs::read_dir`], [`AndroidFs::create_new_file`], [`AndroidFs::create_dir_all`] :  
    ///     If use URI from thoese fucntions, the permissions of the origin directory URI is persisted, not an entry iteself by this function. 
    ///     Because the permissions and validity period of the descendant entry URIs depend on the origin directory.   
    /// 
    /// # Support
    /// All Android versions supported by Tauri. 
    #[maybe_async]
    pub fn persist_uri_permission(&self, uri: &FsUri) -> Result<()> {
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().persist_picker_uri_permission(uri).await
        }
    }

    /// Check a persisted URI permission grant by [`Picker::persist_uri_permission`].  
    /// Returns false if there are only non-persistent permissions or no permissions.
    /// 
    /// # Args
    /// - **uri** :  
    /// URI of the target file or directory.  
    /// This must be a URI taken from following :  
    ///     - [`Picker::pick_files`]  
    ///     - [`Picker::pick_file`]  
    ///     - [`Picker::pick_visual_medias`]  
    ///     - [`Picker::pick_visual_media`]  
    ///     - [`Picker::pick_dir`]  
    ///     - [`Picker::save_file`]  
    ///     - [`AndroidFs::resolve_file_uri`], [`AndroidFs::resolve_dir_uri`], [`AndroidFs::read_dir`], [`AndroidFs::create_new_file`], [`AndroidFs::create_dir_all`] :  
    ///     If use URI from those functions, the permissions of the origin directory URI is checked, not an entry iteself by this function. 
    ///     Because the permissions and validity period of the descendant entry URIs depend on the origin directory.   
    /// 
    /// - **permission** :  
    /// The permission you want to check.  
    /// 
    /// # Support
    /// All Android versions supported by Tauri.
    #[maybe_async]
    pub fn check_persisted_uri_permission(
        &self, 
        uri: &FsUri, 
        permission: UriPermission
    ) -> Result<bool> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().check_persisted_picker_uri_permission(uri, permission).await
        }
    }

    /// Return list of all persisted URIs that have been persisted by [`Picker::persist_uri_permission`] and currently valid.   
    /// 
    /// # Support
    /// All Android versions supported by Tauri.
    #[maybe_async]
    pub fn get_all_persisted_uri_permissions(&self) -> Result<Vec<PersistedUriPermissionState>> {
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls()
                .get_all_persisted_picker_uri_permissions().await
                .map(|v| v.collect())
        }
    }

    /// Relinquish a persisted URI permission grant by [`Picker::persist_uri_permission`].   
    /// Non-persistent permissions are not released.  
    /// 
    /// Returns true if a persisted permission exists for the specified URI and was successfully released; 
    /// otherwise, returns false if no persisted permission existed in the first place.
    /// 
    /// # Args
    /// - ***uri*** :  
    /// URI of the target file or directory.  
    ///
    /// # Support
    /// All Android versions supported by Tauri.
    #[maybe_async]
    pub fn release_persisted_uri_permission(&self, uri: &FsUri) -> Result<bool> {
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().release_persisted_picker_uri_permission(uri).await
        }
    }

    /// Relinquish a all persisted uri permission grants by [`Picker::persist_uri_permission`].   
    /// Non-persistent permissions are not released.   
    /// 
    /// # Support
    /// All Android versions supported by Tauri.
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