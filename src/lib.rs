//! Overview and usage is [here](https://crates.io/crates/tauri-plugin-android-fs)

#![allow(unused_variables)]

mod models;
mod error;
mod api;

#[cfg(target_os = "android")]
mod utils;

pub mod api_level;

pub use models::*;
pub use error::*;
pub use api::*;

#[cfg(target_os = "android")]
pub(crate) use utils::*;

/// Initializes the plugin.
pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("android-fs")
        .setup(|app, api| {
            use tauri::Manager as _;

            let afs = AndroidFs::new(app.clone(), api)?;

            #[cfg(target_os = "android")] {
                // Cleanup temporary files;
                let _ = afs
                    .private_storage()
                    .remove_all_tmp_files();
            }

            app.manage(afs);
            Ok(())
        })
        .build()
}

pub trait AndroidFsExt<R: tauri::Runtime> {

    fn android_fs(&self) -> &AndroidFs<R>;
}

impl<R: tauri::Runtime, T: tauri::Manager<R>> AndroidFsExt<R> for T {

    fn android_fs(&self) -> &AndroidFs<R> {
        self.try_state::<AndroidFs<R>>()
            .map(|i| i.inner())
            .expect("You should call tauri_plugin_android_fs::init() and registier it to your project. See https://crates.io/crates/tauri-plugin-android-fs")
    }
}