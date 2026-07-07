Note: **I’m using a translation tool, so some expressions may be awkward or inaccurate.**

# Overview
This plugin provides a unified file system API across all Android versions supported by Tauri.

# Setup
First, install the plugin to your Tauri project:

`src-tauri/Cargo.toml`

```toml
[dependencies]
tauri-plugin-android-fs = {
  version = "=28.4.0",
  features = [
    # To access public files on older Android versions
    "legacy_storage_permission",
    # To enable notification options
    "notification_permission"
  ]
}
```

Next, register the plugin:

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

Then, configure the APIs that can be called from the frontend JavaScript bindings:

`src-tauri/capabilities/*.json`
```json
{
    "permissions": [
        "android-fs:default"
    ]
}
```

Finally, install the frontend JavaScript bindings:

```bash
pnpm add tauri-plugin-android-fs-api@28.4.0 -E
# or
npm install tauri-plugin-android-fs-api@28.4.0 --save-exact
# or
yarn add tauri-plugin-android-fs-api@28.4.0 --exact
```

**NOTE**: Please ensure that the backend package, `tauri-plugin-android-fs` (crates io), and the frontend package, `tauri-plugin-android-fs-api` (npm), have exactly matching versions.

[![crates.io](https://img.shields.io/crates/v/tauri-plugin-android-fs.svg?color=yellow)](https://crates.io/crates/tauri-plugin-android-fs) [![npm version](https://img.shields.io/npm/v/tauri-plugin-android-fs-api.svg?color=red)](https://www.npmjs.com/package/tauri-plugin-android-fs-api)

# Usage
This plugin operates on files and directories via URIs rather than paths.  

When passing URIs to this plugin's functions, no scope configuration is required.  
This is because the plugin only provides and accepts URIs whose permissions are already managed by the Android system, such as those explicitly selected by the user through a file picker or files created by the app in public directories.

Some functions accept not only URIs but also absolute paths, including app-specific directories. In this case, you need to set the scope configuration for security, [like in plugin-fs](https://v2.tauri.app/reference/javascript/fs/#security).  
You can set a global scope for the plugin, or assign specific scopes to individual commands:

`src-tauri/capabilities/*.json`
```json
{
    "permissions": [
        {
            "identifier": "android-fs:scope",
            "allow": ["$APPDATA/my-data/**/*"],
            "deny": ["$APPDATA/my-data/secret.txt"]
        },
        {
            "identifier": "android-fs:allow-copy-file",
            "allow": ["$APPDATA/my-data/**/*"]
        }
    ]
}
```

# Example

```typescript
import { 
  AndroidFs, 
  AndroidPublicGeneralPurposeDir, 
} from 'tauri-plugin-android-fs-api';

/** 
 * Saves data to '~/Download/MyApp/{fileName}'
 */
async function download(
  fileName: string,
  mimeType: string,
  data: Uint8Array | ReadableStream<Uint8Array>,
): Promise<void> {

  let uri;
  try {
    // Creates a new empty file
    uri = await AndroidFs.createNewPublicFile(
      AndroidPublicGeneralPurposeDir.Download,
      `MyApp/${fileName}`,
      mimeType,
      { isPending: true }
    );

    // Writes data to the file
    if (data instanceof ReadableStream) {
      const writer = await AndroidFs.openWriteFileStream(uri);
      await data.pipeTo(writer);
    }
    else {
      await AndroidFs.writeFile(uri, data);
    }

    // Makes the file visible in other apps and gallery
    await AndroidFs.setPublicFilePending(uri, false);
    await AndroidFs.scanPublicFile(uri);
  }
  // Handles an error and cleanup
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

```json
{
    "permissions": [
        "android-fs:allow-create-new-public-file",
        "android-fs:allow-open-write-file-stream",
        "android-fs:allow-write-file",
        "android-fs:allow-set-public-file-pending",
        "android-fs:allow-scan-public-file",
        "android-fs:allow-remove-file"
    ]
}
```

# API
This plugin provides following APIs:

### 1. APIs to get entries such as files and directories
- `AndroidFs.showOpenFilePicker` 
- `AndroidFs.showOpenDirPicker` 
- `AndroidFs.showSaveFilePicker` 
- `AndroidFs.readDir` 
- `AndroidFs.createNewFile` 
- `AndroidFs.createNewDir` 
- `AndroidFs.createDir` 
- `AndroidFs.createNewPublicFile` 
- `AndroidFs.createNewPublicImageFile` 
- `AndroidFs.createNewPublicVideoFile` 
- `AndroidFs.createNewPublicAudioFile` 
- `AndroidFs.listVolumes`

### 2. APIs to operate entries
- `AndroidFs.copyFile`
- `AndroidFs.truncateFile`
- `AndroidFs.renameFile`
- `AndroidFs.renameDir`
- `AndroidFs.removeFile`
- `AndroidFs.removeEmptyDir`
- `AndroidFs.removeDirAll`
- `AndroidFs.scanPublicFile`
- `AndroidFs.setPublicFilePending`

### 3. APIs to get entry data
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

### 4. APIs to get source URLs
- `AndroidFs.convertFileSrc`
- `AndroidFs.convertThumbnailSrc`

### 5. APIs to read/write files
- `AndroidFs.readFile`
- `AndroidFs.readFileAsBase64`
- `AndroidFs.readFileAsDataURL`
- `AndroidFs.readTextFile`
- `AndroidFs.writeFile`
- `AndroidFs.writeTextFile`

### 6. APIs to stream files
- `AndroidFs.openReadFileStream`
- `AndroidFs.openReadTextFileLinesStream`
- `AndroidFs.openWriteFileStream`
- `AndroidFs.closeAllFileStreams`
- `AndroidFs.countAllFileStreams`

### 7. APIs to send entries to other apps
- `AndroidFs.showViewFileDialog`
- `AndroidFs.showViewDirDialog`
- `AndroidFs.showEditFileDialog`
- `AndroidFs.showShareFileDialog`

### 8. APIs to manage permissions
- `AndroidFs.checkPickerUriPermission`
- `AndroidFs.persistPickerUriPermission`
- `AndroidFs.checkPersistedPickerUriPermission`
- `AndroidFs.releasePersistedPickerUriPermission`
- `AndroidFs.releaseAllPersistedPickerUriPermissions`
- `AndroidFs.checkPublicFilesPermission`
- `AndroidFs.requestPublicFilesPermission`

### 9. Helper
- `isAndroid`
- `getAndroidApiLevel`

# License
This project is licensed under either of

 * MIT license
 * Apache License (Version 2.0)

at your option.