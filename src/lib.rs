//! Overview and usage is [here](https://crates.io/crates/tauri-plugin-android-fs)

#![cfg_attr(not(target_os = "android"), allow(unused))]

mod builder;
mod cmds;
mod config;
mod protocols;
mod scope;
mod utils;

pub mod api;

use utils::*;

pub use api::consts::*;
pub use api::models::*;
pub use builder::*;

/// Creates the plugin builder.
///
/// # Usage
/// `src-tauri/src/lib.rs`
/// ```ignore
/// #[cfg_attr(mobile, tauri::mobile_entry_point)]
/// pub fn run() {
///     tauri::Builder::default()
///         .plugin(
///             tauri_plugin_android_fs::builder()
///                 .build()
///         )
///         .run(tauri::generate_context!())
///         .expect("error while running tauri application");
/// }
/// ```
pub fn builder<R: tauri::Runtime>() -> Builder<R> {
    Builder::new()
}

/// Initializes the plugin.
///
/// This is same as `builder().build()`.
///
/// # Usage
/// `src-tauri/src/lib.rs`
/// ```ignore
/// #[cfg_attr(mobile, tauri::mobile_entry_point)]
/// pub fn run() {
///     tauri::Builder::default()
///         .plugin(tauri_plugin_android_fs::init())
///         .run(tauri::generate_context!())
///         .expect("error while running tauri application");
/// }
/// ```
pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R, Option<config::Config>> {
    Builder::new().build()
}

pub trait AndroidFsExt<R: tauri::Runtime> {
    /// Provides an API for accessing the Android file system.
    ///
    /// It is a blocking-based API. If you need an asynchronous API, use [`AndroidFsExt::android_fs_async`].
    fn android_fs(&self) -> &api::api_sync::AndroidFs<R>;

    /// Provides an asynchronous API for accessing the Android file system.
    fn android_fs_async(&self) -> &api::api_async::AndroidFs<R>;
}

impl<R: tauri::Runtime, T: tauri::Manager<R>> AndroidFsExt<R> for T {
    fn android_fs(&self) -> &api::api_sync::AndroidFs<R> {
        self.try_state::<api::api_sync::AndroidFs<R>>()
            .map(|i| i.inner())
            .expect("tauri_plugin_android_fs should be initialized to use; see https://crates.io/crates/tauri-plugin-android-fs")
    }

    fn android_fs_async(&self) -> &api::api_async::AndroidFs<R> {
        self.try_state::<api::api_async::AndroidFs<R>>()
            .map(|i| i.inner())
            .expect("tauri_plugin_android_fs should be initialized to use; see https://crates.io/crates/tauri-plugin-android-fs")
    }
}
