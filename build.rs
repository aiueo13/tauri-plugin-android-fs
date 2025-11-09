const PERMISSIONS_FOR_ANDROID_9_OR_LOWER: &'static str = r#"
<uses-permission android:name="android.permission.WRITE_EXTERNAL_STORAGE" android:maxSdkVersion="28" />
<uses-permission android:name="android.permission.READ_EXTERNAL_STORAGE" android:maxSdkVersion="28" />
"#;

const PERMISSIONS_FOR_ANDROID_10_OR_LOWER: &'static str = r#"
<uses-permission android:name="android.permission.WRITE_EXTERNAL_STORAGE" android:maxSdkVersion="29" />
<uses-permission android:name="android.permission.READ_EXTERNAL_STORAGE" android:maxSdkVersion="29" />
"#;

fn main() {
	tauri_plugin::Builder::new(&[])
		.android_path("android")
		.build();

	if std::env::var("CARGO_FEATURE_LEGACY_STORAGE_PERMISSION_INCLUDE_ANDROID_10").is_ok() {
		tauri_plugin::mobile::update_android_manifest(
			"ANDROID FS PLUGIN",
			"manifest",
			PERMISSIONS_FOR_ANDROID_10_OR_LOWER.trim().to_string()
		)
		.expect("failed to rewrite AndroidManifest.xml");
	}
	else if std::env::var("CARGO_FEATURE_LEGACY_STORAGE_PERMISSION").is_ok() {
		tauri_plugin::mobile::update_android_manifest(
			"ANDROID FS PLUGIN",
			"manifest",
			PERMISSIONS_FOR_ANDROID_9_OR_LOWER.trim().to_string()
		)
		.expect("failed to rewrite AndroidManifest.xml");
	}
	else {
		// 必要ない場合は上書きして宣言を消す
		tauri_plugin::mobile::update_android_manifest(
			"ANDROID FS PLUGIN",
			"manifest",
			"".to_string()
		)
		.expect("failed to rewrite AndroidManifest.xml");
	}
}