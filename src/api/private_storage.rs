use sync_async::sync_async;
use crate::*;
use super::*;


/// API of file storage intended for the app’s use only.  
/// 
/// # Examples
/// ```no_run
/// async fn example(app: &tauri::AppHandle) {
///     use tauri_plugin_android_fs::AndroidFsExt as _;
/// 
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

    /// Get an absolute path of the app-specific directory on the internal storage.  
    /// App can fully manage entries within this directory via [`std::fs`] and etc.   
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

    /// See [`PrivateStorage::resolve_path`] and [`FileUri::from_path`]
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