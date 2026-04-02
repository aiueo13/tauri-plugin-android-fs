use sync_async::sync_async;
use crate::*;
use super::*;


/// API of utils.
#[sync_async]
pub struct Utils<'a, R: tauri::Runtime> {
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
impl<'a, R: tauri::Runtime> Utils<'a, R> {
    
    #[always_sync]
    fn impls(&self) -> Impls<'_, R> {
        Impls { handle: &self.handle }
    }
}

#[sync_async(
    use(if_async) api_async::{AndroidFs, ProgressNotificationGuard};
    use(if_sync) api_sync::{AndroidFs, ProgressNotificationGuard};
)]
impl<'a, R: tauri::Runtime> Utils<'a, R> {

    /// Displays a notification indicating progress on status bar.  
    /// 
    /// The returned [`ProgressNotificationGuard`] can be used to manage the notification.
    /// - Calling [`ProgressNotificationGuard::update`] will update the notification.  
    /// - Calling [`ProgressNotificationGuard::complete`] will finish the notification as a "success".  
    /// - Calling [`ProgressNotificationGuard::fail`] will finish the notification as a "failure".
    /// - Calling [`ProgressNotificationGuard::cancel`] will finish and close the notification.  
    /// 
    /// By default, it finishes the notification as a "failure" when dropped.   
    /// You can change the drop behavior by using `ProgressNotificationGuard::set_drop_behavior_to_*`.  
    /// 
    /// # Note
    /// This needs two steps: 
    /// 
    /// 1. Declare :  
    ///     By enabling the `notification_permission` feature,  
    ///     you can declare the permissions automatically at build time.  
    ///
    /// 2. Runtime request :  
    ///     By calling [`Utils::request_notification_permission`],
    ///     you can request the permissions from the user at runtime.  
    /// 
    /// # Support
    /// All Android version.
    #[maybe_async]
    pub fn create_progress_notification(
        &self,
        icon: ProgressNotificationIcon,
        title: Option<&str>,
        text: Option<&str>,
        sub_text: Option<&str>,
        progress: Option<u64>,
        progress_max: Option<u64>,
    ) -> Result<ProgressNotificationGuard<R>> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            ProgressNotificationGuard::with_new_notification(
                icon,
                title.map(|s| s.to_string()), 
                text.map(|s| s.to_string()),
                sub_text.map(|s| s.to_string()),
                progress,
                progress_max,
                self.handle.clone()
            ).await
        }
    }

    #[maybe_async]
    pub fn cancel_all_notifications(&self) -> Result<()> {
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().cancel_all_notifications().await
        }
    }

    /// # Note
    /// To request the permissions, you must declare [`POST_NOTIFICATIONS`](https://developer.android.com/reference/android/Manifest.permission#POST_NOTIFICATIONS) in `AndroidManifest.xml`.
    /// By enabling the `notification_permission` feature,
    /// the permissions will be declared automatically.
    #[maybe_async]
    pub fn request_notification_permission(&self) -> Result<bool> {
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().request_notification_permission().await
        }
    }

    #[maybe_async]
    pub fn check_notification_permission(&self) -> Result<bool> {
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().check_notification_permission().await
        }
    }
}