use crate::*;


/// API of file storage that is available to other applications and users.  
/// 
/// # Examples
/// ```
/// fn example(app: &tauri::AppHandle) {
///     use tauri_plugin_android_fs::AndroidFsExt as _;
/// 
///     let api = app.android_fs();
///     let public_storage = api.public_storage();
/// }
/// ```
pub struct PublicStorage<'a, R: tauri::Runtime>(pub(crate) &'a AndroidFs<R>);

impl<'a, R: tauri::Runtime> PublicStorage<'a, R> {

    /// See [`PublicStorage::create_file_in_public_dir`] for description.  
    /// 
    /// This is the same as following: 
    /// ```ignore
    /// create_file_in_public_dir(
    ///     dir,
    ///     format!("{app_name}/{relative_path}"),
    ///     mime_type
    /// );
    /// ```
    pub fn create_file_in_public_app_dir(
        &self,
        dir: impl Into<PublicDir>,
        relative_path: impl AsRef<str>, 
        mime_type: Option<&str>
    ) -> crate::Result<FileUri> {

        on_android!({
            let config = self.0.app.config();
            let app_name = config.product_name.as_deref().unwrap_or("");
            let app_name = match app_name.is_empty() {
                true => &config.identifier,
                false => app_name
            };
            let app_name = app_name.replace('/', " ");
            let relative_path = relative_path.as_ref().trim_start_matches('/');
            let relative_path_with_subdir = format!("{app_name}/{relative_path}");

            self.create_file_in_public_dir(dir, relative_path_with_subdir, mime_type)
        })
    }

    /// Creates a new empty file in the specified public directory
    /// and returns a **persistent read-write** URI.  
    ///  
    /// The created file has following features :   
    /// - Will be registered with the corresponding MediaStore as needed.  
    /// - Always supports remove.
    /// - Not removed when the app is uninstalled.
    /// 
    /// # Args
    /// - ***dir*** :  
    /// The base directory.  
    /// When using [`PublicImageDir`], use only image MIME types for ***mime_type***, which is discussed below.; using other types may cause errors.
    /// Similarly, use only the corresponding media types for [`PublicVideoDir`] and [`PublicAudioDir`].
    /// Only [`PublicGeneralPurposeDir`] supports all MIME types. 
    ///  
    /// - ***relative_path_with_subdir*** :  
    /// The file path relative to the base directory.  
    /// If a file with the same name already exists, a sequential number will be appended to ensure uniqueness.  
    /// Any missing subdirectories in the specified path will be created automatically.  
    /// Please specify a subdirectory in this, 
    /// such as `MyApp/file.txt` or `MyApp/2025-2-11/file.txt`. Do not use `file.txt`.  
    /// As shown above, it is customary to specify the app name at the beginning of the subdirectory, 
    /// and in this case, using [`PublicStorage::create_file_in_public_app_dir`] is recommended.
    ///  
    /// - ***mime_type*** :  
    /// The MIME type of the file to be created.  
    /// If this is None, MIME type is inferred from the extension of ***relative_path_with_subdir***
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
    pub fn create_file_in_public_dir(
        &self,
        dir: impl Into<PublicDir>,
        relative_path_with_subdir: impl AsRef<str>, 
        mime_type: Option<&str>
    ) -> crate::Result<FileUri> {

        on_android!({
            impl_se!(struct Req<'a> { dir: PublicDir, dir_type: &'a str });
            impl_de!(struct Res { name: String, uri: String });

            let dir = dir.into();
            let dir_type = match dir {
                PublicDir::Image(_) => "Image",
                PublicDir::Video(_) => "Video",
                PublicDir::Audio(_) => "Audio",
                PublicDir::GeneralPurpose(_) => "GeneralPurpose",
            };

            let (dir_name, dir_parent_uri) = self.0.api
                .run_mobile_plugin::<Res>("getPublicDirInfo", Req { dir, dir_type })
                .map(|v| (v.name, v.uri))?;
        
            let relative_path = relative_path_with_subdir.as_ref().trim_start_matches('/');
            let relative_path = format!("{dir_name}/{relative_path}");

            let dir_parent_uri = FileUri {
                uri: dir_parent_uri,
                document_top_tree_uri: None
            };

            self.0.create_file(&dir_parent_uri, relative_path, mime_type)
        })
    }

    /// Verify whether [`PublicAudioDir::Audiobooks`] is available on a given device.
    /// 
    /// # Support
    /// All.
    pub fn is_audiobooks_dir_available(&self) -> crate::Result<bool> {
        on_android!({
            impl_de!(struct Res { value: bool });

            self.0.api
                .run_mobile_plugin::<Res>("isAudiobooksDirAvailable", "")
                .map(|v| v.value)
                .map_err(Into::into)
        })
    }

    /// Verify whether [`PublicAudioDir::Recordings`] is available on a given device.
    /// 
    /// # Support
    /// All.
    pub fn is_recordings_dir_available(&self) -> crate::Result<bool> {
        on_android!({
            impl_de!(struct Res { value: bool });

            self.0.api
                .run_mobile_plugin::<Res>("isRecordingsDirAvailable", "")
                .map(|v| v.value)
                .map_err(Into::into)
        })
    }
}