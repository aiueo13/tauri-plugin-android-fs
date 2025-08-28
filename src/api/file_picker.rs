use crate::*;


/// API of file/dir picker.
/// 
/// # Examples
/// ```
/// fn example(app: &tauri::AppHandle) {
///     use tauri_plugin_android_fs::AndroidFsExt as _;
/// 
///     let api = app.android_fs();
///     let file_picker = api.file_picker();
/// }
/// ```
pub struct FilePicker<'a, R: tauri::Runtime>(pub(crate) &'a AndroidFs<R>);

impl<'a, R: tauri::Runtime> FilePicker<'a, R> {

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
    /// This URI works even without any permissions.  
    /// There is no need to use this if there is no special reason.  
    /// System will do its best to launch the dialog in the specified entry 
    /// if it's a directory, or the directory that contains the specified file if not.  
    /// If this is missing or failed to resolve the desired initial location, the initial location is system specific.  
    /// This must be a URI taken from following :   
    ///     - [`AndroidFs::resolve_initial_location`]
    ///     - [`AndroidFs::try_resolve_file_uri`]
    ///     - [`AndroidFs::try_resolve_dir_uri`]
    ///     - [`AndroidFs::resolve_uri`]
    ///     - [`AndroidFs::read_dir`]
    ///     - [`AndroidFs::create_file`]
    ///     - [`AndroidFs::create_dir_all`]
    ///     - [`AndroidFs::rename`]
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
    /// - ***multiple*** :  
    /// Indicates whether multiple file selection is allowed.  
    /// 
    /// # Support
    /// All.
    /// 
    /// # References
    /// <https://developer.android.com/reference/android/content/Intent#ACTION_OPEN_DOCUMENT>
    pub fn pick_files(
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
    
            let _guard = self.0.intent_lock.lock();
            self.0.api
                .run_mobile_plugin::<Res>("showOpenFileDialog", Req { mime_types, multiple, initial_location })
                .map(|v| v.uris)
                .map_err(Into::into)
        })
    }

    /// Opens a system file picker and returns a **read-write** URI.  
    /// If no file is selected or the user cancels, None is returned.  
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
    /// This URI works even without any permissions.  
    /// There is no need to use this if there is no special reason.  
    /// System will do its best to launch the dialog in the specified entry 
    /// if it's a directory, or the directory that contains the specified file if not.  
    /// If this is missing or failed to resolve the desired initial location, the initial location is system specific.  
    /// This must be a URI taken from following :   
    ///     - [`AndroidFs::resolve_initial_location`]
    ///     - [`AndroidFs::try_resolve_file_uri`]
    ///     - [`AndroidFs::try_resolve_dir_uri`]
    ///     - [`AndroidFs::resolve_uri`]
    ///     - [`AndroidFs::read_dir`]
    ///     - [`AndroidFs::create_file`]
    ///     - [`AndroidFs::create_dir_all`]
    ///     - [`AndroidFs::rename`]
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
    /// All.
    /// 
    /// # References
    /// <https://developer.android.com/reference/android/content/Intent#ACTION_OPEN_DOCUMENT>
    pub fn pick_file(
        &self,
        initial_location: Option<&FileUri>,
        mime_types: &[&str],
    ) -> crate::Result<Option<FileUri>> {

        self.pick_files(initial_location, mime_types, false).map(|mut f| f.pop())
    }

    /// Opens a media picker and returns a **readonly** URIs.  
    /// If no file is selected or the user cancels, an empty vec is returned.  
    ///  
    /// By default, returned URI is valid until the app is terminated. 
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
    /// Availability on a given device can be verified by calling [`FilePicker::is_visual_media_picker_available`].  
    /// If not supported, this function behaves the same as [`FilePicker::pick_files`].  
    /// 
    /// # References
    /// <https://developer.android.com/training/data-storage/shared/photopicker>
    pub fn pick_visual_medias(
        &self,
        target: VisualMediaTarget,
        multiple: bool,
    ) -> crate::Result<Vec<FileUri>> {

        on_android!({
            impl_se!(struct Req { multiple: bool, target: VisualMediaTarget });
            impl_de!(struct Res { uris: Vec<FileUri> });
    
            let _guard = self.0.intent_lock.lock();
            self.0.api
                .run_mobile_plugin::<Res>("showOpenVisualMediaDialog", Req { multiple, target })
                .map(|v| v.uris)
                .map_err(Into::into)
        })
    }

    /// Opens a media picker and returns a **readonly** URI.  
    /// If no file is selected or the user cancels, None is returned.  
    ///  
    /// By default, returned URI is valid until the app is terminated. 
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
    /// <https://developer.android.com/training/data-storage/shared/photopicker>
    pub fn pick_visual_media(
        &self,
        target: VisualMediaTarget,
    ) -> crate::Result<Option<FileUri>> {

        self.pick_visual_medias(target, false).map(|mut f| f.pop())
    }

    /// Opens a file picker and returns a **readonly** URIs.  
    /// If no file is selected or the user cancels, an empty vec is returned.  
    ///  
    /// Returned URI is valid until the app is terminated. Can not persist it.
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
    /// - ***multiple*** :  
    /// Indicates whether multiple file selection is allowed.  
    /// 
    /// # Support
    /// All.
    /// 
    /// # References
    /// <https://developer.android.com/reference/android/content/Intent#ACTION_GET_CONTENT>
    pub fn pick_contents(
        &self,
        mime_types: &[&str],
        multiple: bool
    ) -> crate::Result<Vec<FileUri>> {

        on_android!({
            impl_se!(struct Req<'a> { mime_types: &'a [&'a str], multiple: bool });
            impl_de!(struct Res { uris: Vec<FileUri> });

            let _guard = self.0.intent_lock.lock();
            self.0.api
                .run_mobile_plugin::<Res>("showOpenContentDialog", Req { mime_types, multiple })
                .map(|v| v.uris)
                .map_err(Into::into)
        })
    }

    /// Opens a file picker and returns a **readonly** URI.  
    /// If no file is selected or the user cancels, None is returned.  
    ///  
    /// Returned URI is valid until the app is terminated. Can not persist it.
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
    /// All.
    /// 
    /// # References
    /// <https://developer.android.com/reference/android/content/Intent#ACTION_GET_CONTENT>
    pub fn pick_content(
        &self,
        mime_types: &[&str],
    ) -> crate::Result<Option<FileUri>> {

        self.pick_contents(mime_types, false).map(|mut f| f.pop())
    }

    /// Opens a system directory picker, allowing the creation of a new directory or the selection of an existing one, 
    /// and returns a **read-write** directory URI. 
    /// App can fully manage entries within the returned directory.  
    /// If no directory is selected or the user cancels, `None` is returned. 
    /// 
    /// By default, returned URI is valid until the app is terminated. 
    /// If you want to persist it across app restarts, use [`AndroidFs::take_persistable_uri_permission`].
    /// 
    /// This provides a standardized file explorer-style interface,
    /// and also allows file selection from part of third-party apps or cloud storage.
    /// 
    /// # Args  
    /// - ***initial_location*** :  
    /// Indicate the initial location of dialog.    
    /// This URI works even without any permissions.  
    /// There is no need to use this if there is no special reason.  
    /// System will do its best to launch the dialog in the specified entry 
    /// if it's a directory, or the directory that contains the specified file if not.  
    /// If this is missing or failed to resolve the desired initial location, the initial location is system specific.   
    /// This must be a URI taken from following :   
    ///     - [`AndroidFs::resolve_initial_location`]
    ///     - [`AndroidFs::try_resolve_file_uri`]
    ///     - [`AndroidFs::try_resolve_dir_uri`]
    ///     - [`AndroidFs::resolve_uri`]
    ///     - [`AndroidFs::read_dir`]
    ///     - [`AndroidFs::create_file`]
    ///     - [`AndroidFs::create_dir_all`]
    ///     - [`AndroidFs::rename`]
    ///     - [`FilePicker::pick_files`]
    ///     - [`FilePicker::pick_file`]
    ///     - [`FilePicker::pick_dir`]
    ///     - [`FilePicker::save_file`]
    /// 
    /// # Support
    /// All.
    /// 
    /// # References
    /// <https://developer.android.com/reference/android/content/Intent#ACTION_OPEN_DOCUMENT_TREE>
    pub fn pick_dir(
        &self,
        initial_location: Option<&FileUri>,
    ) -> crate::Result<Option<FileUri>> {

        on_android!({
            impl_se!(struct Req<'a> { initial_location: Option<&'a FileUri> });
            impl_de!(struct Res { uri: Option<FileUri> });

            let _guard = self.0.intent_lock.lock();
            self.0.api
                .run_mobile_plugin::<Res>("showManageDirDialog", Req { initial_location })
                .map(|v| v.uri)
                .map_err(Into::into)
        })
    }

    /// Opens a system file saver and returns a **writeonly** URI.  
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
    /// This URI works even without any permissions.  
    /// There is no need to use this if there is no special reason.  
    /// System will do its best to launch the dialog in the specified entry 
    /// if it's a directory, or the directory that contains the specified file if not.  
    /// If this is missing or failed to resolve the desired initial location, the initial location is system specific.   
    /// This must be a URI taken from following :   
    ///     - [`AndroidFs::resolve_initial_location`]
    ///     - [`AndroidFs::try_resolve_file_uri`]
    ///     - [`AndroidFs::try_resolve_dir_uri`]
    ///     - [`AndroidFs::resolve_uri`]
    ///     - [`AndroidFs::read_dir`]
    ///     - [`AndroidFs::create_file`]
    ///     - [`AndroidFs::create_dir_all`]
    ///     - [`AndroidFs::rename`]
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
    /// All.
    /// 
    /// # References
    /// <https://developer.android.com/reference/android/content/Intent#ACTION_CREATE_DOCUMENT>
    pub fn save_file(
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
        
            let _guard = self.0.intent_lock.lock();
            self.0.api
                .run_mobile_plugin::<Res>("showSaveFileDialog", Req { initial_file_name, mime_type, initial_location })
                .map(|v| v.uri)
                .map_err(Into::into)
        })
    }

    /// Verify whether [`FilePicker::pick_visual_medias`] is available on a given device.
    /// 
    /// # Support
    /// All.
    pub fn is_visual_media_picker_available(&self) -> crate::Result<bool> {
        on_android!({
            impl_de!(struct Res { value: bool });

            self.0.api
                .run_mobile_plugin::<Res>("isVisualMediaDialogAvailable", "")
                .map(|v| v.value)
                .map_err(Into::into)
        })
    }
}