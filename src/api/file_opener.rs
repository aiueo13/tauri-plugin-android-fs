use sync_async::sync_async;
use crate::*;
use super::*;


/// API of opening file/dir with other apps.
/// 
/// # Examples
/// ```no_run
/// fn example(app: &tauri::AppHandle) {
///     use tauri_plugin_android_fs::AndroidFsExt as _;
/// 
///     let api = app.android_fs();
///     let file_sender = api.file_sender();
/// }
/// ```
#[sync_async]
pub struct FileOpener<'a, R: tauri::Runtime> {
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
impl<'a, R: tauri::Runtime> FileOpener<'a, R> {
    
    #[always_sync]
    fn impls(&self) -> Impls<'_, R> {
        Impls { handle: &self.handle }
    }
}

#[sync_async(
    use(if_async) api_async::{AndroidFs, FilePicker, PrivateStorage, PublicStorage, WritableStream};
    use(if_sync) api_sync::{AndroidFs, FilePicker, PrivateStorage, PublicStorage, WritableStream};
)]
impl<'a, R: tauri::Runtime> FileOpener<'a, R> {

    /// Show app chooser for sharing files with other apps.   
    /// This function returns immediately after requesting to open the app chooser, 
    /// without waiting for the app’s response. 
    /// 
    /// This sends the files as a single unit.
    /// The available apps depend on the MIME types associated with the files.  
    /// This does not result in an error even if no available apps are found. 
    /// An empty app chooser is displayed.
    /// 
    /// # Args
    /// - ***uris*** :  
    /// Target file URIs to share.  
    /// This all needs to be **readable**.  
    /// URIs converted directly from a path, such as via [`FileUri::from_path`], can **not** be used.   
    /// 
    /// # Support
    /// All Android version.
    /// 
    /// # References
    /// - <https://developer.android.com/reference/android/content/Intent#ACTION_SEND_MULTIPLE>
    /// - <https://developer.android.com/reference/android/content/Intent#ACTION_SEND>
    #[maybe_async]
    pub fn share_files<'b>(
        &self, 
        uris: impl IntoIterator<Item = &'b FileUri>, 
    ) -> Result<()> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().show_share_file_app_chooser(uris).await
        }
    }

    /// Show app chooser for sharing file with other apps.    
    /// This function returns immediately after requesting to open the app chooser, 
    /// without waiting for the app’s response. 
    /// 
    /// The available apps depend on the MIME type associated with the file.  
    /// This does not result in an error even if no available apps are found. 
    /// An empty app chooser is displayed.
    /// 
    /// # Args
    /// - ***uri*** :  
    /// Target file URI to share.  
    /// Must be **readable**.  
    /// URIs converted directly from a path, such as via [`FileUri::from_path`], can **not** be used.  
    /// 
    /// # Support
    /// All Android version.
    /// 
    /// # References
    /// - <https://developer.android.com/reference/android/content/Intent#ACTION_SEND>
    #[maybe_async]
    pub fn share_file(
        &self, 
        uri: &FileUri,
    ) -> Result<()> {
        
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().show_share_file_app_chooser([uri]).await
        }
    }

    /// Show app chooser for opening file with other apps.   
    /// This function returns immediately after requesting to open the app chooser, 
    /// without waiting for the app’s response. 
    /// 
    /// The available apps depend on the MIME type associated with the file.  
    /// This does not result in an error even if no available apps are found. 
    /// An empty app chooser is displayed.
    /// 
    /// # Args
    /// - ***uri*** :  
    /// Target file URI to view.  
    /// Must be **readable**.  
    /// URIs converted directly from a path, such as via [`FileUri::from_path`], can **not** be used.  
    /// 
    /// # Support
    /// All Android version.
    /// 
    /// # References
    /// - <https://developer.android.com/reference/android/content/Intent#ACTION_VIEW>
    #[maybe_async]
    pub fn open_file(
        &self, 
        uri: &FileUri,
    ) -> Result<()> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().show_open_file_app_chooser(uri).await
        }
    }

    /// Show app chooser for opening dir with other apps.   
    /// This function returns immediately after requesting to open the app chooser, 
    /// without waiting for the app’s response. 
    ///   
    /// This does not result in an error even if no available apps are found. 
    /// An empty app chooser is displayed.
    /// 
    /// # Args
    /// - ***uri*** :  
    /// Target dir URI to view.  
    /// Must be **readable**.  
    /// URIs converted directly from a path, such as via [`FileUri::from_path`], can **not** be used.  
    /// 
    /// # Support
    /// All Android version.
    /// 
    /// # References
    /// - <https://developer.android.com/reference/android/content/Intent#ACTION_VIEW>
    #[maybe_async]
    pub fn open_dir(
        &self, 
        uri: &FileUri,
    ) -> Result<()> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().show_open_dir_app_chooser(uri).await
        }
    }

    /// Show app chooser for editing file with other apps.   
    /// This function returns immediately after requesting to open the app chooser, 
    /// without waiting for the app’s response. 
    /// 
    /// The available apps depend on the MIME type associated with the file.  
    /// This does not result in an error even if no available apps are found. 
    /// An empty app chooser is displayed.
    /// 
    /// # Note
    /// I think that this may be the least commonly used request for sending file to app.  
    /// Even if you want to open an image or video editing app, 
    /// [`FileOpener::open_file`] allows you to choose from a wider range of apps in many cases.
    /// 
    /// # Args
    /// - ***uri*** :  
    /// Target file URI to view.  
    /// Must be **read-writeable**.  
    /// URIs converted directly from a path, such as via [`FileUri::from_path`], can **not** be used.  
    /// 
    /// # Support
    /// All Android version.
    /// 
    /// # References
    /// - <https://developer.android.com/reference/android/content/Intent#ACTION_EDIT>
    #[maybe_async]
    pub fn edit_file(
        &self, 
        uri: &FileUri,
    ) -> Result<()> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
           self.impls().show_edit_file_app_chooser(uri).await
        }
    }
}