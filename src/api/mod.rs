macro_rules! on_android {
    ($action: expr) => {{
        #[cfg(not(target_os = "android"))] {
            Err(crate::Error::NotAndroid)
        }
        #[cfg(target_os = "android")] {
            $action
        }
    }};
    ($phantom: ty, $action: expr) => {{
        #[cfg(not(target_os = "android"))] {
            Err::<$phantom, _>(crate::Error::NotAndroid)
        }
        #[cfg(target_os = "android")] {
            $action
        }
    }};
}

macro_rules! impl_se {
    (struct $struct_ident:ident $(< $lifetime:lifetime >)? { $( $name:ident: $ty:ty ),* $(,)? }) => {
        #[derive(serde::Serialize)]
        #[serde(rename_all = "camelCase")]
        struct $struct_ident $(< $lifetime >)? {
            $($name: $ty,)*
        }
    };
}

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
mod private_storage;
mod public_storage;

pub use android_fs::AndroidFs;
pub use private_storage::PrivateStorage;
pub use public_storage::PublicStorage;