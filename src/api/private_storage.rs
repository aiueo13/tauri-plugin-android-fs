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
    /// App can fully manage entries within this directory.   
    /// 
    /// This function does **not** create any directories; it only constructs the path.
    ///
    /// These entries will be deleted when the app is uninstalled and may also be deleted at the user’s initialising request.  
    /// When using [`PrivateDir::Cache`], the system will automatically delete entries as disk space is needed elsewhere on the device.   
    /// 
    /// Since the returned paths can change when the app is moved to an adopted storage device, 
    /// only relative paths should be stored.
    /// 
    /// # Support
    /// All Android version.
    pub fn resolve_path(
        &self, 
        dir: PrivateDir
    ) -> crate::Result<std::path::PathBuf> {

        on_android!({
            impl_de!(struct Paths {
                data: String, 
                cache: String
            });
        
            static PATHS: std::sync::OnceLock<Paths> = std::sync::OnceLock::new();

            if PATHS.get().is_none() {
                let _ = PATHS.set(
                    self.0.api.run_mobile_plugin::<Paths>("getPrivateBaseDirAbsolutePaths", "")?
                );
            }
            let paths = PATHS.get().expect("Should call 'set' before 'get'");

            Ok(match dir {
                PrivateDir::Data => std::path::PathBuf::from(paths.data.to_owned()),
                PrivateDir::Cache => std::path::PathBuf::from(paths.cache.to_owned()),
            })
        })
    }

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