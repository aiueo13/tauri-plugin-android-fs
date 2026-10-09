#[cfg(target_os = "android")]
use std::sync::LazyLock;
use std::sync::Arc;
use crate::*;


/// Plugin builder.
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
pub struct Builder<R: tauri::Runtime> {
    custom_file_providers: Vec<(String, Box<dyn FnOnce(tauri::AppHandle<R>) -> Arc<dyn CustomFileProvider> + Send + 'static>)>,
}

impl<R: tauri::Runtime> Builder<R> {

    pub fn new() -> Builder<R> {
        Self {
            custom_file_providers: Vec::new(),
        }
    }

    /// Registers a provider for providing arbitrary custom files.
    /// 
    /// **Note**: Enable `custom_file_provider` feature to use.
    /// 
    /// This allows large files that do not physically exist on disk, 
    /// such as files on network shares or cloud storage services,
    /// to be accessed quickly,
    /// while also enabling content to be streamed on demand.
    /// 
    /// The specified provider name
    /// and the logical path of a custom file can be passed to [`AndroidFs::build_custom_file_uri`]
    /// to obtain an [`FsUri`] for accessing the custom file.
    /// The URI is an Android Content URI
    /// and can be shared with other applications using [`Opener`], for example.
    /// 
    /// This custom file provider remains valid until the current application process terminates,
    /// after which it can no longer be accessed.
    /// URIs created by a previous application process are rejected 
    /// before they are handled by the provided [`CustomFileProvider`].
    /// 
    /// If only read-only custom files with fixed metadata and data need to be provided, 
    /// the simplified [`AndroidFs::register_simple_readonly_custom_file`] can be used instead.
    /// 
    /// # Security
    /// Custom files are accessed internally through Android's `ContentProvider`,
    /// but it is not exported.
    /// In other words, other applications can only access custom files that are
    /// explicitly shared by this application, such as through an [`Opener`] or
    /// Android `Intent` with the appropriate URI permission granted.
    /// Other applications cannot enumerate or access arbitrary custom files.
    /// 
    /// The logical path passed to [`AndroidFs::build_custom_file_uri`] is percent-encoded and included in the resulting URI, 
    /// so it must not contain any information that should not be exposed.
    ///  
    /// # Support
    /// This API is available on all Android versions supported by Tauri. 
    /// However, some [`CustomFile`] variants are only available on Android 8.0 (API level 26) or higher. 
    /// See the documentation for `CustomFile::from_*` for details.
    /// 
    /// [`Opener`]: crate::api::api_async::Opener
    /// [`AndroidFs::register_simple_readonly_custom_file`]: crate::api::api_async::AndroidFs::register_simple_readonly_custom_file
    /// [`AndroidFs::build_custom_file_uri`]: crate::api::api_async::AndroidFs::build_custom_file_uri
    pub fn register_custom_file_provider<P: CustomFileProvider>(
        mut self,
        provider_name: impl Into<String>,
        init_provider: impl FnOnce(tauri::AppHandle<R>) -> P + Send + 'static
    ) -> Self {

        self.custom_file_providers.push((
            provider_name.into(), 
            Box::new(|app| Arc::new((init_provider)(app))))
        );
        self
    }

    pub fn build(self) -> tauri::plugin::TauriPlugin<R, Option<config::Config>> {
        let builder = tauri::plugin::Builder::<R, Option<config::Config>>::new("android-fs")
            .setup(move |app, api| {
                use tauri::Manager as _;

                #[cfg(target_os = "android")] {
                    let handle = api.register_android_plugin(
                        crate::api::PLUGIN_PACKAGE,
                        crate::api::PLUGIN_MAIN_CLASS
                    )?;
                    let afs_sync = crate::api::api_sync::AndroidFs { handle: handle.clone() };
                    let afs_async = crate::api::api_async::AndroidFs { handle: handle.clone() };
                    app.manage(afs_sync);
                    app.manage(afs_async);

                    app.manage(api::new_simple_custom_files_state(app.app_handle().clone()));

                    #[cfg(feature = "commands")] {
                        app.manage(cmds::new_file_stream_resources_state(app.app_handle().clone()));
                        app.manage(cmds::new_file_writer_resources_state(app.app_handle().clone()));
                    }

                    #[cfg(any(feature = "protocol_content", feature = "protocol_thumbnail"))] {
                        app.manage(protocols::new_config_state(api.config().as_ref(), app));
                    }

                    let mut custom_file_providers = self.custom_file_providers;

                    custom_file_providers.push((
                        api::SIMPLE_CUSTOM_FILE_PROVIDER_NAME.to_string(),
                        Box::new(move |app| {
                            Arc::new(api::SimpleCustomFileProvider::new(app))
                        })
                    ));

                    let file_provider_config = api
                        .config()
                        .as_ref()
                        .map(|c| c.file_provider.clone())
                        .unwrap_or_default();

                    if cfg!(feature = "file_provider") && file_provider_config.enable {

                        custom_file_providers.push((
                            crate::api::FILE_PROVIDER_NAME.to_string(),
                            Box::new(move |app| {
                                let scope = tauri::scope::fs::Scope::new(
                                    &app,
                                    &file_provider_config.scope
                                );

                                let resolve_path = move |path: String| {
                                    let scope = scope.as_ref().ok()?;

                                    let (path, _) = path.rsplit_once("/TpafsMimeType=")?;
                                    let path = std::path::PathBuf::from(path);
                                    if !path.is_absolute() {
                                        return None
                                    }
                                    let path = tauri::path::SafePathBuf::new(path).ok()?;
                                    let path = path.as_ref();

                                    if !scope.is_allowed(path) {
                                        return None
                                    }

                                    Some(path.to_path_buf())
                                };
                                
                                let get_mime_type = move |_resolved_path: std::path::PathBuf, path: String| -> std::io::Result<_> {
                                    let (_, mime_type) = path
                                        .rsplit_once("/TpafsMimeType=")
                                        .ok_or_else(|| std::io::Error::other("invalid uri format"))?;

                                    Ok(Some(mime_type.to_string()))
                                };

                                Arc::new(StdFileProvider::new_custom(app, resolve_path, get_mime_type))
                            })
                        ));
                    }

                    let mut providers = Vec::new();
                    for (provider_name, init_provider) in custom_file_providers {
                        let app = app.app_handle().clone();

                        // setup 呼び出し時では全てのプラグインが初期化済みであるとは限らないので
                        // init_provider 内で何らかのプラグインが使われるとパニックする可能性がある。
                        // よってこの setup 内では provider を初期化せず、
                        // provider が初めにアクセスされた際に lazy に初期化する。
                        let provider = LazyLock::new(move || (init_provider)(app));
                        let provider = move || Arc::clone(&*provider);
                        providers.push((provider_name, provider));
                    }
                    crate::api::CUSTOM_FILE_PROVIDERS.add_providers(providers);
                }
                #[cfg(not(target_os = "android"))] {
                    let afs_sync = crate::api::api_sync::AndroidFs::<R> { handle: Default::default() };
                    let afs_async = crate::api::api_async::AndroidFs::<R> { handle: Default::default() };
                    app.manage(afs_sync);
                    app.manage(afs_async);
                }

                Ok(())
            });

        #[cfg(feature = "commands")]
        let builder = builder
            .js_init_script(format!(
                "window.__TAURI_ANDROID_FS_PLUGIN_INTERNALS__ = {{ isAndroid: {} }};",
                cfg!(target_os = "android")
            ))
            .invoke_handler(tauri::generate_handler![
                cmds::get_android_api_level,
                cmds::get_uri_for_file_path,
                cmds::get_name,
                cmds::get_type,
                cmds::get_metadata,
                cmds::get_file_byte_length,
                cmds::get_file_mime_type,
                cmds::get_file_thumbnail,
                cmds::get_file_thumbnail_as_base64,
                cmds::get_file_thumbnail_as_data_url,
                cmds::register_readonly_custom_file,
                cmds::unregister_custom_file,
                cmds::list_volumes,
                cmds::create_new_public_file,
                cmds::create_new_public_image_file,
                cmds::create_new_public_video_file,
                cmds::create_new_public_audio_file,
                cmds::scan_public_file,
                cmds::set_public_file_pending,
                cmds::request_public_files_permission,
                cmds::check_public_files_permission,
                cmds::create_new_file,
                cmds::create_new_dir,
                cmds::create_dir,
                cmds::count_all_file_streams,
                cmds::close_all_file_streams,
                cmds::open_read_file_stream,
                cmds::open_read_text_file_lines_stream,
                cmds::open_write_file_stream,
                cmds::read_file,
                cmds::read_file_as_base64,
                cmds::read_file_as_data_url,
                cmds::read_text_file,
                cmds::write_file,
                cmds::write_text_file,
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
                cmds::list_all_persisted_picker_uri_permissions,
                cmds::show_open_file_picker,
                cmds::show_open_dir_picker,
                cmds::show_save_file_picker,
                cmds::show_share_file_app_chooser,
                cmds::show_view_file_app_chooser,
                cmds::show_view_dir_app_chooser,
                cmds::show_edit_file_app_chooser,
                cmds::get_mime_type_from_extension,
            ]);

        #[cfg(all(target_os = "android", feature = "protocol_thumbnail"))]
        let builder = builder
            .register_asynchronous_uri_scheme_protocol(
                protocols::protocol_thumbnail::URI_SCHEME, 
                protocols::protocol_thumbnail::protocol,
            );

        #[cfg(all(target_os = "android", feature = "protocol_content"))]
        let builder = builder
            .register_asynchronous_uri_scheme_protocol(
                protocols::protocol_content::URI_SCHEME, 
                protocols::protocol_content::protocol,
            );
    
        builder.build()
    }
}