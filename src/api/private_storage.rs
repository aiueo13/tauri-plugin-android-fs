use sync_async::sync_async;
use crate::*;
use super::*;


/// API of file storage intended for the app’s use only.  
/// 
/// # Examples
/// ```no_run
/// use tauri_plugin_android_fs::{AndroidFsExt, PrivateDir};
/// 
/// async fn example(app: &tauri::AppHandle) {
///     let api = app.android_fs_async();
///     let ps = api.private_storage();
/// 
///     // Resolve the absolute paths.
///     // Files and directories in these locations can be fully managed using `std::fs`.
///     let cache_dir_path: std::path::PathBuf = ps.resolve_path(PrivateDir::Cache).await?;
///     let data_dir_path: std::path::PathBuf = ps.resolve_path(PrivateDir::Data).await?;
///     let nobackup_data_dir_path: std::path::PathBuf = ps.resolve_path(PrivateDir::NoBackupData).await?;
///
///     // These directories may also contain files created by other Tauri plugins
///     // or the WebView runtime. To avoid conflicts, it is recommended to use
///     // a uniquely named subdirectory for your application.
///     let cache_dir_path = cache_dir_path.join("01K6049FVCD4SAGMAB6X20SA5S");
///     let data_dir_path = data_dir_path.join("01K6049FVCD4SAGMAB6X20SA5S");
///     let nobackup_data_dir_path = nobackup_data_dir_path.join("01K6049FVCD4SAGMAB6X20SA5S");
/// }
/// ```
#[sync_async]
pub struct PrivateStorage<'a, R: tauri::Runtime> {
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
impl<'a, R: tauri::Runtime> PrivateStorage<'a, R> {
    
    #[always_sync]
    fn impls(&self) -> Impls<'_, R> {
        Impls { handle: &self.handle }
    }
}

#[sync_async(
    use(if_async) api_async::{AndroidFs, Opener, Picker, PublicStorage};
    use(if_sync) api_sync::{AndroidFs, Opener, Picker, PublicStorage};
)]
impl<'a, R: tauri::Runtime> PrivateStorage<'a, R> {

    /// Returns the absolute path of an app-specific directory on the internal storage.
    ///
    /// Files and directories in this location can be managed directly using [`std::fs`].
    ///
    /// This function does not guarantee directory creation. 
    ///
    /// Since these locations may also contain files created
    /// by other Tauri plugins or by the WebView runtime, 
    /// it is recommended to create a uniquely named subdirectory for your application.
    ///
    /// # Notes
    /// Files in these locations are removed when the app is uninstalled.
    /// 
    /// When using [`PrivateDir::Cache`], the system may automatically delete files
    /// when additional storage space is needed. 
    /// Applications should not rely on this behavior and should clear cache files explicitly.
    /// 
    /// These directories are inaccessible to other apps under normal circumstances.
    /// On rooted devices or when the user has elevated privileges,
    /// their contents may still be accessible.
    /// 
    /// The returned path may change if the app is moved to adopted storage.
    /// Persist only relative paths if the path needs to be stored.
    /// 
    /// Each Android user has a separate app-specific directory.
    /// 
    /// # Support
    /// All Android versions supported by Tauri.
    #[maybe_async]
    pub fn resolve_path(
        &self, 
        dir: PrivateDir
    ) -> Result<std::path::PathBuf> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().private_dir_path(dir).map(Clone::clone)
        }
    }

    /// See [`PrivateStorage::resolve_path`] and [`FsUri::from_path`]
    #[maybe_async]
    pub fn resolve_uri(
        &self, 
        dir: PrivateDir,
        relative_path: impl AsRef<std::path::Path>
    ) -> Result<FsUri> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            let mut path = self.resolve_path(dir).await?;
            path.push(relative_path.as_ref());
            Ok(path.into())
        }
    }
}