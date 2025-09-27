macro_rules! on_android {
    ($action: expr) => {{
        #[cfg(not(target_os = "android"))] {
            Err(crate::Error { msg: std::borrow::Cow::Borrowed("This plugin is only for Android") })
        }
        #[cfg(target_os = "android")] {
            $action
        }
    }};
    ($phantom: ty, $action: expr) => {{
        #[cfg(not(target_os = "android"))] {
            Err::<$phantom, _>(crate::Error { msg: std::borrow::Cow::Borrowed("This plugin is only for Android") })
        }
        #[cfg(target_os = "android")] {
            $action
        }
    }};
}

#[allow(unused)]
macro_rules! impl_se {
    (struct $struct_ident:ident $(< $lifetime:lifetime >)? { $( $name:ident: $ty:ty ),* $(,)? }) => {
        #[derive(serde::Serialize)]
        #[serde(rename_all = "camelCase")]
        struct $struct_ident $(< $lifetime >)? {
            $($name: $ty,)*
        }
    };
}

#[allow(unused)]
macro_rules! impl_de {
    (struct $struct_ident:ident $(< $lifetime:lifetime >)? { $( $name:ident: $ty:ty ),* $(,)? }) => {
        #[derive(serde::Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct $struct_ident $(< $lifetime >)? {
            $($name: $ty,)*
        }
    };
    (struct $struct_ident:ident $(;)?) => {
        #[derive(serde::Deserialize)]
        struct $struct_ident;
    };
}

mod android_fs;
mod file_picker;
mod file_opener;
mod private_storage;
mod public_storage;
mod writable_stream;

pub use android_fs::AndroidFs;
pub use file_picker::FilePicker;
pub use file_opener::{FileOpener, FileSender};
pub use private_storage::PrivateStorage;
pub use public_storage::PublicStorage;
pub use writable_stream::WritableStream;