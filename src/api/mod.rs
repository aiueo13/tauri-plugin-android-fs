#[cfg(target_os = "android")]
mod impls;

mod android_fs;
mod file_opener;
mod file_picker;
mod app_storage;
mod private_storage;
mod public_storage;

pub mod api_async {
    pub use crate::api::android_fs::AsyncAndroidFs as AndroidFs;
    pub use crate::api::file_opener::AsyncFileOpener as FileOpener;
    pub use crate::api::file_picker::AsyncFilePicker as FilePicker;
    pub use crate::api::app_storage::AsyncAppStorage as AppStorage;
    pub use crate::api::private_storage::AsyncPrivateStorage as PrivateStorage;
    pub use crate::api::public_storage::AsyncPublicStorage as PublicStorage;
}

pub mod api_sync {
    pub use crate::api::android_fs::SyncAndroidFs as AndroidFs;
    pub use crate::api::file_opener::SyncFileOpener as FileOpener;
    pub use crate::api::file_picker::SyncFilePicker as FilePicker;
    pub use crate::api::app_storage::SyncAppStorage as AppStorage;
    pub use crate::api::private_storage::SyncPrivateStorage as PrivateStorage;
    pub use crate::api::public_storage::SyncPublicStorage as PublicStorage;
}


/// A guard that removes the file on drop
pub struct TempFileGuard {
    path: std::path::PathBuf
}

impl Drop for TempFileGuard {

    fn drop(&mut self) {
        std::fs::remove_file(&self.path).ok();
    }
}