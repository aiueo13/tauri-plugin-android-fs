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
pub struct PublicStorage<'a, R: tauri::Runtime>(
    #[allow(unused)]
    pub(crate) &'a AndroidFs<R>
);

impl<'a, R: tauri::Runtime> PublicStorage<'a, R> {

    /// Gets a list of currently available storage volumes (internal storage, SD card, USB drive, etc.).
    /// Be aware of TOCTOU.
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
    /// each user may have a different view for the same physical volume
    /// (e.g. when the volume is a built-in emulated storage).
    /// 
    /// # Support
    /// Android 10 (API level 29) or higher.  
    pub fn get_available_volumes(&self) -> Result<Vec<PublicStorageVolume>> {
        on_android!({
            impl_de!(struct Obj { volume_name: String, description: Option<String>, is_primary: bool });
            impl_de!(struct Res { volumes: Vec<Obj> });

            let mut volumes = self.0.api
                .run_mobile_plugin::<Res>("getStorageVolumes", "")
                .map(|v| v.volumes.into_iter())
                .map(|v| v.map(|v| 
                    PublicStorageVolume {
                        description: v.description.unwrap_or_else(|| v.volume_name.clone()),
                        id: PublicStorageVolumeId(v.volume_name),
                        is_primary: v.is_primary,
                    }
                ))
                .map(|v| v.collect::<Vec<_>>())?;

            // is_primary を先頭にする。他はそのままの順序
            volumes.sort_by(|a, b| b.is_primary.cmp(&a.is_primary));

            Ok(volumes)
        })
    }

    /// Gets a primary storage volume.  
    /// This is the most common and recommended storage volume for placing files that can be accessed by other apps or user.
    /// 
    /// A device always has one (and one only) primary storage volume.  
    /// 
    /// The returned primary volume may not currently be accessible 
    /// if it has been mounted by the user on their computer, 
    /// has been removed from the device, or some other problem has happened.  
    /// 
    /// You can find a list of all currently available volumes using [`PublicStorage::get_available_volumes`].
    /// 
    /// # Note
    /// The volume represents the logical view of a storage volume for an individual user:
    /// each user may have a different view for the same physical volume
    /// (e.g. when the volume is a built-in emulated storage).
    /// 
    /// The primary volume provides a separate area for each user in a multi-user environment.
    /// 
    /// # Support
    /// Android 10 (API level 29) or higher.   
    /// 
    /// # References
    /// <https://developer.android.com/reference/android/provider/MediaStore#VOLUME_EXTERNAL_PRIMARY>
    pub fn get_primary_volume(&self) -> Result<PublicStorageVolume> {
        on_android!({
            impl_de!(struct Res { volume_name: String, description: Option<String>, is_primary: bool });

            self.0.api
                .run_mobile_plugin::<Res>("getPrimaryStorageVolume", "")
                .map(|v| 
                    PublicStorageVolume {
                        description: v.description.unwrap_or_else(|| v.volume_name.clone()),
                        id: PublicStorageVolumeId(v.volume_name),
                        is_primary: v.is_primary,
                    }
                )
                .map_err(Into::into)
        })
    }

    /// Creates a new empty file in the app folder of the specified public directory
    /// and returns a **persistent read-write** URI.
    ///
    /// The created file has the following features:  
    /// - It is registered with the appropriate MediaStore as needed.  
    /// - The app can fully manage it until the app is uninstalled.  
    /// - It is **not** removed when the app itself is uninstalled.  
    ///
    /// This is equivalent to:
    /// ```ignore
    /// let app_name = public_storage.app_dir_name()?;
    /// public_storage.create_file(
    ///     volume,
    ///     dir,
    ///     format!("{app_name}/{relative_path}"),
    ///     mime_type
    /// )?;
    /// ```
    /// 
    /// # Args
    /// - ***volume*** :  
    /// The storage volume, such as internal storage, SD card, etc.  
    /// Usually, you don't need to specify this unless there is a special reason.  
    /// If `None` is provided, [`the primary storage volume`](PublicStorage::get_primary_volume) will be used.  
    /// 
    /// - ***dir*** :  
    /// The base directory.  
    /// When using [`PublicImageDir`], use only image MIME types for ***mime_type***, which is discussed below.; using other types may cause errors.
    /// Similarly, use only the corresponding media types for [`PublicVideoDir`] and [`PublicAudioDir`].
    /// Only [`PublicGeneralPurposeDir`] supports all MIME types. 
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
    /// Android 10 (API level 29) or higher.  
    ///
    /// Note :  
    /// - [`PublicAudioDir::Audiobooks`] is not available on Android 9 (API level 28) and lower.
    /// Availability on a given device can be verified by calling [`PublicStorage::is_audiobooks_dir_available`].  
    /// - [`PublicAudioDir::Recordings`] is not available on Android 11 (API level 30) and lower.
    /// Availability on a given device can be verified by calling [`PublicStorage::is_recordings_dir_available`].  
    /// - Others dirs are available in all Android versions.
    pub fn create_file_in_app_dir(
        &self,
        volume: Option<&PublicStorageVolumeId>,
        dir: impl Into<PublicDir>,
        relative_path: impl AsRef<str>, 
        mime_type: Option<&str>
    ) -> crate::Result<FileUri> {

        on_android!({
            let app_dir_name = self.app_dir_name()?;
            let relative_path = relative_path.as_ref().trim_matches('/');
            let relative_path_with_subdir = format!("{app_dir_name}/{relative_path}");

            self.create_file(volume, dir, relative_path_with_subdir, mime_type)
        })
    }

    /// Creates a new empty file in the specified public directory
    /// and returns a **persistent read-write** URI.
    ///
    /// The created file has the following features:  
    /// - It is registered with the appropriate MediaStore as needed.  
    /// - The app can fully manage it until the app is uninstalled.  
    /// - It is **not** removed when the app itself is uninstalled.  
    ///
    /// # Args
    /// - ***volume*** :  
    /// The storage volume, such as internal storage, SD card, etc.  
    /// Usually, you don't need to specify this unless there is a special reason.  
    /// If `None` is provided, [`the primary storage volume`](PublicStorage::get_primary_volume) will be used.  
    /// 
    /// - ***dir*** :  
    /// The base directory.  
    /// When using [`PublicImageDir`], use only image MIME types for ***mime_type***, which is discussed below.; using other types may cause errors.
    /// Similarly, use only the corresponding media types for [`PublicVideoDir`] and [`PublicAudioDir`].
    /// Only [`PublicGeneralPurposeDir`] supports all MIME types. 
    ///  
    /// - ***relative_path_with_subdir*** :  
    /// The file path relative to the base directory.  
    /// Please specify a subdirectory in this, 
    /// such as `MyApp/file.txt` or `MyApp/2025-2-11/file.txt`. Do not use `file.txt`.  
    /// As shown above, it is customary to specify the app name at the beginning of the subdirectory, 
    /// and in this case, using [`PublicStorage::create_file_in_app_dir`] is recommended.  
    /// Any missing subdirectories in the specified path will be created automatically.  
    /// If a file with the same name already exists, 
    /// the system append a sequential number to ensure uniqueness.   
    /// If no extension is present, 
    /// the system may infer one from ***mime_type*** and may append it to the file name. 
    /// But this append-extension operation depends on the model and version.
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
    pub fn create_file(
        &self,
        volume: Option<&PublicStorageVolumeId>,
        dir: impl Into<PublicDir>,
        relative_path_with_subdir: impl AsRef<str>, 
        mime_type: Option<&str>
    ) -> crate::Result<FileUri> {

        on_android!({
            impl_se!(struct Req<'a> { dir: PublicDir, dir_type: &'a str, volume_name: Option<&'a str> });
            impl_de!(struct Res { name: String, uri: String });

            if self.0.api_level()? < api_level::ANDROID_10 {
                return Err(Error { msg: "requires Android 10 (API level 29) or higher".into() })
            }

            let volume_name = volume.map(|v| v.0.as_str());
            let dir = dir.into();
            let dir_type = match dir {
                PublicDir::Image(_) => "Image",
                PublicDir::Video(_) => "Video",
                PublicDir::Audio(_) => "Audio",
                PublicDir::GeneralPurpose(_) => "GeneralPurpose",
            };

            let (dir_name, dir_parent_uri) = self.0.api
                .run_mobile_plugin::<Res>("getPublicDirInfo", Req { dir, dir_type, volume_name })
                .map(|v| (v.name, v.uri))?;
        
            let relative_path = relative_path_with_subdir.as_ref().trim_matches('/');
            let relative_path = format!("{dir_name}/{relative_path}");

            let dir_parent_uri = FileUri {
                uri: dir_parent_uri,
                document_top_tree_uri: None
            };

            self.0.create_file(&dir_parent_uri, relative_path, mime_type)
        })
    }

    /// Recursively create a directory and all of its parent components if they are missing.  
    /// If it already exists, do nothing.
    /// 
    /// [`PublicStorage::create_file`] does this automatically, so there is no need to use it together.
    /// 
    /// # Args  
    /// - ***volume*** :  
    /// The storage volume, such as internal storage, SD card, etc.  
    /// If `None` is provided, [`the primary storage volume`](PublicStorage::get_primary_volume) will be used.  
    /// 
    /// - ***dir*** :  
    /// The base directory.  
    ///  
    /// - ***relative_path*** :  
    /// The directory path relative to the base directory.    
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
    pub fn create_dir_all(
        &self,
        volume: Option<&PublicStorageVolumeId>,
        dir: impl Into<PublicDir>,
        relative_path: impl AsRef<str>, 
    ) -> Result<()> {

        on_android!({
            let relative_path = relative_path.as_ref().trim_matches('/');
            if relative_path.is_empty() {
                return Ok(())
            }

            // TODO:
            // create_file経由ではなく folder作成専用のkotlin apiを作成し呼び出す
            let dir = dir.into();
            let tmp_file_uri = self.create_file(
                volume,
                dir, 
                format!("{relative_path}/TMP-01K3CGCKYSAQ1GHF8JW5FGD4RW"), 
                Some(match dir {
                    PublicDir::Image(_) => "image/png",
                    PublicDir::Audio(_) => "audio/mp3",
                    PublicDir::Video(_) => "video/mp4",
                    PublicDir::GeneralPurpose(_) => "application/octet-stream"
                })
            )?;
            let _ = self.0.remove_file(&tmp_file_uri);

            Ok(())
        })
    }

    /// Recursively create a directory and all of its parent components if they are missing.  
    /// If it already exists, do nothing.
    /// 
    /// [`PublicStorage::create_file_in_app_dir`] does this automatically, so there is no need to use it together.  
    /// 
    /// This is the same as following: 
    /// ```ignore
    /// let app_name = public_storage.app_dir_name()?;
    /// public_storage.create_dir_all(
    ///     volume,
    ///     dir,
    ///     format!("{app_name}/{relative_path}"),
    /// )?;
    /// ```
    /// # Args  
    /// - ***volume*** :  
    /// The storage volume, such as internal storage, SD card, etc.  
    /// If `None` is provided, [`the primary storage volume`](PublicStorage::get_primary_volume) will be used.  
    /// 
    /// - ***dir*** :  
    /// The base directory.  
    ///  
    /// - ***relative_path*** :  
    /// The directory path relative to the base directory.    
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
    pub fn create_dir_all_in_app_dir(
        &self,
        volume: Option<&PublicStorageVolumeId>,
        dir: impl Into<PublicDir>,
        relative_path: impl AsRef<str>, 
    ) -> Result<()> {

        on_android!({
            let app_dir_name = self.app_dir_name()?;
            let relative_path = relative_path.as_ref().trim_start_matches('/');
            let relative_path_with_subdir = format!("{app_dir_name}/{relative_path}");

            self.create_dir_all(volume, dir, relative_path_with_subdir)
        })
    }

    /// Create the specified directory URI that has **no permissions**.  
    /// 
    /// This should only be used as `initial_location` in the file picker. 
    /// It must not be used for any other purpose.  
    /// 
    /// This is useful when selecting save location, 
    /// but when selecting existing entries, `initial_location` is often better with None.
    /// 
    /// # Example
    /// ```rust
    /// use tauri_plugin_android_fs::{AndroidFsExt, InitialLocation, PublicGeneralPurposeDir, PublicImageDir, PublicVideoDir};
    ///
    /// fn example(app: tauri::AppHandle) -> tauri_plugin_android_fs::Result<()> {
    ///     let api = app.android_fs();
    ///     let public_storage = api.public_storage();
    ///
    ///     // Get URI of the top public directory in primary volume
    ///     let initial_location = public_storage.resolve_initial_location(
    ///         InitialLocation::PrimaryTopDir, 
    ///         false
    ///     )?;
    ///
    ///     api.file_picker().pick_file(Some(&initial_location), &[])?;
    ///     api.file_picker().pick_dir(Some(&initial_location))?;
    ///     api.file_picker().save_file(Some(&initial_location), "file_name", Some("image/png"))?;
    ///
    ///
    ///     // Get URI of ~/Pictures/ in primary volume
    ///     let initial_location = public_storage.resolve_initial_location(
    ///         InitialLocation::PrimaryPublicDir { 
    ///             dir: PublicImageDir::Pictures.into(), 
    ///             relative_path: "" 
    ///         }, 
    ///         false
    ///     )?;
    ///
    ///     // Get URI of ~/Documents/sub_dir1/sub_dir2/ in primary volume
    ///     let initial_location = public_storage.resolve_initial_location(
    ///         InitialLocation::PrimaryPublicDir { 
    ///             dir: PublicGeneralPurposeDir::Documents.into(), 
    ///             relative_path: "sub_dir1/sub_dir2" 
    ///         }, 
    ///         true, // Create "sub_dir1" and "sub_dir2" directories if missing
    ///     )?;
    ///
    ///
    ///     let volumes = public_storage.get_available_volumes()?;
    ///     let primary_volume = public_storage.get_primary_volume()?;
    ///
    ///     // Get any available volume other than the primary one 
    ///     // (e.g., SD card or USB drive); 
    ///     // 
    ///     // if none, use the primary volume.
    ///     let volume = volumes.into_iter()
    ///         .find(|v| !v.is_primary)
    ///         .unwrap_or(primary_volume);
    ///
    ///     // Get URI of the top public directory in the specified volume
    ///     let initial_location = public_storage.resolve_initial_location(
    ///         InitialLocation::TopDir {
    ///             volume: &volume.id
    ///         }, 
    ///         false
    ///     )?;
    ///
    ///     // Get URI of ~/Movies/ in the specified volume
    ///     let initial_location = public_storage.resolve_initial_location(
    ///         InitialLocation::PublicDir {
    ///             volume: &volume.id,
    ///             dir: PublicVideoDir::Movies.into(),
    ///             relative_path: ""
    ///         }, 
    ///         false
    ///     )?;
    ///     
    ///
    ///     Ok(())
    /// }
    /// ```
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
    pub fn resolve_initial_location(
        &self,
        initial_location: InitialLocation,
        create_dir_all: bool
    ) -> Result<FileUri> {

        on_android!({
            let volume = initial_location.volume();
            let volume_uid = match volume {
                None => None,
                Some(volume) => self.get_volume_uuid(volume)?
            };

            let mut uri = match volume_uid {
                None => "content://com.android.externalstorage.documents/document/primary%3A".to_string(),
                Some(uid) => format!("content://com.android.externalstorage.documents/document/{uid}%3A")
            };
            
            if let Some((dir, relative_path)) = initial_location.dir_and_relative_path(self.app_dir_name()?) {
                uri.push_str(&format!("{dir}"));

                let relative_path = relative_path.trim_matches('/');
                if !relative_path.is_empty() {
                    uri.push_str("%2F");
                    uri.push_str(&encode_document_id(relative_path));
                }

                let _ = self.create_dir_all(volume, dir, relative_path);
            }

            Ok(FileUri { uri, document_top_tree_uri: None })
        })
    }

    /// Verify whether the basic functions of PublicStorage 
    /// (such as [`PublicStorage::create_file`]) can be performed.
    /// 
    /// If on Android 9 (API level 28) and lower, this returns false.  
    /// If on Android 10 (API level 29) or higher, this returns true.  
    /// 
    /// # Support
    /// All Android version.
    pub fn is_available(&self) -> crate::Result<bool> {
        on_android!({
            Ok(api_level::ANDROID_10 <= self.0.api_level()?)
        })
    }

    /// Verify whether [`PublicAudioDir::Audiobooks`] is available on a given device.   
    /// 
    /// If on Android 9 (API level 28) and lower, this returns false.  
    /// If on Android 10 (API level 29) or higher, this returns true.  
    /// 
    /// # Support
    /// All Android version.
    pub fn is_audiobooks_dir_available(&self) -> crate::Result<bool> {
        on_android!({
            Ok(api_level::ANDROID_10 <= self.0.api_level()?)
        })
    }

    /// Verify whether [`PublicAudioDir::Recordings`] is available on a given device.   
    /// 
    /// If on Android 11 (API level 30) and lower, this returns false.  
    /// If on Android 12 (API level 31) or higher, this returns true.  
    /// 
    /// # Support
    /// All Android version.
    pub fn is_recordings_dir_available(&self) -> crate::Result<bool> {
        on_android!({
            Ok(api_level::ANDROID_12 <= self.0.api_level()?)
        })
    }

    /// Resolve the app dir name from Tauri's config.  
    /// 
    /// # Support
    /// All Android version.
    pub fn app_dir_name(&self) -> crate::Result<&str> {
        on_android!({
            use std::sync::OnceLock;
            
            static APP_DIR_NAME: OnceLock<String> = OnceLock::new();

            if APP_DIR_NAME.get().is_none() {
                let config = self.0.app.config();
                let app_name = config.product_name
                    .as_deref()
                    .filter(|s| !s.is_empty())
                    .unwrap_or(&config.identifier)
                    .replace('/', " ");
                
                // The cell is guaranteed to contain a value when set returns
                let _ = APP_DIR_NAME.set(app_name);
            }

            Ok(&APP_DIR_NAME.get().unwrap())
        })
    }


    #[allow(unused)]
    fn get_volume_uuid(&self, volume: &PublicStorageVolumeId) -> Result<Option<String>> {
        on_android!({
            impl_se!(struct Req<'a> { volume_name: &'a str });
            impl_de!(struct Res { value: Option<String> });

            let volume_name = &volume.0;

            self.0.api
                .run_mobile_plugin::<Res>("getStorageVolumeUuid", Req { volume_name })
                .map(|v| v.value)
                .map_err(Into::into)
        })
    }
}