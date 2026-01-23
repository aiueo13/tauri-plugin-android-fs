Note: **I’m using a translation tool, so there may be some inappropriate expressions.**

# Overview
This plugin provides a unified file system API for all Android versions supported by Tauri.

# Setup
First, install this plugin to your Tauri project:

`src-tauri/Cargo.toml`

```toml
[dependencies]
tauri-plugin-android-fs = { version = "24.2", features = ["legacy_storage_permission"] }
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

```
pnpm add tauri-plugin-android-fs-api
# or
npm add tauri-plugin-android-fs-api
# or
yarn add tauri-plugin-android-fs-api
```

**NOTE**: Please make sure that the Rust-side `tauri-plugin-android-fs` and the JavaScript-side `tauri-plugin-android-fs-api` versions match.

# Usage
This plugin operates on files and directories via URIs rather than paths.  

By using `AndroidFs.getFsPath`, you can obtain a path from a URI and use it with the functions provided by Tauri's file system, [`@tauri-apps/plugin-fs`](https://v2.tauri.app/ja/plugin/file-system/), to read and write files. For those paths, there is no need for you to set [the scope configuration of plugin-fs](https://v2.tauri.app/reference/javascript/fs/#security).

```typescript
import { AndroidFs, AndroidPublicGeneralPurposeDir } from 'tauri-plugin-android-fs-api';
import { writeTextFile } from '@tauri-apps/plugin-fs';

/**
 * Save the text to '~/Download/MyApp/{fileName}'
 */
async function saveText(fileName: string, data: string): Promise<void> {
    const baseDir = AndroidPublicGeneralPurposeDir.Download;
    const relativePath = `MyApp/${fileName}`;
    const mimeType = "text/plain";

    const uri = await AndroidFs.createNewPublicFile(baseDir, relativePath, mimeType);

    try {
        const path = await AndroidFs.getFsPath(uri);
        await writeTextFile(path, data);
        await AndroidFs.scanPublicFile(uri);
    }
    catch (e) {
        await AndroidFs.removeFile(uri).catch(() => {});
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

### 1. APIs to obtain entries such as files and directories.
- `AndroidFs.showOpenFilePicker` 
- `AndroidFs.showOpenDirPicker` 
- `AndroidFs.showSaveFilePicker` 
- `AndroidFs.readDir` 
- `AndroidFs.createNewFile` 
- `AndroidFs.createDirAll` 
- `AndroidFs.createNewPublicFile` 
- `AndroidFs.createNewPublicImageFile` 
- `AndroidFs.createNewPublicVideoFile` 
- `AndroidFs.createNewPublicAudioFile` 

### 3. APIs to retrieve data from entries.
- `AndroidFs.getThumbnail` 
- `AndroidFs.getThumbnailBase64` 
- `AndroidFs.getThumbnailDataUrl` 
- `AndroidFs.getFsPath` 
- `AndroidFs.getName` 
- `AndroidFs.getByteLength` 
- `AndroidFs.getType` 
- `AndroidFs.getMimeType` 
- `AndroidFs.getMetadata` 

### 3. APIs to operate entries.
- `AndroidFs.scanPublicFile`
- `AndroidFs.copyFile`
- `AndroidFs.truncateFile`
- `AndroidFs.removeFile`
- `AndroidFs.removeEmptyDir`
- `AndroidFs.removeDirAll`

### 4. APIs to manage entry permissions
- `AndroidFs.checkPickerUriPermission`
- `AndroidFs.persistPickerUriPermission`
- `AndroidFs.checkPersistedPickerUriPermission`
- `AndroidFs.releasePersistedPickerUriPermission`
- `AndroidFs.releaseAllPersistedPickerUriPermissions`
- `AndroidFs.hasPublicFilesPermission`
- `AndroidFs.requestPublicFilesPermission`

### 5. APIs to send entries to other apps.
- `AndroidFs.showViewFileDialog`
- `AndroidFs.showViewDirDialog`
- `AndroidFs.showShareFileDialog`

### 6. Helper
- `isAndroid`
- `getAndroidApiLevel`


For simplicity, some features and detailed options of the API have been omitted. If you need them, please consider using the [`tauri-plugin-android-fs`](https://crates.io/crates/tauri-plugin-android-fs) on the Rust side.

# License
This project is licensed under either of

 * MIT license
 * Apache License (Version 2.0)

at your option.