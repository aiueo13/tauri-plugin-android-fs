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
            .filter(|v| v.id.media_store_volume_name.is_some())
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
            .map(|v| v.filter(|v| v.id.media_store_volume_name.is_some()))
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
    pub fn create_new_file(
        &self,
        volume_id: Option<&StorageVolumeId>,
        base_dir: impl Into<PublicDir>,
        relative_path: impl AsRef<std::path::Path>, 
        mime_type: Option<&str>
    ) -> crate::Result<FileUri> {

        on_android!({
            impl_se!(struct Req<'a> { media_store_volume_name: &'a str, relative_path: std::path::PathBuf, mime_type: Option<&'a str> });
            impl_de!(struct Res { uri: FileUri });

            if self.0.api_level()? < api_level::ANDROID_10 {
                return Err(Error { msg: "requires Android 10 (API level 29) or higher".into() })
            }

            let consts = self.0.consts()?;
            let relative_path = {
                let mut p = std::path::PathBuf::new();
                p.push(consts.public_dir_name(base_dir)?);
                p.push(validate_relative_path(relative_path.as_ref())?);
                p
            };
            let media_store_volume_name = volume_id
                .map(|v| v.media_store_volume_name.as_ref())
                .unwrap_or(consts.media_store_primary_volume_name.as_ref())
                .ok_or_else(|| Error::with("The storage volume is not available for PublicStorage"))?;

            self.0.api
                .run_mobile_plugin::<Res>("createNewMediaStoreFile", Req {
                    media_store_volume_name, 
                    relative_path,
                    mime_type,
                })
                .map(|v| v.uri)
                .map_err(Into::into)
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
    pub fn resolve_path(
        &self,
        volume_id: Option<&StorageVolumeId>,
        base_dir: impl Into<PublicDir>,
    ) -> Result<std::path::PathBuf> {

        if self.0.api_level()? < api_level::ANDROID_10 {
            return Err(Error { msg: "requires Android 10 (API level 29) or higher".into() })
        }

        let mut path = match volume_id {
            Some(volume_id) => {
                let (vn, tp) = volume_id.media_store_volume_name.as_ref()
                    .zip(volume_id.top_directory_path.as_ref())
                    .ok_or_else(|| Error::with("The storage volume is not available for PublicStorage"))?;
                
                if !self.0.check_media_store_volume_name_available(vn)? {
                    return Err(Error::with("The storage volume is not currently available"))
                }

                tp.clone()
            },
            None => {
                self.get_primary_volume()?
                    .and_then(|v| v.id.top_directory_path)
                    .ok_or_else(|| Error::with("Primary storage volume is not currently available"))?
            }
        };

        path.push(self.0.consts()?.public_dir_name(base_dir)?);
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
    pub fn resolve_initial_location(
        &self,
        volume_id: Option<&StorageVolumeId>,
        base_dir: impl Into<PublicDir>,
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
                uri.uri.push_str(&encode_document_id(relative_path.as_ref()));
            }

            if create_dir_all {
                let _ = self.create_dir_all(volume_id, base_dir, relative_path.as_ref());
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
    /// All Android version.
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
}