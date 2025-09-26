use crate::*;


/// API of file storage intended for the app’s use only.  
/// 
/// # Examples
/// ```
/// fn example(app: &tauri::AppHandle) {
///     use tauri_plugin_android_fs::AndroidFsExt as _;
/// 
///     let api = app.android_fs();
///     let private_storage = api.private_storage();
/// }
/// ```
pub struct PrivateStorage<'a, R: tauri::Runtime>(
    #[allow(unused)]
    pub(crate) &'a AndroidFs<R>
);

impl<'a, R: tauri::Runtime> PrivateStorage<'a, R> {

    /// Get an absolute path of the app-specific directory on the internal storage.  
    /// App can fully manage entries within this directory.   
    /// 
    /// This function does **not** create any directories; it only constructs the path.
    /// 
    /// Since these locations may contain files created by other Tauri plugins or webview systems, 
    /// it is recommended to add a subdirectory with a unique name.
    ///
    /// These entries will be deleted when the app is uninstalled and may also be deleted at the user’s initialising request.  
    /// 
    /// When using [`PrivateDir::Cache`], the system will automatically delete entries as disk space is needed elsewhere on the device. 
    /// But you should not rely on this. The cache should be explicitly cleared by yourself.
    /// 
    /// The system prevents other apps and user from accessing these locations. 
    /// In cases where the device is rooted or the user has special permissions, the user may be able to access this.   
    /// 
    /// Since the returned paths can change when the app is moved to an [adopted storage](https://source.android.com/docs/core/storage/adoptable), 
    /// only relative paths should be stored.
    /// 
    /// # Note
    /// This provides a separate area for each user in a multi-user environment.
    /// 
    /// # Support
    /// All Android version.
    pub fn resolve_path(
        &self, 
        dir: PrivateDir
    ) -> crate::Result<std::path::PathBuf> {

        on_android!({
            impl_de!(struct Paths {
                data: std::path::PathBuf, 
                cache: std::path::PathBuf, 
                no_backup_data: std::path::PathBuf, 
            });
        
            static PATHS: std::sync::OnceLock<Paths> = std::sync::OnceLock::new();

            if PATHS.get().is_none() {
                let _ = PATHS.set(
                    self.0.api.run_mobile_plugin::<Paths>("getPrivateBaseDirAbsolutePaths", "")?
                );
            }
            let paths = PATHS.get().expect("Should call 'set' before 'get'");

            Ok(match dir {
                PrivateDir::Data => paths.data.clone(),
                PrivateDir::Cache => paths.cache.clone(),
                PrivateDir::NoBackupData => paths.no_backup_data.clone(),
            })
        })
    }

    /// Get an absolute path of the app-specific directory on the specified storage volume.  
    /// App can fully manage entries within this directory.  
    /// 
    /// This function does **not** create any directories; it only constructs the path.
    ///    
    /// Since these locations may contain files created by other Tauri plugins or webview systems, 
    /// it is recommended to add a subdirectory with a unique name.
    ///
    /// These entries will be deleted when the app is uninstalled and may also be deleted at the user’s initialising request.   
    /// 
    /// # Note
    /// If you are unsure between this function and [`PrivateStorage::resolve_path`], 
    /// you don’t need to use this one.  
    /// The difference from [`PrivateStorage::resolve_path`] is that these files may be accessed by other apps that have specific permissions,
    /// and it cannot always be available since removable storage can be ejected.  
    /// 
    /// One advantage of using this is that it allows storing large app-specific data/cache on SD cards or other supplementary storage, 
    /// which can be useful on older devices with limited built-in storage capacity. 
    /// However on modern devices, the built-in storage capacity is relatively large,
    /// and there is little advantage in using this.  
    /// 
    /// By using [`StorageVolume { is_emulated, .. }`](StorageVolume), 
    /// you can determine whether this belongs to the same storage volume as [`PrivateStorage::resolve_path`]. 
    /// In this case, there is no advantage in using this instead of `PrivateStorage::resolve_path`. 
    /// It only reduces security.
    /// 
    /// # Args
    /// - ***volume_id*** :  
    /// ID of the storage volume, such as internal storage, SD card, etc.  
    /// If `None` is provided, [`the primary storage volume`](PrivateStorage::get_primary_volume) will be used.  
    /// 
    /// # Support
    /// All Android version. 
    pub fn resolve_outside_path(
        &self, 
        volume_id: Option<&StorageVolumeId>,
        dir: OutsidePrivateDir
    ) -> Result<std::path::PathBuf> {

        if let Some(volume_id) = volume_id {
            let dir_path = volume_id
                .outside_private_dir_path(dir)
                .ok_or_else(|| Error::with("The storage volume has no app-speific directory"))?;
            
            if !self.0.check_storage_volume_available_by_path(dir_path)? {
                return Err(Error::with("The storage volume is not currently available"))
            }

            return Ok(dir_path.clone())
        }

        self.get_primary_volume()?
            .and_then(|v| v.id.outside_private_dir_path(dir).map(Clone::clone))
            .ok_or_else(|| Error::with("Primary storage volume is not currently available"))
    }

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
    /// This typically includes [`primary storage volume`](PrivateStorage::get_primary_volume),
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
    pub fn get_volumes(&self) -> Result<Vec<StorageVolume>> {
        let volumes = self.0.get_available_storage_volumes()?
            .into_iter()
            .filter(|v| v.id.private_data_dir_path.is_some() || v.id.private_cache_dir_path.is_some())
            .collect::<Vec<_>>();

        Ok(volumes)
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
    pub fn get_primary_volume(&self) -> Result<Option<StorageVolume>> {
        self.0.get_primary_storage_volume_if_available()
            .map(|v| v.filter(|v| v.id.private_data_dir_path.is_some() || v.id.private_cache_dir_path.is_some()))
            .map_err(Into::into)
    }


    /// This is same as [`FileUri::from_path`]
    #[deprecated = "Use FileUri::from_path instead"]
    pub fn resolve_uri(
        &self, 
        dir: PrivateDir,
        relative_path: impl AsRef<std::path::Path>
    ) -> crate::Result<FileUri> {

        on_android!({
            let mut path = self.resolve_path(dir)?;
            path.push(validate_relative_path(relative_path.as_ref())?);
            Ok(path.into())
        })
    }
}


#[allow(unused)]
impl<'a, R: tauri::Runtime> PrivateStorage<'a, R> {

    pub(crate) fn create_new_tmp_file(&self) -> crate::Result<(std::fs::File, std::path::PathBuf)> {
        on_android!({
            let tmp_file_path = {
                use std::sync::atomic::{AtomicUsize, Ordering};

                static COUNTER: AtomicUsize = AtomicUsize::new(0);
                let id = COUNTER.fetch_add(1, Ordering::Relaxed);

                let tmp_dir_path = self.resolve_tmp_dir()?;
                let _ = std::fs::create_dir_all(&tmp_dir_path);
            
                tmp_dir_path.join(format!("{id}"))
            };
            
            let tmp_file = std::fs::File::create_new(&tmp_file_path)?;

            Ok((tmp_file, tmp_file_path))
        })
    }

    pub(crate) fn remove_all_tmp_files(&self) -> crate::Result<()> {
        on_android!({
            match std::fs::remove_dir_all(self.resolve_tmp_dir()?) {
                Ok(_) => Ok(()),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
                Err(e) => Err(e.into()),
            }
        })
    }

    pub(crate) fn resolve_tmp_dir(&self) -> crate::Result<std::path::PathBuf> {
        on_android!({
            const TMP_DIR_RELATIVE_PATH: &str = "pluginAndroidFs-tmpDir-01K486FKQ2BZSBGFD34RFH9FWJ";

            let mut path = self.resolve_path(PrivateDir::Cache)?;
            path.push(TMP_DIR_RELATIVE_PATH);
            Ok(path)
        })
    }
}