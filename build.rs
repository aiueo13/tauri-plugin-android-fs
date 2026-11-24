#[path = "src/scope/mod.rs"]
mod scope;

const COMMANDS: &'static [&'static str] = &[
    "get_android_api_level",
    "get_uri_for_file_path",
    "get_name",
    "get_type",
    "get_metadata",
    "get_file_mime_type",
    "get_file_byte_length",
    "get_file_thumbnail",
    "get_file_thumbnail_as_base64",
    "get_file_thumbnail_as_data_url",
    "list_volumes",
    "create_new_public_file",
    "create_new_public_image_file",
    "create_new_public_video_file",
    "create_new_public_audio_file",
    "scan_public_file",
    "set_public_file_pending",
    "request_public_files_permission",
    "check_public_files_permission",
    "create_new_file",
    "create_new_dir",
    "create_dir",
    "truncate_file",
    "copy_file",
    "count_all_file_streams",
    "close_all_file_streams",
    "open_read_file_stream",
    "open_read_text_file_lines_stream",
    "open_write_file_stream",
    "read_file",
    "read_file_as_base64",
    "read_file_as_data_url",
    "read_text_file",
    "write_file",
    "write_text_file",
    "read_dir",
    "rename_file",
    "rename_dir",
    "check_picker_uri_permission",
    "persist_picker_uri_permission",
    "check_persisted_picker_uri_permission",
    "release_persisted_picker_uri_permission",
    "release_all_persisted_picker_uri_permissions",
    "list_all_persisted_picker_uri_permissions",
    "remove_file",
    "remove_empty_dir",
    "remove_dir_all",
    "show_open_file_picker",
    "show_open_dir_picker",
    "show_save_file_picker",
    "show_share_file_app_chooser",
    "show_view_file_app_chooser",
    "show_view_dir_app_chooser",
    "show_edit_file_app_chooser",
    "get_mime_type_from_extension",
    "register_readonly_custom_file",
    "unregister_custom_file"
];

fn main() {
    tauri_plugin::Builder::new(COMMANDS)
        .android_path("android")
        .global_scope_schema(schemars::schema_for!(scope::Scope))
        .build();

    let permissions = {
        let mut p = Vec::new();
        let media_store_image_read_permission = std::env::var("CARGO_FEATURE_MEDIA_STORE_IMAGE_READ_PERMISSION").is_ok();
        let media_store_video_read_permission = std::env::var("CARGO_FEATURE_MEDIA_STORE_VIDEO_READ_PERMISSION").is_ok();
        let media_store_audio_read_permission = std::env::var("CARGO_FEATURE_MEDIA_STORE_AUDIO_READ_PERMISSION").is_ok();
        let notification_permission = std::env::var("CARGO_FEATURE_NOTIFICATION_PERMISSION").is_ok();
        let legacy_storage_permission_include_android_10 = std::env::var("CARGO_FEATURE_LEGACY_STORAGE_PERMISSION_INCLUDE_ANDROID_10").is_ok();
        let legacy_storage_permission = std::env::var("CARGO_FEATURE_LEGACY_STORAGE_PERMISSION").is_ok();

        let read_media_store =
            media_store_image_read_permission ||
            media_store_video_read_permission ||
            media_store_audio_read_permission;

        if notification_permission {
            p.push(r#"<uses-permission android:name="android.permission.POST_NOTIFICATIONS" />"#);
        }

        if legacy_storage_permission_include_android_10 {
            p.push(r#"<uses-permission android:name="android.permission.WRITE_EXTERNAL_STORAGE" android:maxSdkVersion="29" />"#);
        }
	    else if legacy_storage_permission {
            p.push(r#"<uses-permission android:name="android.permission.WRITE_EXTERNAL_STORAGE" android:maxSdkVersion="28" />"#);
        }

        if read_media_store {
            p.push(r#"<uses-permission android:name="android.permission.READ_EXTERNAL_STORAGE" android:maxSdkVersion="32" />"#);
        }
        else if legacy_storage_permission_include_android_10 {
            p.push(r#"<uses-permission android:name="android.permission.READ_EXTERNAL_STORAGE" android:maxSdkVersion="29" />"#);
        }
        else if legacy_storage_permission {
            p.push(r#"<uses-permission android:name="android.permission.READ_EXTERNAL_STORAGE" android:maxSdkVersion="28" />"#);
        }
        
        if media_store_image_read_permission {
            p.push(r#"<uses-permission android:name="android.permission.READ_MEDIA_IMAGES" />"#);
        }

        if media_store_video_read_permission {
            p.push(r#"<uses-permission android:name="android.permission.READ_MEDIA_VIDEO" />"#);
        }

        if media_store_audio_read_permission {
            p.push(r#"<uses-permission android:name="android.permission.READ_MEDIA_AUDIO" />"#);
        }

        if read_media_store {
            p.push(r#"<uses-permission android:name="android.permission.READ_MEDIA_VISUAL_USER_SELECTED" />"#);
        }

        p
    };

    tauri_plugin::mobile::update_android_manifest(
        "ANDROID FS PLUGIN",
        "manifest",
        // 空の文字列の場合でも書き込むことで古い必要ない宣言を上書きして消すことができる。
        permissions.join("\n"),
    ).expect("failed to rewrite AndroidManifest.xml");

    let providers = {
        let mut p = Vec::new();

        if std::env::var("CARGO_FEATURE_CUSTOM_FILE_PROVIDER").is_ok() {
            p.push(vec![
                r#"<provider"#,
                r#"    android:name="okayu.tauri.plugin.android.fs.AFCustomFileProvider""#,
                r#"    android:authorities="${applicationId}.tpafs-custom-file-provider""#,
                r#"    android:exported="false""#,
                r#"    android:grantUriPermissions="true" >"#,
                r#"</provider>"#,
            ].join("\n"));
        }

        p
    };

    tauri_plugin::mobile::update_android_manifest(
        "PROVIDERS FOR ANDROID FS PLUGIN",
        "application",
        // 空の文字列の場合でも書き込むことで古い必要ない宣言を上書きして消すことができる。
        providers.join("\n"),
    ).expect("failed to rewrite AndroidManifest.xml");
}