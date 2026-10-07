#[cfg(target_os = "android")]
mod impls;

mod android_fs;
mod opener;
mod picker;
mod app_storage;
mod private_storage;
mod public_storage;
mod notification;
mod progress_notification_guard;

pub(crate) mod models;
pub(crate) mod consts;

#[cfg(target_os = "android")]
pub(crate) use impls::{CUSTOM_FILE_PROVIDERS, PLUGIN_MAIN_CLASS, PLUGIN_PACKAGE, FILE_PROVIDER_NAME};

pub mod api_async {
    pub use crate::api::android_fs::AsyncAndroidFs as AndroidFs;
    pub use crate::api::opener::AsyncOpener as Opener;
    pub use crate::api::picker::AsyncPicker as Picker;
    pub use crate::api::app_storage::AsyncAppStorage as AppStorage;
    pub use crate::api::private_storage::AsyncPrivateStorage as PrivateStorage;
    pub use crate::api::public_storage::AsyncPublicStorage as PublicStorage;
    pub use crate::api::notification::AsyncNotifications as Notifications;
    pub use crate::api::progress_notification_guard::AsyncProgressNotificationGuard as ProgressNotificationGuard;
}

pub mod api_sync {
    pub use crate::api::android_fs::SyncAndroidFs as AndroidFs;
    pub use crate::api::opener::SyncOpener as Opener;
    pub use crate::api::picker::SyncPicker as Picker;
    pub use crate::api::app_storage::SyncAppStorage as AppStorage;
    pub use crate::api::private_storage::SyncPrivateStorage as PrivateStorage;
    pub use crate::api::public_storage::SyncPublicStorage as PublicStorage;
    pub use crate::api::notification::SyncNotifications as Notifications;
    pub use crate::api::progress_notification_guard::SyncProgressNotificationGuard as ProgressNotificationGuard;
}