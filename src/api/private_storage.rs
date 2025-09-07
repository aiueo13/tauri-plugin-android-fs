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

    /// Get the absolute path of the specified directory.  
    /// App can fully manage entries within this directory without any permission via std::fs.   
    ///
    /// These files will be deleted when the app is uninstalled and may also be deleted at the user’s initialising request.  
    /// When using [`PrivateDir::Cache`], the system will automatically delete files in this directory as disk space is needed elsewhere on the device.   
    /// 
    /// The returned path may change over time if the calling app is moved to an adopted storage device, 
    /// so only relative paths should be persisted.   
    /// 
    /// # Examples
    /// ```no_run
    /// use tauri_plugin_android_fs::{AndroidFs, AndroidFsExt, PrivateDir, PrivateStorage};
    /// 
    /// fn example(app: tauri::AppHandle) {
    ///     let api = app.android_fs().private_storage();
    /// 
    ///     let dir_path = api.resolve_path(PrivateDir::Data).unwrap();
    ///     let file_path = dir_path.join("2025-2-12/data.txt");
    ///     
    ///     // Write file
    ///     std::fs::create_dir_all(file_path.parent().unwrap()).unwrap();
    ///     std::fs::write(&file_path, "aaaa").unwrap();
    /// 
    ///     // Read file
    ///     let _ = std::fs::read_to_string(&file_path).unwrap();
    /// 
    ///     // Remove file
    ///     std::fs::remove_file(&file_path).unwrap();
    /// }
    /// ```
    /// 
    /// # Support
    /// All Android version.
    pub fn resolve_path(&self, dir: PrivateDir) -> crate::Result<std::path::PathBuf> {
        on_android!({
            impl_de!(struct Paths { data: String, cache: String });
        
            static PATHS: std::sync::OnceLock<Paths> = std::sync::OnceLock::new();

            if PATHS.get().is_none() {
                let paths = self.0.api
                    .run_mobile_plugin::<Paths>("getPrivateBaseDirAbsolutePaths", "")?;

                // The cell is guaranteed to contain a value when set returns
                let _ = PATHS.set(paths);
            }

            let paths = PATHS.get().unwrap();

            Ok(match dir {
                PrivateDir::Data => std::path::PathBuf::from(paths.data.to_owned()),
                PrivateDir::Cache => std::path::PathBuf::from(paths.cache.to_owned()),
            })
        })
    }

    /// Get the absolute path of the specified relative path and base directory.  
    /// App can fully manage entries of this path without any permission via std::fs.   
    ///
    /// See [`PrivateStorage::resolve_path`] for details.  
    /// 
    /// # Support
    /// All Android version.
    pub fn resolve_path_with(
        &self,
        dir: PrivateDir,
        relative_path: impl AsRef<str>
    ) -> crate::Result<std::path::PathBuf> {

        on_android!({
            let relative_path = relative_path.as_ref().trim_start_matches('/');
            let path = self.resolve_path(dir)?.join(relative_path);
            Ok(path)
        })
    }

    pub fn resolve_uri(&self, dir: PrivateDir) -> crate::Result<FileUri> {
        on_android!({
            self.resolve_path(dir).map(Into::into)
        })
    }

    pub fn resolve_uri_with(&self, dir: PrivateDir, relative_path: impl AsRef<str>) -> crate::Result<FileUri> {
        on_android!({
            self.resolve_path_with(dir, relative_path).map(Into::into)
        })
    }

    /// Writes a slice as the entire contents of a file.  
    /// 
    /// This function will create a file if it does not exist, and will entirely replace its contents if it does.  
    /// Recursively create parent directories if they are missing.  
    /// 
    /// This internally uses [`PrivateStorage::resolve_path`] , [`std::fs::create_dir_all`], and [`std::fs::write`].  
    /// See [`PrivateStorage::resolve_path`] for details.  
    /// 
    /// # Support
    /// All Android version.
    pub fn write(
        &self, 
        base_dir: PrivateDir, 
        relative_path: impl AsRef<str>, 
        contents: impl AsRef<[u8]>
    ) -> crate::Result<()> {

        on_android!({
            let path = self.resolve_path_with(base_dir, relative_path)?;

            if let Some(parent_dir) = path.parent() {
                std::fs::create_dir_all(parent_dir)?;
            }

            std::fs::write(path, contents)?;
            Ok(())
        })
    }

    /// Open a file in read-only mode.  
    /// 
    /// If you only need to read the entire file contents, consider using [`PrivateStorage::read`]  or [`PrivateStorage::read_to_string`] instead.  
    /// 
    /// This internally uses [`PrivateStorage::resolve_path`] and [`std::fs::File::open`].  
    /// See [`PrivateStorage::resolve_path`] for details.  
    /// 
    /// # Support
    /// All Android version.
    pub fn open_file(
        &self,
        base_dir: PrivateDir, 
        relative_path: impl AsRef<str>, 
    ) -> crate::Result<std::fs::File> {

        on_android!({
            let path = self.resolve_path_with(base_dir, relative_path)?;
            Ok(std::fs::File::open(path)?)
        })
    }

    /// Opens a file in write-only mode.  
    /// This function will create a file if it does not exist, and will truncate it if it does.
    /// 
    /// If you only need to write the contents, consider using [`PrivateStorage::write`]  instead.  
    /// 
    /// This internally uses [`PrivateStorage::resolve_path`] and [`std::fs::File::create`].  
    /// See [`PrivateStorage::resolve_path`] for details.  
    /// 
    /// # Support
    /// All Android version.
    pub fn create_file(
        &self,
        base_dir: PrivateDir, 
        relative_path: impl AsRef<str>, 
    ) -> crate::Result<std::fs::File> {

        on_android!({
            let path = self.resolve_path_with(base_dir, relative_path)?;
            Ok(std::fs::File::create(path)?)
        })
    }

    /// Creates a new file in read-write mode; error if the file exists. 
    /// 
    /// This internally uses [`PrivateStorage::resolve_path`] and [`std::fs::File::create_new`].  
    /// See [`PrivateStorage::resolve_path`] for details.  
    /// 
    /// # Support
    /// All Android version.
    pub fn create_new_file(
        &self,
        base_dir: PrivateDir, 
        relative_path: impl AsRef<str>, 
    ) -> crate::Result<std::fs::File> {

        on_android!({
            let path = self.resolve_path_with(base_dir, relative_path)?;
            Ok(std::fs::File::create_new(path)?)
        })
    }

    /// Reads the entire contents of a file into a bytes vector.  
    /// 
    /// If you need [`std::fs::File`], use [`PrivateStorage::open_file`] insted.  
    /// 
    /// This internally uses [`PrivateStorage::resolve_path`] and [`std::fs::read`].  
    /// See [`PrivateStorage::resolve_path`] for details.  
    /// 
    /// # Support
    /// All Android version.
    pub fn read(
        &self,
        base_dir: PrivateDir, 
        relative_path: impl AsRef<str>, 
    ) -> crate::Result<Vec<u8>> {

        on_android!({
            let path = self.resolve_path_with(base_dir, relative_path)?;
            Ok(std::fs::read(path)?)
        })
    }

    /// Reads the entire contents of a file into a string.  
    /// 
    /// If you need [`std::fs::File`], use [`PrivateStorage::open_file`] insted.  
    /// 
    /// This internally uses [`PrivateStorage::resolve_path`] and [`std::fs::read_to_string`].  
    /// See [`PrivateStorage::resolve_path`] for details.  
    /// 
    /// # Support
    /// All Android version.
    pub fn read_to_string(
        &self,
        base_dir: PrivateDir,
        relative_path: impl AsRef<str>, 
    ) -> crate::Result<String> {

        on_android!({
            let path = self.resolve_path_with(base_dir, relative_path)?;
            Ok(std::fs::read_to_string(path)?)
        })
    }

    /// Returns an iterator over the entries within a directory.
    /// 
    /// This internally uses [`PrivateStorage::resolve_path`] and [`std::fs::read_dir`].  
    /// See [`PrivateStorage::resolve_path`] for details.  
    /// 
    /// # Support
    /// All Android version.
    pub fn read_dir(
        &self,
        base_dir: PrivateDir,
        relative_path: Option<&str>,
    ) -> crate::Result<std::fs::ReadDir> {

        on_android!({
            let path = match relative_path {
                Some(relative_path) => self.resolve_path_with(base_dir, relative_path)?,
                None => self.resolve_path(base_dir)?,
            };
    
            Ok(std::fs::read_dir(path)?)
        })
    }

    /// Removes a file from the filesystem.  
    /// 
    /// This internally uses [`PrivateStorage::resolve_path`] and [`std::fs::remove_file`].  
    /// See [`PrivateStorage::resolve_path`] for details.  
    /// 
    /// # Support
    /// All Android version.
    pub fn remove_file(
        &self,
        base_dir: PrivateDir,
        relative_path: impl AsRef<str>,
    ) -> crate::Result<()> {

        on_android!({
            let path = self.resolve_path_with(base_dir, relative_path)?;
            Ok(std::fs::remove_file(path)?)
        })
    }

    /// Removes an empty directory.  
    /// If you want to remove a directory that is not empty, as well as all of its contents recursively, consider using [`PrivateStorage::remove_dir_all`] instead.  
    /// 
    /// This internally uses [`PrivateStorage::resolve_path`] and [`std::fs::remove_dir`].  
    /// See [`PrivateStorage::resolve_path`] for details.  
    /// 
    /// # Support
    /// All Android version.
    pub fn remove_dir(
        &self,
        base_dir: PrivateDir,
        relative_path: Option<&str>,
    ) -> crate::Result<()> {

        on_android!({
            let path = match relative_path {
                Some(relative_path) => self.resolve_path_with(base_dir, relative_path)?,
                None => self.resolve_path(base_dir)?,
            };
    
            std::fs::remove_dir(path)?;
            Ok(())
        })
    }

    /// Removes a directory at this path, after removing all its contents. Use carefully!  
    /// 
    /// This internally uses [`PrivateStorage::resolve_path`] and [`std::fs::remove_dir_all`].  
    /// See [`PrivateStorage::resolve_path`] for details.  
    /// 
    /// # Support
    /// All Android version.
    pub fn remove_dir_all(
        &self,
        base_dir: PrivateDir,
        relative_path: Option<&str>,
    ) -> crate::Result<()> {

        on_android!({
            let path = match relative_path {
                Some(relative_path) => self.resolve_path_with(base_dir, relative_path)?,
                None => self.resolve_path(base_dir)?,
            };
    
            std::fs::remove_dir_all(path)?;
            Ok(())
        })
    }

    /// Returns Ok(true) if the path points at an existing entity.  
    /// 
    /// This internally uses [`PrivateStorage::resolve_path`] and [`std::fs::exists`].  
    /// See [`PrivateStorage::resolve_path`] for details.  
    /// 
    /// # Support
    /// All Android version.
    pub fn exists(
        &self,
        base_dir: PrivateDir,
        relative_path: impl AsRef<str>
    ) -> crate::Result<bool> {

        on_android!({
            let path = self.resolve_path_with(base_dir, relative_path)?;
            Ok(std::fs::exists(path)?)
        })
    }

    /// Queries the file system to get information about a file, directory.  
    /// 
    /// This internally uses [`PrivateStorage::resolve_path`] and [`std::fs::metadata`].  
    /// See [`PrivateStorage::resolve_path`] for details.  
    /// 
    /// # Support
    /// All Android version.
    pub fn metadata(
        &self,
        base_dir: PrivateDir,
        relative_path: Option<&str>,
    ) -> crate::Result<std::fs::Metadata> {

        on_android!({
            let path = match relative_path {
                Some(relative_path) => self.resolve_path_with(base_dir, relative_path)?,
                None => self.resolve_path(base_dir)?,
            };
    
            Ok(std::fs::metadata(path)?)
        })
    }


    #[allow(unused)]
    pub(crate) fn create_new_tmp_file(&self) -> crate::Result<(std::fs::File, std::path::PathBuf)> {
        on_android!({
            let tmp_file_path = {
                use std::sync::atomic::{AtomicUsize, Ordering};

                static COUNTER: AtomicUsize = AtomicUsize::new(0);
                let id = COUNTER.fetch_add(1, Ordering::Relaxed);

                let tmp_dir_path = self.resolve_tmp_dir()?;
                let _ = std::fs::create_dir_all(&tmp_dir_path);
            
                tmp_dir_path.join(format!("tmp file {id}"))
            };
            
            let tmp_file = std::fs::File::create_new(&tmp_file_path)?;

            Ok((tmp_file, tmp_file_path))
        })
    }

    #[allow(unused)]
    pub(crate) fn remove_all_tmp_files(&self) -> crate::Result<()> {
        on_android!({
            match std::fs::remove_dir_all(self.resolve_tmp_dir()?) {
                Ok(_) => Ok(()),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
                Err(e) => Err(e.into()),
            }
        })
    }

    #[allow(unused)]
    pub(crate) fn resolve_tmp_dir(&self) -> crate::Result<std::path::PathBuf> {
        on_android!({
            const TMP_DIR_RELATIVE_PATH: &str = "pluginAndroidFs-tmpDir-01K486FKQ2BZSBGFD34RFH9FWJ";

            self.resolve_path_with(PrivateDir::Cache, TMP_DIR_RELATIVE_PATH)
        })
    }
}