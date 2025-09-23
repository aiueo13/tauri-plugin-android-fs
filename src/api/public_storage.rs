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
    pub fn get_volumes(&self) -> Result<Vec<StorageVolume>> {
        if self.0.api_level()? < api_level::ANDROID_10 {
            return Err(Error { msg: "requires Android 10 (API level 29) or higher".into() })
        }

        let volumes = self.0.get_available_storage_volumes()?
            .into_iter()
            .filter(|v| v.id.media_store_context.is_some())
            .collect::<Vec<_>>();

        Ok(volumes)
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
    pub fn get_primary_volume(&self) -> Result<Option<StorageVolume>> {
        if self.0.api_level()? < api_level::ANDROID_10 {
            return Err(Error { msg: "requires Android 10 (API level 29) or higher".into() })
        }

        self.0.get_primary_storage_volume_if_available()
            .map(|v| v.filter(|v| v.id.media_store_context.is_some()))
            .map_err(Into::into)
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
    /// - ***use_app_dir*** :   
    /// Determines whether to insert a directory named after the application name 
    /// specified in Tauri's configuration between ***base_dir*** and ***relative_path***.
    /// It is recommended to enable this unless there is a special reason not to.   
    /// See [`PublicStorage::app_dir_name`]
    /// 
    /// - ***relative_path*** :  
    /// The file path relative to the base directory.  
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
    pub fn create_new_file(
        &self,
        volume_id: Option<&StorageVolumeId>,
        base_dir: impl Into<PublicDir>,
        use_app_dir: bool,
        relative_path: impl AsRef<std::path::Path>, 
        mime_type: Option<&str>
    ) -> crate::Result<FileUri> {

        on_android!({
            if self.0.api_level()? < api_level::ANDROID_10 {
                return Err(Error { msg: "requires Android 10 (API level 29) or higher".into() })
            }

            let c = self.0.consts()?;

            let base_dir = base_dir.into();
            let relative_path = validate_relative_path(relative_path.as_ref())?;
            let relative_path = {
                let mut p = std::path::PathBuf::new();
                p.push(c.public_dir_name(base_dir)?);
                if use_app_dir {
                    p.push(self.app_dir_name()?);
                }
                p.push(relative_path);
                p
            };

            let media_store_ctx = volume_id
                .map(|v| v.media_store_context.as_ref())
                .unwrap_or(c.primary_storage_volume_media_store_context.as_ref())
                .ok_or_else(|| Error::with("The storage volume is not available for PublivStorage"))?;
            
            let media_store_content_uri = match base_dir.into() {
                PublicDir::Image(_) => &media_store_ctx.images_content_uri,
                PublicDir::Video(_) => &media_store_ctx.videos_content_uri,
                PublicDir::Audio(_) => &media_store_ctx.audios_content_uri,
                PublicDir::GeneralPurpose(_) => &media_store_ctx.files_content_uri
            };

            let base_uri = FileUri {
                uri: media_store_content_uri.clone(),
                document_top_tree_uri: None
            };

            self.0.create_new_file(&base_uri, relative_path, mime_type)
        })
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
    /// - ***use_app_dir*** :   
    /// Determines whether to insert a directory named after the application name 
    /// specified in Tauri's configuration between ***base_dir*** and ***relative_path***.
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
    pub fn create_dir_all(
        &self,
        volume_id: Option<&StorageVolumeId>,
        base_dir: impl Into<PublicDir>,
        use_app_dir: bool,
        relative_path: impl AsRef<std::path::Path>, 
    ) -> Result<()> {

        on_android!({
            if self.0.api_level()? < api_level::ANDROID_10 {
                return Err(Error { msg: "requires Android 10 (API level 29) or higher".into() })
            }

            let relative_path = validate_relative_path(relative_path.as_ref())?;
            let base_dir = base_dir.into();

            let tmp_file_uri = self.create_new_file(
                volume_id,
                base_dir, 
                use_app_dir,
                relative_path.join("TMP-01K3CGCKYSAQ1GHF8JW5FGD4RW"), 
                Some(match base_dir {
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

    /// Retrieves the absolute path for a specified public directory within the given storage volume.   
    /// This function does **not** create any directories; it only constructs the path.
    /// 
    /// **Please avoid using this whenever possible.**    
    /// Use it only in cases that cannot be handled by [`PublicStorage::create_new_file`] or [`PrivateStorage::resolve_path`], 
    /// such as when you need to pass the absolute path of a user-accessible file as an argument to any database library, debug logger, and etc.  
    ///
    /// You can create files and folders under this directory and read or write **only** them.  
    /// When using [`PublicImageDir`], use only image type for file name extension, 
    /// using other type extension or none may cause errors.
    /// Similarly, use only the corresponding media types for [`PublicVideoDir`] and [`PublicAudioDir`].
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
    /// - ***use_app_dir*** :   
    /// Determines whether to insert a directory named after the application name 
    /// specified in Tauri's configuration under ***base_dir***.  
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
    pub fn resolve_path(
        &self,
        volume_id: Option<&StorageVolumeId>,
        base_dir: impl Into<PublicDir>,
        use_app_dir: bool,
    ) -> Result<std::path::PathBuf> {

        if self.0.api_level()? < api_level::ANDROID_10 {
            return Err(Error { msg: "requires Android 10 (API level 29) or higher".into() })
        }

        let mut path = match volume_id {
            Some(volume_id) => {
                let Some(c) = &volume_id.media_store_context else {
                    return Err(Error::with(format!("The storage volume is not available for PublivStorage: {}", volume_id.top_directory_path.display())))
                };
                if !self.0.check_storage_volume_available_by_media_store_volume_name(&c.volume_name)? {
                    return Err(Error::with(format!("The storage volume is not currently available: {}", volume_id.top_directory_path.display())))
                }
                volume_id.top_directory_path.clone()
            },
            None => {
                let Some(volume) = self.get_primary_volume()? else {
                    return Err(Error::with("Primary storage volume is not currently available"))
                };
                volume.id.top_directory_path
            }
        };

        path.push(self.0.consts()?.public_dir_name(base_dir)?);
        if use_app_dir {
            path.push(self.app_dir_name()?);
        }
        Ok(path)
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
    /// - ***use_app_dir*** :   
    /// Determines whether to insert a directory named after the application name 
    /// specified in Tauri's configuration between ***base_dir*** and ***relative_path***.
    /// 
    /// - ***relative_path*** :  
    /// The directory path relative to the base directory.    
    ///  
    /// # Support
    /// If use `None` to ***volume***, all Android version: 
    /// otherwise: Android 10 (API level 29) or higher
    ///
    /// Note :  
    /// - [`PublicAudioDir::Audiobooks`] is not available on Android 9 (API level 28) and lower.
    /// Availability on a given device can be verified by calling [`PublicStorage::is_audiobooks_dir_available`].  
    /// - [`PublicAudioDir::Recordings`] is not available on Android 11 (API level 30) and lower.
    /// Availability on a given device can be verified by calling [`PublicStorage::is_recordings_dir_available`].  
    /// - Others dirs are available in all Android versions.
    pub fn resolve_initial_location(
        &self,
        volume_id: Option<&StorageVolumeId>,
        base_dir: impl Into<PublicDir>,
        use_app_dir: bool,
        relative_path: impl AsRef<std::path::Path>,
        create_dir_all: bool
    ) -> Result<FileUri> {

        on_android!({
            let base_dir = base_dir.into();
            
            let mut uri = self.resolve_initial_location_top(volume_id)?;
            uri.uri.push_str(self.0.consts()?.public_dir_name(base_dir)?);

            let relative_path = validate_relative_path(relative_path.as_ref())?;
            let relative_path = relative_path.to_string_lossy();
            if !relative_path.is_empty() {
                uri.uri.push_str("%2F");
                uri.uri.push_str(&match use_app_dir {
                    false => encode_document_id(relative_path.as_ref()),
                    true => {
                        let mut r = std::path::PathBuf::new();
                        r.push(self.app_dir_name()?);
                        r.push(relative_path.as_ref());
                        encode_document_id(r.to_string_lossy().as_ref())
                    },
                });
            }

            if create_dir_all {
                let _ = self.create_dir_all(volume_id, base_dir, use_app_dir, relative_path.as_ref());
            }

            Ok(uri)
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
    /// # Args  
    /// - ***volume_id*** :  
    /// ID of the storage volume, such as internal storage, SD card, etc.  
    /// If `None` is provided, [`the primary storage volume`](PublicStorage::get_primary_volume) will be used.  
    /// 
    /// # Support
    /// All Android version: 
    ///
    /// Note :  
    /// - [`PublicAudioDir::Audiobooks`] is not available on Android 9 (API level 28) and lower.
    /// Availability on a given device can be verified by calling [`PublicStorage::is_audiobooks_dir_available`].  
    /// - [`PublicAudioDir::Recordings`] is not available on Android 11 (API level 30) and lower.
    /// Availability on a given device can be verified by calling [`PublicStorage::is_recordings_dir_available`].  
    /// - Others dirs are available in all Android versions.
    pub fn resolve_initial_location_top(
        &self,
        volume_id: Option<&StorageVolumeId>
    ) -> Result<FileUri> {

        on_android!({
            let volume_id = volume_id
                .and_then(|v| v.uuid.as_deref())
                .unwrap_or("primary");

            Ok(FileUri {
                uri: format!("content://com.android.externalstorage.documents/document/{volume_id}%3A"),
                document_top_tree_uri: None 
            })
        })
    }

    /// Verify whether the basic functions of PublicStorage 
    /// (such as [`PublicStorage::create_new_file`]) can be performed.
    /// 
    /// If on Android 9 (API level 28) and lower, this returns false.  
    /// If on Android 10 (API level 29) or higher, this returns true.  
    /// 
    /// # Support
    /// All Android version.
    pub fn is_available(&self) -> crate::Result<bool> {
        Ok(api_level::ANDROID_10 <= self.0.api_level()?)
    }

    /// Verify whether [`PublicAudioDir::Audiobooks`] is available on a given device.   
    /// 
    /// If on Android 9 (API level 28) and lower, this returns false.  
    /// If on Android 10 (API level 29) or higher, this returns true.  
    /// 
    /// # Support
    /// All Android version.
    pub fn is_audiobooks_dir_available(&self) -> crate::Result<bool> {
        Ok(self.0.consts()?.env_dir_audiobooks.is_some())
    }

    /// Verify whether [`PublicAudioDir::Recordings`] is available on a given device.   
    /// 
    /// If on Android 11 (API level 30) and lower, this returns false.  
    /// If on Android 12 (API level 31) or higher, this returns true.  
    /// 
    /// # Support
    /// All Android version.
    pub fn is_recordings_dir_available(&self) -> crate::Result<bool> {
        Ok(self.0.consts()?.env_dir_recordings.is_some())
    }

    /// Resolve the app dir name from Tauri's config.  
    /// 
    /// This uses "productName" in `src-tauri/tauri.conf.json`
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

                let _ = APP_DIR_NAME.set(app_name);
            }

            Ok(&APP_DIR_NAME.get().expect("Should call 'set' before 'get'"))
        })
    }
}