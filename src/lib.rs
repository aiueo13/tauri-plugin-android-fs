//! Overview and usage is [here](https://crates.io/crates/tauri-plugin-android-fs)

#![allow(unused)]

mod models;
mod error;
mod api;
mod utils;

pub use models::*;
pub use error::{Error, Result};
pub use api::{AndroidFs, PrivateStorage, PublicStorage};
pub(crate) use utils::*;

/// Initializes the plugin.
pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    let builder = tauri::plugin::Builder::new("android-fs")
        .setup(|app, api| {
            use tauri::Manager as _;

            let afs = AndroidFs::new(app.clone(), api)?;

            // Cleanup temporary directory
            let _ = afs
                .private_storage()
                .remove_dir_all(PrivateDir::Cache, Some(TMP_DIR_RELATIVE_PATH));

            app.manage(afs);
            Ok(())
        });

    // https://github.com/aiueo13/tauri-plugin-android-fs/issues/1
    #[cfg(all(feature = "avoid-issue1", target_os = "android"))]
    let builder = {
        const SCRIPT: &str = "
            ;(async function () {
                const noop = async () => await window.__TAURI_INTERNALS__.invoke('plugin:android-fs|noop');

                setInterval(noop, 800)
            })();
        ";

        #[tauri::command]
        fn noop() {}

        builder
            .invoke_handler(tauri::generate_handler![noop])
            .js_init_script(SCRIPT.into())  
    };

    builder.build()
}

pub trait AndroidFsExt<R: tauri::Runtime> {

    fn android_fs(&self) -> &AndroidFs<R>;
}

impl<R: tauri::Runtime, T: tauri::Manager<R>> AndroidFsExt<R> for T {

    fn android_fs(&self) -> &AndroidFs<R> {
        self.state::<AndroidFs<R>>().inner()
    }
}