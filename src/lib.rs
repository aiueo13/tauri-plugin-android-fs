//! Overview and usage is [here](https://crates.io/crates/tauri-plugin-android-fs)

#![allow(unused_variables)]

mod models;
mod consts;
mod utils;

pub mod api;

pub use models::*;
pub use consts::*;

#[allow(unused_imports)]
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

            #[cfg(target_os = "android")] {
                let handle = api.register_android_plugin("com.plugin.android_fs", "AndroidFsPlugin")?;
                let afs_sync = crate::api::api_sync::AndroidFs { handle: handle.clone() };
                let afs_async = crate::api::api_async::AndroidFs { handle: handle.clone() };

                // クリーンアップされなかった一時ファイルを全て削除
                afs_sync.impls().remove_all_tmp_files().ok();

                app.manage(afs_sync);
                app.manage(afs_async);
            }
            #[cfg(not(target_os = "android"))] {
                let afs_sync = crate::api::api_sync::AndroidFs::<R> { handle: Default::default() };
                let afs_async = crate::api::api_async::AndroidFs::<R> { handle: Default::default() };
                app.manage(afs_sync);
                app.manage(afs_async);
            }

            Ok(())
        })
        .build()
}

pub trait AndroidFsExt<R: tauri::Runtime> {

    fn android_fs(&self) -> &api::api_sync::AndroidFs<R>;

    fn android_fs_async(&self) -> &api::api_async::AndroidFs<R>;
}

impl<R: tauri::Runtime, T: tauri::Manager<R>> AndroidFsExt<R> for T {

    fn android_fs(&self) -> &api::api_sync::AndroidFs<R> {
        self.try_state::<api::api_sync::AndroidFs<R>>()
            .map(|i| i.inner())
            .expect("should register this plugin by tauri_plugin_android_fs::init(). see https://crates.io/crates/tauri-plugin-android-fs")
    }

    fn android_fs_async(&self) -> &api::api_async::AndroidFs<R> {
        self.try_state::<api::api_async::AndroidFs<R>>()
            .map(|i| i.inner())
            .expect("should register this plugin by tauri_plugin_android_fs::init(). see https://crates.io/crates/tauri-plugin-android-fs")
    }
}