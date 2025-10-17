#[cfg(target_os = "android")]
mod impls;

mod android_fs;
mod file_opener;
mod file_picker;
mod private_storage;
mod public_storage;
mod writable_stream;

pub mod api_async {
    pub use crate::api::android_fs::AsyncAndroidFs as AndroidFs;
    pub use crate::api::file_opener::AsyncFileOpener as FileOpener;
    pub use crate::api::file_picker::AsyncFilePicker as FilePicker;
    pub use crate::api::private_storage::AsyncPrivateStorage as PrivateStorage;
    pub use crate::api::public_storage::AsyncPublicStorage as PublicStorage;
    pub use crate::api::writable_stream::AsyncWritableStream as WritableStream;
}

pub mod api_sync {
    pub use crate::api::android_fs::SyncAndroidFs as AndroidFs;
    pub use crate::api::file_opener::SyncFileOpener as FileOpener;
    pub use crate::api::file_picker::SyncFilePicker as FilePicker;
    pub use crate::api::private_storage::SyncPrivateStorage as PrivateStorage;
    pub use crate::api::public_storage::SyncPublicStorage as PublicStorage;
    pub use crate::api::writable_stream::SyncWritableStream as WritableStream;
}