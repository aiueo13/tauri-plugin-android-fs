Note: **I’m using a translation tool, so there may be some inappropriate expressions.**

# Overview
This plugin provides a unified file system API for all Android versions supported by Tauri.

# Setup
First, install this plugin to your Tauri project:

`src-tauri/Cargo.toml`

```toml
[dependencies]
tauri-plugin-android-fs = { version = "=28.0.0", features = [
    # For `AndroidFs.createNewPublicFile` and related APIs on Android 9 or lower
    "legacy_storage_permission",
    # For notification options
    "notification_permission"
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
pnpm add tauri-plugin-android-fs-api@28.0.0 -E
# or
npm install tauri-plugin-android-fs-api@28.0.0 --save-exact
# or
yarn add tauri-plugin-android-fs-api@28.0.0 --exact
```

**NOTE**: Please make sure that the Rust-side `tauri-plugin-android-fs` and the JavaScript-side `tauri-plugin-android-fs-api` versions match exactly.

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

# Examples

```typescript
import { 
  AndroidFs, 
  AndroidPublicGeneralPurposeDir, 
  AndroidProgressNotificationIconType,
  type AndroidProgressNotificationTemplate 
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

    // Configures a system status bar notification (optional)
    const notification: AndroidProgressNotificationTemplate | undefined = {
      icon: AndroidProgressNotificationIconType.Download,
      title: "{{fileName}}",
      textProgress: "Downloading...",
      textCompletion: "Download complete",
      subText: "{{progress}}"
    };

    // Writes data to the file
    if (data instanceof Uint8Array) {
      await AndroidFs.writeFile(uri, data, { notification });
    }
    else if (data instanceof ReadableStream) {
      const writer = await AndroidFs.openWriteFileStream(uri, { notification });
      await data.pipeTo(writer);
    }
    else {
      throw new TypeError("Unsupported data type");
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

# APIs
This plugin provides following APIs:

### 1. APIs to get entries such as files and directories
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

### 5. APIs to read files
- `AndroidFs.openReadFileStream`
- `AndroidFs.openReadTextFileLinesStream`
- `AndroidFs.readFile`
- `AndroidFs.readFileAsBase64`
- `AndroidFs.readFileAsDataURL`
- `AndroidFs.readTextFile`

### 6. APIs to write to files
- `AndroidFs.openWriteFileStream`
- `AndroidFs.writeFile`
- `AndroidFs.writeTextFile`

### 7. APIs to send entries to other apps
- `AndroidFs.showViewFileDialog`
- `AndroidFs.showViewDirDialog`
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


For simplicity, some features and detailed options of the API have been omitted. If you need them, please consider using the [`tauri-plugin-android-fs`](https://crates.io/crates/tauri-plugin-android-fs) on the Rust side.

# License
This project is licensed under either of

 * MIT license
 * Apache License (Version 2.0)

at your option.