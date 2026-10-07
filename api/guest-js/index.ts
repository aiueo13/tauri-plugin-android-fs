import { convertFileSrc as convertCustomProtocolSrc, invoke } from '@tauri-apps/api/core'
import { createReadableStream, createWritableStream } from 'create-web-stream'

/**
 * @ignore
 * @internal
 */
declare global {
	interface Window {
		__TAURI_ANDROID_FS_PLUGIN_INTERNALS__?: {
			isAndroid: boolean
		}
	}
}

/**
 * Returns whether the Tauri runtime environment is Android.
 *
 * @returns `true` if built for Android; otherwise, `false`.
 */
export function isAndroid(): boolean {
	const isAndroid = window.__TAURI_ANDROID_FS_PLUGIN_INTERNALS__?.isAndroid
	if (isAndroid !== undefined) {
		return isAndroid
	}

	throw new Error("tauri-plugin-android-fs may be not set up; see https://github.com/aiueo13/tauri-plugin-android-fs/blob/main/api/README.md")
}

let cachedApiLevel: Promise<number> | null = null

/**
 * Returns {@link https://developer.android.com/guide/topics/manifest/uses-sdk-element#ApiLevels | the API level} of the current Android device.
 * 
 * @example
 * ```ts
 * import * as AndroidFs from 'tauri-plugin-android-fs-api';
 * 
 * async function isAndroid10orHigher(): Promise<boolean> {
 * 	return AndroidFs.AndroidApiLevel.ANDROID_10 <= await AndroidFs.getAndroidApiLevel()
 * }
 * ```
 *
 * @returns Promise that resolves to the Android API level. This value is constant during the application lifecycle.
 * 
 * @see {@link https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_sync/struct.AndroidFs.html#method.api_level | AndroidFs::api_level }
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
 * @remarks  
 * Tauri does not support Android 7 or lower.
 * 
 * @see {@link https://developer.android.com/guide/topics/manifest/uses-sdk-element#api-level-table | Android API level table}
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
	ANDROID_17: 37,
} as const);

/**
 * URI for a file or directory on Android.
 *
 * @remarks
 * A single entry may have multiple URI representations.
 * Additionally, unlike a file path, a URI must refer to an existing entry.
 * 
 * This corresponds to {@link https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/struct.FsUri.html | tauri_plugin_android_fs::FsUri}.
 */
export type FsUri = {

	/**
	 * @remarks
	 * You do not need to be aware of this value.
	 */
	uri: string,

	/**
	 * @remarks
	 * You do not need to be aware of this value.
	 */
	documentTopTreeUri: string | null
}

/**
 * Checks whether a value is a valid `FsUri`.
 *
 * @param value - The value to check.
 * @returns `true` if the value is a valid `FsUri`; otherwise, `false`.
 */
export function isFsUri(value: unknown): value is FsUri {
	if (typeof value !== "object" || value === null) {
		return false
	}

	const obj = value as Record<string, unknown>

	return (
		typeof obj.uri === "string" &&
		(typeof obj.documentTopTreeUri === "string" || obj.documentTopTreeUri === null)
	);
}

function mapFsPathForInput(uri: FsUri | string | URL): string | FsUri {
	return uri instanceof URL ? uri.toString() : uri
}

export type EntryType =
	| {
		type: "Dir"
	}
	| {
		type: "File",

		/**
		 * @see {@link https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.AndroidFs.html#method.get_mime_type | AndroidFs::get_mime_type}
		 */
		mimeType: string
	}

export const ThumbnailFormat = Object.freeze({
	Jpeg: "jpeg",
	Png: "png",
	Webp: "webp",
} as const)

export type ThumbnailFormat = (typeof ThumbnailFormat)[keyof typeof ThumbnailFormat]

export type GetThumbnailOptions = {

	/**
	 * Preferred thumbnail width in pixels.
	 * 
	 * @defaultValue `512` (or same as `height` if specified)
	 */
	width?: number,

	/**
	 * Preferred thumbnail height in pixels.
	 * 
	 * @defaultValue `512` (or same as `width` if specified)
	 */
	height?: number,

	/**
	 * Image format of a thumbnail.  
	 * 
	 * @remarks
	 * One of: `"jpeg"`, `"png"`, `"webp"`.
	 * 
	 * @defaultValue `"webp"`.  
	 */
	format?: ThumbnailFormat
}

export type ConvertThumbnailSrcOptions = {

	/**
	 * Preferred thumbnail width in pixels.
	 * 
	 * @defaultValue `512` (or same as `height` if specified)
	 */
	width?: number,

	/**
	 * Preferred thumbnail height in pixels.
	 * 
	 * @defaultValue `512` (or same as `width` if specified)
	 */
	height?: number,

	/**
	 * Image format of a thumbnail.  
	 * 
	 * @remarks
	 * One of: `"jpeg"`, `"png"`, `"webp"`. 
	 * 
	 * @defaultValue `"webp"`
	 */
	format?: ThumbnailFormat
}

export type ReadDirOptions = {

	/**
	 * Number of entries to skip from the beginning.
	 *
	 * @defaultValue `0`
	 */
	offset?: number,

	/**
	 * Maximum number of entries to return.
	 *
	 * @remarks
	 * If omitted, all available entries starting from `offset` are returned.
	 */
	limit?: number,
}

export type EntryMetadata = DirMetadata | FileMetadata

export type DirMetadata = {
	type: "Dir",
	name: string,
	lastModified: Date,
}

export type FileMetadata = {
	type: "File",
	name: string,
	lastModified: Date,
	byteLength: number,
	mimeType: string,
};

type EntryMetadataInner =
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

function mutateEntryMetadata<T>(entry: EntryMetadataInner & T): EntryMetadata & T {
	(entry as any).lastModified = new Date(entry.lastModified)
	return entry as unknown as EntryMetadata & T
}

function mutateEntryMetadataArray<T>(entries: Array<EntryMetadataInner & T>): Array<EntryMetadata & T> {
	for (let i = 0; i < entries.length; i++) {
		(entries[i] as any).lastModified = new Date(entries[i].lastModified)
	}

	return entries as unknown as Array<EntryMetadata & T>
}

/**
 * Metadata and URI for a file or directory on Android.
 */
export type EntryMetadataWithUri = EntryMetadata & { uri: FsUri }

type EntryMetadataWithUriInner = EntryMetadataInner & { uri: FsUri }

export type GetUriForFilePathOptions = {

	/**
	 * MIME type for the file.
	 *
	 * @remarks
	 * If not specified, the MIME type is inferred from the extension. 
	 */
	mimeType?: string
}


export type ReadFileAsDataUrlOptions = {

	/**
	 * MIME type for the Data URL.
	 *
	 * @remarks
	 * If not specified, the MIME type provided by a provider of the file will be used.
	 */
	mimeType?: string
}

export type ReadTextFileOptions = {

	/**
	 * Text encoding used to decode the data, such as `"utf-8"`, `"shift_jis"`, or `"iso-8859-2"`.
	 * 
	 * @see {@link https://developer.mozilla.org/en-US/docs/Web/API/Encoding_API/Encodings | available encodings}
	 * @see {@link https://developer.mozilla.org/en-US/docs/Web/API/TextDecoder/TextDecoder#label | WebAPI TextDecoder's label option}
	 * 
	 * @defaultValue `"utf-8"`.
	 */
	encoding?: string,

	/**
	 * Indicates whether decoding errors are treated as fatal.
	 *
	 * @remarks
	 * The behavior is as follows:
	 * - `false`: Invalid byte sequences are replaced with U+FFFD (`�`) and decoding continues.
	 * - `true`: An error is thrown when an invalid byte sequence is encountered.
	 * 
	 * @see {@link https://developer.mozilla.org/en-US/docs/Web/API/TextDecoder/TextDecoder#fatal | WebAPI TextDecoder's fatal option}
	 *
	 * @defaultValue `false`
	 */
	fatal?: boolean,

	/**
	 * Indicates whether to ignore a leading BOM (Byte Order Mark).
	 *
	 * @remarks
	 * The behavior is as follows:
	 * - `false`: The leading BOM is automatically stripped from the decoded result.
	 * - `true`: The leading BOM is preserved as a regular character.
	 * 
	 * @see {@link https://developer.mozilla.org/en-US/docs/Web/API/TextDecoder/TextDecoder#ignorebom | WebAPI TextDecoder's ignoreBOM option}
	 *
	 * @defaultValue `false`
	 */
	ignoreBOM?: boolean,
}

export type WriteFileOptions = {

	/**
	 * Indicates whether to create a new file if it does not exist.
	 * 
	 * @remarks
	 * This option is only valid for files specified by a path. 
	 * It is ignored for files specified by a URI.
	 *
	 * @defaultValue `true`
	 */
	create?: boolean,

	/**
	 * Indicates whether to append data to the end of the file.
	 * 
	 * @remarks
	 * The behavior is as follows:
	 * - `true`: Preserves the existing data and writes the new data to the end of the file.
	 * - `false`: Truncates the existing data and writes the new data.
	 * 
	 * For files selected via File/Directory Picker,
	 * `true` may not be supported and could result in an error.
	 * 
	 * @defaultValue `false`
	 */
	append?: boolean,

	/**
	 * Configuration for the system progress notification.
	 * 
	 * @remarks
	 * The behavior is as follows:
	 * - `undefined` : A progress notification is omitted.  
	 * - Specified: A progress notification is displayed in the Android status bar during the operation.
	 * 
	 * Accepts a custom configuration object or one of the predefined presets: 
	 * - `ProgressNotificationTemplate.Default`
	 * - `ProgressNotificationTemplate.DefaultDownload`
	 * - `ProgressNotificationTemplate.DefaultUpload`
	 * - `ProgressNotificationTemplate.DefaultSave`
	 */
	notification?: ProgressNotificationTemplate,
}

export type WriteTextFileOptions = {

	/**
	 * Indicates whether to create a new file if it does not exist.
	 *
	 * @remarks
	 * This option is only valid for files specified by a path. 
	 * It is ignored for files specified by a URI.
	 * 
	 * @defaultValue `true`
	 */
	create?: boolean,

	/**
	 * Indicates whether to append data to the end of the file.
	 * 
	 * @remarks
	 * The behavior is as follows:
	 * - `true`: Preserves the existing data and writes the new data to the end of the file.
	 * - `false`: Truncates the existing data and writes the new data.
	 * 
	 * For files selected via File/Directory Picker,
	 * `true` may not be supported and could result in an error.
	 * 
	 * @defaultValue `false`
	 */
	append?: boolean,

	/**
	 * Configuration for the system progress notification.
	 * 
	 * @remarks
	 * The behavior is as follows:
	 * - `undefined` : A progress notification is omitted.  
	 * - Specified: A progress notification is displayed in the Android status bar during the operation.
	 * 
	 * Accepts a custom configuration object or one of the predefined presets: 
	 * - `ProgressNotificationTemplate.Default`
	 * - `ProgressNotificationTemplate.DefaultDownload`
	 * - `ProgressNotificationTemplate.DefaultUpload`
	 * - `ProgressNotificationTemplate.DefaultSave`
	 */
	notification?: ProgressNotificationTemplate,
}

export type CopyFileOptions = {

	/**
	 * Indicates whether to create a new file if it does not exist.
	 * 
	 * @remarks
	 * This option is only valid for files specified by a path. 
	 * It is ignored for files specified by a URI.
	 * 
	 * @defaultValue `true`
	 */
	create?: boolean,

	/**
	 * Configuration for the system progress notification.
	 * 
	 * @remarks
	 * The behavior is as follows:
	 * - `undefined` : A progress notification is omitted.  
	 * - Specified: A progress notification is displayed in the Android status bar during the operation.
	 * 
	 * Accepts a custom configuration object or one of the predefined presets: 
	 * - `ProgressNotificationTemplate.Default`
	 * - `ProgressNotificationTemplate.DefaultDownload`
	 * - `ProgressNotificationTemplate.DefaultUpload`
	 * - `ProgressNotificationTemplate.DefaultSave`
	 */
	notification?: ProgressNotificationTemplate,
}

export type OpenWriteFileStreamOptions = {

	/**
	 * Buffer size, in bytes, used when sending data from the frontend to the backend.
	 * 
	 * @remarks
	 * IPC calls are relatively expensive, so larger buffer sizes are generally more efficient. 
	 * However, setting this value too high may cause the UI to freeze or result in out-of-memory errors.
	 * 
	 * @defaultValue `524288` (512 KiB)
	 */
	bufferByteLength?: number,

	/**
	 * `AbortSignal` that allows the write operation to be aborted.
	 * 
	 * @remarks
	 * When aborted, the stream enters an errored state, all subsequent write operations fail,
	 * and the underlying file resources are released immediately.
	 */
	signal?: AbortSignal,

	/**
	 * Indicates whether to create a new file if it does not exist.
	 *
	 * @remarks
	 * This option is only valid for files specified by a path. 
	 * It is ignored for files specified by a URI.
	 * 
	 * @defaultValue `true`
	 */
	create?: boolean,

	/**
	 * Indicates whether to append data to the end of the file.
	 * 
	 * @remarks
	 * The behavior is as follows:
	 * - `true`: Preserves the existing data and writes the new data to the end of the file.
	 * - `false`: Truncates the existing data and writes the new data.
	 * 
	 * For files selected via File/Directory Picker,
	 * `true` may not be supported and could result in an error.
	 * 
	 * @defaultValue `false`
	 */
	append?: boolean,

	/**
	 * Configuration for the system progress notification.
	 * 
	 * @remarks
	 * The behavior is as follows:
	 * - `undefined` : A progress notification is omitted.  
	 * - Specified: A progress notification is displayed in the Android status bar during the operation.
	 * 
	 * Accepts a custom configuration object or one of the predefined presets: 
	 * - `ProgressNotificationTemplate.Default`
	 * - `ProgressNotificationTemplate.DefaultDownload`
	 * - `ProgressNotificationTemplate.DefaultUpload`
	 * - `ProgressNotificationTemplate.DefaultSave`
	 */
	notification?: ProgressNotificationTemplate,
}

export type OpenReadFileStreamOptions = {

	/**
	 * Buffer size, in bytes, used when sending data from the backend to the frontend.
	 * 
	 * @remarks
	 * IPC calls are relatively expensive, so larger buffer sizes are generally more efficient. 
	 * However, setting this value too high may cause the UI to freeze or result in out-of-memory errors.
	 * 
	 * @defaultValue `524288` (512 KiB)
	 */
	bufferByteLength?: number,

	/**
	 * `AbortSignal` that allows the read operation to be aborted.
	 * 
	 * @remarks
	 * When aborted, the stream enters an errored state, all subsequent read operations fail,
	 * and the underlying file resources are released immediately.
	 */
	signal?: AbortSignal,
}

export type OpenReadTextFileLinesStreamItem = {

	/**
	 * Text of a line.
	 * 
	 * @remarks
	 * This value excludes line break characters.
	 * If needed, use `lineBreak`.
	 */
	line: string,

	/**
	 * Line break characters at the end of a line.
	 * 
	 * @remarks
	 * One of: `"\n"`, `"\r\n"`, `null`.
	 * 
	 * This value is `null` 
	 * only if the line is the last line and does not end with a line break.
	 */
	lineBreak: "\n" | "\r\n" | null
}

export type OpenReadTextFileLinesStreamOptions = {

	/**
	 * Text encoding used to decode the data, such as `"utf-8"`, `"shift_jis"`, or `"iso-8859-2"`.
	 * 
	 * @see {@link https://developer.mozilla.org/en-US/docs/Web/API/Encoding_API/Encodings | available encodings}
	 * @see {@link https://developer.mozilla.org/en-US/docs/Web/API/TextDecoder/TextDecoder#label | WebAPI TextDecoder's label option}
	 * 
	 * @defaultValue `"utf-8"`.
	 */
	encoding?: string,

	/**
	 * Indicates whether decoding errors are treated as fatal.
	 *
	 * @remarks
	 * The behavior is as follows:
	 * - `false`: Invalid byte sequences are replaced with U+FFFD (`�`) and decoding continues.
	 * - `true`: An error is thrown when an invalid byte sequence is encountered.
	 * 
	 * @see {@link https://developer.mozilla.org/en-US/docs/Web/API/TextDecoder/TextDecoder#fatal | WebAPI TextDecoder's fatal option}
	 *
	 * @defaultValue `false`
	 */
	fatal?: boolean,

	/**
	 * Indicates whether to ignore a leading BOM (Byte Order Mark).
	 *
	 * @remarks
	 * The behavior is as follows:
	 * - `false`: The leading BOM is automatically stripped from the decoded result.
	 * - `true`: The leading BOM is preserved as a regular character.
	 * 
	 * @see {@link https://developer.mozilla.org/en-US/docs/Web/API/TextDecoder/TextDecoder#ignorebom | WebAPI TextDecoder's ignoreBOM option}
	 *
	 * @defaultValue `false`
	 */
	ignoreBOM?: boolean,

	/**
	 * Buffer size, in bytes, used when sending data from the backend to the frontend.
	 * 
	 * @remarks
	 * IPC calls are relatively expensive, so larger buffer sizes are generally more efficient. 
	 * However, setting this value too high may cause the UI to freeze or result in out-of-memory errors.
	 * 
	 * This value is not guaranteed to be strictly respected.
	 * If a single line exceeds this size, more bytes may be sent in a single IPC transmission.
	 * To prevent OOM errors, use `maxLineByteLength`.
	 * 
	 * @defaultValue `524288` (512 KiB)
	 */
	bufferByteLength?: number,

	/**
	 * `AbortSignal` that allows the read operation to be aborted.
	 * 
	 * @remarks
	 * When aborted, the stream enters an errored state, all subsequent read operations fail,
	 * and the underlying file resources are released immediately.
	 */
	signal?: AbortSignal,

	/**
	 * Maximum byte length of a line before decoding.
	 * 
	 * @remarks
	 * If a line exceeds this limit, an error is thrown. 
	 * This prevents OOM errors when reading minified files or binaries.
	 * 
	 * This excluding line break characters and the initial BOM (if present). 
	 * 
	 * @defaultValue `0` (unlimited)
	 */
	maxLineByteLength?: number,
}

export type OpenFilePickerOptions = {

	/**
	 * MIME types of the files to pick.   
	 * If empty, any file can be selected.  
	 */
	mimeTypes?: string[] | string,

	/**
	 * Indicates whether multiple files can be picked.  
	 * 
	 * @defaultValue `false`
	 */
	multiple?: boolean,

	/**
	 * Preferred picker type.  
	 * 
	 * @remarks
	 * `"Gallery"` is not guaranteed to be used.  
	 * 
	 * If not specified, an appropriate option is selected based on the `mimeTypes`. 
	 */
	pickerType?: "FilePicker" | "Gallery",

	/**
	 * Indicates whether write access to the picked files is required.  
	 * 
	 * @defaultValue `false`
	 */
	needWritePermission?: boolean,

	/**
	 * Indicates whether only files located on the local device are pickable.
	 *   
	 * @defaultValue `false`
	 */
	localOnly?: boolean,

	/**
	 * Initial directory when launching File Picker.  
	 * 
	 * @remarks
	 * If this value is not specified or the desired initial location cannot be resolved,
	 * the initial location is system-specific.
	 * 
	 * One of: 
	 * - `PickerInitialLocation.Any(...)` 
	 * - `PickerInitialLocation.VolumeTop(...)`   
	 * - `PickerInitialLocation.PublicDir(...)`
	 */
	initialLocation?: PickerInitialLocation
}

export type OpenDirPickerOptions = {

	/**
	 * Indicates whether only directories located on the local device are pickable.
	 *   
	 * @defaultValue `false`
	 */
	localOnly?: boolean,

	/**
	 * Initial directory when launching Directory Picker.  
	 * 
	 * @remarks
	 * If this value is not specified or the desired initial location cannot be resolved,
	 * the initial location is system-specific.
	 * 
	 * One of: 
	 * - `PickerInitialLocation.Any(...)` 
	 * - `PickerInitialLocation.VolumeTop(...)`   
	 * - `PickerInitialLocation.PublicDir(...)`
	 */
	initialLocation?: PickerInitialLocation
}

export type SaveFilePickerOptions = {

	/**
	 * Indicates whether only files located on the local device are pickable.
	 *   
	 * @defaultValue `false`
	 */
	localOnly?: boolean,

	/**
	 * Initial directory when launching File Picker.  
	 * 
	 * @remarks
	 * If this value is not specified or the desired initial location cannot be resolved,
	 * the initial location is system-specific.
	 * 
	 * One of: 
	 * - `PickerInitialLocation.Any(...)` 
	 * - `PickerInitialLocation.VolumeTop(...)`   
	 * - `PickerInitialLocation.PublicDir(...)`
	 */
	initialLocation?: PickerInitialLocation
}

export type CreateNewPublicFileOptions = {

	/**
	 * Indicates whether to prompt the user for permissions to access public files if it has not already been granted. 
	 *  
	 * @defaultValue `true`
	 */
	requestPermission?: boolean,

	/**
	 * ID of the storage volume where the file will be created. 
	 * 
	 * @defaultValue {@link https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.PublicStorage.html#method.get_primary_volume | Primary storage volume}
	 */
	volumeId?: StorageVolumeId,

	/**
	 * Indicates whether the file is marked as pending.  
	 * 
	 * @remarks
	 * When set to `true`, the app has exclusive access to the file, 
	 * making it invisible to other apps until `setPublicFilePending(..., false)` is called. 
	 * 
	 * If the file remains pending for more than 7 days, 
	 * the system automatically deletes it.  
	 * 
	 * This is available for Android 11 (API level 30) and higher.  
	 * If unavailable, this option is ignored.
	 * 
	 * @defaultValue `false`
	 */
	isPending?: boolean
}

export const PublicGeneralPurposeDir = Object.freeze({

	/**
	 * Standard directory for user-accessible documents.
	 *
	 * @remarks
	 * This is a subdirectory under the user-visible top-level directory of the storage volume.
	 *
	 * e.g.
	 * - `/storage/emulated/{user-id}/Documents`
	 * - `/storage/{sd-card-id}/Documents`
	 */
	Documents: "Documents",

	/**
	 * Standard directory for user-accessible downloaded files.
	 *
	 * @remarks
	 * This is a subdirectory under the user-visible top-level directory of the storage volume.  
	 *
	 * e.g.
	 * - `/storage/emulated/{user-id}/Download`
	 * - `/storage/{sd-card-id}/Download`
	 */
	Download: "Download",
} as const);

/**
 * Android public directories for image files.
 */
export const PublicImageDir = Object.freeze({

	/**
	 * Standard directory for user-accessible pictures.
	 *
	 * @remarks
	 * This is a subdirectory under the user-visible top-level directory of the storage volume.
	 *
	 * e.g.
	 * - `/storage/emulated/{user-id}/Pictures`
	 * - `/storage/{sd-card-id}/Pictures`
	 */
	Pictures: "Pictures",

	/**
	 * Standard directory for user-accessible pictures and videos captured when the device is mounted as a camera.
	 *
	 * @remarks
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
export const PublicVideoDir = Object.freeze({

	/**
	 * Standard directory for user-accessible movies.
	 *
	 * @remarks
	 * This is a subdirectory under the user-visible top-level directory of the storage volume.
	 * 
	 * e.g.
	 * - `/storage/emulated/{user-id}/Movies`
	 * - `/storage/{sd-card-id}/Movies`
	 */
	Movies: "Movies",

	/**
	 * Standard directory for user-accessible pictures and videos captured when the device is mounted as a camera.
	 *
	 * @remarks
	 * This is a subdirectory under the user-visible top-level directory of the storage volume.
	 *
	 * e.g.
	 * - `/storage/emulated/{user-id}/DCIM`
	 * - `/storage/{sd-card-id}/DCIM`
	 */
	DCIM: "DCIM",

	/**
	 * Standard directory for user-accessible pictures.
	 *
	 * @remarks
	 * This is a subdirectory under the user-visible top-level directory of the storage volume.
	 *
	 * e.g.
	 * - `/storage/emulated/{user-id}/Pictures`
	 * - `/storage/{sd-card-id}/Pictures`
	 */
	Pictures: "Pictures",
} as const);

/**
 * Android public directories for audio files.
 */
export const PublicAudioDir = Object.freeze({

	/**
	 * Standard directory for user-accessible regular music.
	 *
	 * @remarks
	 * This is a subdirectory under the user-visible top-level directory of the storage volume.
	 * 
	 * e.g.
	 * - `/storage/emulated/{user-id}/Music`
	 * - `/storage/{sd-card-id}/Music`
	 */
	Music: "Music",

	/**
	 * Standard directory for user-accessible alams (not as regular music).
	 *
	 * @remarks
	 * This is a subdirectory under the user-visible top-level directory of the storage volume.
	 * 
	 * e.g.
	 * - `/storage/emulated/{user-id}/Alarms`
	 * - `/storage/{sd-card-id}/Alarms`
	 */
	Alarms: "Alarms",

	/**
	 * Standard directory for user-accessible audiobooks (not as regular music).
	 *
	 * @remarks
	 * This is a subdirectory under the user-visible top-level directory of the storage volume.
	 *
	 * e.g.
	 * - `/storage/emulated/{user-id}/Audiobooks`
	 * - `/storage/{sd-card-id}/Audiobooks`
	 *
	 * This is available for Android 10 (API level 29) and higher.  
	 * If unavailable, the `~/Music/Audiobooks` folder will be used instead.  
	 */
	Audiobooks: "Audiobooks",

	/**
	 * Standard directory for user-accessible notifications (not as regular music).
	 *
	 * @remarks
	 * This is a subdirectory under the user-visible top-level directory of the storage volume.
	 *
	 * e.g.
	 * - `/storage/emulated/{user-id}/Notifications`
	 * - `/storage/{sd-card-id}/Notifications`
	 */
	Notifications: "Notifications",

	/**
	 * Standard directory for user-accessible podcasts (not as regular music).
	 *
	 * @remarks
	 * This is a subdirectory under the user-visible top-level directory of the storage volume.
	 *
	 * e.g.
	 * - `/storage/emulated/{user-id}/Podcasts`
	 * - `/storage/{sd-card-id}/Podcasts`
	 */
	Podcasts: "Podcasts",

	/**
	 * Standard directory for user-accessible ringtones (not as regular music).
	 *
	 * @remarks
	 * This is a subdirectory under the user-visible top-level directory of the storage volume.
	 *
	 * e.g.
	 * - `/storage/emulated/{user-id}/Ringtones`
	 * - `/storage/{sd-card-id}/Ringtones`
	 */
	Ringtones: "Ringtones",

	/**
	 * Standard directory for user-accessible recordings (not as regular music).
	 *
	 * @remarks
	 * This is a subdirectory under the user-visible top-level directory of the storage volume.
	 *
	 * e.g.
	 * - `/storage/emulated/{user-id}/Recordings`
	 * - `/storage/{sd-card-id}/Recordings`
	 *
	 * This is available for Android 12 (API level 31) or higher.  
	 * If unavailable, the `~/Music/Recordings` folder will be used instead.
	 */
	Recordings: "Recordings",
} as const);

/**
 * Android public directories.
 */
export const PublicDir = Object.freeze({

	/**
	 * Standard directory for user-accessible documents.
	 *
	 * @remarks
	 * This is a subdirectory under the user-visible top-level directory of the storage volume.
	 *
	 * e.g.
	 * - `/storage/emulated/{user-id}/Documents`
	 * - `/storage/{sd-card-id}/Documents`
	 */
	Documents: "Documents",

	/**
	 * Standard directory for user-accessible downloaded files.
	 *
	 * @remarks
	 * This is a subdirectory under the user-visible top-level directory of the storage volume.  
	 *
	 * e.g.
	 * - `/storage/emulated/{user-id}/Download`
	 * - `/storage/{sd-card-id}/Download`
	 */
	Download: "Download",

	/**
	 * Standard directory for user-accessible pictures.
	 *
	 * @remarks
	 * This is a subdirectory under the user-visible top-level directory of the storage volume.
	 *
	 * e.g.
	 * - `/storage/emulated/{user-id}/Pictures`
	 * - `/storage/{sd-card-id}/Pictures`
	 */
	Pictures: "Pictures",

	/**
	 * Standard directory for user-accessible movies.
	 *
	 * @remarks
	 * This is a subdirectory under the user-visible top-level directory of the storage volume.
	 * 
	 * e.g.
	 * - `/storage/emulated/{user-id}/Movies`
	 * - `/storage/{sd-card-id}/Movies`
	 */
	Movies: "Movies",

	/**
	 * Standard directory for user-accessible pictures and videos captured when the device is mounted as a camera.
	 *
	 * @remarks
	 * This is a subdirectory under the user-visible top-level directory of the storage volume.
	 *
	 * e.g.
	 * - `/storage/emulated/{user-id}/DCIM`
	 * - `/storage/{sd-card-id}/DCIM`
	 */
	DCIM: "DCIM",

	/**
	 * Standard directory for user-accessible regular music.
	 *
	 * @remarks
	 * This is a subdirectory under the user-visible top-level directory of the storage volume.
	 * 
	 * e.g.
	 * - `/storage/emulated/{user-id}/Music`
	 * - `/storage/{sd-card-id}/Music`
	 */
	Music: "Music",

	/**
	 * Standard directory for user-accessible alams (not as regular music).
	 *
	 * @remarks
	 * This is a subdirectory under the user-visible top-level directory of the storage volume.
	 * 
	 * e.g.
	 * - `/storage/emulated/{user-id}/Alarms`
	 * - `/storage/{sd-card-id}/Alarms`
	 */
	Alarms: "Alarms",

	/**
	 * Standard directory for user-accessible audiobooks (not as regular music).
	 *
	 * @remarks
	 * This is a subdirectory under the user-visible top-level directory of the storage volume.
	 *
	 * e.g.
	 * - `/storage/emulated/{user-id}/Audiobooks`
	 * - `/storage/{sd-card-id}/Audiobooks`
	 *
	 * This is available for Android 10 (API level 29) and higher.  
	 * If unavailable, the `~/Music/Audiobooks` folder will be used instead.  
	 */
	Audiobooks: "Audiobooks",

	/**
	 * Standard directory for user-accessible notifications (not as regular music).
	 *
	 * @remarks
	 * This is a subdirectory under the user-visible top-level directory of the storage volume.
	 *
	 * e.g.
	 * - `/storage/emulated/{user-id}/Notifications`
	 * - `/storage/{sd-card-id}/Notifications`
	 */
	Notifications: "Notifications",

	/**
	 * Standard directory for user-accessible podcasts (not as regular music).
	 *
	 * @remarks
	 * This is a subdirectory under the user-visible top-level directory of the storage volume.
	 *
	 * e.g.
	 * - `/storage/emulated/{user-id}/Podcasts`
	 * - `/storage/{sd-card-id}/Podcasts`
	 */
	Podcasts: "Podcasts",

	/**
	 * Standard directory for user-accessible ringtones (not as regular music).
	 *
	 * @remarks
	 * This is a subdirectory under the user-visible top-level directory of the storage volume.
	 *
	 * e.g.
	 * - `/storage/emulated/{user-id}/Ringtones`
	 * - `/storage/{sd-card-id}/Ringtones`
	 */
	Ringtones: "Ringtones",

	/**
	 * Standard directory for user-accessible recordings (not as regular music).
	 *
	 * @remarks
	 * This is a subdirectory under the user-visible top-level directory of the storage volume.
	 *
	 * e.g.
	 * - `/storage/emulated/{user-id}/Recordings`
	 * - `/storage/{sd-card-id}/Recordings`
	 *
	 * This is available for Android 12 (API level 31) or higher.  
	 * If unavailable, the `~/Music/Recordings` folder will be used instead.
	 */
	Recordings: "Recordings",
} as const);

export type PublicGeneralPurposeDir = (typeof PublicGeneralPurposeDir)[keyof typeof PublicGeneralPurposeDir]
export type PublicImageDir = (typeof PublicImageDir)[keyof typeof PublicImageDir]
export type PublicVideoDir = (typeof PublicVideoDir)[keyof typeof PublicVideoDir]
export type PublicAudioDir = (typeof PublicAudioDir)[keyof typeof PublicAudioDir]
export type PublicDir = (typeof PublicDir)[keyof typeof PublicDir];

export type StorageVolumeMetadata = {

	/**
	 * User-visible description of this storage volume.  
	 * 
	 * @remarks
	 * This is determined by the manufacturer and is often localized based on the user’s language.
	 * 
	 * e.g.
	 * - `"Internal shared storage"`
	 * - `"SD card"`
	 */
	description: string,

	/**
	 * Indicates whether this is the primary storage volume. 
	 * 
	 * @remarks
	 * A device always has exactly one primary storage volume.
	 * 
	 * The primary volume may be inaccessible if it is mounted on a computer by the user, 
	 * removed from the device, or experiencing other issues.  
	 * Therefore, the primary storage volume is not guaranteed to be included in the list.
	 */
	isPrimary: boolean,

	/**
	 * Indicates whether this volume is physically removable. 
	 * 
	 * @remarks
	 * If `false`, this is the device's built-in storage.
	 */
	isRemovable: boolean,

	/**
	 * Indicates whether this volume is a stable part of the device.
	 * 
	 * @remarks
	 * For example, a device's built-in storage and physical media slots under protective covers are considered stable,
	 * while USB flash drives connected to handheld devices are not.
	 */
	isStable: boolean,

	/**
	 * Indicates whether this volume is emulated.
	 * 
	 * @remarks
	 * An emulated volume is backed by a private user data partition, 
	 * such as internal storage or {@link https://source.android.com/docs/core/storage/adoptable | adoptable storage}.
	 */
	isEmulated: boolean,

	/**
	 * Indicates whether this is a read-only storage volume.
	 * 
	 * @remarks
	 * e.g., SD card in read-only mode.
	 */
	isReadOnly: boolean,

	/**
	 * Indicates whether public files can be placed on this storage volume.
	 *
	 * @remarks
	 * This does not indicate whether the volume is currently writable
	 * (i.e., whether public files can actually be created on it).
	 * For that information, refer to `isReadOnly`.
	 */
	isAvailableForPublicFiles: boolean,

	/**
	 * ID of this storage volume.
	 * 
	 * @remarks
	 * Since the storage volume ID can change, 
	 * it should not be persisted across app restarts.
	 */
	id: StorageVolumeId
}

/**
 * ID of the storage volume on Android.
 * 
 * @remarks
 * Since the storage volume ID can change, 
 * it should not be persisted across app restarts.
 */
export type StorageVolumeId = string;

export const UriPermissionState = Object.freeze({
	Read: "Read",
	Write: "Write",
	ReadAndWrite: "ReadAndWrite",
	ReadOrWrite: "ReadOrWrite"
} as const)

export type UriPermissionState = typeof UriPermissionState[keyof typeof UriPermissionState]

export type PersistedUriPermission = {
	type: "Dir" | "File", 
	uri: FsUri, 
	hasReadPermission: boolean, 
	hasWritePermission: boolean,
};

export const ListVolumesPurpose = Object.freeze({
	CreatePublicFile: "CreatePublicFile",
	PickerInitialLocation: "PickerInitialLocation"
} as const)

export type ListVolumesPurpose = (typeof ListVolumesPurpose)[keyof typeof ListVolumesPurpose]

export type ListVolumesOptions = {

	/**
	 * Purpose for listing storage volumes.
	 *
	 * @remarks
	 * The behavior is as follows:
	 * - `"CreatePublicFile"`:
	 * Lists only volumes available for `createNewPublicFile` and its related functions.
	 * This excludes non-writable volumes (e.g., a read-only SD card). 
	 * Additionally, on Android 9 and below, it excludes secondary storage volumes that are inaccessible to `createNewPublicFile` due to Android platform restrictions.
	 * In other words, it returns only volumes where `StorageVolumeMetadata.isReadOnly` is `false` and `StorageVolumeMetadata.isAvailableForPublicFiles` is `true`.
	 * - `"PickerInitialLocation"`:
	 * Lists all volumes available for use as an initial picker location.
	 * This includes all detected volumes.
	 * - `undefined`: Lists only volumes available for all purposes.
	 */
	purpose?: ListVolumesPurpose
}

/**
 * Initial location when launching File/Directory Picker on Android.
 */
export type PickerInitialLocation =
	| { type: "Any", uri: FsUri }
	| { type: "VolumeTop", volumeId?: StorageVolumeId }
	| {
		type: "PublicDir"
		baseDir: PublicDir
		relativePath?: string
		volumeId?: StorageVolumeId
	}

type PickerInitialLocationInner =
	| { type: "Any", uri: FsUri }
	| { type: "VolumeTop", volumeId: StorageVolumeId | null }
	| {
		type: "PublicDir"
		baseDir: PublicDir
		relativePath: string | null
		volumeId: StorageVolumeId | null
	}

function mapPickerInitialLocationForInput(
	i?: PickerInitialLocation | undefined | null
): PickerInitialLocationInner | null {

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

export type PickerInitialLocationPublicDirOptions = {

	/**
	 * Relative path from the public directory.
	 */
	relativePath?: string

	/**
	 * ID of the storage volume that the public directory belongs to.  
	 * 
	 * @defaultValue {@link https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.PublicStorage.html#method.get_primary_volume | Primary storage volume}
	 */
	volumeId?: StorageVolumeId
}

/**
 * Initial location when launching File/Directory Picker on Android.
 */
export const PickerInitialLocation = Object.freeze({

	/**
	 * Builds an initial picker location at the specified directory,
	 * or in the directory containing the specified file.
	 * 
	 * @param uri - URI of the target entry.
	 */
	Any(uri: FsUri): PickerInitialLocation {
		return {
			type: "Any",
			uri,
		}
	},

	/**
	 * Builds an initial picker location at the top of the storage volume.
	 *
	 * @param volumeId - ID of the target storage volume. Defaults to {@link https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.PublicStorage.html#method.get_primary_volume | Primary storage volume}.
	 */
	VolumeTop(
		volumeId?: StorageVolumeId
	): PickerInitialLocation {

		return {
			type: "VolumeTop",
			volumeId,
		}
	},

	/**
	 * Builds an initial picker location inside the public directory.
	 *
	 * @param baseDir - Target public directory. One of: `"Documents"`, `"Download"`, `"Pictures"`, `"DCIM"`, `"Movies"`, `"Music"`, `"Alarms"`, `"Audiobooks"`, `"Notifications"`, `"Podcasts"`, `"Ringtones"`, `"Recordings"`.
	 * @param options - Optional settings: `relativePath`, `volumeId`. See `PickerInitialLocationPublicDirOptions` for details.
	 */
	PublicDir(
		baseDir: PublicDir,
		options?: PickerInitialLocationPublicDirOptions
	): PickerInitialLocation {

		return {
			type: "PublicDir",
			baseDir,
			relativePath: options?.relativePath,
			volumeId: options?.volumeId
		}
	},
} as const)

export const ProgressNotificationIconType = Object.freeze({

	/**
	 * Application icon
	 */
	App: "App",

	/**
	 * Download icon
	 */
	Download: "Download",

	/**
	 * Upload icon
	 */
	Upload: "Upload",

	/**
	 * Save icon
	 */
	Save: "Save",
} as const);

export type ProgressNotificationIconType = (typeof ProgressNotificationIconType)[keyof typeof ProgressNotificationIconType]

export const ProgressNotificationTemplate = Object.freeze({

	/**
	 * Default application notification settings.
	 */
	Default: Object.freeze({
		icon: ProgressNotificationIconType.App,
		title: "{{fileName}}",
		subTextProgress: "{{progress}}",
		subTextCompletion: "{{progress}}"
	} as const satisfies ProgressNotificationTemplate),

	/**
	 * Default download notification settings.
	 */
	DefaultDownload: Object.freeze({
		icon: ProgressNotificationIconType.Download,
		title: "{{fileName}}",
		subTextProgress: "{{progress}}",
		subTextCompletion: "{{progress}}"
	} as const satisfies ProgressNotificationTemplate),

	/**
	 * Default upload notification settings.
	 */
	DefaultUpload: Object.freeze({
		icon: ProgressNotificationIconType.Upload,
		title: "{{fileName}}",
		subTextProgress: "{{progress}}",
		subTextCompletion: "{{progress}}"
	} as const satisfies ProgressNotificationTemplate),

	/**
	 * Default save notification settings.
	 */
	DefaultSave: Object.freeze({
		icon: ProgressNotificationIconType.Save,
		title: "{{fileName}}",
		subTextProgress: "{{progress}}",
		subTextCompletion: "{{progress}}"
	} as const satisfies ProgressNotificationTemplate),
} as const)

export type ProgressNotificationTemplate = {

	/**
	 * Icon of the notification.
	 * 
	 * @remarks
	 * One of: `"App"`, `"Download"`, `"Upload"`, `"Save"`.
	 */
	icon: ProgressNotificationIconType,

	/**
	 * Total length of the data in bytes.
	 * 
	 * @remarks
	 * If not specified, the progress bar will be indeterminate mode.
	 */
	expectedByteLength?: number,

	/**
	 * Whether to force the progress bar into an indeterminate state (a continuous loop animation).
	 */
	forceIndeterminateProgressBar?: boolean,

	/**
	 * Title of the notification.
	 * 
	 * @remarks
	 * To set a specific title depending on the state, 
	 * use `titleProgress`, `titleCompletion`, and `titleFailure`.
	 *
	 * The following placeholders are supported:
	 * - `"{{fileName}}"`: Name of the file
	 * - `"{{progress}}"`: Formatted number of bytes processed so far (e.g. `"2.2 KB"`, `"100.1 MB"`)
	 * - `"{{progressMax}}"`: Formatted total number of bytes (e.g. `"5.0 MB"`, or `"--"` if `expectedByteLength` is undefined)
	 * - `"{{percentage}}"`: Progress percentage from 0 to 100 (e.g. `"45"`, or `"--"` if `expectedByteLength` is undefined)
	 * 
	 * @see [Notification.Builder.setContentTitle](https://developer.android.com/reference/android/app/Notification.Builder#setContentTitle(java.lang.CharSequence))
	 * @see [Layout of the content title](https://developer.android.com/develop/ui/views/notifications/progress-centric#anatomy)
	 */
	title?: string;

	/**
	 * Title of the notification while the process is in progress.
	 * 
	 * @remarks
	 * The following placeholders are supported:
	 * - `"{{fileName}}"`: Name of the file
	 * - `"{{progress}}"`: Formatted number of bytes processed so far (e.g. `"2.2 KB"`, `"100.1 MB"`)
	 * - `"{{progressMax}}"`: Formatted total number of bytes (e.g. `"5.0 MB"`, or `"--"` if `expectedByteLength` is undefined)
	 * - `"{{percentage}}"`: Progress percentage from 0 to 100 (e.g. `"45"`, or `"--"` if `expectedByteLength` is undefined)
	 * 
	 * @see [Notification.Builder.setContentTitle](https://developer.android.com/reference/android/app/Notification.Builder#setContentTitle(java.lang.CharSequence))
	 * @see [Layout of the content title](https://developer.android.com/develop/ui/views/notifications/progress-centric#anatomy)
	 */
	titleProgress?: string;

	/**
	 * Title of the notification upon successful completion.
	 * 
	 * @remarks
	 * The following placeholders are supported:
	 * - `"{{fileName}}"`: Name of the file
	 * - `"{{progress}}"`: Formatted number of bytes processed so far (e.g. `"2.2 KB"`, `"100.1 MB"`)
	 * - `"{{progressMax}}"`: Formatted total number of bytes (always same as `{{progress}}`)
	 * - `"{{percentage}}"`: Progress percentage from 0 to 100 (always `"100"`)
	 * 
	 * @see [Notification.Builder.setContentTitle](https://developer.android.com/reference/android/app/Notification.Builder#setContentTitle(java.lang.CharSequence))
	 * @see [Layout of the content title](https://developer.android.com/develop/ui/views/notifications/progress-centric#anatomy)
	 */
	titleCompletion?: string;

	/**
	 * Title of the notification when the process fails.
	 * 
	 * @remarks
	 * The following placeholders are supported:
	 * - `"{{fileName}}"`: Name of the file
	 * - `"{{progress}}"`: Formatted number of bytes processed so far (e.g. `"2.2 KB"`, `"100.1 MB"`)
	 * - `"{{progressMax}}"`: Formatted total number of bytes (e.g. `"5.0 MB"`, or `"--"` if `expectedByteLength` is undefined)
	 * - `"{{percentage}}"`: Progress percentage from 0 to 100 (e.g. `"45"`, or `"--"` if `expectedByteLength` is undefined)
	 * 
	 * @see [Notification.Builder.setContentTitle](https://developer.android.com/reference/android/app/Notification.Builder#setContentTitle(java.lang.CharSequence))
	 * @see [Layout of the content title](https://developer.android.com/develop/ui/views/notifications/progress-centric#anatomy)
	 */
	titleFailure?: string;

	/**
	 * Message body of the notification.
	 * 
	 * @remarks
	 * To set a specific message depending on the state, 
	 * use `textProgress`, `textCompletion`, and `textFailure`.
	 * 
	 * The following placeholders are supported:
	 * - `"{{fileName}}"`: Name of the file
	 * - `"{{progress}}"`: Formatted number of bytes processed so far (e.g. `"2.2 KB"`, `"100.1 MB"`)
	 * - `"{{progressMax}}"`: Formatted total number of bytes (e.g. `"5.0 MB"`, or `"--"` if `expectedByteLength` is undefined)
	 * - `"{{percentage}}"`: Progress percentage from 0 to 100 (e.g. `"45"`, or `"--"` if `expectedByteLength` is undefined)
	 * 
	 * @see [Notification.Builder.setContentText](https://developer.android.com/reference/android/app/Notification.Builder#setContentText(java.lang.CharSequence))
	 * @see [Layout of the content text](https://developer.android.com/develop/ui/views/notifications/progress-centric#anatomy)
	 */
	text?: string;

	/**
	 * Message body of the notification while the process is in progress.
	 * 
	 * @remarks
	 * The following placeholders are supported:
	 * - `"{{fileName}}"`: Name of the file
	 * - `"{{progress}}"`: Formatted number of bytes processed so far (e.g. `"2.2 KB"`, `"100.1 MB"`)
	 * - `"{{progressMax}}"`: Formatted total number of bytes (e.g. `"5.0 MB"`, or `"--"` if `expectedByteLength` is undefined)
	 * - `"{{percentage}}"`: Progress percentage from 0 to 100 (e.g. `"45"`, or `"--"` if `expectedByteLength` is undefined)
	 * 
	 * @see [Notification.Builder.setContentText](https://developer.android.com/reference/android/app/Notification.Builder#setContentText(java.lang.CharSequence))
	 * @see [Layout of the content text](https://developer.android.com/develop/ui/views/notifications/progress-centric#anatomy)
	 */
	textProgress?: string;

	/**
	 * Message body of the notification upon successful completion.
	 * 
	 * @remarks
	 * The following placeholders are supported:
	 * - `"{{fileName}}"`: Name of the file
	 * - `"{{progress}}"`: Formatted number of bytes processed so far (e.g. `"2.2 KB"`, `"100.1 MB"`)
	 * - `"{{progressMax}}"`: Formatted total number of bytes (always same as `{{progress}}`)
	 * - `"{{percentage}}"`: Progress percentage from 0 to 100 (always `"100"`)
	 * 
	 * @see [Notification.Builder.setContentText](https://developer.android.com/reference/android/app/Notification.Builder#setContentText(java.lang.CharSequence))
	 * @see [Layout of the content text](https://developer.android.com/develop/ui/views/notifications/progress-centric#anatomy)
	 */
	textCompletion?: string;

	/**
	 * Message body of the notification when the process fails.
	 * 
	 * @remarks
	 * The following placeholders are supported:
	 * - `"{{fileName}}"`: Name of the file
	 * - `"{{progress}}"`: Formatted number of bytes processed so far (e.g. `"2.2 KB"`, `"100.1 MB"`)
	 * - `"{{progressMax}}"`: Formatted total number of bytes (e.g. `"5.0 MB"`, or `"--"` if `expectedByteLength` is undefined)
	 * - `"{{percentage}}"`: Progress percentage from 0 to 100 (e.g. `"45"`, or `"--"` if `expectedByteLength` is undefined)
	 * 
	 * @see [Notification.Builder.setContentText](https://developer.android.com/reference/android/app/Notification.Builder#setContentText(java.lang.CharSequence))
	 * @see [Layout of the content text](https://developer.android.com/develop/ui/views/notifications/progress-centric#anatomy)
	 */
	textFailure?: string;

	/**
	 * Sub text of the notification.
	 * 
	 * @remarks
	 * To set a specific sub text depending on the state, 
	 * use `subTextProgress`, `subTextCompletion`, and `subTextFailure`.
	 * 
	 * The following placeholders are supported:
	 * - `"{{fileName}}"`: Name of the file
	 * - `"{{progress}}"`: Formatted number of bytes processed so far (e.g. `"2.2 KB"`, `"100.1 MB"`)
	 * - `"{{progressMax}}"`: Formatted total number of bytes (e.g. `"5.0 MB"`, or `"--"` if `expectedByteLength` is undefined)
	 * - `"{{percentage}}"`: Progress percentage from 0 to 100 (e.g. `"45"`, or `"--"` if `expectedByteLength` is undefined)
	 * 
	 * @see [Notification.Builder.setSubText](https://developer.android.com/reference/android/app/Notification.Builder#setSubText(java.lang.CharSequence))
	 * @see [Layout of the sub text](https://developer.android.com/develop/ui/views/notifications/progress-centric#anatomy)
	 */
	subText?: string;

	/**
	 * Sub text of the notification while the process is in progress.
	 * 
	 * @remarks
	 * The following placeholders are supported:
	 * - `"{{fileName}}"`: Name of the file
	 * - `"{{progress}}"`: Formatted number of bytes processed so far (e.g. `"2.2 KB"`, `"100.1 MB"`)
	 * - `"{{progressMax}}"`: Formatted total number of bytes (e.g. `"5.0 MB"`, or `"--"` if `expectedByteLength` is undefined)
	 * - `"{{percentage}}"`: Progress percentage from 0 to 100 (e.g. `"45"`, or `"--"` if `expectedByteLength` is undefined)
	 * 
	 * @see [Notification.Builder.setSubText](https://developer.android.com/reference/android/app/Notification.Builder#setSubText(java.lang.CharSequence))
	 * @see [Layout of the sub text](https://developer.android.com/develop/ui/views/notifications/progress-centric#anatomy)
	 */
	subTextProgress?: string;

	/**
	 * Sub text of the notification upon successful completion.
	 * 
	 * The following placeholders are supported:
	 * - `"{{fileName}}"`: Name of the file
	 * - `"{{progress}}"`: Formatted number of bytes processed so far (e.g. `"2.2 KB"`, `"100.1 MB"`)
	 * - `"{{progressMax}}"`: Formatted total number of bytes (always same as `{{progress}}`)
	 * - `"{{percentage}}"`: Progress percentage from 0 to 100 (always `"100"`)
	 * 
	 * @see [Notification.Builder.setSubText](https://developer.android.com/reference/android/app/Notification.Builder#setSubText(java.lang.CharSequence))
	 * @see [Layout of the sub text](https://developer.android.com/develop/ui/views/notifications/progress-centric#anatomy)
	 */
	subTextCompletion?: string;

	/**
	 * Sub text of the notification when the process fails.
	 * 
	 * @remarks
	 * The following placeholders are supported:
	 * - `"{{fileName}}"`: Name of the file
	 * - `"{{progress}}"`: Formatted number of bytes processed so far (e.g. `"2.2 KB"`, `"100.1 MB"`)
	 * - `"{{progressMax}}"`: Formatted total number of bytes (e.g. `"5.0 MB"`, or `"--"` if `expectedByteLength` is undefined)
	 * - `"{{percentage}}"`: Progress percentage from 0 to 100 (e.g. `"45"`, or `"--"` if `expectedByteLength` is undefined)
	 * 
	 * @see [Notification.Builder.setSubText](https://developer.android.com/reference/android/app/Notification.Builder#setSubText(java.lang.CharSequence))
	 * @see [Layout of the sub text](https://developer.android.com/develop/ui/views/notifications/progress-centric#anatomy)
	 */
	subTextFailure?: string;
};


/**
 * Gets the name of a file or directory.  
 * 
 * @remarks
 * Includes the file extension if present.
 *
 * @param uri - URI or path of the target file or directory.
 * 
 * @returns Promise that resolves to the name of the entry.
 * 
 * @throws 
 * The returned Promise rejects with an error in the following cases:
 * - When the entry does not exist.
 * - When the app does not have read permissions for the entry.
 * - When an unexpected error occurs.
 * 
 * @see {@link https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.AndroidFs.html#method.get_name | AndroidFs::get_name}
 */
export async function getName(uri: FsUri | string | URL): Promise<string> {
	return await invoke('plugin:android-fs|get_name', {
		uri: mapFsPathForInput(uri)
	})
}

/**
 * Gets the type of a file or directory.
 *
 * @param uri - URI or path of the target file or directory.
 * 
 * @returns Promise that resolves to the type of the entry. The resolved value will be an object of type `EntryType`, which can be either `{ type: "Dir" }` for directories or `{ type: "File", mimeType: string }` for files.
 * 
 * @throws 
 * The returned Promise rejects with an error in the following cases:
 * - When the entry does not exist.
 * - When the app does not have read permissions for the entry.
 * - When an unexpected error occurs.
 * 
 * @see {@link https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.AndroidFs.html#method.get_type | AndroidFs::get_type}
 */
export async function getType(uri: FsUri | string | URL): Promise<EntryType> {
	return await invoke('plugin:android-fs|get_type', {
		uri: mapFsPathForInput(uri)
	})
}

/**
 * Gets metadata of a file or directory.  
 * 
 * @param uri - URI or path of the target file or directory.
 * 
 * @returns Promise that resolves to metadata of the entry. It includes the type (`"Dir"` or `"File"`), name, last modified date, and for files also byte length and MIME type.
 * 
 * @throws
 * The returned Promise rejects with an error in the following cases:
 * - When the entry does not exist.
 * - When the app does not have read permissions for the entry.
 * - When an unexpected error occurs.
 * 
 * @see {@link https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.AndroidFs.html#method.get_info | AndroidFs::get_info}
 */
export async function getMetadata(uri: FsUri | string | URL): Promise<EntryMetadata> {
	const md = await invoke<EntryMetadataInner>('plugin:android-fs|get_metadata', {
		uri: mapFsPathForInput(uri)
	})

	return mutateEntryMetadata(md)
}

/**
 * Gets the file size in bytes of a file.  
 *
 * @param uri - URI or path of the target file.
 * 
 * @returns Promise that resolves to a non-negative integer representing the file size in bytes.
 *
 * @throws 
 * The returned Promise rejects with an error in the following cases:
 * - When the entry is a directory, not a file.
 * - When the file does not exist.
 * - When the app does not have read permissions for the file.
 * - When an unexpected error occurs.
 * 
 * @see {@link https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.AndroidFs.html#method.get_len | AndroidFs::get_len}
 */
export async function getFileByteLength(uri: FsUri | string | URL): Promise<number> {
	return await invoke('plugin:android-fs|get_file_byte_length', {
		uri: mapFsPathForInput(uri)
	})
}

/**
 * Gets the MIME type of a file.
 *
 * @param uri - URI or path of the target file.
 * 
 * @returns Promise that resolves to the MIME type of the file.
 *
 * @throws 
 * The returned Promise rejects with an error in the following cases:
 * - When the entry is a directory, not a file.
 * - When the file does not exist.
 * - When the app does not have read permissions for the file.
 * - When an unexpected error occurs.
 * 
 * @see {@link https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.AndroidFs.html#method.get_mime_type | AndroidFs::get_mime_type}
 */
export async function getFileMimeType(uri: FsUri | string | URL): Promise<string> {
	return await invoke('plugin:android-fs|get_file_mime_type', {
		uri: mapFsPathForInput(uri)
	})
}

/**
 * Gets the thumbnail of a file as the Data URL.  
 * 
 * @remarks
 * This does not perform caching.
 *
 * @param uri - URI or path of the target file.
 * @param options - Optional settings: `width`, `height`, `format`. See `GetThumbnailOptions` for details.
 * 
 * @returns Promise that resolves to a Data URL string of the thumbnail, or `null` if none exists. The actual dimensions will not exceed approximately twice the specified width or height, while always maintaining the original aspect ratio.
 * 
 * @throws 
 * The returned Promise rejects with an error in the following cases:
 * - When the entry is a directory, not a file.
 * - When the file does not exist.
 * - When the app does not have read permissions for the file.
 * - When an unexpected error occurs.
 * 
 * @see {@link https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.AndroidFs.html#method.get_thumbnail | AndroidFs::get_thumbnail}
 */
export async function getFileThumbnailAsDataURL(
	uri: FsUri | string | URL,
	options?: GetThumbnailOptions
): Promise<string | null> {

	const format: ThumbnailFormat = options?.format ?? "webp"
	const width = options?.width ?? options?.height ?? 512
	const height = options?.height ?? options?.width ?? 512
	const thumbnail = await invoke<ArrayBuffer>('plugin:android-fs|get_file_thumbnail_as_data_url', {
		uri: mapFsPathForInput(uri),
		width,
		height,
		format
	})

	return thumbnail.byteLength === 0 ? null : decodeUtf8(thumbnail)
}

/**
 * Gets the thumbnail of a file as the base64-encoded string.  
 * 
 * @remarks
 * This does not perform caching.
 *
 * @param uri - URI or path of the target file.
 * @param options - Optional settings: `width`, `height`, `format`. See `GetThumbnailOptions` for details.
 * 
 * @returns Promise that resolves to a base64-encoded string of the thumbnail, or `null` if none exists. The actual dimensions will not exceed approximately twice the specified width or height, while always maintaining the original aspect ratio.
 * 
 * @throws 
 * The returned Promise rejects with an error in the following cases:
 * - When the entry is a directory, not a file.
 * - When the file does not exist.
 * - When the app does not have read permissions for the file.
 * - When an unexpected error occurs.
 * 
 * @see {@link https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.AndroidFs.html#method.get_thumbnail | AndroidFs::get_thumbnail}
 */
export async function getFileThumbnailAsBase64(
	uri: FsUri | string | URL,
	options?: GetThumbnailOptions
): Promise<string | null> {

	const format: ThumbnailFormat = options?.format ?? "webp"
	const width = options?.width ?? options?.height ?? 512
	const height = options?.height ?? options?.width ?? 512
	const thumbnail = await invoke<ArrayBuffer>('plugin:android-fs|get_file_thumbnail_as_base64', {
		uri: mapFsPathForInput(uri),
		width,
		height,
		format
	})

	return thumbnail.byteLength === 0 ? null : decodeUtf8(thumbnail)
}

/**
 * Gets the thumbnail of a file as bytes.  
 * 
 * @remarks
 * This does not perform caching.
 *
 * @param uri - URI or path of the target file.
 * @param options - Optional settings: `width`, `height`, `format`. See `GetThumbnailOptions` for details.
 * 
 * @returns Promise that resolves to bytes of the thumbnail, or `null` if none exists. The actual dimensions will not exceed approximately twice the specified width or height, while always maintaining the original aspect ratio.
 * 
 * @throws 
 * The returned Promise rejects with an error in the following cases:
 * - When the entry is a directory, not a file.
 * - When the file does not exist.
 * - When the app does not have read permissions for the file.
 * - When an unexpected error occurs.
 * 
 * @see {@link https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.AndroidFs.html#method.get_thumbnail | AndroidFs::get_thumbnail}
 */
export async function getFileThumbnail(
	uri: FsUri | string | URL,
	options?: GetThumbnailOptions
): Promise<Uint8Array<ArrayBuffer> | null> {

	const format: ThumbnailFormat = options?.format ?? "webp"
	const width = options?.width ?? options?.height ?? 512
	const height = options?.height ?? options?.width ?? 512
	const thumbnail = await invoke<ArrayBuffer>('plugin:android-fs|get_file_thumbnail', {
		uri: mapFsPathForInput(uri),
		width,
		height,
		format
	})

	return thumbnail.byteLength === 0 ? null : new Uint8Array(thumbnail)
}

/**
 * Gets the path usable with {@link https://v2.tauri.app/ja/plugin/file-system/ | @tauri-apps/plugin-fs}.
 * 
 * @remarks
 * Paths **derived from this plugin's URI** are supported only for reading and writing files.
 * No guarantees are provided for other operations or directory handling.
 * For these paths, you do not need to configure {@link https://v2.tauri.app/reference/javascript/fs/#security | the scope configuration} of the Tauri file system.
 * 
 * Caution is required when using `@tauri-apps/plugin-fs` for file operations. 
 * Writing files can occasionally be very slow. 
 * Additionally, files obtained from third-party apps via a file picker may not be openable, readable, writable, or seekable.  
 * 
 * Therefore, it is strongly recommended to use the dedicated APIs provided by this plugin instead.
 * 
 * @param uri - URI or path of the target file or directory.
 * @returns A string of the path. Note that while it is called a "path", it may actually be a URI that is compatible with `@tauri-apps/plugin-fs`.
 */
export function getFsPath(uri: FsUri | string | URL): string {
	if (isFsUri(uri)) {
		return uri.uri
	}
	if (typeof uri === "string") {
		return uri
	}
	return uri.toString()
}

/**
 * Constructs a URI for a file at the specified absolute path.
 *
 * @remarks
 * This is Available on Android 8 (API level 26) or higher.
 *
 * This function only constructs a URI. To actually access the file
 * using the returned URI, the file provider feature must be enabled
 * and configured as described below.
 *
 * ### 1. Enable the file provider feature
 * Enable the `file_provider` feature. 
 * 
 * `src-tauri/Cargo.toml`:
 * ```toml
 * [dependencies]
 * tauri-plugin-android-fs = { features = ["file_provider"], ... }
 * ```
 *
 * ### 2. Configure the file provider
 * Configure the file provider to allow the file to be loaded
 * and specify its scope as with other APIs.
 * 
 * `src-tauri/tauri.conf.json`:
 * ```json
 * {
 *   "plugins": {
 *     "android-fs": {
 *       "fileProvider": {
 *         "enable": true,
 *         "scope": {
 *           "allow": [
 *             "$APPDATA/my-data/shared/*",
 *             "$APPDATA/my-data/shared/document/thumbnail/*"
 *           ]
 *         }
 *       }
 *     }
 *   }
 * }
 * ```
 *
 * NOTE:
 * Ensure that `serde_json` is present in your Rust dependencies. 
 * It is included by default in Tauri project templates, but if it has been removed, add it back.
 * If it is missing, the project will fail to build.
 *
 * @param path - Absolute path of the target file.
 * @param options - Optional settings: `mimeType`. See `GetUriForFilePathOptions` for details.
 * 
 * @returns Promise that resolves to a URI of the file. The URI remains valid until the current application process terminates, after which it can no longer be accessed. The URI is an Android Content URI and can be shared with other applications using `showShareFileAppChooser` and etc.
 * @throws The returned Promise rejects with an error if Android 7 or lower.
 */
export async function getUriForFilePath(
	path: string,
	options?: GetUriForFilePathOptions
): Promise<FsUri> {

	const mimeType = options?.mimeType ?? null
	return await invoke("plugin:android-fs|get_uri_for_file_path", { path, mimeType })
}

/**
 * Converts a file URI into a URL that can be used to load thumbnails in HTML `<img>` elements.
 * 
 * @remarks
 * This is backed by Tauri’s custom protocol.
 * 
 * ## Setup
 * This function only constructs a URL.  
 * To actually load a file using the returned URL, follow the steps below.
 * 
 * #### 1. Enable protocol feature
 * Enable protocol_thumbnail feature.
 * 
 * `src-tauri/Cargo.toml`
 * ```toml
 * [dependencies]
 * tauri-plugin-android-fs = { features = ["protocol_thumbnail"], ... }
 * ```
 * 
 * #### 2. Configuration
 * Set the configuration to allow files to be loaded.
 * If you are using absolute paths, you must configure the scope as with other APIs.
 * 
 * `src-tauri/tauri.conf.json`
 * ```json
 * {
 *   "plugins": {
 *     "android-fs": {
 *       "thumbnailProtocol": {
 *         "enable": true,
 *         "scope": {
 *           "allow": [
 *             "$APPDATA/my-data/*",
 *             "$APPDATA/my-data/shared/image/*"
 *           ]
 *         }
 *       }
 *     }
 *   }
 * }
 * ```
 * 
 * NOTE:
 * Ensure that `serde_json` is present in your Rust dependencies.  
 * It is included by default in Tauri project templates, but if it has been removed, add it back.  
 * If it is missing, the project will fail to build.
 * 
 * #### 3. Content Security Policy (CSP)
 * If you are using a CSP, 
 * add `http://android-fs-thumbnail.localhost` to {@link https://v2.tauri.app/reference/config/#csp-1 | app.security.csp} in `src-tauri/tauri.conf.json`.
 * 
 * @param uri - URI or path of the target file.
 * @param options - Optional settings: `width`, `height`, `format`. See `ConvertThumbnailSrcOptions` for details.
 */
export function convertFileThumbnailSrc(
	uri: FsUri | string | URL,
	options?: ConvertThumbnailSrcOptions,
): string {

	let srcUrl = convertCustomProtocolSrc(
		JSON.stringify(mapFsPathForInput(uri)),
		"android-fs-thumbnail"
	)

	let sep = "?"
	if (options?.width != null) {
		srcUrl += sep + "w=" + options.width
		sep = "&"
	}
	if (options?.height != null) {
		srcUrl += sep + "h=" + options.height
		sep = "&"
	}
	if (options?.format != null) {
		srcUrl += sep + "f=" + options?.format
		sep = "&"
	}

	return srcUrl
}

/**
 * Converts a file URI into a URL that can be loaded by HTML `<img>`, `<video>`, and `<audio>` elements.
 * 
 * @remarks
 * This is backed by Tauri’s custom protocol.
 * 
 * ## Known Issues (As of October 5, 2026)
 * File loading may fail. 
 * This issue occurs frequently with `<video>` and `<audio>` elements, but is not limited to them, 
 * because Tauri’s custom protocol currently {@link https://github.com/tauri-apps/tauri/issues/12019 | cannot handle range requests on Android}.
 * As a workaround, 
 * you can host your own server on the Tauri backend 
 * or use {@link https://www.npmjs.com/package/tauri-plugin-android-player-api?activeTab=readme | tauri_plugin_android_player} to use Android native player.
 * 
 * ## Setup
 * This function only constructs a URL.  
 * To actually load a file using the returned URL, follow the steps below.
 * 
 * #### 1. Enable protocol feature
 * Enable protocol_content feature.
 * 
 * `src-tauri/Cargo.toml`
 * ```toml
 * [dependencies]
 * tauri-plugin-android-fs = { features = ["protocol_content"], ... }
 * ```
 * 
 * #### 2. Configuration
 * Set the configuration to allow files to be loaded.
 * If you are using absolute paths, you must configure the scope as with other APIs.
 * 
 * `src-tauri/tauri.conf.json`
 * ```json
 * {
 *   "plugins": {
 *     "android-fs": {
 *       "contentProtocol": {
 *         "enable": true,
 *         "scope": {
 *           "allow": [
 *             "$APPDATA/my-data/*",
 *             "$APPDATA/my-data/shared/image/*"
 *           ]
 *         }
 *       }
 *     }
 *   }
 * }
 * ```
 * 
 * NOTE:
 * Ensure that `serde_json` is present in your Rust dependencies.  
 * It is included by default in Tauri project templates, but if it has been removed, add it back.  
 * If it is missing, the project will fail to build.
 * 
 * #### 3. Content Security Policy (CSP)
 * If you are using a CSP, 
 * add `http://android-fs-content.localhost` to {@link https://v2.tauri.app/reference/config/#csp-1 | app.security.csp} in `src-tauri/tauri.conf.json`.
 * 
 * @param uri - URI or path of the target file.
 */
export function convertFileSrc(uri: FsUri | string | URL): string {
	return convertCustomProtocolSrc(
		JSON.stringify(mapFsPathForInput(uri)),
		"android-fs-content"
	)
}

/**
 * Retrieves available storage volumes (e.g., internal storage, SD cards, or USB drives).
 * 
 * @param options - Optional settings: `purpose`. See `ListVolumesOptions` for details.
 * 
 * @returns Promise that resolves to an array of the storage volumes. 
 * 
 * @see {@link https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.AndroidFs.html#method.get_volumes | AndroidFs::get_volumes}
 */
export async function listVolumes(
	options?: ListVolumesOptions
): Promise<StorageVolumeMetadata[]> {

	const purpose = options?.purpose
	const volumes = await invoke<StorageVolumeMetadata[]>('plugin:android-fs|list_volumes')

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
 * Requests permission from the user to create public files and access them.  
 * 
 * @remarks
 * This is intended for `createNewPublicFile` and its related functions. 
 * However, since those functions automatically request permission by default, calling this method explicitly is usually unnecessary.
 * 
 * @returns Promise that resolves to a boolean indicating whether the app is allowed to create files in public storage and access them.
 * 
 * @see {@link https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.PublicStorage.html#method.request_permission | PublicStorage::request_permission}
 */
export async function requestPublicFilesPermission(): Promise<boolean> {
	return await invoke('plugin:android-fs|request_public_files_permission')
}

/**
 * Checks whether the app has permission to create public files and access them.
 * 
 * @remarks
 * The app can request this permission explicitly using `requestPublicFilesPermission`.
 * 
 * @returns Promise that resolves to a boolean indicating whether the app is allowed to create files in public storage and access them.
 * 
 * @see {@link https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.PublicStorage.html#method.check_permission | PublicStorage::check_permission}
 */
export async function checkPublicFilesPermission(): Promise<boolean> {
	return await invoke('plugin:android-fs|check_public_files_permission')
}

/**
 * Triggers the Android MediaScanner to scan a public file,
 * making it visible in media applications (e.g., Gallery, Music player).
 * 
 * @param uri - URI of the file to be scanned.  
 * @returns Promise that resolves when the scan request has been successfully initiated.
 * 
 * @throws
 * The returned Promise rejects with an error in the following cases:
 * - When the entry is a directory, not a file.
 * - When the file does not exist.
 * - When a provider of the file is not the Android MediaStore.
 * - When the app does not have read/write permissions for the file.
 * - When an unexpected error occurs.
 * 
 * @see {@link https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.PublicStorage.html#method.scan | PublicStorage::scan}
 */
export async function scanPublicFile(uri: FsUri): Promise<void> {
	return await invoke('plugin:android-fs|scan_public_file', {
		uri,
	})
}

/**
 * Specifies whether a public file is marked as pending.  
 * 
 * @remarks
 * This is available for Android 11 (API level 30) or higher.  
 * If unavailable, this does nothing. 
 * 
 * @param uri - URI of the target file.  
 * @param isPending - Indicates whether the file is pending. When `true`, the app has exclusive access and the file is invisible to others. Files pending for over 7 days are automatically deleted by the Android system.
 * 
 * @returns Promise that resolves when the operation is completed.
 * 
 * @throws
 * The returned Promise rejects with an error in the following cases:
 * - When the entry is a directory, not a file.
 * - When the file does not exist.
 * - When a provider of the file is not the Android MediaStore.
 * - When the app does not have read/write permissions for the file.
 * - When an unexpected error occurs.
 * 
 * @see {@link https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.PublicStorage.html#method.set_pending | PublicStorage::set_pending}
 */
export async function setPublicFilePending(
	uri: FsUri,
	isPending: boolean
): Promise<void> {

	await invoke('plugin:android-fs|set_public_file_pending', {
		uri,
		isPending
	})
}

/**
 * Creates a new empty file in a public directory
 * 
 * @param baseDir - Base directory in which to create the new file. One of: `"Documents"`, `"Download"`.
 * @param relativePath - Relative path from the base directory. If a file with the same name already exists, a sequential number is appended to ensure uniqueness. Any missing directories in this path will be created recursively.
 * @param mimeType - MIME type of the file to create. If `null`, it is inferred from the extension of `relativePath`.
 * @param options - Optional settings: `requestPermission`, `volumeId`, `isPending`. See `CreateNewPublicFileOptions` for details.
 * 
 * @returns Promise that resolves to a URI of the created file, with persisted read/write permissions that depend on `checkPublicFilesPermission`.
 * 
 * @throws 
 * The returned Promise rejects with an error in the following cases:
 * - When the storage volume is currently unavailable
 * - When the app does not have read/write permissions for the public files.
 * - When an unexpected error occurs.
 * 
 * @see {@link https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.PublicStorage.html#method.create_new_file | PublicStorage::create_new_file}
 */
export async function createNewPublicFile(
	baseDir: PublicGeneralPurposeDir,
	relativePath: string,
	mimeType: string | null,
	options?: CreateNewPublicFileOptions
): Promise<FsUri> {

	const requestPermission: boolean = options?.requestPermission ?? true
	const volumeId: StorageVolumeId | null = options?.volumeId ?? null
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
 * Creates a new empty image file in a public directory
 * 
 * @param baseDir - Base directory in which to create the new file. One of: `"Pictures"`, `"DCIM"`, `"Documents"`, `"Download"`.
 * @param relativePath - Relative path from the base directory. If a file with the same name already exists, a sequential number is appended to ensure uniqueness. Any missing directories in this path will be created recursively.
 * @param mimeType - MIME type of the file to create. If `null`, it is inferred from the extension of `relativePath`.
 * @param options - Optional settings: `requestPermission`, `volumeId`, `isPending`. See `CreateNewPublicFileOptions` for details.
 * 
 * @returns Promise that resolves to a URI of the created file, with persisted read/write permissions that depend on `checkPublicFilesPermission`.
 * 
 * @throws 
 * The returned Promise rejects with an error in the following cases:
 * - When the storage volume is currently unavailable
 * - When the app does not have read/write permissions for the public files.
 * - When the MIME type is not an image type.
 * - When an unexpected error occurs.
 * 
 * @see {@link https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.PublicStorage.html#method.create_new_file | PublicStorage::create_new_file}
 */
export async function createNewPublicImageFile(
	baseDir: PublicImageDir | PublicGeneralPurposeDir,
	relativePath: string,
	mimeType: string | null,
	options?: CreateNewPublicFileOptions
): Promise<FsUri> {

	const requestPermission: boolean = options?.requestPermission ?? true
	const volumeId: StorageVolumeId | null = options?.volumeId ?? null
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
 * Creates a new empty video file in a public directory
 * 
 * @param baseDir - Base directory in which to create the new file. One of: `"Movies"`, `"DCIM"`, `"Documents"`, `"Download"`.
 * @param relativePath - Relative path from the base directory. If a file with the same name already exists, a sequential number is appended to ensure uniqueness. Any missing directories in this path will be created recursively.
 * @param mimeType - MIME type of the file to create. If `null`, it is inferred from the extension of `relativePath`.
 * @param options - Optional settings: `requestPermission`, `volumeId`, `isPending`. See `CreateNewPublicFileOptions` for details.
 * 
 * @returns Promise that resolves to a URI of the created file, with persisted read/write permissions that depend on `checkPublicFilesPermission`.
 * 
 * @throws 
 * The returned Promise rejects with an error in the following cases:
 * - When the storage volume is currently unavailable
 * - When the app does not have read/write permissions for the public files.
 * - When the MIME type is not a video type.
 * - When an unexpected error occurs.
 * 
 * @see {@link https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.PublicStorage.html#method.create_new_file | PublicStorage::create_new_file}
 */
export async function createNewPublicVideoFile(
	baseDir: PublicVideoDir | PublicGeneralPurposeDir,
	relativePath: string,
	mimeType: string | null,
	options?: CreateNewPublicFileOptions
): Promise<FsUri> {

	const requestPermission: boolean = options?.requestPermission ?? true
	const volumeId: StorageVolumeId | null = options?.volumeId ?? null
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
 * Creates a new empty audio file in a public directory.
 * 
 * @param baseDir - Base directory in which to create the new file. One of: `"Music"`, `"Alarms"`, `"Audiobooks"`, `"Notifications"`, `"Podcasts"`, `"Ringtones"`, `"Recordings"`, `"Documents"`, `"Download"`.
 * @param relativePath - Relative path from the base directory. If a file with the same name already exists, a sequential number is appended to ensure uniqueness. Any missing directories in this path will be created recursively.
 * @param mimeType - MIME type of the file to create. If `null`, it is inferred from the extension of `relativePath`.
 * @param options - Optional settings: `requestPermission`, `volumeId`, `isPending`. See `CreateNewPublicFileOptions` for details.
 * 
 * @returns Promise that resolves to a URI of the created file, with persisted read/write permissions that depend on `checkPublicFilesPermission`.
 * 
 * @throws 
 * The returned Promise rejects with an error in the following cases:
 * - When the storage volume is currently unavailable
 * - When the app does not have read/write permissions for the public files.
 * - When the MIME type is not an audio type.
 * - When an unexpected error occurs.
 * 
 * @see {@link https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.PublicStorage.html#method.create_new_file | PublicStorage::create_new_file}
 */
export async function createNewPublicAudioFile(
	baseDir: PublicAudioDir | PublicGeneralPurposeDir,
	relativePath: string,
	mimeType: string | null,
	options?: CreateNewPublicFileOptions
): Promise<FsUri> {

	const requestPermission: boolean = options?.requestPermission ?? true
	const volumeId: StorageVolumeId | null = options?.volumeId ?? null
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
 * Creates a new empty file in a directory.  
 * 
 * @param baseDirUri - URI of the base directory in which to create the new file. 
 * @param relativePath - Relative path from the base directory. If an entry with the same name already exists, a sequential number is appended to ensure uniqueness. Any missing parent directories in this path will be created recursively.
 * @param mimeType - MIME type of the file to create. If `null`, it is inferred from the extension of `relativePath`.
 * 
 * @returns Promise that resolves to a URI of the created file, with permissions that depend on the base directory.
 * 
 * @throws 
 * The returned Promise rejects with an error in the following cases:
 * - When the base directory is a file, not a directory.
 * - When the base directory does not exist.
 * - When the app does not have read/write permissions for the base directory.
 * - When a provider of the base directory via Directory Picker does not support the create-file or read-directory operations.
 * - When an unexpected error occurs.
 * 
 * @see {@link https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.AndroidFs.html#method.create_new_file | AndroidFs::create_new_file}
 */
export async function createNewFile(
	baseDirUri: FsUri,
	relativePath: string,
	mimeType: string | null
): Promise<FsUri> {

	return await invoke('plugin:android-fs|create_new_file', {
		baseDirUri,
		relativePath,
		mimeType,
	})
}

/**
 * Creates a new directory in a directory.  
 * 
 * @param baseDirUri - URI of the base directory in which to create the new directory. 
 * @param relativePath - Relative path from the base directory. If an entry with the same name already exists, a sequential number is appended to ensure uniqueness. Any missing parent directories in this path will be created recursively.
 * @param mimeType - MIME type of the directory to create. If `null`, it is inferred from the extension of `relativePath`.
 * 
 * @returns Promise that resolves to a URI of the created directory, with permissions that depend on the base directory.
 * 
 * @throws 
 * The returned Promise rejects with an error in the following cases:
 * - When the base directory is a file, not a directory.
 * - When the base directory does not exist.
 * - When the app does not have read/write permissions for the base directory.
 * - When a provider of the base directory via Directory Picker does not support the create-directory or read-directory operations.
 * - When an unexpected error occurs.
 * 
 * @see {@link https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.AndroidFs.html#method.create_new_dir | AndroidFs::create_new_dir}
*/
export async function createNewDir(
	baseDirUri: FsUri,
	relativePath: string,
): Promise<FsUri> {

	return await invoke('plugin:android-fs|create_new_dir', {
		baseDirUri,
		relativePath,
	})
}

/**
 * Creates a new directory in a directory if missing.
 * 
 * @param baseDirUri - URI of the base directory in which to create the directory. 
 * @param relativePath - Relative path from the base directory. Any missing parent directories in this path will be created recursively.
 * 
 * @returns Promise that resolves to a URI of the created or existing directory. The permissions depend on the base directory.
 * 
 * @throws 
 * The returned Promise rejects with an error in the following cases:
 * - When the base directory is a file, not a directory.
 * - When the base directory does not exist.
 * - When the app does not have read/write permissions for the base directory.
 * - When a provider of the base directory via Directory Picker does not support the create-directory or read-directory operations.
 * - When an unexpected error occurs.
 * 
 * @see {@link https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.AndroidFs.html#method.create_dir_all | AndroidFs::create_dir_all}
 */
export async function createDir(
	baseDirUri: FsUri,
	relativePath: string,
): Promise<FsUri> {

	return await invoke('plugin:android-fs|create_dir', {
		baseDirUri,
		relativePath,
	})
}

/**
 * Opens a file in read mode and resolves to a {@link https://developer.mozilla.org/en-US/docs/Web/API/ReadableStream | ReadableStream}.
 * 
 * @remarks
 * The caller is responsible for releasing the returned stream.
 * The stream is released in the following cases:
 * - When the stream or its reader is canceled. 
 * - When all data has been successfully read from the stream.
 * - When a read operation fails with an error. 
 * - When an abort event is received from the provided `AbortSignal`.
 * - When `closeAllFileStreams` is called.
 * 
 * The stream provides a {@link https://developer.mozilla.org/en-US/docs/Web/API/ReadableStreamDefaultReader | ReadableStreamDefaultReader}
 * and a {@link https://developer.mozilla.org/en-US/docs/Web/API/ReadableStreamBYOBReader | ReadableStreamBYOBReader}.
 * 
 * There is also {@link https://crates.io/crates/tauri-plugin-fs-stream | tauri-plugin-fs-stream}, which provides APIs not only for Android but for all platforms.
 * 
 * @param uri - URI or path of the file to read. 
 * @param options - Optional settings: `bufferByteLength`, `signal`. See `OpenReadFileStreamOptions` for details.
 * 
 * @returns Promise that resolves to a `ReadableStream<Uint8Array<ArrayBuffer>>` backed by the file opened in read-only mode. This stream maintains a one-to-one correspondence with the file descriptor.
 * 
 * @throws
 * The returned Promise rejects with an error in the following cases:
 * - When the entry is a directory, not a file.
 * - When the file does not exist.
 * - When the app does not have read permissions for the file.
 * - When an unexpected error occurs.
 * 
 * @see {@link https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.AndroidFs.html#method.open_file_readable | AndroidFs::open_file_readable}
 */
export async function openReadFileStream(
	uri: FsUri | string | URL,
	options?: OpenReadFileStreamOptions
): Promise<ReadableStream<Uint8Array<ArrayBuffer>>> {

	throwIfAborted(options?.signal)
	const bufferByteLength = mapBufferByteLengthForInput(options?.bufferByteLength)
	const { open, read, close } = resolveCmdReadFileStream("plugin:android-fs|open_read_file_stream")

	try {
		await open({
			uri: mapFsPathForInput(uri)
		})

		return createReadableStream(
			{
				read: () => read(bufferByteLength),
				release: () => close()
			},
			{ signal: options?.signal }
		)
	}
	catch (e) {
		await close().catch(() => { })
		throw e
	}
}

/**
 * Opens a file in read mode and resolves to a {@link https://developer.mozilla.org/en-US/docs/Web/API/ReadableStream | ReadableStream} of text lines.
 * 
 * @remarks
 * The returned stream yields decoded text line by line.
 * For the structure of each item, see `OpenReadTextFileLinesStreamItem`.
 * 
 * The caller is responsible for releasing the stream.
 * The stream is released in the following cases:
 * - When the stream or its reader is canceled. 
 * - When all data has been successfully read from the stream.
 * - When a read operation fails with an error. 
 * - When an abort event is received from the provided `AbortSignal`.
 * - When `closeAllFileStreams` is called.
 * 
 * There is also {@link https://crates.io/crates/tauri-plugin-fs-stream | tauri-plugin-fs-stream}, which provides APIs not only for Android but for all platforms.
 * 
 * @param uri - URI or path of the file to read. 
 * @param options - Optional settings: `encoding`, `fatal`, `ignoreBOM`, `maxLineByteLength`, `bufferByteLength`, `signal`. See `OpenReadTextFileLinesStreamOptions` for details.
 * 
 * @returns Promise that resolves to a `ReadableStream<OpenReadTextFileLinesStreamItem>` backed by the file opened in read-only mode. This stream maintains a one-to-one correspondence with the file descriptor.
 *
 * @throws 
 * The returned Promise rejects with an error in the following cases:
 * - When the entry is a directory, not a file.
 * - When the file does not exist.
 * - When the app does not have read permissions for the file.
 * - When an unexpected error occurs.
 * 
 * @see {@link https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.AndroidFs.html#method.open_file_readable | AndroidFs::open_file_readable}
 */
export async function openReadTextFileLinesStream(
	uri: FsUri | string | URL,
	options?: OpenReadTextFileLinesStreamOptions,
): Promise<ReadableStream<OpenReadTextFileLinesStreamItem>> {

	throwIfAborted(options?.signal)
	const maxLineByteLength = mapMaxLineByteLengthForInput(options?.maxLineByteLength)
	const bufferByteLength = mapBufferByteLengthForInput(options?.bufferByteLength)
	const label = mapEncodingLabelForInput(options?.encoding)
	const fatal = options?.fatal ?? false
	const ignoreBOM = options?.ignoreBOM ?? false
	const { open, read, close } = resolveCmdReadFileStream("plugin:android-fs|open_read_text_file_lines_stream")

	try {
		await open({
			uri: mapFsPathForInput(uri),
			label,
			maxLineByteLength,
			ignoreBOM
		})

		return createTextLinesReadableStream(
			{
				read: () => read(bufferByteLength),
				release: close
			},
			{ label, fatal },
			options?.signal
		)
	}
	catch (e) {
		await close().catch(() => { })
		throw e
	}
}

/**
 * Opens a file in write mode and resolves to a {@link https://developer.mozilla.org/en-US/docs/Web/API/WritableStream | WritableStream}.  
 * 
 * @remarks
 * The caller is responsible for releasing the returned stream.
 * The stream is released in the following cases:
 * - When the stream or its writer is closed.
 * - When the stream or its writer is aborted.
 * - When a write operation fails with an error.
 * - When an abort event is received from the provided `AbortSignal`.
 * - When `closeAllFileStreams` is called.
 * 
 * There is also {@link https://crates.io/crates/tauri-plugin-fs-stream | tauri-plugin-fs-stream}, which provides APIs not only for Android but for all platforms.
 * 
 * @param uri - URI or path of the file to be written.
 * @param options - Optional settings: `append`, `create`, `bufferByteLength`, `signal`, `notification`. See `OpenWriteFileStreamOptions` for details.
 * 
 * @returns Promise that resolves to a `WritableStream<Uint8Array<ArrayBufferLike>>` backed by the file opened in write mode. This stream maintains a one-to-one correspondence with the file descriptor.
 *
 * @throws
 * The returned Promise rejects with an error in the following cases:
 * - When the entry is a directory, not a file.
 * - When the app does not have write permissions for the file.
 * - When `options.create` is `false` or a URI is specified, and the file does not exist.
 * - When `options.append` is `true`, and a provider of the file via File/Directory Picker does not support append mode.
 * - When an unexpected error occurs.
 * 
 * @see {@link https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.AndroidFs.html#method.open_file_writable | AndroidFs::open_file_writable}
 * @see {@link https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.AndroidFs.html#method.open_file | AndroidFs::open_file}
 * @see {@link https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/enum.FileAccessMode.html#variant.WriteAppend | AndroidFs::FileAccessMode::WriteAppend}
 */
export async function openWriteFileStream(
	uri: FsUri | string | URL,
	options?: OpenWriteFileStreamOptions
): Promise<WritableStream<Uint8Array<ArrayBufferLike>>> {

	throwIfAborted(options?.signal)
	const bufferByteLength = mapBufferByteLengthForInput(options?.bufferByteLength)
	const { open, write, close } = resolveCmdWriteFileStream("plugin:android-fs|open_write_file_stream")

	try {
		await open({
			uri: mapFsPathForInput(uri),
			append: options?.append ?? false,
			create: options?.create ?? true,
			notification: options?.notification ?? null
		})

		return createWritableStream(
			{
				write,
				release: (t) => close(t === "Close" ? "Ok" : "Err")
			},
			{
				signal: options?.signal,
				bufferSize: bufferByteLength,
				strictBufferSize: false,
				useBufferView: true,
			}
		)
	}
	catch (e) {
		await close("Err").catch(() => { })
		throw e
	}
}

/**
 * Forcibly disposes of all file streams.
 *
 * @remarks
 * All backend file resources owned by stream instances
 * created by this plugin are disconnected from the frontend and released.
 * 
 * After this operation,
 * any read or write attempts on existing streams will result in an error, 
 * except for buffering in the frontend
 * 
 * This affects streams created by the following methods:
 * - `openReadFileStream`
 * - `openReadTextFileLinesStream`
 * - `openWriteFileStream`
 * 
 * This is intended for debugging or testing. 
 * Instead, use the stream's APIs or abort the associated {@link https://developer.mozilla.org/en-US/docs/Web/API/AbortController | AbortController}.
 * 
 * @returns Promise that resolves when the operation completes successfully.
 */
export async function closeAllFileStreams(): Promise<void> {
	await invoke("plugin:android-fs|close_all_file_streams")
}

/**
 * Retrieves the number of currently active file streams.
 * 
 * @remarks
 * This counts all backend file resources owned by stream instances created by this plugin
 * that have not yet been disconnected from the frontend and released.
 * 
 * This applies to streams created by the following methods:
 * - `openReadFileStream`
 * - `openReadTextFileLinesStream`
 * - `openWriteFileStream`
 * 
 * @returns Promise that resolves to a number of currently active file streams.
 */
export async function countAllFileStreams(): Promise<number> {
	return await invoke("plugin:android-fs|count_all_file_streams")
}

/**
 * Reads the entire file contents as raw bytes.
 *
 * @param uri - URI or path of the target file.
 *
 * @returns Promise that resolves to a `Uint8Array<ArrayBuffer>` containing the entire file contents.
 * 
 * @throws 
 * The returned Promise rejects with an error in the following cases:
 * - When the entry is a directory, not a file.
 * - When the file does not exist.
 * - When the app does not have read permissions for the file.
 * - When an unexpected error occurs.
 *
 * @see {@link https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.AndroidFs.html#method.read_file | AndroidFs::read_file}
 */
export async function readFile(
	uri: FsUri | string | URL,
): Promise<Uint8Array<ArrayBuffer>> {

	const bytes = await invoke<ArrayBuffer>('plugin:android-fs|read_file', {
		uri: mapFsPathForInput(uri),
	})

	return new Uint8Array(bytes)
}

/**
 * Reads the entire file contents as a {@link https://developer.mozilla.org/en-US/docs/Glossary/Base64 | Base64-encoded string}.
 *
 * @param uri - URI or path of the target file.
 *
 * @returns Promise that resolves to a Base64-encoded string representing the entire file contents.
 * 
 * @throws 
 * The returned Promise rejects with an error in the following cases:
 * - When the entry is a directory, not a file.
 * - When the file does not exist.
 * - When the app does not have read permissions for the file.
 * - When an unexpected error occurs.
 *
 * @see {@link https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.AndroidFs.html#method.read_file | AndroidFs::read_file}
 */
export async function readFileAsBase64(
	uri: FsUri | string | URL,
): Promise<string> {

	const base64 = await invoke<ArrayBuffer>('plugin:android-fs|read_file_as_base64', {
		uri: mapFsPathForInput(uri),
	})

	return decodeUtf8(base64)
}

/**
 * Reads the entire file contents as a {@link https://developer.mozilla.org/en-US/docs/Web/URI/Reference/Schemes/data | Data URL}.
 *
 * @param uri - URI or path of the target file.
 * @param options - Optional settings: `mimeType`. See `ReadFileAsDataUrlOptions` for details.
 * 
 * @returns Promise that resolves to a Data URL string representing the entire file contents.
 * 
 * @throws 
 * The returned Promise rejects with an error in the following cases:
 * - When the entry is a directory, not a file.
 * - When the file does not exist.
 * - When the app does not have read permissions for the file.
 * - When an unexpected error occurs.
 *
 * @see {@link https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.AndroidFs.html#method.read_file | AndroidFs::read_file}
 * @see {@link https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.AndroidFs.html#method.get_mime_type | AndroidFs::get_mime_type}
 */
export async function readFileAsDataURL(
	uri: FsUri | string | URL,
	options?: ReadFileAsDataUrlOptions
): Promise<string> {

	const mimeType = options?.mimeType ?? null
	const dataUrl = await invoke<ArrayBuffer>('plugin:android-fs|read_file_as_data_url', {
		uri: mapFsPathForInput(uri),
		mimeType
	})

	return decodeUtf8(dataUrl)
}

/**
 * Reads the entire contents of a file and decodes it as text.
 * 
 * @param uri - URI or file path of the target file.
 * @param options - Optional settings: `encoding`, `fatal`, `ignoreBOM`. See `ReadTextFileOptions` for details.
 * 
 * @returns Promise that resolves to a decoded text representing the entire file contents.
 * 
 * @throws 
 * The returned Promise rejects with an error in the following cases:
 * - When the entry is a directory, not a file.
 * - When the file does not exist.
 * - When the app does not have read permissions for the file.
 * - When `options.fatal` is `true`, and the file contains an invalid byte sequence for `options.encoding`.
 * - When an unexpected error occurs.
 *
 * @see {@link https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.AndroidFs.html#method.open_file_readable | AndroidFs::open_file_readable}
 * @see {@link https://developer.mozilla.org/en-US/docs/Web/API/TextDecoder | WebAPI TextDecoder}
 */
export async function readTextFile(
	uri: FsUri | string | URL,
	options?: ReadTextFileOptions
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
 * Writes data to a file.   
 * 
 * @param uri - URI or path of the file to write to. 
 * @param data - Bytes to write.
 * @param options - Optional settings: `append`, `create`, `notification`. See `WriteFileOptions` for details.
 * 
 * @returns Promise that resolves when the operation completes successfully.
 * 
 * @throws
 * The returned Promise rejects with an error in the following cases:
 * - When the entry is a directory, not a file.
 * - When the app does not have write permissions for the file.
 * - When `options.create` is `false` or a URI is specified, and the file does not exist.
 * - When `options.append` is `true`, and a provider of the file via File/Directory Picker does not support append mode.
 * - When an unexpected error occurs.
 * 
 * @see {@link https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.AndroidFs.html#method.open_file_writable | AndroidFs::open_file_writable}
 * @see {@link https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.AndroidFs.html#method.open_file | AndroidFs::open_file}
 * @see {@link https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/enum.FileAccessMode.html#variant.WriteAppend | AndroidFs::FileAccessMode::WriteAppend}
 */
export async function writeFile(
	uri: FsUri | string | URL,
	data: Uint8Array<ArrayBufferLike>,
	options?: WriteFileOptions
): Promise<void> {

	const n = options?.notification
	const notification = n != null ? { ...n, forceIndeterminateProgressBar: true } : null
	const { open, write, close } = resolveCmdWriteFileStream("plugin:android-fs|write_file")

	try {
		await open({
			uri: mapFsPathForInput(uri),
			append: options?.append ?? false,
			create: options?.create ?? true,
			notification
		})
		await write(data)
		await close("Ok")
	}
	catch (e) {
		await close("Err").catch(() => { })
		throw e
	}
}

/**
 * Writes text data to a file as UTF-8.      
 * 
 * @param uri - URI or path of the file to write to. 
 * @param data - String to write.
 * @param options - Optional settings: `append`, `create`, `notification`. See `WriteTextFileOptions` for details.
 * 
 * @returns Promise that resolves when the operation completes successfully.
 * 
 * @throws
 * The returned Promise rejects with an error in the following cases:
 * - When the entry is a directory, not a file.
 * - When the app does not have write permissions for the file.
 * - When `options.create` is `false` or a URI is specified, and the file does not exist.
 * - When `options.append` is `true`, and a provider of the file via File/Directory Picker does not support append mode.
 * - When an unexpected error occurs.
 * 
 * @see {@link https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.AndroidFs.html#method.open_file_writable | AndroidFs::open_file_writable}
 * @see {@link https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.AndroidFs.html#method.open_file | AndroidFs::open_file}
 * @see {@link https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/enum.FileAccessMode.html#variant.WriteAppend | AndroidFs::FileAccessMode::WriteAppend}
 * @see {@link https://developer.mozilla.org/en-us/docs/Web/API/TextEncoder | WebAPI TextEncoder}
 */
export async function writeTextFile(
	uri: FsUri | string | URL,
	data: string,
	options?: WriteTextFileOptions
): Promise<void> {

	const n = options?.notification
	const notification = n != null ? { ...n, forceIndeterminateProgressBar: true } : null
	const { open, write, close } = resolveCmdWriteFileStream("plugin:android-fs|write_text_file")

	try {
		await open({
			uri: mapFsPathForInput(uri),
			append: options?.append ?? false,
			create: options?.create ?? true,
			notification
		})
		await write(data)
		await close("Ok")
	}
	catch (e) {
		await close("Err").catch(() => { })
		throw e
	}
}

/**
 * Copies the contents of a source file to a destination file.
 * 
 * @remarks
 * The existing content of the destination file will be truncated before writing. 
 * 
 * @param srcUri - URI or path of the source file to copy. 
 * @param destUri - URI or path of the destination file to copy. 
 * @param options - Optional settings: `create`, `notification`. See `CopyFileOptions` for details.
 * 
 * @returns Promise that resolves when the operation completes successfully.
 * 
 * @throws
 * The returned Promise rejects with an error in the following cases:
 * - When the src/dest entry is a directory, not a file.
 * - When the src file does not exist.
 * - When `options.create` is `false` or a URI is specified, and the dest file does not exist.
 * - When the app does not have read permissions for the src file.
 * - When the app does not have write permissions for the dest file.
 * - When an unexpected error occurs.
 * 
 * @see {@link https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.AndroidFs.html#method.copy | AndroidFs::copy}
 */
export async function copyFile(
	srcUri: FsUri | string | URL,
	destUri: FsUri | string | URL,
	options?: CopyFileOptions,
): Promise<void> {

	const create = options?.create ?? true
	const notification = options?.notification ?? null

	await invoke('plugin:android-fs|copy_file', {
		srcUri: mapFsPathForInput(srcUri),
		destUri: mapFsPathForInput(destUri),
		create,
		notification,
	})
}

/**
 * Truncates a file to zero length.
 * 
 * @param uri - URI of the file to truncate.
 * 
 * @returns Promise that resolves when the operation completes successfully.
 * 
 * @throws
 * The returned Promise rejects with an error in the following cases:
 * - When the entry is a directory, not a file.
 * - When the file does not exist.
 * - When the app does not have write permissions for the file.
 * - When a provider of the file via File/Directory Picker does not support the truncate operation.
 * - When an unexpected error occurs.
 * 
 * @see {@link https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.AndroidFs.html#method.open_file_writable | AndroidFs::open_file_writable}
 */
export async function truncateFile(uri: FsUri): Promise<void> {
	await invoke('plugin:android-fs|truncate_file', { uri })
}

/**
 * Renames a file and returns its new URI.
 * 
 * @remarks
 * For URIs from File/Directory Picker,
 * all permissions are lost after this operation, including for the returned new URI.
 * 
 * @param uri - URI of the file to rename.
 * @param name - New name, including the file extension if needed. If an entry with the same name already exists, a sequential number is automatically appended.
 * 
 * @returns Promise that resolves to a new URI of the file.
 * 
 * @throws
 * The returned Promise rejects with an error in the following cases:
 * - When the entry is a directory, not a file.
 * - When the file does not exist.
 * - When the app does not have write permissions for the file.
 * - When a provider of the file via File/Directory Picker does not support the rename operation.
 * - When an unexpected error occurs.
 * 
 * @see {@link https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.AndroidFs.html#method.rename | AndroidFs::rename}
 */
export async function renameFile(
	uri: FsUri,
	name: string
): Promise<FsUri> {

	return await invoke('plugin:android-fs|rename_file', {
		uri,
		name
	})
}

/**
 * Renames a directory and returns its new URI.
 * 
 * @remarks
 * For URIs from Directory Picker,
 * all permissions are lost after this operation, including for the returned new URI.
 * 
 * @param uri - URI of the directory to rename.
 * @param name - New name. If an entry with the same name already exists, a sequential number is automatically appended.
 * 
 * @returns Promise that resolves to a new URI of the directory.
 * 
 * @throws
 * The returned Promise rejects with an error in the following cases:
 * - When the entry is a file, not a directory.
 * - When the directory does not exist.
 * - When the app does not have write permissions for the directory.
 * - When a provider of the directory via Directory Picker does not support the rename operation.
 * - When an unexpected error occurs.
 * 
 * @see {@link https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.AndroidFs.html#method.rename | AndroidFs::rename}
 */
export async function renameDir(
	uri: FsUri,
	name: string
): Promise<FsUri> {

	return await invoke('plugin:android-fs|rename_dir', {
		uri,
		name
	})
}

/**
 * Removes a file.
 * 
 * @param uri - URI of the file to remove.
 * 
 * @returns Promise that resolves when the operation completes successfully.
 * 
 * @throws 
 * The returned Promise rejects with an error in the following cases:
 * - When the entry is a directory, not a file.
 * - When the file does not exist.
 * - When the app does not have write permissions for the file.
 * - When a provider of the file via File/Directory Picker does not support the remove operation.
 * - When an unexpected error occurs.
 * 
 * @see {@link https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.AndroidFs.html#method.remove_file | AndroidFs::remove_file}
 */
export async function removeFile(uri: FsUri): Promise<void> {
	await invoke('plugin:android-fs|remove_file', { uri })
}

/**
 * Removes a directory and all of its contents recursively.
 * 
 * @param uri - URI of the file to remove.
 * 
 * @returns Promise that resolves when the operation completes successfully.
 * 
 * @throws 
 * The returned Promise rejects with an error in the following cases:
 * - When the entry is a file, not a directory.
 * - When the directory does not exist.
 * - When the app does not have read/write permissions for the directory.
 * - When a provider of the directory via Directory Picker does not support the remove or read-directory operations.
 * - When an unexpected error occurs.
 * 
 * @see {@link https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.AndroidFs.html#method.remove_dir_all | AndroidFs::remove_dir_all}
 */
export async function removeDirAll(uri: FsUri): Promise<void> {
	await invoke('plugin:android-fs|remove_dir_all', { uri })
}

/**
 * Removes a empty directory.
 * 
 * @param uri - URI of the file to remove.
 * 
 * @returns Promise that resolves when the operation completes successfully.
 * 
 * @throws 
 * The returned Promise rejects with an error in the following cases:
 * - When the entry is a file, not a directory.
 * - When the directory does not exist.
 * - When the directory is not empty.
 * - When the app does not have read/write permissions for the directory.
 * - When a provider of the directory via Directory Picker does not support the remove or read-directory operations.
 * - When an unexpected error occurs.
 * 
 * @see {@link https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.AndroidFs.html#method.remove_dir | AndroidFs::remove_dir}
 */
export async function removeEmptyDir(uri: FsUri): Promise<void> {
	await invoke('plugin:android-fs|remove_empty_dir', { uri })
}

/**
 * Retrieves metadata and URIs for the immediate children of a directory.
 * 
 * @param uri - URI of the directory to read.
 * @param options - Optional settings: `offset`, `limit`. See `ReadDirOptions` for details.
 *
 * @returns Promise that resolves to an array of entries. Each entry includes metadata and the URI of the file or directory. 
 * 
 * @throws
 * The returned Promise rejects with an error in the following cases:
 * - When the entry is a file, not a directory.
 * - When the directory does not exist.
 * - When the app does not have read permissions for the directory.
 * - When a provider of the directory via Directory Picker does not support the read-directory operation.
 * - When an unexpected error occurs.
 * 
 * @see {@link https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.AndroidFs.html#method.read_dir | AndroidFs::read_dir}
 */
export async function readDir(
	uri: FsUri,
	options?: ReadDirOptions
): Promise<EntryMetadataWithUri[]> {

	const offset = options?.offset ?? null
	const limit = options?.limit ?? null
	const entries = await invoke<EntryMetadataWithUriInner[]>('plugin:android-fs|read_dir', {
		uri,
		offset,
		limit,
	})

	return mutateEntryMetadataArray(entries)
}

/**
 * Opens the system file picker to let the user select an existing file.
 * 
 * @param options - Optional settings: `mimeTypes`, `multiple`, `pickerType`, `needWritePermission`, `localOnly`, `initialLocation`. See `OpenFilePickerOptions` for details.
 * 
 * @returns Promise that resolves to an array of URIs representing the picked files, or an empty array if no files are selected. By default, the app has read access to the URIs, and this permission remains valid until the app or device is terminated. The app can gain persistent access to the files using `persistPickerUriPermission`.
 * 
 * @see {@link https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.Picker.html#method.pick_files | Picker::pick_files}
 * @see {@link https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.Picker.html#method.pick_visual_medias | Picker::pick_visual_medias}
 */
export async function showOpenFilePicker(
	options?: OpenFilePickerOptions
): Promise<FsUri[]> {

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
 * Opens the system directory picker to let the user select a new or existing directory.
 * 
 * @param options - Optional settings: `localOnly`, `initialLocation`. See `OpenDirPickerOptions` for details.
 * 
 * @returns Promise that resolves to a URI representing the picked directory (which may be newly created or already existing), or `null` if no directory is selected. By default, the app has read-write access to the URI, and this permission remains valid until the app or device is terminated. The app can gain persistent access to the directory using `persistPickerUriPermission`. Permissions for derived entries, such as `readDir` and `createNewFile`, depend on the permissions granted to this picked directory.
 * 
 * @see {@link https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.Picker.html#method.pick_dir | Picker::pick_dir}
 */
export async function showOpenDirPicker(
	options?: OpenDirPickerOptions
): Promise<FsUri | null> {

	const localOnly = options?.localOnly ?? false
	const initialLocation = mapPickerInitialLocationForInput(options?.initialLocation)

	return await invoke("plugin:android-fs|show_open_dir_picker", {
		localOnly,
		initialLocation
	})
}

/**
 * Opens the system file saver to let the user specify a file destination.
 * 
 * @param defaultFileName - Initial file name. The user may change this value before saving the file.
 * @param mimeType - MIME type of the file to save. If `null`, it is inferred from the extension of `defaultFileName`.
 * @param options - Optional settings: `localOnly`, `initialLocation`. See `SaveFilePickerOptions` for details.
 * 
 * @returns Promise that resolves to a URI representing the picked file (which may be a newly created empty file or an existing file), or `null` if no file is selected. By default, the app has write access to the URI, and this permission remains valid until the app or device is terminated. The app can gain persistent access to the file using `persistPickerUriPermission`.
 * 
 * @see {@link https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.Picker.html#method.save_file | Picker::save_file}
 */
export async function showSaveFilePicker(
	defaultFileName: string,
	mimeType: string | null,
	options?: SaveFilePickerOptions
): Promise<FsUri | null> {

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
 * Shows the app chooser for sharing files with other applications.
 * 
 * @remarks
 * This sends the files as a single unit.  
 * 
 * Available applications depend on the MIME types associated with the files.   
 * This does not result in an error even if no compatible applications are found; 
 * instead, an empty app chooser is displayed.
 * 
 * @param uris - URIs of the target files.
 * 
 * @returns Promise that resolves after the app chooser is launched.
 * 
 * @throws 
 * The returned Promise rejects with an error in the following cases:
 * - When the entries contain a directory.
 * - When the files do not exist.
 * - When the app does not have read permissions for the files.
 * 
 * @see {@link https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.Opener.html#method.share_files | Opener::share_files}
 */
export async function showShareFileAppChooser(
	uris: FsUri | FsUri[]
): Promise<void> {

	await invoke("plugin:android-fs|show_share_file_app_chooser", {
		uris: Array.isArray(uris) ? uris : [uris]
	})
}

/**
 * Shows the app chooser for opening a file with other applications.
 * 
 * @remarks 
 * Available applications depend on the MIME types associated with the file.   
 * This does not result in an error even if no compatible applications are found; 
 * instead, an empty app chooser is displayed.
 * 
 * @param uri - URI of the target file.
 * 
 * @returns Promise that resolves after the app chooser is launched.
 * 
 * @throws 
 * The returned Promise rejects with an error in the following cases:
 * - When the entry is a directory, not a file.
 * - When the file does not exist.
 * - When the app does not have read permissions for the file.
 * 
 * @see {@link https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.Opener.html#method.open_file | Opener::open_file}
 */
export async function showViewFileAppChooser(uri: FsUri): Promise<void> {
	await invoke("plugin:android-fs|show_view_file_app_chooser", { uri })
}

/**
 * Shows the app chooser for opening a directory with other applications.
 * 
 * @remarks 
 * This does not result in an error even if no compatible applications are found; 
 * instead, an empty app chooser is displayed.
 * 
 * @param uri - URI of the target directory.
 * 
 * @returns Promise that resolves after the app chooser is launched.
 * 
 * @throws 
 * The returned Promise rejects with an error in the following cases:
 * - When the entry is a file, not a directory.
 * - When the directory does not exist.
 * - When the app does not have read permissions for the directory.
 * 
 * @see {@link https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.Opener.html#method.open_dir | Opener::open_dir}
 */
export async function showViewDirAppChooser(uri: FsUri): Promise<void> {
	await invoke("plugin:android-fs|show_view_dir_app_chooser", { uri })
}

/**
 * Shows the app chooser for editing a file with other applications.
 * 
 * @remarks 
 * Available applications depend on the MIME types associated with the file.   
 * This does not result in an error even if no compatible applications are found; 
 * instead, an empty app chooser is displayed.
 * 
 * OPINION:
 * I think this is probably the least commonly used request for sending a file to an app.
 * Even if you want to open an image or video editing app,
 * `showViewFileAppChooser` often lets you choose from a wider range of apps.
 *
 * @param uri - URI of the target file.
 * 
 * @returns Promise that resolves after the app chooser is launched.
 * 
 * @throws 
 * The returned Promise rejects with an error in the following cases:
 * - When the entry is a directory, not a file.
 * - When the file does not exist.
 * - When the app does not have read/write permissions for the file.
 * 
 * @see {@link https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.Opener.html#method.edit_file | Opener::edit_file}
 */
export async function showEditFileAppChooser(uri: FsUri): Promise<void> {
	await invoke("plugin:android-fs|show_edit_file_app_chooser", { uri })
}

/**
 * Checks the permission state of a URI granted by File/Directory Picker.
 * 
 * @param uri - URI of the target file or directory.
 * @param state - Permission state to check. One of: `"Read"`, `"Write"`, `"ReadAndWrite"`, `"ReadOrWrite"`.
 * 
 * @returns Promise that resolves to a boolean indicating whether the specified permission is granted.
 * 
 * @see {@link https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.Picker.html#method.check_uri_permission | Picker::check_uri_permission}
 */
export async function checkPickerUriPermission(
	uri: FsUri,
	state: UriPermissionState
): Promise<boolean> {

	return await invoke("plugin:android-fs|check_picker_uri_permission", { uri, state })
}

/**
 * Takes a persistent permission to access the file or directory (and its descendants) selected via File/Directory Picker.  
 * 
 * @remarks
 * This prolongs an already acquired permission rather than acquiring a new one.
 * 
 * Android imposes a strict limit on the total number of URIs
 * that can be made persistent at the same time. 
 * Therefore, it is highly recommended to release unnecessary persisted URIs
 * via `releasePersistedPickerUriPermission`
 * or `releaseAllPersistedPickerUriPermissions`.
 * 
 * Persisted permissions may be revoked by other applications or the user, 
 * by modifying set permissions, or by moving/removing entries.   
 * To verify validity,
 * use `checkPersistedPickerUriPermission`
 * or `checkPickerUriPermission`.
 * 
 * @param uri - URI of the target file or directory.
 * 
 * @returns Promise that resolves when the operation is complete.
 * 
 * @throws
 * The returned Promise rejects with an error in the following cases:
 * - When the entry does not exist.
 * - When the app does not have any permissions for the entry.
 * - When a provider of the entry via File/Directory Picker does not support the persist-permissions operation.
 * - When an unexpected error occurs.
 * 
 * @see {@link https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.Picker.html#method.persist_picker_uri_permission | Picker::persist_picker_uri_permission}
 * @see {@link https://stackoverflow.com/questions/71099575/should-i-release-persistableuripermission-when-a-new-storage-location-is-chosen/71100621#71100621 | Android Persistable URI Permission Limit Discussion}
 */
export async function persistPickerUriPermission(uri: FsUri): Promise<void> {
	await invoke("plugin:android-fs|persist_picker_uri_permission", { uri })
}

/**
 * Checks the persisted permission state of a URI granted via `persistPickerUriPermission`.
 * 
 * @param uri - URI of the target file or directory.
 * @param state - Permission state to check. One of: `"Read"`, `"Write"`, `"ReadAndWrite"`, `"ReadOrWrite"`.
 * 
 * @returns Promise that resolves to a boolean; `false` if only non-persistent permissions exist or if there are no permissions.
 * 
 * @see {@link https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.Picker.html#method.check_persisted_picker_uri_permission | Picker::check_persisted_picker_uri_permission}
 */
export async function checkPersistedPickerUriPermission(
	uri: FsUri,
	state: UriPermissionState
): Promise<boolean> {

	return await invoke("plugin:android-fs|check_persisted_picker_uri_permission", { uri, state })
}

/**
 * Relinquishes a persisted permission of a URI granted via `persistPickerUriPermission`.
 * 
 * @param uri - URI of the target file or directory.
 * @returns Promise that resolves to a boolean; `true` if a persisted permission exists for the specified URI and was successfully released, or `false` if no persisted permission existed.
 *
 * @see {@link https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.Picker.html#method.release_persisted_picker_uri_permission | Picker::release_persisted_picker_uri_permission}
 */
export async function releasePersistedPickerUriPermission(uri: FsUri): Promise<boolean> {
	return await invoke("plugin:android-fs|release_persisted_picker_uri_permission", { uri })
}

/**
 * Relinquishes all persisted permissions of URIs granted via `persistPickerUriPermission`.
 * 
 * @returns Promise that resolves when the operation is complete.
 * 
 * @see {@link https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.Picker.html#method.release_all_persisted_picker_uri_permissions | Picker::release_all_persisted_picker_uri_permissions}
 */
export async function releaseAllPersistedPickerUriPermissions(): Promise<void> {
	await invoke("plugin:android-fs|release_all_persisted_picker_uri_permissions")
}

/**
 * Gets all persisted permissions of URIs granted via `persistPickerUriPermission`.
 *
 * @returns Promise that resolves to an array containing currently available persisted permissions. Each entry always has at least one of read or write permissions.
 * 
 * @see {@link https://docs.rs/tauri-plugin-android-fs/latest/tauri_plugin_android_fs/api/api_async/struct.Picker.html#method.get_all_persisted_picker_uri_permissions | Picker::get_all_persisted_picker_uri_permissions}
 */
export async function listAllPersistedPickerUriPermissions(): Promise<PersistedUriPermission[]> {
	return await invoke("plugin:android-fs|list_all_persisted_picker_uri_permissions")
}

/**
 * Gets the MIME type associated with a file extension.
 *
 * @param extension - File extension. A leading `.` is optional, and the extension is automatically normalized to lowercase.
 * @returns A promise that resolves to the MIME type, or null if the MIME type cannot be determined from the specified extension.
 * 
 * @see {@link https://developer.android.com/reference/kotlin/android/webkit/MimeTypeMap | AndroidAPI MimeTypeMap}
 */
export async function getMimeTypeFromExtension(extension: string): Promise<string | null> {
	return await invoke("plugin:android-fs|get_mime_type_from_extension", { extension })
}



/** 512 KiB */
const DEFAULT_BUFFER_SIZE_FOR_IPC = 512 * 1024;

function mapBufferByteLengthForInput(s?: number): number {
	const bufferSize = s ?? DEFAULT_BUFFER_SIZE_FOR_IPC
	if (!isNonzeroSafeInt(bufferSize)) {
		throw new Error("Invalid bufferByteLength: expected a non-zero safe unsigned integer (1..Number.MAX_SAFE_INTEGER)")
	}
	return bufferSize
}

function mapEncodingLabelForInput(label?: string): string {
	try {
		return (new TextDecoder(label)).encoding
	}
	catch {
		throw new RangeError("Bad encoding label")
	}
}

function mapMaxLineByteLengthForInput(s?: number): number {
	if (s == null) return 0

	if (!Number.isSafeInteger(s) || s < 0) {
		throw new Error("Invalid maxLineByteLength: expected a safe unsigned integer");
	}

	return s
}

const UTF8_DECODER = new TextDecoder()
const UTF8_ENCODER = new TextEncoder()

function decodeUtf8(bytes: AllowSharedBufferSource): string {
	return UTF8_DECODER.decode(bytes)
}
function encodeUtf8(text: string): Uint8Array<ArrayBuffer> {
	return UTF8_ENCODER.encode(text)
}

type CmdReadFileStreamHandler = {
	open: (args: Record<string, unknown>) => Promise<void>
	read: (len: number) => Promise<Uint8Array<ArrayBuffer> | null>,
	close: () => Promise<void>,
}
function resolveCmdReadFileStream(cmdName: string): CmdReadFileStreamHandler {
	// Tauri IPC の制約により、戻り値は全て ArrayBuffer 型となる。
	type CmdEvents = {
		Open: Record<string, unknown>,
		Read: { id: number, len: number },
		Close: { id: number },
	}
	type CmdType = keyof CmdEvents
	type CmdArgs<T extends CmdType> = CmdEvents[T]
	function cmd<T extends CmdType>(type: T, args: CmdArgs<T>): Promise<ArrayBuffer> {
		return invoke(cmdName, { event: { type, args } })
	}


	let id: Promise<number> | null = null

	return {
		open: async (args) => {
			if (id !== null) throw new Error("File already opened")
			id = cmd("Open", args).then(ridFromBytes)
			await id
		},

		read: async (len) => {
			if (id === null) throw new Error("File not opened")
			const data = await cmd("Read", { id: await id, len, })
			return data.byteLength === 0 ? null : new Uint8Array(data)
		},

		close: async () => {
			if (id === null) return
			await cmd("Close", { id: await id })
		}
	}
}

type CmdWriteFileStreamHandler = {
	open: (args: Record<string, unknown>) => Promise<void>,
	write: (data: Uint8Array<ArrayBufferLike> | string) => Promise<void>,
	close: (result: "Err" | "Ok") => Promise<void>,
}
function resolveCmdWriteFileStream(cmdName: string): CmdWriteFileStreamHandler {
	// Tauri IPC の制約により、大きいバイトを送る際は body で、それ以外の値は headers で送信する。
	type CmdEvents = {
		Open: { in: { body: Uint8Array, args: Record<string, unknown> }, out: { id: number, supportsRawIpcRequestBody: boolean } },
		Write: { in: { body: Uint8Array | { data: string, format: "dataUrlToDecodedData" | "textToUtf8" }, args: { id: number } }, out: void },
		Close: { in: { body: {}, args: { id: number, error: boolean } }, out: void },
	}
	type CmdType = keyof CmdEvents
	type CmdInputBody<T extends CmdType> = CmdEvents[T]["in"]["body"]
	type CmdInputArgs<T extends CmdType> = CmdEvents[T]["in"]["args"]
	type CmdOutput<T extends CmdType> = CmdEvents[T]["out"]
	function cmd<T extends CmdType>(type: T, body: CmdInputBody<T>, args: CmdInputArgs<T>): Promise<CmdOutput<T>> {
		return invoke(cmdName, body, {
			headers: {
				"tpafs-cmd-type": type,
				"tpafs-cmd-args": encodeURIComponent(JSON.stringify(args))
			}
		})
	}


	const PAYLOAD_FOR_CHECKING_RAW_IPC_REQUEST_BODY_SUPPORTED = new Uint8Array([0]);

	let state: Promise<{ id: number, supportsRawIpcRequestBody: boolean }> | null = null

	return {
		open: async (args) => {
			if (state !== null) throw new Error("File already opened")
			state = cmd(
				"Open",
				PAYLOAD_FOR_CHECKING_RAW_IPC_REQUEST_BODY_SUPPORTED,
				args
			)
			await state
		},

		write: async (chunk) => {
			if (state === null) throw new Error("File not opened")
			const { id, supportsRawIpcRequestBody } = await state

			if (supportsRawIpcRequestBody) {
				const data = typeof chunk === "string"
					? encodeUtf8(chunk)
					: chunk

				await cmd("Write", data, { id })
			}
			else {
				if (typeof chunk === "string") {
					await cmd(
						"Write",
						{
							data: chunk,
							format: "textToUtf8"
						},
						{ id }
					)
				}
				else {
					// IPC のリクエストで raw Body を送れない場合、
					// 大きな配列に対して非常に非効率な形式にシリアライズされる。
					// よって、まだマシな dataURL としてデータを送る。
					// Data URL を用いる理由は web API の FileReader で比較的効率的に作成できるため。
					// <https://github.com/tauri-apps/tauri/issues/10573>
					await cmd(
						"Write",
						{
							data: await bytesToDataUrl(chunk),
							format: "dataUrlToDecodedData"
						},
						{ id }
					)
				}
			}
		},

		close: async (result) => {
			if (state === null) return
			const { id } = await state
			const error = result === "Err"
			await cmd("Close", {}, { id, error })
		},
	}
}

function createTextLinesReadableStream(
	handler: {
		/** null か空で EOF */
		read: () => Promise<Uint8Array<ArrayBuffer> | null>,
		release?: () => Promise<void>
	},
	options?: {
		fatal?: boolean,
		label?: string,
	},
	signal?: AbortSignal
): ReadableStream<{ line: string, lineBreak: "\n" | "\r\n" | null }> {

	/*
	 * bytes は以下の形式のレコードが連続したものであり、
	 * 各レコードが分断されることはない。
	 * 
	 * - err flag (u8, 0 = ok, 1 = err)
	 * - line break type (u8, 0 = null, 1 = "\n", 2 = "\r\n")
	 * - line bytes len (u64, big endian)
	 * - line bytes (variable bytes)
	 * 
	 * err flag が 0 の場合、正常にその行が読み込まれたことを指す。
	 * この場合、line bytes には BOM 処理されたテキストが格納される。
	 * 
	 * err flag が 1 の場合、その行でエラーが発生したことを示す。
	 * この場合、line bytes には utf-8 形式のエラーメッセージが格納され、
	 * この呼び出しでの最後の行となる。
	 * 
	 * エラー発生後の呼び出しの挙動は未定義。
	 */
	const ERR_FLAG_LEN = 1;
	const LINE_BREAK_TYPE_LEN = 1;
	const LINE_LEN_LEN = 8;

	const ERR_FLAG_OFFSET = 0;
	const LINE_BREAK_TYPE_OFFSET = ERR_FLAG_OFFSET + ERR_FLAG_LEN;
	const LINE_LEN_OFFSET = LINE_BREAK_TYPE_OFFSET + LINE_BREAK_TYPE_LEN;
	const LINE_OFFSET = LINE_LEN_OFFSET + LINE_LEN_LEN;

	const LINE_BREAK_NULL = 0
	const LINE_BREAK_LF = 1
	const LINE_BREAK_CRLF = 2

	let abortListener: (() => void) | null = null
	let decoder: TextDecoder | null = null
	let buffer: Uint8Array<ArrayBuffer> | null = null

	let cleanupPromise: Promise<void> | null = null
	function cleanup(): Promise<void> {
		if (cleanupPromise === null) {
			cleanupPromise = (async () => {
				buffer = null
				decoder = null
				if (signal != null && abortListener != null) {
					signal.removeEventListener("abort", abortListener)
					abortListener = null
				}
				if (handler.release) {
					await handler.release()
				}
			})()
		}
		return cleanupPromise
	}

	// エラーはその原因となった行を読み込んだ際に発生させたいため、
	// 1回の pull では1回だけ enqueue　を行う。
	// 複数回行うとエラーが発生した行ではない箇所で read してもエラーになってしまう。
	return new ReadableStream({
		start(controller) {
			if (signal) {
				abortListener = () => {
					cleanup().catch(() => { })
					controller.error(signal.reason ?? newAbortError())
				}
				signal.addEventListener("abort", abortListener);
			}
		},

		async pull(controller) {
			try {
				throwIfAborted(signal)
				if (buffer == null || buffer.byteLength === 0) {
					buffer = await handler.read()
					throwIfAborted(signal)
				}
				if (buffer == null || buffer.byteLength === 0) {
					await cleanup()
					controller.close()
					return
				}

				if (buffer.byteLength < LINE_OFFSET) {
					throw new Error("Invalid data: Chunk ended with partial header.")
				}
				const lineLen = trySafeU64FromBytes(
					buffer.subarray(LINE_LEN_OFFSET, LINE_LEN_OFFSET + LINE_LEN_LEN),
					"bigEndian"
				)

				if (buffer.byteLength < LINE_OFFSET + lineLen) {
					throw new Error("Invalid data: Line split detected.")
				}
				const lineBytes = buffer.subarray(LINE_OFFSET, LINE_OFFSET + lineLen)

				const errFlag = buffer[ERR_FLAG_OFFSET]
				if (numToFlag(errFlag)) {
					throw new Error(decodeUtf8(lineBytes))
				}

				const lineBreakType = buffer[LINE_BREAK_TYPE_OFFSET]
				let lineBreak: "\n" | "\r\n" | null = null
				if (lineBreakType === LINE_BREAK_LF) lineBreak = "\n"
				else if (lineBreakType === LINE_BREAK_CRLF) lineBreak = "\r\n"
				else if (lineBreakType === LINE_BREAK_NULL) lineBreak = null
				else throw new Error("Invalid lineBreakType")

				if (decoder == null) {
					decoder = new TextDecoder(options?.label, {
						fatal: options?.fatal,
						ignoreBOM: true
					})
				}
				const line = decoder.decode(lineBytes)

				throwIfAborted(signal)
				controller.enqueue({ line, lineBreak })
				buffer = buffer.subarray(LINE_OFFSET + lineLen)
			}
			catch (e) {
				await cleanup().catch(() => { })
				throw e
			}
		},

		async cancel() {
			await cleanup()
		}
	})
}

function throwIfAborted(signal: AbortSignal | undefined | null) {
	if (signal?.aborted === true) {
		throw (signal?.reason ?? newAbortError())
	}
}

function newAbortError(): DOMException {
	return new DOMException("The operation was aborted.", "AbortError")
}

async function bytesToDataUrl(bytes: Uint8Array<ArrayBufferLike>): Promise<string> {
	const buffer = bytes.buffer instanceof ArrayBuffer
		? bytes as Uint8Array<ArrayBuffer>
		: new Uint8Array(bytes)

	const blob = new Blob([buffer], { type: "application/octet-stream" })
	return await blobToDataUrl(blob)
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

function isNonzeroSafeInt(num: number): boolean {
	return isSafeInt(num) && num !== 0
}

function isSafeInt(num: number): boolean {
	return Number.isSafeInteger(num) && 0 <= num && num <= Number.MAX_SAFE_INTEGER
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
		throw new Error("Expected 4 bytes for u32");
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

function numToFlag(flag: number): boolean {
	if (flag === 1) return true
	if (flag === 0) return false
	throw new Error("Invalid flag value")
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
		throw new Error("Expected 8 bytes for u64");
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