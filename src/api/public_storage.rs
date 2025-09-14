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

            if self.0.api_level()? < api_level::ANDROID_10 {
                return Err(Error { msg: "requires Android 10 (API level 29) or higher".into() })
            }

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

            // primary volume を先頭にする。他はそのままの順序
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

            if self.0.api_level()? < api_level::ANDROID_10 {
                return Err(Error { msg: "requires Android 10 (API level 29) or higher".into() })
            }

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

    /// Creates a new empty file in the specified public directory of the storage volume.  
    /// This returns a **persistent read-write** URI.
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
        volume: Option<&PublicStorageVolumeId>,
        base_dir: impl Into<PublicDir>,
        use_app_dir: bool,
        relative_path: impl AsRef<std::path::Path>, 
        mime_type: Option<&str>
    ) -> crate::Result<FileUri> {

        on_android!({
            if self.0.api_level()? < api_level::ANDROID_10 {
                return Err(Error { msg: "requires Android 10 (API level 29) or higher".into() })
            }

            let base_dir = base_dir.into();
            let relative_path = validate_relative_path(relative_path.as_ref())?;
            let base_dir_uri = self.get_dir_uri(volume, base_dir)?;
         
            let relative_path = {
                let mut p = std::path::PathBuf::new();
                p.push(self.get_dir_name(base_dir)?);
                if use_app_dir {
                    p.push(self.app_dir_name()?);
                }
                p.push(relative_path);
                p
            };

            self.0.create_new_file(&base_dir_uri, relative_path, mime_type)
        })
    }

    /// Recursively create a directory and all of its parent components if they are missing.  
    /// If it already exists, do nothing.
    /// 
    /// [`PublicStorage::create_new_file`] does this automatically, so there is no need to use it together.
    /// 
    /// # Args  
    /// - ***volume*** :  
    /// The storage volume, such as internal storage, SD card, etc.  
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
        volume: Option<&PublicStorageVolumeId>,
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
                volume,
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
    /// You can create files and folders under this directory and read or write only them. 
    /// 
    /// **Please avoid using this whenever possible.**    
    /// Use it only in cases that cannot be handled by [`PublicStorage::create_new_file`] or [`PrivateStorage::resolve_path`], 
    /// such as when you need to pass the absolute path of a user-accessible file as an argument to any database library.  
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
    /// - ***volume*** :  
    /// The storage volume, such as internal storage, SD card, etc.  
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
        volume: Option<&PublicStorageVolumeId>,
        base_dir: impl Into<PublicDir>,
        use_app_dir: bool,
    ) -> Result<std::path::PathBuf> {

        on_android!({
            if self.0.api_level()? < api_level::ANDROID_10 {
                return Err(Error::with("requires Android 10 (API level 29) or higher"))
            }

            let mut path = self.get_volume_path(volume)?;
            path.push(self.get_dir_name(base_dir)?);
            if use_app_dir {
                path.push(self.app_dir_name()?);
            }
            Ok(path)
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
    /// - ***volume*** :  
    /// The storage volume, such as internal storage, SD card, etc.  
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
        volume: Option<&PublicStorageVolumeId>,
        base_dir: impl Into<PublicDir>,
        use_app_dir: bool,
        relative_path: impl AsRef<std::path::Path>,
        create_dir_all: bool
    ) -> Result<FileUri> {

        on_android!({
            let base_dir = base_dir.into();
            
            let mut uri = self.resolve_initial_location_top(volume)?;
            uri.uri.push_str(self.get_dir_name(base_dir)?);

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
                let _ = self.create_dir_all(volume, base_dir, use_app_dir, relative_path.as_ref());
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
    /// - ***volume*** :  
    /// The storage volume, such as internal storage, SD card, etc.  
    /// If `None` is provided, [`the primary storage volume`](PublicStorage::get_primary_volume) will be used.  
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
    pub fn resolve_initial_location_top(
        &self,
        volume: Option<&PublicStorageVolumeId>
    ) -> Result<FileUri> {

        on_android!({
            let volume_uid = match volume {
                None => None,
                Some(volume) => {
                    if self.0.api_level()? < api_level::ANDROID_10 {
                        return Err(Error { msg: "requires Android 10 (API level 29) or higher".into() })
                    }
                    self.get_volume_uuid(volume)?
                }
            };

            let uri = match volume_uid {
                None => "content://com.android.externalstorage.documents/document/primary%3A".to_string(),
                Some(uid) => format!("content://com.android.externalstorage.documents/document/{uid}%3A")
            };

            Ok(FileUri { uri, document_top_tree_uri: None })
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


#[allow(unused)]
impl<'a, R: tauri::Runtime> PublicStorage<'a, R> {

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

    fn get_volume_path(&self,
        volume: Option<&PublicStorageVolumeId>, 
    ) -> Result<std::path::PathBuf> {

        on_android!({
            impl_se!(struct Req<'a> { volume_name: Option<&'a str> });
            impl_de!(struct Res { path: String });

            let volume_name = volume.map(|v| v.0.as_str());

            self.0.api
                .run_mobile_plugin::<Res>("getStorageVolumePath", Req { volume_name })
                .map(|v| {
                    use std::str::FromStr;
                    
                    let r = std::path::PathBuf::from_str(&v.path);
                    std::result::Result::<_, std::convert::Infallible>::unwrap(r)
                })
                .map_err(Into::into)
        })
    }

    fn get_dir_uri(
        &self, 
        volume: Option<&PublicStorageVolumeId>, 
        dir: impl Into<PublicDir>,
    ) -> Result<FileUri> {

        on_android!({
            impl_se!(struct Req<'a> { dir_type: &'a str, volume_name: Option<&'a str> });
            impl_de!(struct Res { uri: String });

            let volume_name = volume.map(|v| v.0.as_str());
            let dir_type = match dir.into() {
                PublicDir::Image(_) => "Image",
                PublicDir::Video(_) => "Video",
                PublicDir::Audio(_) => "Audio",
                PublicDir::GeneralPurpose(_) => "GeneralPurpose",
            };

            self.0.api
                .run_mobile_plugin::<Res>("getPublicDirUri", Req { dir_type, volume_name })
                .map(|v| FileUri { uri: v.uri, document_top_tree_uri: None })
                .map_err(Into::into)
        })
    }

    fn get_dir_name(&self, dir: impl Into<PublicDir>) -> Result<&str> {
        on_android!({
            impl_de!(struct EnvironmentDirs {
                pictures: String,
                dcim: String,
                movies: String,
                music: String,
                alarms: String,
                notifications: String,
                podcasts: String,
                ringtones: String,
                documents: String,
                download: String,
                audiobooks: Option<String>,
                recordings: Option<String>,
            });
            
            static DIRS: std::sync::OnceLock<EnvironmentDirs> = std::sync::OnceLock::new();

            if DIRS.get().is_none() {
                let _ = DIRS.set(
                    self.0.api.run_mobile_plugin::<EnvironmentDirs>("getEnvironmentDirs", "")?
                );
            }
            let e = DIRS.get().expect("Should call 'set' before 'get'");


            return Ok(match dir.into() {
                PublicDir::Image(dir) => match dir {
                    PublicImageDir::Pictures => &e.pictures,
                    PublicImageDir::DCIM => &e.dcim,
                },
                PublicDir::Video(dir) => match dir {
                    PublicVideoDir::Movies => &e.movies,
                    PublicVideoDir::DCIM => &e.dcim,
                },
                PublicDir::Audio(dir) => match dir  {
                    PublicAudioDir::Music => &e.music,
                    PublicAudioDir::Alarms => &e.alarms,
                    PublicAudioDir::Notifications => &e.notifications,
                    PublicAudioDir::Podcasts => &e.podcasts,
                    PublicAudioDir::Ringtones => &e.ringtones,
                    PublicAudioDir::Recordings => e.recordings.as_ref().ok_or_else(|| Error { msg: "requires API level 31 or higher".into() })?,
                    PublicAudioDir::Audiobooks => e.audiobooks.as_ref().ok_or_else(|| Error { msg: "requires API level 29 or higher".into() })?,
                },
                PublicDir::GeneralPurpose(dir) => match dir {
                    PublicGeneralPurposeDir::Documents => &e.documents,
                    PublicGeneralPurposeDir::Download => &e.download,
                }
            })
        })
    }
}