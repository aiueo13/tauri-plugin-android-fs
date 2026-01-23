//! Overview and usage is [here](https://crates.io/crates/tauri-plugin-android-fs)

#![allow(unused_variables)]

mod models;
mod cmds;
mod consts;

pub mod api;

pub use models::*;
pub use consts::*;

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
                app.manage(afs_sync);
                app.manage(afs_async);

                let app_handle = app.app_handle().clone();
                std::thread::spawn(move || {
                    let afs = app_handle.android_fs();

                    // 前回作成した一時ファイルを全て削除
                    afs.impls().remove_all_temp_files().ok();
                });
            }
            #[cfg(not(target_os = "android"))] {
                let afs_sync = crate::api::api_sync::AndroidFs::<R> { handle: Default::default() };
                let afs_async = crate::api::api_async::AndroidFs::<R> { handle: Default::default() };
                app.manage(afs_sync);
                app.manage(afs_async);
            }

            Ok(())
        })
        .js_init_script(format!(
            "window.__TAURI_ANDROID_FS_PLUGIN_INTERNALS__ = {{ isAndroid: {} }};",
            cfg!(target_os = "android")
        ))
        .invoke_handler(tauri::generate_handler![
            cmds::get_android_api_level,
            cmds::get_name,
            cmds::get_byte_length,
            cmds::get_mime_type,
            cmds::get_type,
            cmds::get_metadata,
            cmds::get_fs_path,
            cmds::get_thumbnail,
            cmds::get_thumbnail_base64,
            cmds::get_thumbnail_data_url,
            cmds::list_volumes,
            cmds::create_new_public_file,
            cmds::create_new_public_image_file,
            cmds::create_new_public_video_file,
            cmds::create_new_public_audio_file,
            cmds::scan_public_file,
            cmds::request_public_files_permission,
            cmds::has_public_files_permission,
            cmds::create_new_file,
            cmds::create_dir_all,
            cmds::copy_file,
            cmds::truncate_file,
            cmds::read_dir,
            cmds::rename_file,
            cmds::rename_dir,
            cmds::remove_file,
            cmds::remove_empty_dir,
            cmds::remove_dir_all,
            cmds::check_picker_uri_permission,
            cmds::persist_picker_uri_permission,
            cmds::check_persisted_picker_uri_permission,
            cmds::release_persisted_picker_uri_permission,
            cmds::release_all_persisted_picker_uri_permissions,
            cmds::show_open_file_picker,
            cmds::show_open_dir_picker,
            cmds::show_save_file_picker,
            cmds::show_share_file_dialog,
            cmds::show_view_file_dialog,
            cmds::show_view_dir_dialog,

            /* deprecacted */
            cmds::persist_uri_permission,
            cmds::check_persisted_uri_permission,
            cmds::release_persisted_uri_permission,
            cmds::release_all_persisted_uri_permissions,
        ])
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