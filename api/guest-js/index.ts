import { invoke } from '@tauri-apps/api/core'

/** @ignore */
declare global {
	interface Window {
		__TAURI_ANDROID_FS_PLUGIN_INTERNALS__?: {
			isAndroid: boolean
		}
	}
}

/**
 * Returns whether the current runtime environment is Android.
 *
 * @returns `true` if the Tauri app is built for Android; otherwise, `false`.
 * @throws An error if the Tauri backend does not exist or `tauri-plugin-android-fs` is not set up.
 * @since 22.0.0
 */
export function isAndroid(): boolean {
	const isAndroid = window.__TAURI_ANDROID_FS_PLUGIN_INTERNALS__?.isAndroid
	if (isAndroid !== undefined) {
		return isAndroid
	}

	throw Error("tauri-plugin-android-fs may be not set up. See https://github.com/aiueo13/tauri-plugin-android-fs/blob/main/api/README.md")
}

let cachedApiLevel: Promise<number> | null = null

/**
 * Returns [the API level](https://developer.android.com/guide/topics/manifest/uses-sdk-element#ApiLevels) of the running Android device.
 * 
 * @example
 * ```ts
 * import { getAndroidApiLevel, AndroidApiLevel } from 'tauri-plugin-android-fs-api';
 * 
 * async function isAndroid10orHigher(): Promise<boolean> {
 * 	return AndroidApiLevel.ANDROID_10 <= await getAndroidApiLevel()
 * }
 * 
 * ```
 *
 * @returns A Promise that resolves to the Android API level. This value does not change while the application is running, so it is cached on the JavaScript side.
 * @throws The Promise will be rejected with an error, if the current runtime environment is not Android.
 * @see [AndroidFs::api_level](https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_sync/struct.AndroidFs.html#method.api_level)
 * @since 24.2.0
 */
export async function getAndroidApiLevel(): Promise<number> {
	if (!cachedApiLevel) {
		cachedApiLevel = invoke('plugin:android-fs|get_android_api_level')
	}

	return cachedApiLevel
}

/**
 * Android API level.
 * 
 * **NOTE** :  
 * Tauri does not support Android 7 or lower.
 * 
 * @see <https://developer.android.com/guide/topics/manifest/uses-sdk-element#api-level-table>
 */
export const AndroidApiLevel = Object.freeze({
	ANDROID_7: 24,
	ANDROID_7_1: 25,
	ANDROID_8: 26,
	ANDROID_8_1: 27,
	ANDROID_9: 28,
	ANDROID_10: 29,
	ANDROID_11: 30,
	ANDROID_12: 31,
	ANDROID_12_L: 32,
	ANDROID_13: 33,
	ANDROID_14: 34,
	ANDROID_15: 35,
	ANDROID_16: 36,
} as const);

/**
 * URI or path of the file or directory.
 *
 * The type can be `string` or `URL`; 
 * `URL` values must be FS URIs, while `string` values accept both paths and FS URIs.
 * 
 * Corresponds to the path type used by [`@tauri-apps/plugin-fs`](https://v2.tauri.app/ja/plugin/file-system/) on the frontend
 * and [tauri_plugin_fs::FilePath](https://docs.rs/tauri-plugin-fs/2/tauri_plugin_fs/enum.FilePath.html) in Rust.
 */
export type FsPath = string | URL;

function mapFsPathForInput(uri: FsPath | AndroidFsUri): string | AndroidFsUri {
	return uri instanceof URL ? uri.toString() : uri
}

/**
 * URI of the file or directory on Android.
 * 
 * Unlike a path, this must refer to an existing entry.  
 * Additionally, there can be multiple URI representations for the same entry.  
 * 
 * Corresponds to [tauri_plugin_android_fs::FileUri](https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/struct.FileUri.html) in Rust.
 */
export type AndroidFsUri = {

	/**
	 * ### Note
	 * You do not need to be aware of this value.
	 */
	uri: string,

	/**
	 * ### Note
	 * You do not need to be aware of this value.
	 */
	documentTopTreeUri: string | null
}

/**
 * Type of the file or directory on Android.
 */
export type AndroidEntryType =
	| { type: "Dir" }
	| { type: "File", mimeType: string }

/**
 * Image format of thumbnail.
 */
export type AndroidThumbnailFormat = "jpeg" | "png" | "webp"

/**
 * Options of `AndroidFs.getThumbnail` and its related functions.
 */
export type AndroidGetThumbnailOptions = {

	/**
	 * An image format of the thumbnail.  
	 * One of `"jpeg"`, `"png"`, `"webp"`.  
	 * Defaults to `"jpeg"`.  
	 */
	format?: AndroidThumbnailFormat
}

/**
 * Metadata of the file or directory on Android.
 */
export type AndroidEntryMetadata = AndroidDirMetadata | AndroidFileMetadata

/**
 * Metadata of the directory on Android.
 */
export type AndroidDirMetadata = {
	type: "Dir",
	name: string,
	lastModified: Date,
}

/**
 * Metadata of the file on Android.
 */
export type AndroidFileMetadata = {
	type: "File";
	name: string,
	lastModified: Date,
	byteLength: number,
	mimeType: string,
};

type AndroidEntryMetadataInner =
	| {
		type: "Dir",
		name: string,
		lastModified: number,
	}
	| {
		type: "File";
		name: string,
		lastModified: number,
		byteLength: number,
		mimeType: string,
	};

/**
 * Metadata and URI of the file or directory on Android.
 */
export type AndroidEntryMetadataWithUri = AndroidEntryMetadata & { uri: AndroidFsUri }

type AndroidEntryMetadataWithUriInner = AndroidEntryMetadataInner & { uri: AndroidFsUri }

/**
 * Options of `AndroidFs.readTextFile`
 */
export type AndroidReadTextFileOptions = {

	/**
	 * Text encoding passed to `TextDecoder`.  
	 * If the specified encoding is not supported by the runtime, a `RangeError` may be thrown by `TextDecoder`.
	 *
	 * Defaults to `"utf-8"`.
	 * 
	 * e.g.
	 * - `"utf-8"`
	 * - `"shift_jis"`
	 * - `"iso-8859-2"`
	 * - `"koi8-r"`
	 * - `"gbk"`
	 */
	encoding?: string,

	/**
	 * Indicates whether decoding errors should be treated as fatal.
	 *
	 * - `false`: Invalid byte sequences are replaced with U+FFFD (`�`) and decoding continues.
	 * - `true`: A `TypeError` is thrown when an invalid byte sequence is encountered.
	 *
	 * This option is forwarded to `TextDecoderOptions.fatal` of `TextDecoder` constructor.
	 *
	 * Defaults to `false`.
	 */
	fatal?: boolean,

	/**
	 * Indicates whether to ignore a leading BOM (Byte Order Mark).
	 *
	 * - `false`: A leading BOM is automatically stripped from the decoded result.
	 * - `true`: A leading BOM is preserved and treated as a normal character.
	 *
	 * This option is forwarded to `TextDecoderOptions.ignoreBOM` of `TextDecoder` constructor.
	 *
	 * Defaults to `false`.
	 */
	ignoreBOM?: boolean,
}

/**
 * Options of `AndroidFs.writeFile`
 */
export type AndroidWriteFileOptions = {

	/**
	 * The buffer size, in bytes, used when sending data from the frontend to the backend while writing from a `ReadableStream`.
	 * 
	 * IPC calls are relatively expensive (several milliseconds to tens of milliseconds per no-op call), 
	 * so larger buffer sizes are generally more efficient. 
	 * But if it is too large, the UI may freeze or run out of memory.
	 * 
	 * Defaults to `512000` (500 KiB).
	 */
	bufferByteLength?: number,
}

/**
 * Options of `AndroidFs.openWriteFileStream`
 */
export type AndroidOpenWriteFileStreamOptions = {

	/**
	 * The buffer size, in bytes, used when sending data from the frontend to the backend.
	 * 
	 * IPC calls are relatively expensive (several milliseconds to tens of milliseconds per no-op call), 
	 * so larger buffer sizes are generally more efficient. 
	 * But if it is too large, the UI may freeze or run out of memory.
	 * 
	 * Defaults to `512000` (500 KiB).
	 */
	bufferByteLength?: number,
}

/**
 * Options of `AndroidFs.openReadFileStream`
 */
export type AndroidOpenReadFileStreamOptions = {

	/**
	 * The buffer size, in bytes, used when sending data from the backend to the frontend.
	 * 
	 * IPC calls are relatively expensive (several milliseconds to tens of milliseconds per no-op call), 
	 * so larger buffer sizes are generally more efficient. 
	 * But if it is too large, the UI may freeze or run out of memory.
	 * 
	 * Defaults to `512000` (500 KiB).
	 */
	bufferByteLength?: number,
}

/**
 * Options of `AndroidFs.openReadTextFileLinesStream`
 */
export type AndroidOpenReadTextFileLinesStreamOptions = {

	/**
	 * The buffer size, in bytes, used when sending data from the backend to the frontend.
	 * 
	 * IPC calls are relatively expensive (several milliseconds to tens of milliseconds per no-op call), 
	 * so larger buffer sizes are generally more efficient. 
	 * But if it is too large, the UI may freeze or run out of memory.
	 * 
	 * This value is not guaranteed to be strictly respected. 
	 * If a single line exceeds this size, 
	 * more bytes may be sent in a single IPC transmission.
	 * 
	 * Defaults to `512000` (500 KiB).
	 */
	bufferByteLength?: number,

	/**
	 * Indicates whether decoding errors should be treated as fatal.
	 *
	 * - `false`: Invalid byte sequences are replaced with U+FFFD (`�`) and decoding continues.
	 * - `true`: A `TypeError` is thrown when an invalid byte sequence is encountered.
	 *
	 * This option is forwarded to `TextDecoderOptions.fatal` of `TextDecoder` constructor.
	 *
	 * Defaults to `false`.
	 */
	fatal?: boolean,

	/**
	 * Indicates whether to ignore a leading BOM (Byte Order Mark).
	 *
	 * - `false`: A leading BOM is automatically stripped from the decoded result.
	 * - `true`: A leading BOM is preserved and treated as a normal character.
	 *
	 * This option is forwarded to `TextDecoderOptions.ignoreBOM` of `TextDecoder` constructor.
	 *
	 * Defaults to `false`.
	 */
	ignoreBOM?: boolean,

	/**
	 * The maximum length of a line in bytes, excluding line breaks character. 
	 * If a line exceeds this limit, an error is thrown. 
	 * This prevents OOM errors when reading minified files or binaries.
	 * 
	 * Defaults to `0` (unlimited).
	 */
	maxLineByteLength?: number;
}

/**
 * Options of file picker on Android.
 */
export type AndroidOpenFilePickerOptions = {

	/**
	 * The MIME types of the files to pick.   
	 * If empty, any file can be selected.  
	 */
	mimeTypes?: string[] | string,

	/**
	 * Indicates whether multiple files can be picked.  
	 * Defaults to `false`.
	 */
	multiple?: boolean,

	/**
	 * Preferable picker type.  
	 * This is not necessarily guaranteed to be used.  
	 * By default, the appropriate option will be selected according to the `mimeTypes`. 
	 */
	pickerType?: "FilePicker" | "Gallery",

	/**
	 * Indicates whether write access to the picked files is required.  
	 * Defaults to `false`.
	 */
	needWritePermission?: boolean,

	/**
	 * Indicates whether only files located on the local device should be pickable.  
	 * Defaults to `false`.
	 */
	localOnly?: boolean,

	/**
	 * Initial directory when launching the file picker.  
	 * 
	 * If this option is omitted or the desired initial location cannot be resolved,
	 * the initial location is system-specific.
	 * 
	 * One of: 
	 * - `AndroidPickerInitialLocation.Any(...)` 
	 * - `AndroidPickerInitialLocation.VolumeTop(...)`   
	 * - `AndroidPickerInitialLocation.PublicDir(...)`
	 */
	initialLocation?: AndroidPickerInitialLocation
}

/**
 * Options of file picker on Android.
 */
export type AndroidOpenDirPickerOptions = {

	/**
	 * Indicates whether only directories located on the local device should be pickable.  
	 * Defaults to `false`.
	 */
	localOnly?: boolean,

	/**
	 * Initial directory when launching the directory picker.  
	 * 
	 * If this option is omitted or the desired initial location cannot be resolved,
	 * the initial location is system-specific.
	 * 
	 * One of: 
	 * - `AndroidPickerInitialLocation.Any(...)` 
	 * - `AndroidPickerInitialLocation.VolumeTop(...)`   
	 * - `AndroidPickerInitialLocation.PublicDir(...)`
	 */
	initialLocation?: AndroidPickerInitialLocation
}

/**
 * Options of file picker on Android.
 */
export type AndroidSaveFilePickerOptions = {

	/**
	 * Indicates whether only files located on the local device should be pickable.  
	 * Defaults to `false`.
	 */
	localOnly?: boolean,

	/**
	 * Initial directory when launching the file picker.  
	 * 
	 * If this option is omitted or the desired initial location cannot be resolved,
	 * the initial location is system-specific.
	 * 
	 * One of: 
	 * - `AndroidPickerInitialLocation.Any(...)` 
	 * - `AndroidPickerInitialLocation.VolumeTop(...)`   
	 * - `AndroidPickerInitialLocation.PublicDir(...)`
	 */
	initialLocation?: AndroidPickerInitialLocation
}

/**
 * Options of `AndroidFs.createNewPublicFile` and etc.
 */
export type AndroidCreateNewPublicFileOptions = {

	/**
	 * Indicates whether to prompt the user for permission if it has not already been granted.  
	 * Defaults to `true`.
	 */
	requestPermission?: boolean,

	/**
	 * ID of the storage volume where the file will be created.  
	 * Defaults to primary storage volume.
	 */
	volumeId?: AndroidStorageVolumeId,

	/**
	 * Indicates whether the file will be marked as pending.  
	 * When set to `true`, the app has exclusive access to the file, 
	 * and it becomes invisible to other apps until `AndroidFs.setPublicFilePending(..., false)` is called. 
	 * 
	 * If it remains `true` for more than 7 days, 
	 * the system will automatically delete the file.  
	 * 
	 * Defaults to `false`.
	 *
	 * **NOTE** :  
	 * This is available for Android 11 or higher.  
	 * If unavailable, this will be ignored. 
	 */
	isPending?: boolean
}

/**
 * Android public directories for general-purpose files.
 */
export const AndroidPublicGeneralPurposeDir = Object.freeze({

	/**
	 * Resolves to the `~/Documents` folder.  
	 * This is a subdirectory under the user-visible top-level directory of the storage volume.
	 *
	 * e.g.
	 * - `/storage/emulated/{user-id}/Documents`
	 * - `/storage/{sd-card-id}/Documents`
	 */
	Documents: "Documents",

	/**
	 * Resolves to the `~/Download` folder.  
	 * This is a subdirectory under the user-visible top-level directory of the storage volume.  
	 *
	 * e.g.
	 * - `/storage/emulated/{user-id}/Download`
	 * - `/storage/{sd-card-id}/Download`
	 *
	 * **NOTE** :   
	 * This is not the plural `Downloads`, but the singular `Download`.
	 * <https://developer.android.com/reference/android/os/Environment#DIRECTORY_DOWNLOADS>
	 */
	Download: "Download",
} as const);

/**
 * Android public directories for image files.
 */
export const AndroidPublicImageDir = Object.freeze({

	/**
	 * Resolves to the `~/Pictures` folder.  
	 * This is a subdirectory under the user-visible top-level directory of the storage volume.
	 *
	 * e.g.
	 * - `/storage/emulated/{user-id}/Pictures`
	 * - `/storage/{sd-card-id}/Pictures`
	 */
	Pictures: "Pictures",

	/**
	 * Resolves to the `~/DCIM` folder.  
	 * This is a subdirectory under the user-visible top-level directory of the storage volume.
	 *
	 * e.g.
	 * - `/storage/emulated/{user-id}/DCIM`
	 * - `/storage/{sd-card-id}/DCIM`
	 */
	DCIM: "DCIM",
} as const);

/**
 * Android public directories for video files.
 */
export const AndroidPublicVideoDir = Object.freeze({

	/**
	 * Resolves to the `~/Movies` folder.  
	 * This is a subdirectory under the user-visible top-level directory of the storage volume.
	 * 
	 * 
	 * e.g.
	 * - `/storage/emulated/{user-id}/Movies`
	 * - `/storage/{sd-card-id}/Movies`
	 */
	Movies: "Movies",

	/**
	 * Resolves to the `~/DCIM` folder.  
	 * This is a subdirectory under the user-visible top-level directory of the storage volume.
	 *
	 * e.g.
	 * - `/storage/emulated/{user-id}/DCIM`
	 * - `/storage/{sd-card-id}/DCIM`
	 */
	DCIM: "DCIM",
} as const);

/**
 * Android public directories for audio files.
 */
export const AndroidPublicAudioDir = Object.freeze({

	/**
	 * Resolves to the `~/Music` folder.  
	 * This is a subdirectory under the user-visible top-level directory of the storage volume.
	 * 
	 * e.g.
	 * - `/storage/emulated/{user-id}/Music`
	 * - `/storage/{sd-card-id}/Music`
	 */
	Music: "Music",

	/**
	 * Resolves to the `~/Alarms` folder.  
	 * This is a subdirectory under the user-visible top-level directory of the storage volume.
	 * 
	 * e.g.
	 * - `/storage/emulated/{user-id}/Alarms`
	 * - `/storage/{sd-card-id}/Alarms`
	 */
	Alarms: "Alarms",

	/**
	 * Resolves to the `~/Audiobooks` folder.  
	 * This is a subdirectory under the user-visible top-level directory of the storage volume.
	 *
	 * e.g.
	 * - `/storage/emulated/{user-id}/Audiobooks`
	 * - `/storage/{sd-card-id}/Audiobooks`
	 *
	 * **NOTE** :  
	 * This is available for Android 10 (API level 29) and higher.  
	 * If unavailable, the `~/Music/Audiobooks` folder will be used instead.  
	 */
	Audiobooks: "Audiobooks",

	/**
	 * Resolves to the `~/Notifications` folder.  
	 * This is a subdirectory under the user-visible top-level directory of the storage volume.
	 *
	 * e.g.
	 * - `/storage/emulated/{user-id}/Notifications`
	 * - `/storage/{sd-card-id}/Notifications`
	 */
	Notifications: "Notifications",

	/**
	 * Resolves to the `~/Podcasts` folder.  
	 * This is a subdirectory under the user-visible top-level directory of the storage volume.
	 *
	 * e.g.
	 * - `/storage/emulated/{user-id}/Podcasts`
	 * - `/storage/{sd-card-id}/Podcasts`
	 */
	Podcasts: "Podcasts",

	/**
	 * Resolves to the `~/Ringtones` folder.  
	 * This is a subdirectory under the user-visible top-level directory of the storage volume.
	 *
	 * e.g.
	 * - `/storage/emulated/{user-id}/Ringtones`
	 * - `/storage/{sd-card-id}/Ringtones`
	 */
	Ringtones: "Ringtones",

	/**
	 * Resolves to the `~/Recordings` folder.  
	 * This is a subdirectory under the user-visible top-level directory of the storage volume.
	 *
	 * e.g.
	 * - `/storage/emulated/{user-id}/Recordings`
	 * - `/storage/{sd-card-id}/Recordings`
	 *
	 * **NOTE** :  
	 * This is available for Android 12 (API level 31) or higher.  
	 * If unavailable, the `~/Music/Recordings` folder will be used instead.
	 */
	Recordings: "Recordings",
} as const);

/**
 * Android public directories.
 */
export const AndroidPublicDir = Object.freeze({

	/**
	 * Resolves to the `~/Documents` folder.  
	 * This is a subdirectory under the user-visible top-level directory of the storage volume.
	 *
	 * e.g.
	 * - `/storage/emulated/{user-id}/Documents`
	 * - `/storage/{sd-card-id}/Documents`
	 */
	Documents: "Documents",

	/**
	 * Resolves to the `~/Download` folder.  
	 * This is a subdirectory under the user-visible top-level directory of the storage volume.  
	 *
	 * e.g.
	 * - `/storage/emulated/{user-id}/Download`
	 * - `/storage/{sd-card-id}/Download`
	 *
	 * **NOTE** :   
	 * This is not the plural `Downloads`, but the singular `Download`.
	 * <https://developer.android.com/reference/android/os/Environment#DIRECTORY_DOWNLOADS>
	 */
	Download: "Download",

	/**
	 * Resolves to the `~/Pictures` folder.  
	 * This is a subdirectory under the user-visible top-level directory of the storage volume.
	 *
	 * e.g.
	 * - `/storage/emulated/{user-id}/Pictures`
	 * - `/storage/{sd-card-id}/Pictures`
	 */
	Pictures: "Pictures",

	/**
	 * Resolves to the `~/Movies` folder.  
	 * This is a subdirectory under the user-visible top-level directory of the storage volume.
	 * 
	 * 
	 * e.g.
	 * - `/storage/emulated/{user-id}/Movies`
	 * - `/storage/{sd-card-id}/Movies`
	 */
	Movies: "Movies",

	/**
	 * Resolves to the `~/DCIM` folder.  
	 * This is a subdirectory under the user-visible top-level directory of the storage volume.
	 *
	 * e.g.
	 * - `/storage/emulated/{user-id}/DCIM`
	 * - `/storage/{sd-card-id}/DCIM`
	 */
	DCIM: "DCIM",

	/**
	 * Resolves to the `~/Music` folder.  
	 * This is a subdirectory under the user-visible top-level directory of the storage volume.
	 * 
	 * e.g.
	 * - `/storage/emulated/{user-id}/Music`
	 * - `/storage/{sd-card-id}/Music`
	 */
	Music: "Music",

	/**
	 * Resolves to the `~/Alarms` folder.  
	 * This is a subdirectory under the user-visible top-level directory of the storage volume.
	 * 
	 * e.g.
	 * - `/storage/emulated/{user-id}/Alarms`
	 * - `/storage/{sd-card-id}/Alarms`
	 */
	Alarms: "Alarms",

	/**
	 * Resolves to the `~/Audiobooks` folder.  
	 * This is a subdirectory under the user-visible top-level directory of the storage volume.
	 *
	 * e.g.
	 * - `/storage/emulated/{user-id}/Audiobooks`
	 * - `/storage/{sd-card-id}/Audiobooks`
	 *
	 * **NOTE** :  
	 * This is available for Android 10 (API level 29) and higher.  
	 * If unavailable, the `~/Music/Audiobooks` folder will be used instead.  
	 */
	Audiobooks: "Audiobooks",

	/**
	 * Resolves to the `~/Notifications` folder.  
	 * This is a subdirectory under the user-visible top-level directory of the storage volume.
	 *
	 * e.g.
	 * - `/storage/emulated/{user-id}/Notifications`
	 * - `/storage/{sd-card-id}/Notifications`
	 */
	Notifications: "Notifications",

	/**
	 * Resolves to the `~/Podcasts` folder.  
	 * This is a subdirectory under the user-visible top-level directory of the storage volume.
	 *
	 * e.g.
	 * - `/storage/emulated/{user-id}/Podcasts`
	 * - `/storage/{sd-card-id}/Podcasts`
	 */
	Podcasts: "Podcasts",

	/**
	 * Resolves to the `~/Ringtones` folder.  
	 * This is a subdirectory under the user-visible top-level directory of the storage volume.
	 *
	 * e.g.
	 * - `/storage/emulated/{user-id}/Ringtones`
	 * - `/storage/{sd-card-id}/Ringtones`
	 */
	Ringtones: "Ringtones",

	/**
	 * Resolves to the `~/Recordings` folder.  
	 * This is a subdirectory under the user-visible top-level directory of the storage volume.
	 *
	 * e.g.
	 * - `/storage/emulated/{user-id}/Recordings`
	 * - `/storage/{sd-card-id}/Recordings`
	 *
	 * **NOTE** :  
	 * This is available for Android 12 (API level 31) or higher.  
	 * If unavailable, the `~/Music/Recordings` folder will be used instead.
	 */
	Recordings: "Recordings",
} as const);

export type AndroidPublicGeneralPurposeDir = (typeof AndroidPublicGeneralPurposeDir)[keyof typeof AndroidPublicGeneralPurposeDir]
export type AndroidPublicImageDir = (typeof AndroidPublicImageDir)[keyof typeof AndroidPublicImageDir]
export type AndroidPublicVideoDir = (typeof AndroidPublicVideoDir)[keyof typeof AndroidPublicVideoDir]
export type AndroidPublicAudioDir = (typeof AndroidPublicAudioDir)[keyof typeof AndroidPublicAudioDir]
export type AndroidPublicDir = (typeof AndroidPublicDir)[keyof typeof AndroidPublicDir];

/**
 * Information about the storage volume on Android.
 */
export type AndroidStorageVolumeInfo = {

	/**
	 * A user-visible description of this storage volume.  
	 * This can be determined by the manufacturer and is often localized according to the user’s language.
	 * 
	 * e.g.
	 * - `Internal shared storage`
	 * - `SDCARD`
	 * - `SD card`
	 * - `Virtual SD card`
	 */
	description: string,

	/**
	 * Indicates whether this is primary storage volume. 
	 * A device always has one (and one only) primary storage volume.
	 * 
	 * **NOTE** :  
	 * The primary volume may not be accessible if it has been mounted by the user on their computer, 
	 * has been removed from the device, or some other problem has happened.  
	 * Therefore, the primary storage volume is not necessarily included.
	 */
	isPrimary: boolean,

	/**
	 * Indicates whether this is physically removable. 
	 * If `false`, this is device's built-in storage.
	 */
	isRemovable: boolean,

	/**
	 * Indicates whether this is stable part of the device.
	 * 
	 * For example, a device's built-in storage and physical media slots under protective covers are considered stable,
	 * while USB flash drives connected to handheld devices are not.
	 */
	isStable: boolean,

	/**
	 * Indicates whether this is backed by private user data partition, 
	 * either internal storage or [adopted storage](https://source.android.com/docs/core/storage/adoptable).
	 * 
	 * On most recent devices, the primary storage volume will often have this set to true.
	 */
	isEmulated: boolean,

	/**
	 * Indicates whether this is read-only storage volume.
	 * 
	 * e.g. SD card with readonly mode.
	 */
	isReadOnly: boolean,

	/**
	 * Indicates whether public files can be placed on this storage volume.
	 *
	 * **Note** :  
	 * This does not indicate whether the volume is writable
	 * (that is, whether public files can actually be created on it).
	 * For that information, refer to `isReadOnly`.
	 */
	isAvailableForPublicFiles: boolean,

	/**
	 * ID of this storage volume.
	 * 
	 * Since storage volume ID can change, this should be not stored.
	 */
	id: AndroidStorageVolumeId
}

/**
 * ID of the storage volume on Android.
 * 
 * Since storage volume ID can change, this should be not stored.
 */
export type AndroidStorageVolumeId = string;

/**
 * State of the URI permission on Android.
 */
export const AndroidUriPermissionState = Object.freeze({
	Read: "Read",
	Write: "Write",
	ReadAndWrite: "ReadAndWrite",
	ReadOrWrite: "ReadOrWrite"
} as const)

/**
 * State of the URI permission on Android.
 */
export type AndroidUriPermissionState = typeof AndroidUriPermissionState[keyof typeof AndroidUriPermissionState]

/**
 * Options of `AndroidFs.listVolumes`.
 */
export type AndroidListVolumesOptions = {

	/**
	 * Purpose for listing storage volumes.
	 *
	 * - `"CreatePublicFile"`:
	 * Lists only volumes that are available for `AndroidFs.createNewPublicFile` and its related functions.
	 * This does not include volumes that are not writable (e.g., a read-only SD card), 
	 * and, on Android 9 and below, it does not include volumes other than primary storage that are inaccessible to `AndroidFs.createNewPublicFile` due to Android platform restrictions.
	 * In other words, it returns only volumes whose `isReadOnly` and `isAvailableForPublicFiles` properties of `AndroidStorageVolumeInfo` are false and true respectively.
	 * 
	 * - `"PickerInitialLocation"`:
	 * Lists only volumes that are available for use as a picker initial location.
	 * This includes all volumes.
	 *
	 * By default, only volumes that are available for both purposes are listed.
	 */
	purpose?: "CreatePublicFile" | "PickerInitialLocation"
}

/**
 * Initial location when launching file/directory picker.
 */
export type AndroidPickerInitialLocation =
	| { type: "Any", uri: AndroidFsUri }
	| { type: "VolumeTop", volumeId?: AndroidStorageVolumeId }
	| {
		type: "PublicDir"
		baseDir: AndroidPublicDir
		relativePath?: string
		volumeId?: AndroidStorageVolumeId
	}

type AndroidPickerInitialLocationInner =
	| { type: "Any", uri: AndroidFsUri }
	| { type: "VolumeTop", volumeId: AndroidStorageVolumeId | null }
	| {
		type: "PublicDir"
		baseDir: AndroidPublicDir
		relativePath: string | null
		volumeId: AndroidStorageVolumeId | null
	}

function mapPickerInitialLocationForInput(
	i?: AndroidPickerInitialLocation | undefined | null
): AndroidPickerInitialLocationInner | null {

	if (i == null) {
		return null
	}
	if (i.type === "PublicDir") {
		return {
			type: "PublicDir",
			baseDir: i.baseDir,
			relativePath: i.relativePath ?? null,
			volumeId: i.volumeId ?? null
		}
	}
	if (i.type === "VolumeTop") {
		return {
			type: "VolumeTop",
			volumeId: i.volumeId ?? null
		}
	}
	return i
}

/**
 * Options of `AndroidPickerInitialLocation.PublicDir`.
 */
export type AndroidPickerInitialLocationPublicDirOptions = {

	/**
	 * Relative path from the target public directory.
	 */
	relativePath?: string

	/**
	 * ID of the storage volume that the target public directory belongs to.  
	 * 
	 * Defaults to primary storage volume.
	 */
	volumeId?: AndroidStorageVolumeId
}

/**
 * Initial location when launching file/directory picker.
 */
export const AndroidPickerInitialLocation = Object.freeze({

	/**
	 * Builds an initial picker location at the specified directory, or in the directory containing the specified file.
	 * 
	 * @param uri - URI of the target entry.
	 */
	Any(uri: AndroidFsUri): AndroidPickerInitialLocation {
		return {
			type: "Any",
			uri,
		}
	},

	/**
	 * Builds an initial picker location at the top of the storage volume.
	 *
	 * @param volumeId - ID of the target storage volume. Defaults to primary storage volume.
	 */
	VolumeTop(
		volumeId?: AndroidStorageVolumeId
	): AndroidPickerInitialLocation {

		return {
			type: "VolumeTop",
			volumeId,
		}
	},

	/**
	 * Builds an initial picker location inside the public directory.
	 *
	 * @param baseDir - The target public directory. One of: `"Documents"`, `"Download"`, `"Pictures"`, `"DCIM"`, `"Movies"`, `"Music"`, `"Alarms"`, `"Audiobooks"`, `"Notifications"`, `"Podcasts"`, `"Ringtones"`, `"Recordings"`.
	 * @param options - Additional options for the initial location.
	 * @param options.relativePath - Relative path from the target public directory.
	 * @param options.volumeId - ID of the storage volume that the target public directory belongs to. Defaults to primary storage volume.
	 */
	PublicDir(
		baseDir: AndroidPublicDir,
		options?: AndroidPickerInitialLocationPublicDirOptions
	): AndroidPickerInitialLocation {

		return {
			type: "PublicDir",
			baseDir,
			relativePath: options?.relativePath,
			volumeId: options?.volumeId
		}
	},
} as const)

export class AndroidFs {

	private constructor() { }


	/**
	 * Gets a name of the specified file or directory.  
	 * Includes the file extension if it exists.
	 *
	 * @param uri - The URI or path of the target file or directory.
	 * 
	 * @returns A Promise that resolves to the name of the target.
	 * @throws The Promise will be rejected with an error, if the specified entry does not exist or if the read permission is missing.
	 * 
	 * @see [AndroidFs::get_name](https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.AndroidFs.html#method.get_name)
	 * @since 22.0.0
	 */
	public static async getName(uri: AndroidFsUri | FsPath): Promise<string> {
		return await invoke('plugin:android-fs|get_name', {
			uri: mapFsPathForInput(uri)
		})
	}

	/**
	 * Gets a file size in bytes of the specified file.  
	 *
	 * @param uri - The URI or path of the target file.
	 * 
	 * @returns A Promise that resolves to a non-negative integer representing the length in bytes.
	 * @throws The Promise will be rejected with an error, if the specified entry does not exist, if the entry is a directory, or if the read permission is missing.
	 * 
	 * @see [AndroidFs::get_len](https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.AndroidFs.html#method.get_len)
	 * @since 22.2.0
	 */
	public static async getByteLength(uri: AndroidFsUri | FsPath): Promise<number> {
		return await invoke('plugin:android-fs|get_byte_length', {
			uri: mapFsPathForInput(uri)
		})
	}

	/**
	 * Gets a type of the specified file or directory.
	 *
	 * @param uri - The URI or path of the target file or directory.
	 * 
	 * @returns A Promise that resolves to the type of the entry. The resolved value will be an object of type `AndroidEntryType`, which can be either `{ type: "Dir" }` for directories or `{ type: "File", mimeType: string }` for files.
	 * @throws The Promise will be rejected with an error, if the specified entry does not exist or if the read permission is missing.
	 * 
	 * @see [AndroidFs::get_type](https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.AndroidFs.html#method.get_type)
	 * @since 22.0.0
	 */
	public static async getType(uri: AndroidFsUri | FsPath): Promise<AndroidEntryType> {
		return await invoke('plugin:android-fs|get_type', {
			uri: mapFsPathForInput(uri)
		})
	}

	/**
	 * Gets a MIME type of the specified file.
	 *
	 * @param uri - The URI or path of the target file.
	 * 
	 * @returns A Promise that resolves to the MIME type as a string.
	 * @throws The Promise will be rejected with an error, if the specified entry does not exist, if the entry is a directory, or if the read permission is missing.
	 * 
	 * @see [AndroidFs::get_mime_type](https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.AndroidFs.html#method.get_mime_type)
	 * @since 22.0.0
	 */
	public static async getMimeType(uri: AndroidFsUri | FsPath): Promise<string> {
		return await invoke('plugin:android-fs|get_mime_type', {
			uri: mapFsPathForInput(uri)
		})
	}

	/**
	 * Gets metadata of the specified file or directory.  
	 * 
	 * @param uri - The URI or path of the target file or directory.
	 * 
	 * @returns A Promise that resolves to metadata of the target. It includes the type (`"Dir"` or `"File"`), name, last modified date, and for files also byte length and MIME type.
	 * @throws The Promise will be rejected with an error, if the specified entry does not exist or if the read permission is missing.
	 * 
	 * @see [AndroidFs::get_info](https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.AndroidFs.html#method.get_info)
	 * @since 22.0.0
	 */
	public static async getMetadata(uri: AndroidFsUri | FsPath): Promise<AndroidEntryMetadata> {
		let md = await invoke<AndroidEntryMetadataInner>('plugin:android-fs|get_metadata', {
			uri: mapFsPathForInput(uri)
		})
		const lastModified = new Date(md.lastModified)

		return md.type === "Dir"
			? { type: "Dir", name: md.name, lastModified, }
			: { type: "File", name: md.name, lastModified, byteLength: md.byteLength, mimeType: md.mimeType };
	}

	/**
	 * Gets a data URL representing a thumbnail of the specified file.  
	 * This does not perform caching.
	 *
	 * @param uri - The URI or path of the target file.
	 * @param width - The preferred width of the thumbnail in pixels. 
	 * @param height - The preferred height of the thumbnail in pixels.
	 * @param options - Optional settings.
	 * @param options.format - The image format of the thumbnail. One of: `"jpeg"`, `"png"`, `"webp"`. Defaults to `"jpeg"`.
	 * 
	 * @returns A Promise that resolves to a string containing the thumbnail as a data URL, or `null` if the file does not have a thumbnail. The actual thumbnail dimensions will not exceed approximately twice the specified width or height, and the original aspect ratio of the file is always maintained.
	 * @throws The Promise will be rejected with an error, if the specified entry does not exist, if the entry is a directory, or if the read permission is missing.
	 * 
	 * @see [AndroidFs::get_thumbnail](https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.AndroidFs.html#method.get_thumbnail)
	 * @since 22.0.0
	 */
	public static async getThumbnailDataUrl(
		uri: AndroidFsUri | FsPath,
		width: number,
		height: number,
		options?: AndroidGetThumbnailOptions
	): Promise<string | null> {

		const format: AndroidThumbnailFormat = options?.format ?? "jpeg"

		return await invoke('plugin:android-fs|get_thumbnail_data_url', {
			uri: mapFsPathForInput(uri),
			width,
			height,
			format
		})
	}

	/**
	 * Gets a base64-encoded string representing a thumbnail of the specified file.   
	 * This does not perform caching.
	 *
	 * @param uri - The URI or path of the target file.
	 * @param width - The preferred width of the thumbnail in pixels. 
	 * @param height - The preferred height of the thumbnail in pixels.
	 * @param options - Optional settings.
	 * @param options.format - The image format of the thumbnail. One of: `"jpeg"`, `"png"`, `"webp"`. Defaults to `"jpeg"`.
	 * 
	 * @returns A Promise that resolves to the thumbnail as a base64-encoded string using "+" and "/" characters and containing no line breaks (a single line), or `null` if the file does not have a thumbnail. The actual thumbnail dimensions will not exceed approximately twice the specified width or height, and the original aspect ratio of the file is always maintained.
	 * @throws The Promise will be rejected with an error, if the specified entry does not exist, if the entry is a directory, or if the read permission is missing.
	 * 
	 * @see [AndroidFs::get_thumbnail](https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.AndroidFs.html#method.get_thumbnail)
	 * @since 22.0.0
	 */
	public static async getThumbnailBase64(
		uri: AndroidFsUri | FsPath,
		width: number,
		height: number,
		options?: AndroidGetThumbnailOptions
	): Promise<string | null> {

		const format: AndroidThumbnailFormat = options?.format ?? "jpeg"

		return await invoke('plugin:android-fs|get_thumbnail_base64', {
			uri: mapFsPathForInput(uri),
			width,
			height,
			format
		})
	}

	/**
	 * Gets a thumbnail bytes of the specified file.  
	 * This does not perform caching.
	 *
	 * @param uri - The URI or path of the target file.
	 * @param width - The preferred width of the thumbnail in pixels. 
	 * @param height - The preferred height of the thumbnail in pixels.
	 * @param options - Optional settings.
	 * @param options.format - The image format of the thumbnail. One of: `"jpeg"`, `"png"`, `"webp"`. Defaults to `"jpeg"`.
	 *
	 * @returns A Promise that resolves to a `ArrayBuffer` containing the thumbnail bytes, or `null` if the file does not have a thumbnail. The actual thumbnail dimensions will not exceed approximately twice the specified width or height, and the original aspect ratio of the file is always maintained.
	 * @throws The Promise will be rejected with an error, if the specified entry does not exist, if the entry is a directory, or if the read permission is missing.
	 * 
	 * @see [AndroidFs::get_thumbnail](https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.AndroidFs.html#method.get_thumbnail)
	 * @since 22.0.0
	 */
	public static async getThumbnail(
		uri: AndroidFsUri | FsPath,
		width: number,
		height: number,
		options?: AndroidGetThumbnailOptions
	): Promise<ArrayBuffer | null> {

		const format: AndroidThumbnailFormat = options?.format ?? "jpeg"

		const thumbnail = await invoke<ArrayBuffer>('plugin:android-fs|get_thumbnail', {
			uri: mapFsPathForInput(uri),
			width,
			height,
			format
		})

		return thumbnail.byteLength === 0 ? null : thumbnail
	}

	/**
	 * Gets a path usable with Tauri's file system ([`@tauri-apps/plugin-fs`](https://v2.tauri.app/ja/plugin/file-system/)).
	 * 
	 * Paths **derived from this plugin's URI** are supported only for reading and writing files.
	 * No guarantees are provided for other operations or for directory handling.
	 * And for those paths, there is no need for you to set [the scope configuration](https://v2.tauri.app/reference/javascript/fs/#security) of Tauri's file system.
	 * 
	 * **WARNING** :  
	 * When reading or writing files using plugin-fs, caution is required. 
	 * Writing to files can sometimes be very slow. 
	 * Also, files obtained from third-party apps via a file picker may not be openable, readable, writable, or seekable.  
	 * For this reason, it is strongly recommended to use the APIs provided by this plugin, such as `AndroidFs.openReadFileStream`, `AndroidFs.openWriteFileStream`, `AndroidFs.writeFile` and etc.
	 * 
	 * @param uri - The URI or path of the target file or directory.
	 * @returns A Promise that resolves to the path. Note that although it says "Path", it may actually be a URI that can be used with `@tauri-apps/plugin-fs`.
	 * @since 22.0.0
	 */
	public static async getFsPath(uri: AndroidFsUri | FsPath): Promise<FsPath> {
		return await invoke<string>('plugin:android-fs|get_fs_path', {
			uri: mapFsPathForInput(uri)
		})
	}

	/**
	 * Retrieves information about the available Android storage volumes (e.g., `Internal storage`, `SD card` and `USB drive`).
	 *
	 * @param options - Optional settings.
	 * @param options.purpose - Purpose of storage volumes. One of: `"CreatePublicFile"`, `"PickerInitialLocation"`. By default, only volumes that are available for both purposes are listed.
	 * @returns A Promise that resolves to an array of the storage volumes. 
	 * 
	 * @see [PublicStorage::get_volumes](https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.PublicStorage.html#method.get_volumes)
	 * @since 22.2.0
	 */
	public static async listVolumes(
		options?: AndroidListVolumesOptions
	): Promise<AndroidStorageVolumeInfo[]> {

		const purpose = options?.purpose
		const volumes = await invoke<AndroidStorageVolumeInfo[]>('plugin:android-fs|list_volumes')

		if (purpose == null || purpose === "CreatePublicFile") {
			return volumes
				.filter(v => !v.isReadOnly)
				.filter(v => v.isAvailableForPublicFiles)
		}
		else {
			purpose satisfies "PickerInitialLocation"
			return volumes
		}
	}

	/**
	 * Requests permission from the user to create public files, if necessary.
	 * 
	 * This is intended for `AndroidFs.createNewPublicFile` and its related functions, 
	 * but since those functions request permission automatically by default, 
	 * this is not strictly necessary.
	 * 
	 * @returns A Promise that resolves to a boolean indicating whether the app is allowed to create files in public storage and read/write the files it creates.
	 * @see [PublicStorage::request_permission](https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.PublicStorage.html#method.request_permission)
	 * @since 22.0.0
	 */
	public static async requestPublicFilesPermission(): Promise<boolean> {
		return await invoke('plugin:android-fs|request_public_files_permission')
	}

	/**
	 * Checks whether the app has permission to create public files.
	 * 
	 * The app can request it by `AndroidFs.requestPublicFilesPermission`.
	 * 
	 * @returns A Promise that resolves to a boolean indicating whether the app is allowed to create files in public storage and read/write the files it creates.
	 * @see [PublicStorage::has_permission](https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.PublicStorage.html#method.request_permission)
	 * @since 22.0.0
	 */
	public static async hasPublicFilesPermission(): Promise<boolean> {
		return await invoke('plugin:android-fs|has_public_files_permission')
	}

	/**
	 * Triggers the Android MediaScanner to scan a public file,
	 * making it visible in media applications like the Gallery, Music player, etc.
	 * 
	 * @param uri - The URI of the file to be scanned.  
	 *
	 * @returns A Promise that resolves when the scan request has been initiated.
	 * @throws The Promise will be rejected with an error, if the specified entry does not exist, if the required permission is missing, or if the entry is not public files.  
	 * 
	 * @see [PublicStorage::scan](https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.PublicStorage.html#method.scan)
	 * @since 22.0.0
	 */
	public static async scanPublicFile(
		uri: AndroidFsUri
	): Promise<void> {

		return await invoke('plugin:android-fs|scan_public_file', {
			uri,
		})
	}

	/**
	 * Specifies whether the public file is marked as pending.  
	 * 
	 * **NOTE** :  
	 * This is available for Android 11 or higher.  
	 * If unavailable, this does nothing. 
	 * 
	 * @param uri - The URI of the target file.  
	 * @param isPending - Indicates whether the file will be marked as pending. When set to `true`, the app has exclusive access to the file, and it becomes invisible to other apps. If it remains `true` for more than 7 days, the system will automatically delete the file.   
	 * 
	 * @returns A Promise that resolves when the operation is completed.
	 * @throws The Promise will be rejected with an error, if the specified entry does not exist, if the required permission is missing, or if the entry is not public files.  
	 * 
	 * @see [PublicStorage::set_pending](https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.PublicStorage.html#method.set_pending)
	 * @since 25.0.0
	 */
	public static async setPublicFilePending(
		uri: AndroidFsUri,
		isPending: boolean
	): Promise<void> {

		return await invoke('plugin:android-fs|set_public_file_pending', {
			uri,
			isPending
		})
	}

	/**
	 * Creates a new empty file at the specified location.
	 * 
	 * @example
	 * ```ts
	 * import { AndroidFs, AndroidPublicGeneralPurposeDir } from 'tauri-plugin-android-fs-api';
	 *
	 * async function saveText(fileName: string, data: string): Promise<void> {
	 * 	const baseDir = AndroidPublicGeneralPurposeDir.Download;
	 * 	const relativePath = "MyApp/" + fileName;
	 * 	const mimeType = "text/plain";
	 * 
	 * 	const uri = await AndroidFs.createNewPublicFile(baseDir, relativePath, mimeType);
	 * 
	 * 	try {
	 * 		await AndroidFs.writeTextFile(uri, data);
	 * 		await AndroidFs.scanPublicFile(uri);
	 * 	}
	 * 	catch (e) {
	 * 		await AndroidFs.removeFile(uri).catch(() => {});
	 * 		throw e;
	 * 	}
	 * }
	 * ```
	 * 
	 * @param baseDir - The base directory in which to create the new file. One of: `"Documents"`, `"Download"`.
	 * @param relativePath - The file's relative path from the base directory. If a file with the same name already exists, a sequential number is appended to ensure uniqueness. If the directories in this path do not exist, they will be created recursively.
	 * @param mimeType - The MIME type of the file to create. If `null`, this is inferred from the extension of `relativePath`.
	 * @param options - Optional settings.
	 * @param options.requestPermission - Indicates whether to prompt the user for permission if it has not already been granted. Defaults to `true`.
	 * @param options.volumeId - ID of the storage volume where the file will be created. Defaults to the primary storage volume.
	 * @param options.isPending - Indicates whether the file will be marked as pending. When set to `true`, the app has exclusive access to the file, and it becomes invisible to other apps until `AndroidFs.setPublicFilePending(..., false)` is called. If it remains `true` for more than 7 days, the system will automatically delete the file. Note this is available for Android 11 or higher. If unavailable, this will be ignored. Defaults to `false`.
	 * 
	 * @return A Promise that resolves to the URI of the created file, with persisted read and write permissions that depends on `AndroidFs.hasPublicFilesPermission`.
	 * @throws The Promise will be rejected with an error, if the storage is currently unavailable or the required permission is missing.
	 * 
	 * @see [PublicStorage::create_new_file](https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.PublicStorage.html#method.create_new_file)
	 * @since 22.0.0
	 */
	public static async createNewPublicFile(
		baseDir: AndroidPublicGeneralPurposeDir,
		relativePath: string,
		mimeType: string | null,
		options?: AndroidCreateNewPublicFileOptions
	): Promise<AndroidFsUri> {

		const requestPermission: boolean = options?.requestPermission ?? true
		const volumeId: AndroidStorageVolumeId | null = options?.volumeId ?? null
		const isPending: boolean = options?.isPending ?? false

		return await invoke('plugin:android-fs|create_new_public_file', {
			volumeId,
			baseDir,
			relativePath,
			mimeType,
			requestPermission,
			isPending
		})
	}

	/**
	 * Creates a new empty image file at the specified location.
	 * 
	 * @example
	 * ```ts
	 * import { AndroidFs, AndroidPublicImageDir } from 'tauri-plugin-android-fs-api';
	 *
	 * async function saveImage(
	 *   fileName: string,
	 *   data: Uint8Array | ReadableStream<Uint8Array>,
	 *   mimeType: string
	 * ): Promise<void> {
	 *
	 *   const baseDir = AndroidPublicImageDir.Pictures;
	 *   const relativePath = "MyApp/" + fileName;
	 * 
	 *   const uri = await AndroidFs.createNewPublicImageFile(baseDir, relativePath, mimeType);
	 * 
	 *   try {
	 *     await AndroidFs.writeFile(uri, data);
	 *     await AndroidFs.scanPublicFile(uri);
	 *   }
	 *   catch (e) {
	 *     await AndroidFs.removeFile(uri).catch(() => {});
	 *     throw e;
	 *   }
	 * }
	 * ```
	 * 
	 * @param baseDir - The base directory in which to create the new file. One of: `"Pictures"`, `"DCIM"`, `"Documents"`, `"Download"`.
	 * @param relativePath - The file's relative path from the base directory. If a file with the same name already exists, a sequential number is appended to ensure uniqueness. If the directories in this path do not exist, they will be created recursively.
	 * @param mimeType - The MIME type of the file to create. If `null`, this is inferred from the extension of `relativePath`.
	 * @param options - Optional settings.
	 * @param options.requestPermission - Indicates whether to prompt the user for permission if it has not already been granted. Defaults to `true`.
	 * @param options.volumeId - ID of the storage volume where the file will be created. Defaults to the primary storage volume.
	 * @param options.isPending - Indicates whether the file will be marked as pending. When set to `true`, the app has exclusive access to the file, and it becomes invisible to other apps until `AndroidFs.setPublicFilePending(..., false)` is called. If it remains `true` for more than 7 days, the system will automatically delete the file. Note this is available for Android 11 or higher. If unavailable, this will be ignored. Defaults to `false`.
	 * 
	 * @return A Promise that resolves to the URI of the created file, with persisted read and write permissions that depends on `AndroidFs.hasPublicFilesPermission`.
	 * @throws The Promise will be rejected with an error, if the `mimeType` is not an image type, if the storage is currently unavailable or the required permission is missing.
	 * 
	 * @see [PublicStorage::create_new_file](https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.PublicStorage.html#method.create_new_file)
	 * @since 22.0.0
	 */
	public static async createNewPublicImageFile(
		baseDir: AndroidPublicImageDir | AndroidPublicGeneralPurposeDir,
		relativePath: string,
		mimeType: string | null,
		options?: AndroidCreateNewPublicFileOptions
	): Promise<AndroidFsUri> {

		const requestPermission: boolean = options?.requestPermission ?? true
		const volumeId: AndroidStorageVolumeId | null = options?.volumeId ?? null
		const isPending: boolean = options?.isPending ?? false

		return await invoke('plugin:android-fs|create_new_public_image_file', {
			volumeId,
			baseDir,
			relativePath,
			mimeType,
			requestPermission,
			isPending
		})
	}

	/**
	 * Creates a new empty video file at the specified location.
	 * 
	 * @example
	 * ```ts
	 * import { AndroidFs, AndroidPublicVideoDir } from 'tauri-plugin-android-fs-api';
	 *
	 * async function saveVideo(
	 *   fileName: string,
	 *   data: Uint8Array | ReadableStream<Uint8Array>,
	 *   mimeType: string
	 * ): Promise<void> {
	 *
	 *   const baseDir = AndroidPublicVideoDir.Movies;
	 *   const relativePath = "MyApp/" + fileName;
	 * 
	 *   const uri = await AndroidFs.createNewPublicVideoFile(baseDir, relativePath, mimeType);
	 *
	 *   try {
	 *     await AndroidFs.writeFile(path, data);
	 *     await AndroidFs.scanPublicFile(uri);
	 *   }
	 *   catch (e) {
	 *     await AndroidFs.removeFile(uri).catch(() => {});
	 *     throw e;
	 *   }
	 * }
	 * ```
	 * 
	 * @param baseDir - The base directory in which to create the new file. One of: `"Movies"`, `"DCIM"`, `"Documents"`, `"Download"`.
	 * @param relativePath - The file's relative path from the base directory. If a file with the same name already exists, a sequential number is appended to ensure uniqueness. If the directories in this path do not exist, they will be created recursively.
	 * @param mimeType - The MIME type of the file to create. If `null`, this is inferred from the extension of `relativePath`.
	 * @param options - Optional settings.
	 * @param options.requestPermission - Indicates whether to prompt the user for permission if it has not already been granted. Defaults to `true`.
	 * @param options.volumeId - ID of the storage volume where the file will be created. Defaults to the primary storage volume.
	 * @param options.isPending - Indicates whether the file will be marked as pending. When set to `true`, the app has exclusive access to the file, and it becomes invisible to other apps until `AndroidFs.setPublicFilePending(..., false)` is called. If it remains `true` for more than 7 days, the system will automatically delete the file. Note this is available for Android 11 or higher. If unavailable, this will be ignored. Defaults to `false`.
	 * 
	 * @return A Promise that resolves to the URI of the created file, with persisted read and write permissions that depends on `AndroidFs.hasPublicFilesPermission`.
	 * @throws The Promise will be rejected with an error, if the `mimeType` is not a video type, if the storage is currently unavailable or the required permission is missing.
	 * 
	 * @see [PublicStorage::create_new_file](https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.PublicStorage.html#method.create_new_file)
	 * @since 22.0.0
	 */
	public static async createNewPublicVideoFile(
		baseDir: AndroidPublicVideoDir | AndroidPublicGeneralPurposeDir,
		relativePath: string,
		mimeType: string | null,
		options?: AndroidCreateNewPublicFileOptions
	): Promise<AndroidFsUri> {

		const requestPermission: boolean = options?.requestPermission ?? true
		const volumeId: AndroidStorageVolumeId | null = options?.volumeId ?? null
		const isPending: boolean = options?.isPending ?? false

		return await invoke('plugin:android-fs|create_new_public_video_file', {
			volumeId,
			baseDir,
			relativePath,
			mimeType,
			requestPermission,
			isPending
		})
	}

	/**
	 * Creates a new empty audio file at the specified location.
	 * 
	 * @example
	 * ```ts
	 * import { AndroidFs, AndroidPublicAudioDir } from 'tauri-plugin-android-fs-api';
	 *
	 * async function saveAudio(
	 *   fileName: string,
	 *   data: Uint8Array | ReadableStream<Uint8Array>,
	 *   mimeType: string
	 * ): Promise<void> {
	 *
	 *   const baseDir = AndroidPublicAudioDir.Music;
	 *   const relativePath = "MyApp/" + fileName;
	 * 
	 *   const uri = await AndroidFs.createNewPublicAudioFile(baseDir, relativePath, mimeType);
	 *
	 *   try {
	 *     await AndroidFs.writeFile(path, data);
	 *     await AndroidFs.scanPublicFile(uri);
	 *   }
	 *   catch (e) {
	 *     await AndroidFs.removeFile(uri).catch(() => {});
	 *     throw e;
	 *   }
	 * }
	 * ```
	 * 
	 * @param baseDir - The base directory in which to create the new file. One of: `"Music"`, `"Alarms"`, `"Audiobooks"`, `"Notifications"`, `"Podcasts"`, `"Ringtones"`, `"Recordings"`, `"Documents"`, `"Download"`.
	 * @param relativePath - The file's relative path from the base directory. If a file with the same name already exists, a sequential number is appended to ensure uniqueness. If the directories in this path do not exist, they will be created recursively.
	 * @param mimeType - The MIME type of the file to create. If `null`, this is inferred from the extension of `relativePath`.
	 * @param options - Optional settings.
	 * @param options.requestPermission - Indicates whether to prompt the user for permission if it has not already been granted. Defaults to `true`.
	 * @param options.volumeId - ID of the storage volume where the file will be created. Defaults to the primary storage volume.
	 * @param options.isPending - Indicates whether the file will be marked as pending. When set to `true`, the app has exclusive access to the file, and it becomes invisible to other apps until `AndroidFs.setPublicFilePending(..., false)` is called. If it remains `true` for more than 7 days, the system will automatically delete the file. Note this is available for Android 11 or higher. If unavailable, this will be ignored. Defaults to `false`.
	 * 
	 * @return A Promise that resolves to the URI of the created file, with persisted read and write permissions that depends on `AndroidFs.hasPublicFilesPermission`.
	 * @throws The Promise will be rejected with an error, if the `mimeType` is not a audio type, if the storage is currently unavailable or the required permission is missing.
	 * 
	 * @see [PublicStorage::create_new_file](https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.PublicStorage.html#method.create_new_file)
	 * @since 22.0.0
	 */
	public static async createNewPublicAudioFile(
		baseDir: AndroidPublicAudioDir | AndroidPublicGeneralPurposeDir,
		relativePath: string,
		mimeType: string | null,
		options?: AndroidCreateNewPublicFileOptions
	): Promise<AndroidFsUri> {

		const requestPermission: boolean = options?.requestPermission ?? true
		const volumeId: AndroidStorageVolumeId | null = options?.volumeId ?? null
		const isPending: boolean = options?.isPending ?? false

		return await invoke('plugin:android-fs|create_new_public_audio_file', {
			volumeId,
			baseDir,
			relativePath,
			mimeType,
			requestPermission,
			isPending
		})
	}

	/**
	 * Creates a new empty file at the specified location.  
	 * 
	 * @param baseDirUri - The URI of the base directory in which to create the new file. 
	 * @param relativePath - The file's relative path from the base directory. If a file with the same name already exists, a sequential number is appended to ensure uniqueness. If the directories in this path do not exist, they will be created recursively.
	 * @param mimeType - The MIME type of the file to create. If `null`, this is inferred from the extension of `relativePath`.
	 * 
	 * @returns A Promise that resolves to the URI of the created file, with permissions that depend on the base direcotry.
	 * @throws The Promise will be rejected with an error, if the base directory does not exist, is not a directory, lacks read/write permissions, or if the file provider does not support creating files or directories.
	 * 
	 * @see [AndroidFs::create_new_file](https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.AndroidFs.html#method.create_new_file)
	 * @since 22.0.0
	 */
	public static async createNewFile(
		baseDirUri: AndroidFsUri,
		relativePath: string,
		mimeType: string | null
	): Promise<AndroidFsUri> {

		return await invoke('plugin:android-fs|create_new_file', {
			baseDirUri,
			relativePath,
			mimeType,
		})
	}

	/**
	 * Creates a directory and it's parents at the specified location if they are missing.
	 * 
	 * @param baseDirUri - The URI of the base directory in which to create the directory. 
	 * @param relativePath - The directory's relative path from the base directory. 
	 * 
	 * @returns A Promise that resolves to the URI of the created directory, or the existing directory if one already exists at the specified location. The permissions depend on the base directory.
	 * @throws The Promise will be rejected with an error, if the base directory does not exist, is not a directory, lacks read/write permissions, or if the file provider does not support creating directories.
	 * 
	 * @see [AndroidFs::create_dir_all](https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.AndroidFs.html#method.create_dir_all)
	 * @since 22.1.0
	 */
	public static async createDirAll(
		baseDirUri: AndroidFsUri,
		relativePath: string,
	): Promise<AndroidFsUri> {

		return await invoke('plugin:android-fs|create_dir_all', {
			baseDirUri,
			relativePath,
		})
	}

	/**
	 * Opens the file with read-only mode and resolves to a `ReadableStream`.  
	 * 
	 * The returned `ReadableStream` must always be released by the caller.
	 * Failure to do so may cause file descriptor resource leaks.
	 * The returned ReadableStream is released in the following cases:
	 * - When the ReadableStream or its Reader is canceled. 
	 * - When the ReadableStream's Reader has been fully read. 
	 * - When the ReadableStream's Reader's read operation ends with an error. 
	 * 
	 * These releases may be performed multiple times without issue.
	 * 
	 * @param uri - The URI or path of the file to read. 
	 * @param options - Optional settings.
	 * @param options.bufferByteLength - The buffer size, in bytes, used when sending data from the backend to the frontend. IPC calls are relatively expensive (several milliseconds to tens of milliseconds per no-op call), so larger buffer sizes are generally more efficient. But if it is too large, the UI may freeze or run out of memory. Defaults to `512000` (500 KiB).
	 * 
	 * @returns A Promise that resolves to a `ReadableStream<Uint8Array<ArrayBuffer>>` backed by the file opened in read-only mode. This stream has a one-to-one correspondence with the file descriptor.
	 * @throws The Promise will be rejected with an error, if the specified entry does not exist, if the entry is a directory, or if the read permission is missing.
	 * 
	 * @see [AndroidFs::open_file_readable](https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.AndroidFs.html#method.open_file_readable)
	 * @since 25.1.0
	 */
	public static async openReadFileStream(
		uri: AndroidFsUri | FsPath,
		options?: AndroidOpenReadFileStreamOptions
	): Promise<ReadableStream<Uint8Array<ArrayBuffer>>> {

		const bufferSize = tryMapBufferSizeForInput(options?.bufferByteLength)
		const { open, read, close } = await resolveReadFileStreamEvents(
			"plugin:android-fs|open_read_file_stream",
			uri,
		)

		try {
			await open()
			return await createReadableStream({
				read: () => read(bufferSize),
				release: close
			})
		}
		catch (e) {
			await close().catch(() => { })
			throw e
		}
	}

	/**
	 * Opens the file with read-only mode and resolves to a `ReadableStream` of text lines. 
	 *  
	 * The stream yields decoded text line by line as UTF-8. 
	 * Line breaks are not included in the emitted strings. 
	 * 
	 * The returned `ReadableStream` must always be released by the caller.
	 * Failure to do so may cause file descriptor resource leaks.
	 * The returned ReadableStream is released in the following cases:
	 * - When the ReadableStream or its Reader is canceled. 
	 * - When the ReadableStream's Reader has been fully read. 
	 * - When the ReadableStream's Reader's read operation ends with an error. 
	 * 
	 * These releases may be performed multiple times without issue.
	 * 
	 * @param uri - The URI or path of the file to read. 
	 * @param options - Optional settings.
	 * @param options.maxLineByteLength - The maximum length of a line in bytes, excluding line breaks character. If a line exceeds this limit, an error is thrown. This prevents OOM errors when reading minified files or binaries. Defaults to `0` (unlimited).
	 * @param options.fatal - Indicates whether an error is thrown when an invalid byte sequence is encountered. If `false`, invalid byte sequences are replaced with U+FFFD (`�`) and decoding continues. Defaults to `false`.
	 * @param options.ignoreBOM - Indicates whether a leading BOM is preserved and treated as a normal character. Defaults to `false`.
	 * @param options.bufferByteLength - The buffer size, in bytes, used when sending data from the backend to the frontend. IPC calls are relatively expensive (several milliseconds to tens of milliseconds per no-op call), so larger buffer sizes are generally more efficient. But if it is too large, the UI may freeze or run out of memory. This value is not guaranteed to be strictly respected. If a single line exceeds this size, more bytes may be sent in a single IPC transmission. Defaults to `512000` (500 KiB).
	 *
	 * @returns A Promise that resolves to a `ReadableStream<string>` backed by the file opened in read-only mode. This stream has a one-to-one correspondence with the file descriptor.
	 * @throws The Promise will be rejected with an error, if the specified entry does not exist, if the entry is a directory, or if the read permission is missing.
	 * 
	 * @see [AndroidFs::open_file_readable](https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.AndroidFs.html#method.open_file_readable)
	 * @since 25.1.0
	 */
	public static async openReadTextFileLinesStream(
		uri: AndroidFsUri | FsPath,
		options?: AndroidOpenReadTextFileLinesStreamOptions,
	): Promise<ReadableStream<string>> {

		const maxLineByteLength = tryMapMaxLineByteLength(options?.maxLineByteLength)
		const bufferSize = tryMapBufferSizeForInput(options?.bufferByteLength)
		const fatal = options?.fatal ?? false
		const ignoreBOM = options?.ignoreBOM ?? false
		const { open, read, close } = await resolveReadFileStreamEvents(
			"plugin:android-fs|open_read_text_file_lines_stream",
			uri,
		)

		try {
			await open()

			return await createTextLinesReadableStream(
				{
					read: () => read(bufferSize, { fatal, maxLineByteLength }),
					release: close
				},
				{ fatal, ignoreBOM }
			)
		}
		catch (e) {
			await close().catch(() => { })
			throw e
		}
	}

	/**
	 * Opens the file with write-only mode and resolves to a `WritableStream`.  
	 * Existing content of the file will be truncated.  
	 * 
	 * The returned `WritableStream` must always be released by the caller.
	 * Failure to do so may cause file descriptor resource leaks.
	 * The returned WritableStream is released in the following cases:
	 * - When the WritableStream or its Writer is closed. 
	 * - When the WritableStream or its Writer is aborted. 
	 * - When the WritableStream's Writer's write operation ends with an error. 
	 * 
	 * These releases may be performed multiple times without issue.
	 * 
	 * @param uri - The URI or path of the file to write to. If the path is specified and the entry does not exist, it will be created.
	 * @param options - Optional settings.
	 * @param options.bufferByteLength - The buffer size, in bytes, used when sending data from the frontend to the backend. IPC calls are relatively expensive (several milliseconds to tens of milliseconds per no-op call), so larger buffer sizes are generally more efficient. But if it is too large, the UI may freeze or run out of memory. Defaults to `512000` (500 KiB).
	 * 
	 * @returns A Promise that resolves to a `WritableStream<Uint8Array<ArrayBufferLike>>` backed by the file opened in write-able mode. This stream has a one-to-one correspondence with the file descriptor.
	 * 
	 * @see [AndroidFs::open_file_writable](https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.AndroidFs.html#method.open_file_writable)
	 * @since 25.1.0
	 */
	public static async openWriteFileStream(
		uri: AndroidFsUri | FsPath,
		options?: AndroidOpenWriteFileStreamOptions
	): Promise<WritableStream<Uint8Array<ArrayBufferLike>>> {

		const bufferSize = tryMapBufferSizeForInput(options?.bufferByteLength)
		const { open, write, close } = await resolveWriteFileStreamEvents(
			"plugin:android-fs|open_write_file_stream",
			uri,
		)

		try {
			await open()
			return await createBufferedWritableStream(bufferSize, {
				write,
				release: close
			})
		}
		catch (e) {
			await close().catch(() => { })
			throw e
		}
	}

	/**
	 * Reads the entire contents of the specified file as raw bytes.
	 * 
	 * For large files, consider using `AndroidFs.openReadFileStream`.
	 *
	 * @param uri - The URI or path of the target file.
	 *
	 * @returns A Promise that resolves to a `Uint8Array` containing all bytes of the file.
	 * @throws The Promise will be rejected with an error, if the specified entry does not exist, if the entry is a directory, or if the read permission is missing.
	 *
	 * @see [AndroidFs::read_file](https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.AndroidFs.html#method.read_file)
	 * @since 25.1.0
	 */
	public static async readFile(
		uri: AndroidFsUri | FsPath,
	): Promise<Uint8Array<ArrayBuffer>> {

		const buffer = await invoke<ArrayBuffer>('plugin:android-fs|read_file', {
			uri: mapFsPathForInput(uri),
		})

		return new Uint8Array(buffer)
	}

	/**
	 * Reads the entire contents of the specified file and decodes it as text.
	 * 
	 * For large files, consider using `AndroidFs.openReadFileStream` with [`TextDecoderStream`](https://developer.mozilla.org/ja/docs/Web/API/TextDecoderStream) or [`TextDecoder`](https://developer.mozilla.org/en-US/docs/Web/API/TextDecoder/TextDecoder).
	 *
	 * @param uri - The URI or path of the target file.
	 * @param options - Optional settings. They are passed to [the TextDecoder constructor](https://developer.mozilla.org/en-US/docs/Web/API/TextDecoder/TextDecoder).
	 * @param options.encoding - The text encoding to use for decoding. Defaults to `"utf-8"`.
	 * @param options.fatal - Indicates whether an error is thrown when an invalid byte sequence is encountered. If `false`, invalid byte sequences are replaced with U+FFFD (`�`) and decoding continues. Defaults to `false`.
	 * @param options.ignoreBOM - Indicates whether a leading BOM is preserved and treated as a normal character. Defaults to `false`.
	 * 
	 * @returns A Promise that resolves to the decoded text content of the file.
	 * @throws The Promise will be rejected with an error, if the specified entry does not exist, if the entry is a directory, if the read permission is missing, or if decoding fails.
	 *
	 * @see [AndroidFs::open_file_readable](https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.AndroidFs.html#method.open_file_readable)
	 * @since 25.1.0
	 */
	public static async readTextFile(
		uri: AndroidFsUri | FsPath,
		options?: AndroidReadTextFileOptions
	): Promise<string> {

		const bytes = await invoke<ArrayBuffer>('plugin:android-fs|read_text_file', {
			uri: mapFsPathForInput(uri),
		})
		const decoder = new TextDecoder(
			options?.encoding ?? "utf-8",
			{
				fatal: options?.fatal,
				ignoreBOM: options?.ignoreBOM
			}
		)

		return decoder.decode(bytes)
	}

	/**
	 * Writes bytes to the file.   
	 * Existing content of the file will be truncated.   
	 * 
	 * @param uri - The URI or path of the file to write to. If the path is specified and the entry does not exist, a new file will be created.
	 * @param data - The bytes to write, either as a `Uint8Array` or a `ReadableStream<Uint8Array>`.
	 * @param options - Optional settings.
	 * @param options.bufferByteLength - The buffer size, in bytes, used when sending data from the frontend to the backend while writing from a `ReadableStream`. IPC calls are relatively expensive (several milliseconds to tens of milliseconds per no-op call), so larger buffer sizes are generally more efficient. But if it is too large, the UI may freeze or run out of memory. Defaults to `512000` (500 KiB).
	 * 
	 * @returns A Promise that resolves when the data has been successfully written.
	 * 
	 * @see [AndroidFs::open_file_writable](https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.AndroidFs.html#method.open_file_writable)
	 * @since 25.1.0
	 */
	public static async writeFile(
		uri: AndroidFsUri | FsPath,
		data: Uint8Array<ArrayBufferLike> | ReadableStream<Uint8Array<ArrayBufferLike>>,
		options?: AndroidWriteFileOptions
	): Promise<void> {

		if (data instanceof Uint8Array) {
			await invoke("plugin:android-fs|write_file", {
				event: {
					type: "WriteOnce",
					uri: mapFsPathForInput(uri),
					data: await mapBytesForInput(data)
				}
			})
		}
		else if (data instanceof ReadableStream) {
			const bufferSize = tryMapBufferSizeForInput(options?.bufferByteLength)
			const { open, write, close } = await resolveWriteFileStreamEvents(
				"plugin:android-fs|write_file",
				uri,
			)

			try {
				await open()
				const file = await createBufferedWritableStream(bufferSize, {
					write,
					release: async () => { }
				})
				await data.pipeTo(file)
			}
			finally {
				await close()
			}
		}
		else {
			throw new Error("Unsupported data type")
		}
	}

	/**
	 * Writes text data to the file as UTF-8.   
	 * Existing content of the file will be truncated.   
	 * 
	 * @param uri - The URI or path of the file to write to. If the path is specified and the entry does not exist, a new file will be created.
	 * @param data - The text data to write.
	 *
	 * @returns A Promise that resolves when the data has been successfully written.
	 * 
	 * @see [AndroidFs::open_file_writable](https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.AndroidFs.html#method.open_file_writable)
	 * @since 25.1.0
	 */
	public static async writeTextFile(
		uri: AndroidFsUri | FsPath,
		data: string
	): Promise<void> {

		return await invoke("plugin:android-fs|write_text_file", {
			uri: mapFsPathForInput(uri),

			// Android で body や ArrayBuffer, number などを送信すると
			// 非常に非効率な文字列にシリアライズされ、著しく非効率になる。
			// よって plugin-fs のようにエンコードした後の ArrayBuffer を body として送ることはしない。
			// https://github.com/tauri-apps/tauri/issues/10573
			data
		})
	}

	/**
	 * Copies the contents of the source file to the destination file.   
	 * Existing content of the destination file will be truncated.   
	 * 
	 * @param srcUri - The URI or path of the source file to copy. 
	 * @param destUri - The URI or path of the destination file. If the path is specified and the entry does not exist, a new file will be created.
	 * 
	 * @returns A Promise that resolves when the copying is complete.
	 * 
	 * @see [AndroidFs::copy](https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.AndroidFs.html#method.copy)
	 * @since 22.0.0
	 */
	public static async copyFile(
		srcUri: AndroidFsUri | FsPath,
		destUri: AndroidFsUri | FsPath,
	): Promise<void> {

		return await invoke('plugin:android-fs|copy_file', {
			srcUri: mapFsPathForInput(srcUri),
			destUri: mapFsPathForInput(destUri)
		})
	}

	/**
	 * Deletes the existing content and sets the file size to zero.
	 * 
	 * @param uri - The URI of the file to truncate.
	 * 
	 * @returns A Promise that resolves when the truncation is complete.
	 * @throws The Promise will be rejected with an error, if the entry does not exist, if the entry is not a file, if write permission is missing, or if the file provider does not support truncation.
	 * 
	 * @see [AndroidFs::open_file_writable](https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.AndroidFs.html#method.open_file_writable)
	 * @since 22.0.0
	 */
	public static async truncateFile(uri: AndroidFsUri): Promise<void> {
		return await invoke('plugin:android-fs|truncate_file', { uri })
	}

	/**
	 * Renames the specified file and returns its new URI.
	 * 
	 * **NOTE**:  
	 * For URIs from the file picker, all permissions are lost after this operation, including for the new URI.
	 * 
	 * @param uri - The URI of the file to rename.
	 * @param name - New name, including the file extension if needed. If a entry with the same name already exists, a sequential number is appended to ensure uniqueness.
	 * 
	 * @returns A Promise that resolves to the new URI of the target file.
	 * @throws The Promise will be rejected with an error, if the entry does not exist, if the entry is not a file, if write permission is missing, or if the file provider does not support rename.
	 * 
	 * @see [AndroidFs::rename](https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.AndroidFs.html#method.rename)
	 * @since 24.1.0
	 */
	public static async renameFile(
		uri: AndroidFsUri,
		name: string
	): Promise<AndroidFsUri> {

		return await invoke('plugin:android-fs|rename_file', {
			uri,
			name
		})
	}

	/**
	 * Renames the specified directory and returns its new URI.
	 * 
	 * **NOTE**:  
	 * For URIs from the directory picker, all permissions are lost after this operation, including for the new URI.
	 * 
	 * @param uri - The URI of the directory to rename.
	 * @param name- New name. If a entry with the same name already exists, a sequential number is appended to ensure uniqueness.
	 * 
	 * @returns A Promise that resolves to the new URI of the target directory.
	 * @throws The Promise will be rejected with an error, if the entry does not exist, if the entry is not a directory, if write permission is missing, or if the file provider does not support rename.
	 * 
	 * @see [AndroidFs::rename](https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.AndroidFs.html#method.rename)
	 * @since 24.1.0
	 */
	public static async renameDir(
		uri: AndroidFsUri,
		name: string
	): Promise<AndroidFsUri> {

		return await invoke('plugin:android-fs|rename_dir', {
			uri,
			name
		})
	}

	/**
	 * Removes the specified file.
	 * 
	 * @param uri - The URI of the file to remove.
	 * 
	 * @returns A Promise that resolves when the removing is complete.
	 * @throws The Promise will be rejected with an error, if the entry does not exist, if the entry is not a file, if write permission is missing, or if the file provider does not support removing.
	 * 
	 * @see [AndroidFs::remove_file](https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.AndroidFs.html#method.remove_file)
	 * @since 22.0.0
	 */
	public static async removeFile(uri: AndroidFsUri): Promise<void> {
		return await invoke('plugin:android-fs|remove_file', { uri })
	}

	/**
	 * Removes the specified directory and all of its contents recursively.
	 * 
	 * @param uri - The URI of the directory to remove.
	 * 
	 * @returns A Promise that resolves when the removing is complete.
	 * @throws The Promise will be rejected with an error, if the entry does not exist, if the entry is not a directory, if write permission is missing, or if the file provider does not support removing.
	 * 
	 * @see [AndroidFs::remove_dir_all](https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.AndroidFs.html#method.remove_dir_all)
	 * @since 22.0.0
	 */
	public static async removeDirAll(uri: AndroidFsUri): Promise<void> {
		return await invoke('plugin:android-fs|remove_dir_all', { uri })
	}

	/**
	 * Removes the specified directory if empty.
	 * 
	 * @param uri - The URI of the direcotry to remove.
	 * 
	 * @returns A Promise that resolves when the removing is complete.
	 * @throws The Promise will be rejected with an error, if the entry does not exist, if the entry is not an empty directory, if write permission is missing, or if the file provider does not support removing.
	 * 
	 * @see [AndroidFs::remove_dir](https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.AndroidFs.html#method.remove_dir)
	 * @since 22.0.0
	 */
	public static async removeEmptyDir(uri: AndroidFsUri): Promise<void> {
		return await invoke('plugin:android-fs|remove_empty_dir', { uri })
	}

	/**
	 * Retrieves metadata and URIs for the child files and subdirectories of the specified directory.
	 * 
	 * @param uri - The URI of the direcotry to read.
	 * 
	 * @returns A Promise that resolves to an array of entries, each containing metadata and the URI of a file or directory.
	 * @throws The Promise will be rejected with an error, if the entry does not exist, if the entry is not a directory, if read permission is missing.
	 * 
	 * @see [AndroidFs::read_dir](https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.AndroidFs.html#method.read_dir)
	 * @since 22.0.0
	 */
	public static async readDir(uri: AndroidFsUri): Promise<AndroidEntryMetadataWithUri[]> {
		const entries = await invoke<AndroidEntryMetadataWithUriInner[]>('plugin:android-fs|read_dir', { uri })
		const buffer: AndroidEntryMetadataWithUri[] = new Array(entries.length)

		for (let i = 0; i < entries.length; i++) {
			const e = entries[i];
			const lastModified = new Date(e.lastModified);

			buffer[i] = e.type === "Dir"
				? { type: "Dir", name: e.name, uri: e.uri, lastModified }
				: { type: "File", name: e.name, uri: e.uri, lastModified, byteLength: e.byteLength, mimeType: e.mimeType };
		}

		return buffer
	}

	/**
	 * Opens a system file picker and allows the user to pick one or more files.
	 * 
	 * @param options - Optional configuration for the file picker.
	 * @param options.mimeTypes - The MIME types of the files to pick. If empty, any file can be selected.
	 * @param options.multiple - Indicates whether multiple files can be picked. Defaults to `false`.
	 * @param options.pickerType - Preferable picker type. One of: `"FilePicker"`, `"Gallery"`. This is not necessarily guaranteed to be used. By default, the appropriate option will be selected according to the `mimeTypes`.
	 * @param options.needWritePermission - Indicates whether write access to the picked files is required. Defaults to `false`.
	 * @param options.localOnly - Indicates whether only files located on the local device should be pickable. Defaults to `false`.
	 * @param options.initialLocation - Initial directory when launching the file picker. If this option is omitted or the desired initial location cannot be resolved,the initial location is system-specific. One of: `AndroidPickerInitialLocation.Any(...)`, `AndroidPickerInitialLocation.VolumeTop(...)`, `AndroidPickerInitialLocation.PublicDir(...)`.
	 * 
	 * @returns A Promise that resolves to an array of URI representing the picked files, or an empty array if unpicked. By default, the app has read access to the URIs, and this permission remains valid until the app or device is terminated. The app will be able to gain persistent access to the files by using `AndroidFs.persistPickerUriPermission`.
	 * 
	 * @see [FilePicker::pick_files](https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.FilePicker.html#method.pick_files)
	 * @see [FilePicker::pick_visual_medias](https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.FilePicker.html#method.pick_visual_medias)
	 * @since 22.0.0
	 */
	public static async showOpenFilePicker(
		options?: AndroidOpenFilePickerOptions
	): Promise<AndroidFsUri[]> {

		const _mimeTypes: string[] | string = options?.mimeTypes ?? []
		const mimeTypes: string[] = Array.isArray(_mimeTypes) ? _mimeTypes : [_mimeTypes]
		const multiple: boolean = options?.multiple ?? false
		const pickerType: "FilePicker" | "Gallery" | null = options?.pickerType ?? null
		const needWritePermission: boolean = options?.needWritePermission ?? false
		const localOnly = options?.localOnly ?? false
		const initialLocation = mapPickerInitialLocationForInput(options?.initialLocation)

		return await invoke("plugin:android-fs|show_open_file_picker", {
			mimeTypes,
			multiple,
			pickerType,
			needWritePermission,
			localOnly,
			initialLocation,
		})
	}

	/**
	 * Opens a system directory picker and allows the user to pick one directory.
	 * 
	 * @param options - Optional configuration for the directory picker.
	 * @param options.localOnly - Indicates whether only directories located on the local device should be pickable. Defaults to `false`.
	 * @param options.initialLocation - Initial directory when launching the directory picker. If this option is omitted or the desired initial location cannot be resolved,the initial location is system-specific. One of: `AndroidPickerInitialLocation.Any(...)`, `AndroidPickerInitialLocation.VolumeTop(...)`, `AndroidPickerInitialLocation.PublicDir(...)`.
	 * 
	 * @returns A Promise that resolves to a URI representing the picked directory, or `null` if unpicked. The directory may be a newly created directory, or it may be an existing directory. By default, the app has read-write access to the URI, and this permission remains valid until the app or device is terminated. The app will be able to gain persistent access to the directory by using `AndroidFs.persistPickerUriPermission`. Permissions for entries derived from this directory, such as `AndroidFs.readDir` and `AndroidFs.createNewFile`, depend on the permissions granted to this picked directory itself.
	 * 
	 * @see [FilePicker::pick_dir](https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.FilePicker.html#method.pick_dir)
	 * @since 22.0.0
	 */
	public static async showOpenDirPicker(
		options?: AndroidOpenDirPickerOptions
	): Promise<AndroidFsUri | null> {

		const localOnly = options?.localOnly ?? false
		const initialLocation = mapPickerInitialLocationForInput(options?.initialLocation)

		return await invoke("plugin:android-fs|show_open_dir_picker", {
			localOnly,
			initialLocation
		})
	}

	/**
	 * Opens a system file saver and allows the user to pick one file.
	 * 
	 * @param defaultFileName - An initial file name. The user may change this value before picking the file.
	 * @param mimeType - The MIME type of the file to pick. If `null`, this is inferred from the extension of `defaultFileName`.
	 * @param options - Optional configuration for the file saver.
	 * @param options.localOnly - Indicates whether only files located on the local device should be pickable. Defaults to `false`.
	 * @param options.initialLocation - Initial directory when launching the directory picker. If this option is omitted or the desired initial location cannot be resolved,the initial location is system-specific. One of: `AndroidPickerInitialLocation.Any(...)`, `AndroidPickerInitialLocation.VolumeTop(...)`, `AndroidPickerInitialLocation.PublicDir(...)`.
	 * 
	 * @return A Promise that resolves to a URI representing the picked file, or `null` if unpicked. The file may be a newly created file with no content, or it may be an existing file with the requested MIME type. By default, the app has write access to the URI, and this permission remains valid until the app or device is terminated. The app will be able to gain persistent access to the file by using `AndroidFs.persistPickerUriPermission`.
	 * 
	 * @see [FilePicker::save_file](https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.FilePicker.html#method.save_file)
	 * @since 22.0.0
	 */
	public static async showSaveFilePicker(
		defaultFileName: string,
		mimeType: string | null,
		options?: AndroidSaveFilePickerOptions
	): Promise<AndroidFsUri | null> {

		const localOnly = options?.localOnly ?? false
		const initialLocation = mapPickerInitialLocationForInput(options?.initialLocation)

		return await invoke("plugin:android-fs|show_save_file_picker", {
			defaultFileName,
			mimeType,
			localOnly,
			initialLocation
		})
	}

	/**
	 * Show app chooser for sharing the files with other apps.
	 * 
	 * This sends the files as a single unit.  
	 * The available apps depend on the MIME types associated with the files.   
	 * This does not result in an error even if no available apps are found. 
	 * An empty app chooser is displayed.
	 * 
	 * @param uris - The URIs of the target files.
	 * 
	 * @returns A promise that resolves after the app chooser is launched.
	 * @throws The Promise will be rejected with an error, if the app does not have read permission for the files.
	 * 
	 * @see [FileOpener::share_files](https://docs.rs/tauri-plugin-android-fs/21.0.0/tauri_plugin_android_fs/api/api_async/struct.FileOpener.html#method.share_files)
	 * @since 22.0.0
	 */
	public static async showShareFileDialog(
		uris: AndroidFsUri | AndroidFsUri[]
	): Promise<void> {

		return await invoke("plugin:android-fs|show_share_file_dialog", {
			uris: Array.isArray(uris) ? uris : [uris]
		})
	}

	/**
	 * Show app chooser for opening the file with other apps.
	 * 
	 * The available apps depend on the MIME types associated with the file.   
	 * This does not result in an error even if no available apps are found. 
	 * An empty app chooser is displayed.
	 * 
	 * @param uri - The URI of the target file.
	 * 
	 * @returns A promise that resolves after the app chooser is launched.
	 * @throws The Promise will be rejected with an error, if the app does not have read permission for the file.
	 * 
	 * @see [FileOpener::open_file](https://docs.rs/tauri-plugin-android-fs/21.0.0/tauri_plugin_android_fs/api/api_async/struct.FileOpener.html#method.open_file) 
	 * @since 22.0.0
	 */
	public static async showViewFileDialog(uri: AndroidFsUri): Promise<void> {
		return await invoke("plugin:android-fs|show_view_file_dialog", { uri })
	}

	/**
	 * Show app chooser for opening the directory with other apps.
	 * 
	 * This does not result in an error even if no available apps are found. 
	 * An empty app chooser is displayed.
	 * 
	 * @param uri - The URI of the target directory.
	 * 
	 * @returns A promise that resolves after the app chooser is launched.
	 * @throws The Promise will be rejected with an error, if the app does not have read permission for the directory.
	 * 
	 * @see [FileOpener::open_dir](https://docs.rs/tauri-plugin-android-fs/21.0.0/tauri_plugin_android_fs/api/api_async/struct.FileOpener.html#method.open_dir)
	 * @since 22.0.0
	 */
	public static async showViewDirDialog(uri: AndroidFsUri): Promise<void> {
		return await invoke("plugin:android-fs|show_view_dir_dialog", { uri })
	}

	/**
	 * Check a URI permission state granted by the file/directory picker.
	 * 
	 * @param uri - The URI of the target file or directory.
	 * @param state - Permission to check. One of `"Read"`, `"Write"`, `"ReadAndWrite"`, `"ReadOrWrite"`.
	 * 
	 * @returns A Promise that resolves to a boolean.
	 * 
	 * @see [FilePicker::check_uri_permission](https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.FilePicker.html#method.check_uri_permission)
	 * @since 24.1.0
	 */
	public static async checkPickerUriPermission(
		uri: AndroidFsUri,
		state: AndroidUriPermissionState
	): Promise<boolean> {

		return await invoke("plugin:android-fs|check_picker_uri_permission", { uri, state })
	}

	/**
	 * Takes a persistent permission to access the file or directory (and its descendants) selected via the file/directory picker.  
	 * This prolongs an already acquired permission rather than acquiring a new one.
	 * 
	 * Note that there is [`a limit to the total number of URIs that can be made persistent`](https://stackoverflow.com/questions/71099575/should-i-release-persistableuripermission-when-a-new-storage-location-is-chosen/71100621#71100621) using this function. 
	 * Therefore, it is recommended to release unnecessary persisted URIs via `AndroidFs.releasePersistedPickerUriPermission` or `AndroidFs.releaseAllPersistedPickerUriPermissions`.
	 * 
	 * Persisted permissions may also be revoked by other apps or the user, 
	 * by modifying the set permissions, or by moving/removing entries. 
	 * To verify, use `AndroidFs.checkPersistedPickerUriPermission` or `AndroidFs.checkPickerUriPermission`.
	 * 
	 * @param uri - The URI of the target file or directory.
	 * 
	 * @returns A Promise that resolves when the operation is complete.
	 * 
	 * @see [FilePicker::persist_picker_uri_permission](https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.FilePicker.html#method.persist_picker_uri_permission)
	 * @since 24.1.0
	 */
	public static async persistPickerUriPermission(uri: AndroidFsUri): Promise<void> {
		return await invoke("plugin:android-fs|persist_picker_uri_permission", { uri })
	}

	/**
	 * Check a persisted permission state of the URI granted via `AndroidFs.persistPickerUriPermission`.
	 * 
	 * @param uri - The URI of the target file or directory.
	 * @param state - Permission to check. One of `"Read"`, `"Write"`, `"ReadAndWrite"`, `"ReadOrWrite"`.
	 * 
	 * @returns A Promise that resolves to a boolean: `false` if only non-persistent permissions exist or if there are no permissions.
	 * 
	 * @see [FilePicker::check_persisted_picker_uri_permission](https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.FilePicker.html#method.check_persisted_picker_uri_permission)
	 * @since 24.1.0
	 */
	public static async checkPersistedPickerUriPermission(
		uri: AndroidFsUri,
		state: AndroidUriPermissionState
	): Promise<boolean> {

		return await invoke("plugin:android-fs|check_persisted_picker_uri_permission", { uri, state })
	}

	/**
	 * Relinquish a persisted permission of the URI granted via `AndroidFs.persistPickerUriPermission`.
	 * 
	 * @param uri - The URI of the target file or directory.
	 * 
	 * @returns A Promise that resolves to a boolean; `true` if a persisted permission exists for the specified URI and was successfully released. `false` if no persisted permission existed.
	 *
	 * @see [FilePicker::release_persisted_picker_uri_permission](https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.FilePicker.html#method.release_persisted_picker_uri_permission)
	 * @since 24.1.0
	 */
	public static async releasePersistedPickerUriPermission(uri: AndroidFsUri): Promise<boolean> {
		return await invoke("plugin:android-fs|release_persisted_picker_uri_permission", { uri })
	}

	/**
	 * Relinquish a all persisted permission of the URI granted via `AndroidFs.persistPickerUriPermission`.
	 * 
	 * @returns A Promise that resolves when the operation is complete.
	 * 
	 * @see [FilePicker::release_all_persisted_picker_uri_permissions](https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.FilePicker.html#method.release_all_persisted_picker_uri_permissions)
	 * @since 24.1.0
	 */
	public static async releaseAllPersistedPickerUriPermissions(): Promise<void> {
		return await invoke("plugin:android-fs|release_all_persisted_picker_uri_permissions")
	}
}


/** 500 KiB */
const DEFAULT_BUFFER_SIZE_FOR_IPC = 500 * 1024;

function tryMapBufferSizeForInput(s?: number): number {
	const bufferSize = s ?? DEFAULT_BUFFER_SIZE_FOR_IPC
	if (!isNonzeroU32(bufferSize)) {
		throw new Error(`Invalid bufferByteLength: expected a non-zero 32-bit unsigned integer, got ${bufferSize}`);
	}
	return bufferSize
}

function tryMapMaxLineByteLength(s?: number): number {
	if (s == null) return 0

	if (!Number.isSafeInteger(s) || s < 0) {
		throw new Error(`Invalid maxLineByteLength: expected a safe unsigned integer, got ${s}`);
	}

	return s
}

/** 
 * Android で frontend から body や ArrayBuffer, number[] などを送信すると
 * 非常に非効率な文字列にシリアライズされ、著しく非効率になる。
 * よって DataURL にエンコードして送信する。
 * https://github.com/tauri-apps/tauri/issues/10573
 */
async function mapBytesForInput(bytes: Uint8Array<ArrayBufferLike>): Promise<string> {
	const buffer = bytes.buffer instanceof ArrayBuffer
		? bytes as Uint8Array<ArrayBuffer>
		: new Uint8Array(bytes)

	const blob = new Blob([buffer], { type: "application/octet-stream" })
	return await blobToDataUrl(blob)
}

type ReadFileStreamEvents = {
	open: (options?: Record<any, any>) => Promise<void>
	read: (len: number, options?: Record<any, any>) => Promise<Uint8Array<ArrayBuffer> | null>,
	close: (options?: Record<any, any>) => Promise<void>,
}
async function resolveReadFileStreamEvents(
	cmd: string,
	uri: AndroidFsUri | FsPath,
): Promise<ReadFileStreamEvents> {

	// tauri::ipc::Response の制約のため全てのイベントで ArrayBuffer を返す
	type CmdEvents = {
		// ridFromBytes で id にできる bytes を返す
		// 呼び出すたびに新しくファイルを開く
		Open: { uri: AndroidFsUri | string },
		// 読み込んだ bytes を返す
		Read: { id: number, len: number },
		// 常に空の bytes を返す
		// 何回呼び出してもいい
		Close: { id: number },
	}
	type CmdType = keyof CmdEvents
	type CmdInput<T extends CmdType> = CmdEvents[T]
	function dispatch<T extends CmdType>(type: T, input: CmdInput<T>): Promise<ArrayBuffer> {
		return invoke(cmd, { event: { type, ...input } })
	}

	let id: number | null = null

	return {
		open: async (options) => {
			if (id !== null) throw new Error("File already opened")
			const idBytes = await dispatch("Open", { ...options, uri: mapFsPathForInput(uri) })
			id = ridFromBytes(idBytes)
		},
		read: async (len, options) => {
			if (id === null) throw new Error("File not opened")
			const data = await dispatch("Read", { ...options, id, len, })
			return data.byteLength === 0 ? null : new Uint8Array(data)
		},
		close: async (options) => {
			if (id === null) return
			await dispatch("Close", { ...options, id })
		}
	}
}

type WriteFileStreamEvents = {
	open: (options?: Record<any, any>) => Promise<void>,
	write: (data: Uint8Array<ArrayBufferLike>, options?: Record<any, any>) => Promise<void>,
	close: (options?: Record<any, any>) => Promise<void>,
}
async function resolveWriteFileStreamEvents(
	cmd: string,
	uri: AndroidFsUri | FsPath,
): Promise<WriteFileStreamEvents> {

	// Android で frontend から body や ArrayBuffer, number[] などを送信すると
	// 非常に非効率な文字列にシリアライズされ、著しく非効率になる。
	// よってそれらは送信しない。
	// https://github.com/tauri-apps/tauri/issues/10573
	type CmdEvents = {
		// 呼び出すたびに新しくファイルを開く
		Open: { i: { uri: AndroidFsUri | string }, o: number }
		// 
		Write: { i: { id: number; data: string }, o: void }
		// 何回呼び出してもいい
		Close: { i: { id: number }, o: void }
	}
	type CmdType = keyof CmdEvents
	type CmdInput<T extends CmdType> = CmdEvents[T]["i"]
	type CmdOutput<T extends CmdType> = CmdEvents[T]["o"]
	function dispatch<T extends CmdType>(type: T, input: CmdInput<T>): Promise<CmdOutput<T>> {
		return invoke(cmd, { event: { type, ...input } })
	}


	let id: number | null = null

	return {
		open: async (options) => {
			if (id !== null) throw new Error("File already opened")
			id = await dispatch("Open", { ...options, uri: mapFsPathForInput(uri) })
		},
		write: async (chunk, options) => {
			if (id === null) throw new Error("File not opened")
			await dispatch("Write", { ...options, id, data: await mapBytesForInput(chunk) })
		},
		close: async (options) => {
			if (id === null) return
			await dispatch("Close", { ...options, id })
		},
	}
}

let _isReadableByteStreamAvailable: boolean | null = null
function isReadableByteStreamAvailable() {
	if (_isReadableByteStreamAvailable === null) {
		try {
			new ReadableStream({ type: "bytes" })
			_isReadableByteStreamAvailable = true
		}
		catch {
			_isReadableByteStreamAvailable = false
		}
	}

	return _isReadableByteStreamAvailable
}

async function createTextLinesReadableStream(
	handler: {
		/**
		 * null または空配列で EOF。
		 * 
		 * bytes は以下の形式。
		 * それぞれの行は分断されない。     
		 * | line len (u64, big endian) | line bytes |    
		 * | line len (u64, big endian) | line bytes |    
		 * | line len (u64, big endian) | line bytes |    
		 * ...    
		 */
		read: () => Promise<Uint8Array<ArrayBuffer> | null>,
		release?: () => Promise<void>
	},
	options?: {
		fatal?: boolean,
		ignoreBOM?: boolean
	}
): Promise<ReadableStream<string>> {

	let releasePromise: Promise<void> | null = null
	const releaseOnce = () => {
		if (!releasePromise) {
			releasePromise = (handler.release ?? (async () => { }))()
		}
		return releasePromise
	}

	let decoder: TextDecoder | null = null

	return new ReadableStream({
		async pull(controller) {
			try {
				if (decoder == null) {
					decoder = new TextDecoder("utf-8", {
						fatal: options?.fatal,
						ignoreBOM: options?.ignoreBOM
					})
				}

				let buffer = await handler.read()
				if (buffer == null || buffer.byteLength === 0) {
					decoder = null
					await releaseOnce()
					controller.close()
					return
				}

				while (buffer != null && 0 < buffer.byteLength) {
					if (buffer.byteLength < 8) {
						throw new Error(`Invalid data: Chunk ended with partial header. (${buffer.byteLength} bytes remained)`)
					}
					const lineSize = trySafeU64FromBytes(buffer.subarray(0, 8), "bigEndian")
					if (buffer.byteLength < 8 + lineSize) {
						throw new Error(`Invalid data: Line split detected. Expected ${lineSize} bytes body, but only ${buffer.byteLength - 8} bytes remained in chunk.`)
					}

					const text = decoder.decode(buffer.subarray(8, 8 + lineSize))
					controller.enqueue(text)

					if (buffer.byteLength === 8 + lineSize) {
						buffer = null
					}
					else {
						buffer = buffer.subarray(8 + lineSize)
					}

					if (!decoder.ignoreBOM) {
						decoder = new TextDecoder("utf-8", {
							fatal: options?.fatal,
							ignoreBOM: true
						})
					}
				}
			}
			catch (e) {
				decoder = null
				await releaseOnce()
				throw e
			}
		},

		async cancel() {
			decoder = null
			await releaseOnce()
		}
	})
}

async function createReadableStream(
	handler: {
		/** null または空配列で EOF */
		read: () => Promise<Uint8Array<ArrayBuffer> | null>,
		release?: () => Promise<void>
	},
): Promise<ReadableStream<Uint8Array<ArrayBuffer>>> {

	let releasePromise: Promise<void> | null = null
	const releaseOnce = () => {
		if (!releasePromise) {
			releasePromise = (handler.release ?? (async () => { }))()
		}
		return releasePromise
	}

	if (!isReadableByteStreamAvailable()) {
		return new ReadableStream({
			async pull(controller) {
				try {
					const data = await handler.read()
					if (data == null || data.byteLength === 0) {
						await releaseOnce()
						controller.close()
						return
					}

					controller.enqueue(data)
				}
				catch (e) {
					await releaseOnce().catch(() => { })
					throw e
				}
			},

			async cancel() {
				await releaseOnce()
			}
		})
	}

	let buffer: Uint8Array<ArrayBuffer> | null = null

	// autoAllocateChunkSize を指定すると stream.getReader() でも byob が使われるようになるが、
	// この実装で byob を用いてもコピーが増えるだけで恩恵が少ないため指定しない。
	// また type: "bytes" で strategy を指定すると (正確には size を定義すると) エラーになる点にも注意。
	return new ReadableStream({
		type: "bytes",

		async pull(controller) {
			try {
				if (buffer == null || buffer.byteLength === 0) {
					buffer = await handler.read()
				}
				if (buffer == null || buffer.byteLength === 0) {
					buffer = null
					await releaseOnce()

					// byobRequest がある場合、respond を呼ばないと promise　が解決されない。
					// controller.close() の後だと respond(0) を読んでもエラーにはならない。
					// https://github.com/whatwg/streams/issues/1170
					controller.close()
					controller.byobRequest?.respond(0)
					return
				}

				const byob = controller.byobRequest
				// byobRequest がある場合、respond を呼ばないと promise　が解決されないことに注意
				if (byob != null) {
					// respond する前なので null にならない
					const v = byob.view!!
					const view = new Uint8Array(v.buffer, v.byteOffset, v.byteLength)
					const nread = Math.min(buffer.byteLength, view.byteLength)

					view.set(buffer.subarray(0, nread))
					buffer = buffer.subarray(nread)
					byob.respond(nread)
				}
				else {
					controller.enqueue(buffer)
					buffer = null
				}
			}
			catch (e) {
				buffer = null
				await releaseOnce().catch(() => { })

				// byobRequest が存在する場合、controller.close() を呼んだだけでは
				// Promise は解決されず、respond() も呼ぶ必要がある。
				// controller.error() も同様の挙動になる可能性がある。(要検証)
				// 少なくとも throw すれば Promise は解決されるため、現状はこの実装とする。
				throw e
			}
		},

		async cancel() {
			buffer = null
			await releaseOnce()
		}
	})
}

/**
 * chunk はクロージャーの中でのみ用いるべきであり、それ以降は参照すべきでない。
 * 必要な場合はコピーしてから用いる必要がある。
 */
async function createBufferedWritableStream(
	bufferSize: number,
	handler: {
		write: (chunk: Uint8Array<ArrayBuffer>) => Promise<void>,
		release?: () => Promise<void>
	},
): Promise<WritableStream<Uint8Array<ArrayBufferLike>>> {

	if (!Number.isSafeInteger(bufferSize) || bufferSize <= 0) {
		throw new Error("bufferSize must be a positive safe integer")
	}

	let releasePromise: Promise<void> | null = null
	const releaseOnce = () => {
		if (!releasePromise) {
			releasePromise = (handler.release ?? (async () => { }))()
		}
		return releasePromise
	}

	let buffer: Uint8Array<ArrayBuffer> | null = new Uint8Array(bufferSize)
	let bufferOffset = 0;

	return new WritableStream<Uint8Array<ArrayBufferLike>>({
		async write(src) {
			try {
				if (buffer == null) throw new Error("Buffer missing")

				let srcOffset = 0;

				while (srcOffset < src.byteLength) {
					const n = Math.min(bufferSize - bufferOffset, src.byteLength - srcOffset)
					buffer.set(src.subarray(srcOffset, srcOffset + n), bufferOffset)
					bufferOffset += n
					srcOffset += n

					if (bufferOffset === bufferSize) {
						await handler.write(buffer)
						bufferOffset = 0
					}
				}
			}
			catch (e) {
				buffer = null
				await releaseOnce().catch(() => { })
				throw e
			}
		},

		async close() {
			try {
				if (0 < bufferOffset && buffer != null) {
					await handler.write(buffer.subarray(0, bufferOffset))
				}
			}
			finally {
				buffer = null
				await releaseOnce()
			}
		},

		async abort() {
			buffer = null
			await releaseOnce()
		}
	})
}

async function blobToDataUrl(blob: Blob): Promise<string> {
	return new Promise((resolve, reject) => {
		const reader = new FileReader()

		reader.onload = () => {
			const result = reader.result
			unsub()
			if (typeof result === "string") {
				resolve(result)
			}
			else {
				reject(new Error("FileReader result is not a string"))
			}
		}
		reader.onerror = () => {
			unsub()
			reject(reader.error ?? new Error("FileReader failed"))
		}
		reader.onabort = () => {
			unsub()
			reject(new Error("FileReader aborted"))
		}

		function unsub() {
			reader.onload = null
			reader.onerror = null
			reader.onabort = null
		}

		try {
			reader.readAsDataURL(blob)
		}
		catch (err) {
			unsub()
			reject(err)
		}
	})
}

function isNonzeroU32(num: number): boolean {
	return isU32(num) && num !== 0
}

function isU32(num: number): boolean {
	return Number.isInteger(num) && 0 <= num && num <= 0xFFFFFFFF
}

function ridFromBytes(bytes: ArrayBufferView | ArrayBuffer): number {
	return u32FromBytes(bytes, "bigEndian")
}

function u32FromBytes(
	input: ArrayBufferView | ArrayBuffer,
	endian: "bigEndian" | "littleEndian"
): number {

	const bytes = input instanceof Uint8Array
		? input
		: input instanceof ArrayBuffer
			? new Uint8Array(input)
			: new Uint8Array(input.buffer, input.byteOffset, input.byteLength);

	if (bytes.length !== 4) {
		throw new Error(`Expected 4 bytes for u32, got ${bytes.length}`);
	}

	if (endian === "bigEndian") {
		// Big Endian: [0xAA, 0xBB, 0xCC, 0xDD] -> 0xAABBCCDD
		return ((bytes[0] << 24) | (bytes[1] << 16) | (bytes[2] << 8) | bytes[3]) >>> 0;
	}
	else {
		// Little Endian: [0xDD, 0xCC, 0xBB, 0xAA] -> 0xAABBCCDD
		return (bytes[0] | (bytes[1] << 8) | (bytes[2] << 16) | (bytes[3] << 24)) >>> 0;
	}
}

function trySafeU64FromBytes(
	input: ArrayBufferView | ArrayBuffer,
	endian: "bigEndian" | "littleEndian"
): number {

	const bytes = input instanceof Uint8Array
		? input
		: input instanceof ArrayBuffer
			? new Uint8Array(input)
			: new Uint8Array(input.buffer, input.byteOffset, input.byteLength);

	if (bytes.length !== 8) {
		throw new Error(`Expected 8 bytes for u64, got ${bytes.length}`);
	}

	if (endian === "bigEndian") {
		// bytes[0]: bits 56-63 (全ビット禁止)
		// bytes[1]: bits 48-55 (上位3ビット: 53, 54, 55 が禁止)
		if (bytes[0] !== 0 || (bytes[1] & 0b1110_0000) !== 0) {
			throw new Error("u64 exceeds Number.MAX_SAFE_INTEGER");
		}

		return (
			(bytes[0] * (2 ** 56)) +
			(bytes[1] * (2 ** 48)) +
			(bytes[2] * (2 ** 40)) +
			(bytes[3] * (2 ** 32)) +
			(bytes[4] * (2 ** 24)) +
			(bytes[5] * (2 ** 16)) +
			(bytes[6] * (2 ** 8)) +
			(bytes[7])
		)
	}
	else {
		// little endian
		// bytes[7]: bits 56-63 (全ビット禁止)
		// bytes[6]: bits 48-55 (上位3ビット: 53, 54, 55 が禁止)
		if (bytes[7] !== 0 || (bytes[6] & 0b1110_0000) !== 0) {
			throw new Error("u64 exceeds Number.MAX_SAFE_INTEGER");
		}

		return (
			(bytes[0]) +
			(bytes[1] * (2 ** 8)) +
			(bytes[2] * (2 ** 16)) +
			(bytes[3] * (2 ** 24)) +
			(bytes[4] * (2 ** 32)) +
			(bytes[5] * (2 ** 40)) +
			(bytes[6] * (2 ** 48)) +
			(bytes[7] * (2 ** 56))
		)
	}
}