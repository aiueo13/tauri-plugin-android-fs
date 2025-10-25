use sync_async::sync_async;
use crate::*;
use super::*;


/// API of file/dir picker.
/// 
/// # Examples
/// ```no_run
/// fn example(app: &tauri::AppHandle) {
///     use tauri_plugin_android_fs::AndroidFsExt as _;
/// 
///     let api = app.android_fs();
///     let file_picker = api.file_picker();
/// }
/// ```
#[sync_async]
pub struct FilePicker<'a, R: tauri::Runtime> {
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
impl<'a, R: tauri::Runtime> FilePicker<'a, R> {
    
    #[always_sync]
    fn impls(&self) -> Impls<'_, R> {
        Impls { handle: &self.handle }
    }
}

#[sync_async(
    use(if_async) api_async::{AndroidFs, FileOpener, PrivateStorage, PublicStorage, WritableStream};
    use(if_sync) api_sync::{AndroidFs, FileOpener, PrivateStorage, PublicStorage, WritableStream};
)]
impl<'a, R: tauri::Runtime> FilePicker<'a, R> {

    /// Opens a system file picker and returns a **read-write** URIs.  
    /// If no file is selected or the user cancels, an empty vec is returned.  
    /// 
    /// By default, returned URI is valid until the app or device is terminated. 
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
    /// This URI works even without any permissions.  
    /// There is no need to use this if there is no special reason.  
    /// System will do its best to launch the dialog in the specified entry 
    /// if it's a directory, or the directory that contains the specified file if not.  
    /// If this is missing or failed to resolve the desired initial location, the initial location is system specific.  
    /// This must be a URI taken from following or it's derivative :   
    ///     - [`PublicStorage::resolve_initial_location`]
    ///     - [`PublicStorage::resolve_initial_location_top`]
    ///     - [`FilePicker::pick_files`]
    ///     - [`FilePicker::pick_file`]
    ///     - [`FilePicker::pick_dir`]
    ///     - [`FilePicker::save_file`]
    /// 
    /// - ***mime_types*** :  
    /// The MIME types of the file to be selected.  
    /// However, there is no guarantee that the returned file will match the specified types.  
    /// If left empty, all file types will be available (equivalent to `["*/*"]`).  
    ///  
    /// # Support
    /// All Android version.
    /// 
    /// # References
    /// - <https://developer.android.com/reference/android/content/Intent#ACTION_OPEN_DOCUMENT>
    #[maybe_async]
    pub fn pick_files(
        &self,
        initial_location: Option<&FileUri>,
        mime_types: &[&str],
    ) -> Result<Vec<FileUri>> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().show_pick_file_dialog(initial_location, mime_types, true).await
        }
    }

    /// Opens a system file picker and returns a **read-write** URI.  
    /// If no file is selected or the user cancels, None is returned.  
    /// 
    /// By default, returned URI is valid until the app or device is terminated. 
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
    /// This URI works even without any permissions.  
    /// There is no need to use this if there is no special reason.  
    /// System will do its best to launch the dialog in the specified entry 
    /// if it's a directory, or the directory that contains the specified file if not.  
    /// If this is missing or failed to resolve the desired initial location, the initial location is system specific.  
    /// This must be a URI taken from following or it's derivative :   
    ///     - [`PublicStorage::resolve_initial_location`]
    ///     - [`PublicStorage::resolve_initial_location_top`]
    ///     - [`FilePicker::pick_files`]
    ///     - [`FilePicker::pick_file`]
    ///     - [`FilePicker::pick_dir`]
    ///     - [`FilePicker::save_file`]
    /// 
    /// - ***mime_types*** :  
    /// The MIME types of the file to be selected.  
    /// However, there is no guarantee that the returned file will match the specified types.  
    /// If left empty, all file types will be available (equivalent to `["*/*"]`).  
    ///  
    /// # Support
    /// All Android version.
    /// 
    /// # References
    /// - <https://developer.android.com/reference/android/content/Intent#ACTION_OPEN_DOCUMENT>
    #[maybe_async]
    pub fn pick_file(
        &self,
        initial_location: Option<&FileUri>,
        mime_types: &[&str],
    ) -> Result<Option<FileUri>> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().show_pick_file_dialog(initial_location, mime_types, false)
                .await
                .map(|mut i| i.pop())
        }
    }

    /// Opens a media picker and returns a **readonly** URIs.  
    /// If no file is selected or the user cancels, an empty vec is returned.  
    ///  
    /// By default, returned URI is valid until the app or device is terminated. 
    /// If you want to persist it across app restarts, use [`AndroidFs::take_persistable_uri_permission`].
    ///  
    /// This media picker provides a gallery, 
    /// sorted by date from newest to oldest. 
    /// 
    /// # Args  
    /// - ***target*** :  
    /// The media type of the file to be selected.  
    /// Images or videos, or both.  
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
    /// Availability on a given device can be verified by calling [`FilePicker::is_visual_media_picker_available`].  
    /// If not supported, this function behaves the same as [`FilePicker::pick_files`].  
    /// 
    /// # References
    /// - <https://developer.android.com/training/data-storage/shared/photopicker>
    #[maybe_async]
    pub fn pick_visual_medias(
        &self,
        target: VisualMediaTarget<'_>,
    ) -> Result<Vec<FileUri>> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().show_pick_visual_media_dialog(target, true).await
        }
    }

    /// Opens a media picker and returns a **readonly** URI.  
    /// If no file is selected or the user cancels, None is returned.  
    ///  
    /// By default, returned URI is valid until the app or device is terminated. 
    /// If you want to persist it across app restarts, use [`AndroidFs::take_persistable_uri_permission`].
    ///  
    /// This media picker provides a gallery, 
    /// sorted by date from newest to oldest. 
    /// 
    /// # Args  
    /// - ***target*** :  
    /// The media type of the file to be selected.  
    /// Images or videos, or both.  
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
    /// Availability on a given device can be verified by calling [`FilePicker::is_visual_media_picker_available`].  
    /// If not supported, this function behaves the same as [`FilePicker::pick_file`].  
    /// 
    /// # References
    /// - <https://developer.android.com/training/data-storage/shared/photopicker>
    #[maybe_async]
    pub fn pick_visual_media(
        &self,
        target: VisualMediaTarget<'_>,
    ) -> Result<Option<FileUri>> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().show_pick_visual_media_dialog(target, false)
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
    /// Recent devices often have the similar behaviour as [`FilePicker::pick_visual_medias`] or [`FilePicker::pick_files`].  
    /// In older versions, third-party apps often handle request instead.
    /// 
    /// # Args  
    /// - ***mime_types*** :  
    /// The MIME types of the file to be selected.  
    /// However, there is no guarantee that the returned file will match the specified types.  
    /// If left empty, all file types will be available (equivalent to `["*/*"]`).  
    ///  
    /// # Support
    /// All Android version.
    /// 
    /// # References
    /// - <https://developer.android.com/reference/android/content/Intent#ACTION_GET_CONTENT>
    #[maybe_async]
    pub fn pick_contents(
        &self,
        mime_types: &[&str],
    ) -> Result<Vec<FileUri>> {

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
    /// Recent devices often have the similar behaviour as [`FilePicker::pick_visual_media`] or [`FilePicker::pick_file`].  
    /// In older versions, third-party apps often handle request instead.
    /// 
    /// # Args  
    /// - ***mime_types*** :  
    /// The MIME types of the file to be selected.  
    /// However, there is no guarantee that the returned file will match the specified types.  
    /// If left empty, all file types will be available (equivalent to `["*/*"]`).  
    ///  
    /// # Support
    /// All Android version.
    /// 
    /// # References
    /// - <https://developer.android.com/reference/android/content/Intent#ACTION_GET_CONTENT>
    #[maybe_async]
    pub fn pick_content(
        &self,
        mime_types: &[&str],
    ) -> Result<Option<FileUri>> {

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
    /// If you want to persist it across app restarts, use [`AndroidFs::take_persistable_uri_permission`].
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
    ///     - [`PublicStorage::resolve_initial_location`]
    ///     - [`PublicStorage::resolve_initial_location_top`]
    ///     - [`FilePicker::pick_files`]
    ///     - [`FilePicker::pick_file`]
    ///     - [`FilePicker::pick_dir`]
    ///     - [`FilePicker::save_file`]
    /// 
    /// # Support
    /// All Android version.
    /// 
    /// # References
    /// - <https://developer.android.com/reference/android/content/Intent#ACTION_OPEN_DOCUMENT_TREE>
    #[maybe_async]
    pub fn pick_dir(
        &self,
        initial_location: Option<&FileUri>,
    ) -> Result<Option<FileUri>> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().show_pick_dir_dialog(initial_location).await
        }
    }

    /// Opens a system file saver and returns a **writeonly** URI.  
    /// The returned file may be a newly created file with no content,
    /// or it may be an existing file with the requested MIME type.  
    /// If the user cancels, `None` is returned. 
    /// 
    /// By default, returned URI is valid until the app or device is terminated. 
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
    /// This URI works even without any permissions.  
    /// There is no need to use this if there is no special reason.  
    /// System will do its best to launch the dialog in the specified entry 
    /// if it's a directory, or the directory that contains the specified file if not.  
    /// If this is missing or failed to resolve the desired initial location, the initial location is system specific.   
    /// This must be a URI taken from following or it's derivative :   
    ///     - [`PublicStorage::resolve_initial_location`]
    ///     - [`PublicStorage::resolve_initial_location_top`]
    ///     - [`FilePicker::pick_files`]
    ///     - [`FilePicker::pick_file`]
    ///     - [`FilePicker::pick_dir`]
    ///     - [`FilePicker::save_file`]
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
    /// # Support
    /// All Android version.
    /// 
    /// # References
    /// - <https://developer.android.com/reference/android/content/Intent#ACTION_CREATE_DOCUMENT>
    #[maybe_async]
    pub fn save_file(
        &self,
        initial_location: Option<&FileUri>,
        initial_file_name: impl AsRef<str>,
        mime_type: Option<&str>,
    ) -> Result<Option<FileUri>> {
        
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
           self.impls().show_save_file_dialog(initial_location, initial_file_name, mime_type).await 
        }
    }

    /// Verify whether [`FilePicker::pick_visual_medias`] is available on a given device.
    /// 
    /// # Support
    /// All Android version.
    #[maybe_async]
    pub fn is_visual_media_picker_available(&self) -> Result<bool> {
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().is_visual_media_picker_available().await
        }
    }
}