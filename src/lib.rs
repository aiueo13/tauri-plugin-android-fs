//! Overview and usage is [here](https://crates.io/crates/tauri-plugin-android-fs)

#![allow(unused_variables)]

mod models;
mod api;
mod consts;

pub use models::*;
pub use api::*;
pub use consts::*;

#[cfg(target_os = "android")]
mod utils;
#[cfg(target_os = "android")]
pub(crate) use utils::*;

/// Initializes the plugin.
/// 
/// # Usage
/// `src-tauri/src/lib.rs`
/// ```
/// #[cfg_attr(mobile, tauri::mobile_entry_point)]
/// pub fn run() {
///     tauri::Builder::default()
///         .plugin(tauri_plugin_android_fs::init())
///         .run(tauri::generate_context!())
///         .expect("error while running tauri application");
/// }
/// ```
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
            .expect("You should register this plugin by tauri_plugin_android_fs::init(). See https://crates.io/crates/tauri-plugin-android-fs")
    }
}