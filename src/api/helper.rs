use sync_async::sync_async;
use crate::*;
use super::*;


#[sync_async]
pub struct Helper<'a, R: tauri::Runtime> {
    #[cfg(target_os = "android")]
    pub(crate) handle: &'a tauri::plugin::PluginHandle<R>,

    #[cfg(not(target_os = "android"))]
    pub(crate) handle: &'a tauri::AppHandle<R>,
}

#[cfg(target_os = "android")]
#[sync_async(
    use(if_sync) impls::SyncImpls as Impls;
    use(if_async) impls::AsyncImpls as Impls;
)]
impl<'a, R: tauri::Runtime> Helper<'a, R> {
    
    #[always_sync]
    fn impls(&self) -> Impls<'_, R> {
        Impls { handle: &self.handle }
    }
}

#[sync_async(
    use(if_async) api_async::{AndroidFs, Opener, Picker, PublicStorage, PrivateStorage};
    use(if_sync) api_sync::{AndroidFs, Opener, Picker, PublicStorage, PrivateStorage};
)]
impl<'a, R: tauri::Runtime> Helper<'a, R> {

    #[maybe_async]
    pub fn open_app_settings(&self) -> Result<()> {
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().open_app_settings().await
        }
    }

    #[maybe_async]
    pub fn show_open_app_settings_dialog(
        &self,
        message: &str,
        open_button_label: &str,
    ) -> Result<()> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().show_open_app_settings_dialog(
                message, 
                open_button_label, 
            ).await
        }
    }

    /// Get the api level of this Android device.
    ///
    /// The correspondence table between API levels and Android versions can be found following.
    /// <https://developer.android.com/guide/topics/manifest/uses-sdk-element#api-level-table>
    ///
    /// If you want the constant value of the API level from an Android version, there is the [`api_level`] module.
    ///
    /// # Table
    /// | Android version  | API Level |
    /// |------------------|-----------|
    /// | 16.0             | 36        |
    /// | 15.0             | 35        |
    /// | 14.0             | 34        |
    /// | 13.0             | 33        |
    /// | 12L              | 32        |
    /// | 12.0             | 31        |
    /// | 11.0             | 30        |
    /// | 10.0             | 29        |
    /// | 9.0              | 28        |
    /// | 8.1              | 27        |
    /// | 8.0              | 26        |
    /// | 7.1 - 7.1.2      | 25        |
    /// | 7.0              | 24        |
    ///
    /// Tauri does not support Android versions below 7.
    #[always_sync]
    pub fn api_level(&self) -> Result<i32> {
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().api_level()
        }
    }

    /// Gets the application id of this Android app.
    #[always_sync]
    pub fn app_id(&self) -> Result<&'static str> {
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            Ok(&self.impls().consts()?.app_id)
        }
    }
}