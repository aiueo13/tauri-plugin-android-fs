Note: **I’m using a translation tool, so there may be some inappropriate expressions.**

# Overview
This plugin provides a unified file system API for all Android versions supported by Tauri.

# Setup
First, install this plugin to your Tauri project:

`src-tauri/Cargo.toml`

```toml
[dependencies]
tauri-plugin-android-fs = { version = "=27.0.0", features = [
    # For `AndroidFs.createNewPublicFile` and related APIs on Android 9 or lower
    "legacy_storage_permission"
] }
```

Next, register this plugin in your Tauri project:

`src-tauri/src/lib.rs`

```rust
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_android_fs::init()) // This
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

Then, set the APIs that can be called from the Javascript:

`src-tauri/capabilities/*.json`
```json
{
    "permissions": [
        "android-fs:default"
    ]
}
```

Finally, install the JavaScript Guest bindings using whichever JavaScript package manager you prefer:

```bash
pnpm add tauri-plugin-android-fs-api@27.0.0 -E
# or
npm install tauri-plugin-android-fs-api@27.0.0 --save-exact
# or
yarn add tauri-plugin-android-fs-api@27.0.0 --exact
```

**NOTE**: Please make sure that the Rust-side `tauri-plugin-android-fs` and the JavaScript-side `tauri-plugin-android-fs-api` versions match exactly.

# Usage
This plugin operates on files and directories via URIs rather than paths.  

```typescript
import { AndroidFs, AndroidPublicImageDir } from 'tauri-plugin-android-fs-api';

/** 
 * Saves an image to '~/Pictures/MyApp/{fileName}'
 */
async function saveImage(
	fileName: string,
	mimeType: string,
	data: Uint8Array | ReadableStream<Uint8Array>,
): Promise<void> {

	let uri;
	try {
		// Creates a new empty file
		uri = await AndroidFs.createNewPublicImageFile(
			AndroidPublicImageDir.Pictures,
			`MyApp/${fileName}`,
			mimeType,
			{ isPending: true }
		);

		// Writes data to the file
		if (data instanceof Uint8Array) {
			await AndroidFs.writeFile(uri, data);
		}
		else if (data instanceof ReadableStream) {
			const writer = await AndroidFs.openWriteFileStream(uri);
			await data.pipeTo(writer);
		}
		else {
			throw new TypeError("Unsupported data type");
		}

		// Makes the file visible in other apps and gallery
		await AndroidFs.setPublicFilePending(uri, false);
		await AndroidFs.scanPublicFile(uri);
	}
	// Handles error and cleanup
	catch (e) {
		if (data instanceof ReadableStream) {
			await data.cancel(e).catch(() => { });
		}
		if (uri != null) {
			await AndroidFs.removeFile(uri).catch(() => { });
		}
		throw e;
	}
}
```

When passing URIs to this plugin's functions, no scope configuration is required.  
Some functions accept not only URIs but also absolute paths. In this case, you need to set the scope configuration, [like in plugin-fs](https://v2.tauri.app/reference/javascript/fs/#security).


`src-tauri/capabilities/*.json`
```json
{
    "permissions": [
        {
            "identifier": "android-fs:scope",
            "allow": [
                "$APPDATA/my-data/**/*"
            ],
            "deny": [
                "$APPDATA/my-data/secret.txt"
            ]
        }
    ]
}
```

And you can also assign a specific scope to a particular command.

`src-tauri/capabilities/*.json`
```json
{
    "permissions": [
        {
            "identifier": "android-fs:allow-copy-file",
            "allow": [
                "$APPDATA/my-data/**/*"
            ],
            "deny": [
                "$APPDATA/my-data/secret.txt"
            ]
        }
    ]
}
```

**Note**: A dedicated `my-data` subdirectory is used because resolved directories may already contain files created by the WebView system or other Tauri plugins. This helps prevent file name collisions and unintended access.

# APIs
This plugin provides following APIs:

### 1. APIs to obtain entries such as files and directories
- `AndroidFs.showOpenFilePicker` 
- `AndroidFs.showOpenDirPicker` 
- `AndroidFs.showSaveFilePicker` 
- `AndroidFs.readDir` 
- `AndroidFs.createNewFile` 
- `AndroidFs.createDir` 
- `AndroidFs.createNewPublicFile` 
- `AndroidFs.createNewPublicImageFile` 
- `AndroidFs.createNewPublicVideoFile` 
- `AndroidFs.createNewPublicAudioFile` 

### 2. APIs to retrieve entry data
- `AndroidFs.getFsPath` 
- `AndroidFs.getMetadata` 
- `AndroidFs.getName` 
- `AndroidFs.getType` 
- `AndroidFs.getMimeType` 
- `AndroidFs.getByteLength` 
- `AndroidFs.getThumbnail` 
- `AndroidFs.getThumbnailAsBytes` 
- `AndroidFs.getThumbnailAsBase64` 
- `AndroidFs.getThumbnailAsDataURL` 

### 3. APIs to operate entries
- `AndroidFs.copyFile`
- `AndroidFs.truncateFile`
- `AndroidFs.renameFile`
- `AndroidFs.renameDir`
- `AndroidFs.removeFile`
- `AndroidFs.removeEmptyDir`
- `AndroidFs.removeDirAll`
- `AndroidFs.scanPublicFile`
- `AndroidFs.setPublicFilePending`

### 4. APIs to read files
- `AndroidFs.openReadFileStream`
- `AndroidFs.openReadTextFileLinesStream`
- `AndroidFs.readFile`
- `AndroidFs.readFileAsBase64`
- `AndroidFs.readFileAsDataURL`
- `AndroidFs.readTextFile`

### 5. APIs to write to files
- `AndroidFs.openWriteFileStream`
- `AndroidFs.writeFile`
- `AndroidFs.writeTextFile`

### 6. APIs to manage permissions
- `AndroidFs.checkPickerUriPermission`
- `AndroidFs.persistPickerUriPermission`
- `AndroidFs.checkPersistedPickerUriPermission`
- `AndroidFs.releasePersistedPickerUriPermission`
- `AndroidFs.releaseAllPersistedPickerUriPermissions`
- `AndroidFs.hasPublicFilesPermission`
- `AndroidFs.requestPublicFilesPermission`

### 7. APIs to send entries to other apps
- `AndroidFs.showViewFileDialog`
- `AndroidFs.showViewDirDialog`
- `AndroidFs.showShareFileDialog`

### 8. Helper
- `isAndroid`
- `getAndroidApiLevel`


For simplicity, some features and detailed options of the API have been omitted. If you need them, please consider using the [`tauri-plugin-android-fs`](https://crates.io/crates/tauri-plugin-android-fs) on the Rust side.

# License
This project is licensed under either of

 * MIT license
 * Apache License (Version 2.0)

at your option.