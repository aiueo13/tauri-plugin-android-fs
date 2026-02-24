#[path = "src/cmds/scope.rs"]
mod scope;

const PERMISSIONS_FOR_ANDROID_9_OR_LOWER: &'static str = r#"
<uses-permission android:name="android.permission.WRITE_EXTERNAL_STORAGE" android:maxSdkVersion="28" />
<uses-permission android:name="android.permission.READ_EXTERNAL_STORAGE" android:maxSdkVersion="28" />
"#;

const PERMISSIONS_FOR_ANDROID_10_OR_LOWER: &'static str = r#"
<uses-permission android:name="android.permission.WRITE_EXTERNAL_STORAGE" android:maxSdkVersion="29" />
<uses-permission android:name="android.permission.READ_EXTERNAL_STORAGE" android:maxSdkVersion="29" />
"#;

const COMMANDS: &'static [&'static str] = &[
    "get_android_api_level",
    "get_name",
    "get_byte_length",
    "get_type",
    "get_mime_type",
    "get_metadata",
    "get_thumbnail",
    "get_thumbnail_base64",
    "get_thumbnail_data_url",
    "get_fs_path",
    "list_volumes",
    "create_new_public_file",
    "create_new_public_image_file",
    "create_new_public_video_file",
    "create_new_public_audio_file",
    "scan_public_file",
    "set_public_file_pending",
    "request_public_files_permission",
    "has_public_files_permission",
    "create_new_file",
    "create_dir_all",
    "truncate_file",
    "copy_file",
    "count_all_file_streams",
    "close_all_file_streams",
    "open_read_file_stream",
    "open_read_text_file_lines_stream",
    "open_write_file_stream",
    "read_file",
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
    "remove_file",
    "remove_empty_dir",
    "remove_dir_all",
    "show_open_file_picker",
    "show_open_dir_picker",
    "show_save_file_picker",
    "show_share_file_dialog",
    "show_view_file_dialog",
    "show_view_dir_dialog",
];

fn main() {
    tauri_plugin::Builder::new(COMMANDS)
        .android_path("android")
        .global_scope_schema(schemars::schema_for!(scope::ScopeSchema))
        .build();

    if std::env::var("CARGO_FEATURE_LEGACY_STORAGE_PERMISSION_INCLUDE_ANDROID_10").is_ok() {
        tauri_plugin::mobile::update_android_manifest(
            "ANDROID FS PLUGIN",
            "manifest",
            PERMISSIONS_FOR_ANDROID_10_OR_LOWER.trim().to_string(),
		).expect("failed to rewrite AndroidManifest.xml");
    }
	else if std::env::var("CARGO_FEATURE_LEGACY_STORAGE_PERMISSION").is_ok() {
        tauri_plugin::mobile::update_android_manifest(
            "ANDROID FS PLUGIN",
            "manifest",
            PERMISSIONS_FOR_ANDROID_9_OR_LOWER.trim().to_string(),
        ).expect("failed to rewrite AndroidManifest.xml");
    }
	else {
        // 必要ない場合は上書きして宣言を消す
        tauri_plugin::mobile::update_android_manifest(
            "ANDROID FS PLUGIN",
            "manifest",
            "".to_string(),
        ).expect("failed to rewrite AndroidManifest.xml");
    }
}
