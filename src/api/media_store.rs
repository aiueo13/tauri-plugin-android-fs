use sync_async::sync_async;
use crate::*;
use super::*;


/// API for accessing the MediaStore.
///
/// MediaStore is an Android-provided mechanism for managing and searching
/// media files stored on the device, such as images, videos, and audio.
/// MediaStore maintains an indexed database of media files. 
/// Each file is generally represented by a row, and its metadata is stored in columns.
/// 
/// This is a low-level API.
/// To create files to MediaStore, consider using [`PublicStorage`] instead of this API.
/// To retrieve images and videos for general use, consider using [`Picker`] instead.
/// This API is most useful for apps such as gallery and music apps
/// that need to list the media files on the device or look up
/// detailed information about them.
/// 
/// Using this API requires requesting the appropriate permissions.
/// See [`MediaStore::request_read_permission`] for the required setup and important caveats.
/// 
/// # Examples
/// For example, when building an image gallery app, you will need following:
/// 1. Defining permissions to access images
/// 2. Requesting permissions to access images
/// 3. Enumerating directories that contain images
/// 4. Enumerating the images in a directory
/// 5. Retrieving image metadata and accessing files
///
/// This section shows an implementation example for each of these.
/// 
/// **1. Defining permissions to access images**:
/// By adding `media_store_image_read_permission` feature,
/// the required permissions for each Android version
/// can be automatically declared in the app.
/// See [`MediaStore::request_read_permission`], for details.
/// 
/// **2. Requesting permissions to access images**
/// ```no_run
/// use tauri_plugin_android_fs::{AndroidFsExt, MediaStoreReadPermissionState, MediaStoreReadPermissionType};
/// 
/// /// Requests permission from the user to access images.
/// ///
/// /// Returns `true` if the permission is granted.
/// /// 
/// /// Returns `false` if the permission has not been granted at the time the request completes.
/// /// However, the user may grant the permission later through Settings.
/// pub async fn request_images_permission<R: tauri::Runtime>(
///     app: &tauri::AppHandle<R>
/// ) -> tauri_plugin_android_fs::Result<bool> {
/// 
///     let media_store = app.android_fs_async().media_store();
/// 
///     // If the user has granted access to only selected images, not all images,
///     // treat the permission as granted without displaying the dialog.
///     // Removing this check allows the user to change which images the app can access.
///     if media_store.check_partial_read_permission(MediaStoreReadPermissionType::Image).await? {
///         return Ok(true)
///     }
/// 
///     match media_store.request_read_permission(MediaStoreReadPermissionType::Image).await? {
///         MediaStoreReadPermissionState::Granted => Ok(true),
///         MediaStoreReadPermissionState::Denied => Ok(false),
///         // If "Don't ask again" was selected when requesting the permission previously,
///         // or if the user denies the same permission twice in a row on Android 11 or higher,
///         // the permission dialog may no longer be displayed, and the request will always be denied.
///         // In this case, the user must grant the permission to the app through Settings.
///         MediaStoreReadPermissionState::PermanentlyDenied => {
///             // Opens the Settings screen to direct the user to the app’s permission settings. 
///             // This API returns control immediately after opening the Settings screen, 
///             // so it is not possible to wait for the user
///             // to take action and determine whether permission has been granted.
///             let helper = app.android_fs_async().helper();
///             helper.show_open_app_settings_dialog(
///                 "Permission is required to access images. Please grant the image permission in Settings.", 
///                 "Open Settings"
///             ).await?;
/// 
///             Ok(false)
///         },
///     }
/// }
/// ```
/// 
/// **3. Enumerating directories that contain images**
/// ```no_run
/// use std::{collections::{HashMap, HashSet}, path::PathBuf, time::{Duration, SystemTime}};
/// use tauri_plugin_android_fs::{AndroidFsExt, MediaStoreColumn, MediaStoreQueryFilesOptions, MediaStoreTable};
/// 
/// #[derive(Debug, Hash, PartialEq, Eq, Clone)]
/// pub struct ImageDirectory {
///     pub id: i64,
///     pub name: String,
///     pub latest_file_added: SystemTime,
///     pub latest_file_modified: SystemTime,
///     pub latest_file_taken: Option<SystemTime>,
///     pub image_count: u64,
///     pub total_file_size: u64,
/// }
/// 
/// /// Lists all directories that contains images,
/// /// along with metadata aggregated over their images.
/// /// 
/// /// # Performance
/// /// On my mid-range Android 16 device,
/// /// this processing around 100,000 images takes approximately 1–3 seconds.
/// pub async fn list_image_directories<R: tauri::Runtime>(
///     app: &tauri::AppHandle<R>
/// ) -> tauri_plugin_android_fs::Result<HashSet<ImageDirectory>> {
/// 
///     let media_store = app.android_fs_async().media_store();
/// 
///     // Define the columns to retrieve (i.e., file metadata).
///     // See MediaColumns and FileColumns for the available columns.
///     // - https://developer.android.com/reference/android/provider/MediaStore.MediaColumns
///     // - https://developer.android.com/reference/android/provider/MediaStore.Files.FileColumns
///     let projection = &[
///         MediaStoreColumn::integer("_id"),
///         MediaStoreColumn::string("_data"), 
///         MediaStoreColumn::integer("parent"), 
///         MediaStoreColumn::integer("_size"), 
///         MediaStoreColumn::integer("date_added"),
///         MediaStoreColumn::integer("date_modified"),
///         MediaStoreColumn::integer("datetaken"),
///     ];
///     let id_i = MediaStoreColumn::index_of("_id", projection)?;
///     let path_i = MediaStoreColumn::index_of("_data", projection)?;
///     let parent_directory_id_i = MediaStoreColumn::index_of("parent", projection)?;
///     let file_size_i = MediaStoreColumn::index_of("_size", projection)?;
///     let date_added_i = MediaStoreColumn::index_of("date_added", projection)?;
///     let date_modified_i = MediaStoreColumn::index_of("date_modified", projection)?;
///     let date_taken_i = MediaStoreColumn::index_of("datetaken", projection)?;
/// 
///     // The value of "media_type" column for images.
///     // - https://developer.android.com/reference/android/provider/MediaStore.Files.FileColumns#MEDIA_TYPE_IMAGE
///     let media_type_image = 1;
/// 
///     // Enumerate files using pagination to avoid excessive memory usage.
///     // Using a small page size requires frequent query calls, which is inefficient. 
///     // Therefore, use a page size as large as reasonably possible.
///     let page_size = 5000;
///     let mut last_file_id = Option::<i64>::None;
///     let mut directories = HashMap::<i64, ImageDirectory>::new();
///     loop {
///         // Pagination is based on file IDs rather than offsets.
///         // This prevents pagination from becoming misaligned if files are added during enumeration.
///         let queried_files = media_store.query_files(
///             MediaStoreTable::File, 
///             projection,
///             MediaStoreQueryFilesOptions {
///                 limit: Some(page_size),
///                 sort_order: Some("_id ASC"),
///                 selection: match last_file_id {
///                     Some(_) => Some("media_type = ? AND _id > ?"),
///                     None => Some("media_type = ?"),
///                 },
///                 selection_args: (match last_file_id {
///                     Some(id) => Some(vec![media_type_image.to_string(), id.to_string()]),
///                     None => Some(vec![media_type_image.to_string()]),
///                 }).as_deref(),
///                 ..Default::default()
///             }
///         ).await?;
/// 
///         for file in &queried_files {
///             let id = file.get_required_integer(id_i)?;
///             last_file_id = Some(id);
/// 
///             // These columns should not normally be None,
///             // but filter out inconsistent items just in case.
///             let Some(path) = file.get_string(path_i) else { continue; };
///             let Some(parent_directory_id) = file.get_integer(parent_directory_id_i) else { continue; };
///             let Some(file_size) = file.get_integer_as_u64(file_size_i) else { continue; };
///             let Some(date_added) = file.get_integer_as_u64(date_added_i) else { continue; };
///             let Some(date_modified) = file.get_integer_as_u64(date_modified_i) else { continue; };
/// 
///             // The photo's capture time may not always be available for all files.
///             let date_taken = file.get_integer_as_u64(date_taken_i).filter(|d| *d != 0);
/// 
///             let date_added = SystemTime::UNIX_EPOCH + Duration::from_secs(date_added);
///             let date_modified = SystemTime::UNIX_EPOCH + Duration::from_secs(date_modified);
///             let date_taken = date_taken.map(|date_taken| {
///                 // Note that, unlike added and modified, 
///                 // taken is measured in milliseconds, not seconds.
///                 SystemTime::UNIX_EPOCH + Duration::from_millis(date_taken)
///             });
/// 
///             // Update the directory metadata if it already exists.
///             if let Some(directory) = directories.get_mut(&parent_directory_id) {
///                 if directory.latest_file_added < date_added {
///                     directory.latest_file_added = date_added;
///                 }
///                 if directory.latest_file_modified < date_modified {
///                     directory.latest_file_modified = date_modified;
///                 }
///                 if let Some(date_taken) = date_taken {
///                     if directory.latest_file_taken.is_none_or(|d| d < date_taken) {
///                         directory.latest_file_taken = Some(date_taken);
///                     }
///                 }
///                 directory.image_count += 1;
///                 directory.total_file_size += file_size;
///             }
///             // Add a new directory metadata if it does not exist.
///             else {
///                 let parent_directory_name = PathBuf::from(path)
///                     .parent()
///                     .and_then(|p| p.file_name())
///                     .map(|p| p.to_string_lossy().to_string())
///                     .unwrap_or_default();
/// 
///                 directories.insert(parent_directory_id, ImageDirectory {
///                     id: parent_directory_id,
///                     name: parent_directory_name,
///                     latest_file_modified: date_modified,
///                     latest_file_added: date_added,
///                     latest_file_taken: date_taken,
///                     image_count: 1,
///                     total_file_size: file_size,
///                 });
///             }
///         }
/// 
///         if (queried_files.len() as u64) < page_size {
///             break;
///         }
///     }
/// 
///     Ok(directories.into_values().collect())
/// }
/// ```
/// 
/// **4. Enumerating the images in a directory**
/// ```no_run
/// use std::time::{Duration, SystemTime};
/// use tauri_plugin_android_fs::{AndroidFsExt, FsUri, MediaStoreColumn, MediaStoreQueryFilesOptions, MediaStoreTable};
/// 
/// #[derive(Debug, Hash, PartialEq, Eq, Clone)]
/// pub struct ImageFile {
///     pub uri: FsUri,
///     pub name: String,
///     pub file_size: u64,
///     pub mime_type: String,
///     pub added: SystemTime,
///     pub modified: SystemTime,
///     pub taken: Option<SystemTime>,
/// }
/// 
/// #[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
/// pub enum ImageFileSortKey {
///     Name,
///     FileSize,
///     MimeType,
///     Added,
///     Modified,
///     Taken,
/// }
/// 
/// /// Lists the images in a directory, sorted and paginated.
/// /// 
/// /// `directory_id` is the MediaStore "parent" column, 
/// /// i.e., the `id` of an [`ImageDirectory`] returned by [`list_image_directories`].
/// pub async fn list_image_files_in_directory<R: tauri::Runtime>(
///     directory_id: i64,
///     sort_key: ImageFileSortKey,
///     sort_direction_is_desc: bool,
///     offset: Option<u64>,
///     limit: Option<u64>,
///     app: &tauri::AppHandle<R>,
/// ) -> tauri_plugin_android_fs::Result<Vec<ImageFile>> {
/// 
///     let media_store = app.android_fs_async().media_store();
/// 
///     // Define the columns to retrieve (i.e., file metadata).
///     // See MediaColumns and FileColumns for the available columns.
///     // - https://developer.android.com/reference/android/provider/MediaStore.MediaColumns
///     // - https://developer.android.com/reference/android/provider/MediaStore.Files.FileColumns
///     let projection = &[
///         MediaStoreColumn::string("_display_name"),
///         MediaStoreColumn::integer("_size"),
///         MediaStoreColumn::string("mime_type"),
///         MediaStoreColumn::integer("date_added"),
///         MediaStoreColumn::integer("date_modified"),
///         MediaStoreColumn::integer("datetaken"),
///     ];
///     let name_i = MediaStoreColumn::index_of("_display_name", projection)?;
///     let file_size_i = MediaStoreColumn::index_of("_size", projection)?;
///     let mime_type_i = MediaStoreColumn::index_of("mime_type", projection)?;
///     let date_added_i = MediaStoreColumn::index_of("date_added", projection)?;
///     let date_modified_i = MediaStoreColumn::index_of("date_modified", projection)?;
///     let date_taken_i = MediaStoreColumn::index_of("datetaken", projection)?;
/// 
///     // The value of "media_type" column for images.
///     // - https://developer.android.com/reference/android/provider/MediaStore.Files.FileColumns#MEDIA_TYPE_IMAGE
///     let media_type_image = 1;
/// 
///     let primary_sort_column = match sort_key {
///         ImageFileSortKey::Name => "_display_name".to_string(),
///         ImageFileSortKey::FileSize => "_size".to_string(),
///         ImageFileSortKey::MimeType => "mime_type".to_string(),
///         ImageFileSortKey::Added => "date_added".to_string(),
///         ImageFileSortKey::Modified => "date_modified".to_string(),
///         ImageFileSortKey::Taken => "datetaken".to_string(),
///     };
///     let sort_direction = match sort_direction_is_desc {
///         true => "DESC",
///         false => "ASC",
///     };
/// 
///     let queried_files = media_store.query_files(
///         MediaStoreTable::File,
///         projection, 
///         MediaStoreQueryFilesOptions {
///             selection: Some("media_type = ? AND parent = ?"),
///             selection_args: Some(&[media_type_image.to_string(), directory_id.to_string()]),
///             sort_order: Some(&format!("{primary_sort_column} {sort_direction}, _id")),
///             offset,
///             limit,
///             ..Default::default()
///         }
///     ).await?;
/// 
///     let mut files = Vec::new();
///     for file in queried_files {
///         // These columns should not normally be None,
///         // but filter out inconsistent items just in case.
///         let Some(name) = file.get_string(name_i) else { continue; };
///         let Some(file_size) = file.get_integer_as_u64(file_size_i) else { continue; };
///         let Some(date_added) = file.get_integer_as_u64(date_added_i) else { continue; };
///         let Some(date_modified) = file.get_integer_as_u64(date_modified_i) else { continue; };
///        
///         let date_taken = file.get_integer_as_u64(date_taken_i).filter(|d| *d != 0);
///         let mime_type = file.get_string(mime_type_i);
/// 
///         let mime_type = mime_type.unwrap_or_else(|| "application/octet-stream".into());
///         let date_added = SystemTime::UNIX_EPOCH + Duration::from_secs(date_added);
///         let date_modified = SystemTime::UNIX_EPOCH + Duration::from_secs(date_modified);
///         let date_taken = date_taken.map(|date_taken| {
///             // Note that, unlike added and modified, 
///             // taken is measured in milliseconds, not seconds.
///             SystemTime::UNIX_EPOCH + Duration::from_millis(date_taken)
///         });
/// 
///         files.push(ImageFile {
///             uri: file.uri,
///             name,
///             mime_type,
///             file_size,
///             added: date_added,
///             modified: date_modified,
///             taken: date_taken,
///         });
///     }
/// 
///     Ok(files)
/// }
/// ```
/// 
/// **5. Retrieving image metadata and accessing files**
/// ```no_run
/// use tauri_plugin_android_fs::{AndroidFsExt, FsUri, ImageFormat, MediaStoreColumn, Size, api_level};
/// 
/// pub async fn handle_image_file<R: tauri::Runtime>(
///     uri: &FsUri,
///     app: &tauri::AppHandle<R>
/// ) -> tauri_plugin_android_fs::Result<()> {
/// 
///     let fs = app.android_fs_async();
/// 
///     // File data can be retrieved using the general Android FS API.
///     {
///         let file: std::fs::File = fs.open_file_readable(uri).await?;
///         let name = fs.get_name(&uri).await?;
///         let mime_type = fs.get_mime_type(&uri).await?;
///         let size = fs.get_len(&uri).await?;
///         let thumbnail = fs.get_thumbnail(uri, Size { width: 500, height: 500 }, ImageFormat::Webp).await?;
///     }
/// 
///     // Image data can be retrieved by querying MediaStore.
///     {
///         let media_store = fs.media_store();
/// 
///         // Define the columns to retrieve (i.e., file metadata).
///         // See MediaColumns, FileColumns and ImageColumns for the available columns.
///         // - https://developer.android.com/reference/android/provider/MediaStore.MediaColumns
///         // - https://developer.android.com/reference/android/provider/MediaStore.Files.FileColumns
///         // - https://developer.android.com/reference/android/provider/MediaStore.Images.ImageColumns
///         let projection = &[
///             MediaStoreColumn::integer("width"),
///             MediaStoreColumn::integer("height"),
///             MediaStoreColumn::integer("orientation"),
///             // MediaColumns.RESOLUTION is only available on Android 11 (API level 30) or higher, 
///             // so specify `since_api_level` to skip querying it on Android 10 or lower 
///             // and treat its result value as null.
///             // Passing an undefined column will cause the query to fail on some Android versions.
///             MediaStoreColumn::string("resolution").since_api_level(api_level::ANDROID_11),
///         ];
///         let width_i = MediaStoreColumn::index_of("width", projection)?;
///         let height_i = MediaStoreColumn::index_of("height", projection)?;
///         let orientation_i = MediaStoreColumn::index_of("orientation", projection)?;
///         let resolution_description_i = MediaStoreColumn::index_of("resolution", projection)?;
///         
///         // Note for multiple files, 
///         // using `query_files` with specified conditions is more efficient
///         // like `list_image_files_in_directory`.
///         let queried_file = media_store.query_file(uri, projection).await?;
/// 
///         let width: Option<u64> = queried_file.get_integer_as_u64(width_i);
///         let height: Option<u64> = queried_file.get_integer_as_u64(height_i);
///         // User-visible image size description.
///         let resolution_description: Option<String> = queried_file.get_string(resolution_description_i);
///         // Degrees, such as 0, 90, 180, or 270.
///         let orientation: Option<i64> = queried_file.get_integer(orientation_i);
///     }
/// 
///     Ok(())
/// }
/// ```
#[sync_async(
    use(if_async) api_async::{Picker, PublicStorage};
    use(if_sync) api_sync::{Picker, PublicStorage};
)]
pub struct MediaStore<'a, R: tauri::Runtime> {
    #[cfg(target_os = "android")]
    pub(crate) handle: &'a tauri::plugin::PluginHandle<R>,

    #[cfg(not(target_os = "android"))]
    #[allow(unused)]
    pub(crate) handle: &'a tauri::AppHandle<R>,
}

#[cfg(target_os = "android")]
#[sync_async(
    use(if_sync) impls::SyncImpls as Impls;
    use(if_async) impls::AsyncImpls as Impls;
)]
impl<'a, R: tauri::Runtime> MediaStore<'a, R> {
    
    #[always_sync]
    fn impls(&self) -> Impls<'_, R> {
        Impls { handle: &self.handle }
    }
}

#[sync_async(
    use(if_async) api_async::{AndroidFs, Opener, Picker, PublicStorage, PrivateStorage, Helper};
    use(if_sync) api_sync::{AndroidFs, Opener, Picker, PublicStorage, PrivateStorage, Helper};
)]
impl<'a, R: tauri::Runtime> MediaStore<'a, R> {

    /// Requests the permission to read files from MediaStore 
    /// and returns the resulting state.
    ///
    /// On Android 10 (API level 29) or higher, 
    /// files created by the app itself in MediaStore, including by [`PublicStorage`],
    /// can be accessed without this permission.
    /// On Android 10,
    /// when accessing files using absolute paths and native APIs (such as [`std::fs`]),
    /// in addition to this permission, [`android:requestLegacyExternalStorage`](https://developer.android.com/training/data-storage/use-cases?hl=ja#opt-out-in-production-app)
    /// must be set to `true` in the app's manifest file to opt out of scoped storage on Android 10.
    /// 
    /// # Required
    /// Enable the feature that corresponds to the media your app needs to read from MediaStore:
    /// - **Image**: `media_store_image_read_permission`
    /// - **Video**: `media_store_video_read_permission`
    /// - **Audio**: `media_store_audio_read_permission`
    /// 
    /// Each of these features guarantees that the app can access at least the corresponding media type. 
    /// Note, however, that on Android 12 (API level 31) or lower,
    /// enabling just one of these features is enough to gain access to all media types, 
    /// because the legacy `READ_EXTERNAL_STORAGE` permission which covers every media and non media type 
    /// is used instead of the granular media permissions.
    /// 
    /// # Returned value
    /// The returned [`MediaStoreReadPermissionState`] indicates the app's permission state
    /// for the specified permissions.
    /// 
    /// - `Granted`:
    /// The permission has been granted, but the level of access depends on the current state
    ///  With "full access", all files of the target media can be accessed, 
    /// and requesting the permission again does not display a prompt. 
    /// With "partial access", only files selected by the user can be accessed,
    /// and requesting the permission again displays a prompt allowing the user to change the selection.
    /// [`MediaStore::check_partial_read_permission`] can be used to determine 
    /// whether the app currently has partial access. 
    /// Note that audio does not support partial access.
    /// 
    /// - `Denied`:
    /// The permission request was denied,
    /// but the prompt can be shown again on the next request unlike `PermanentlyDenied`.
    /// 
    /// - `PermanentlyDenied`:
    /// The permission request was denied.
    /// Unlike `Denied`, requesting the permission has no effect in this state:
    /// due to an Android limitation,
    /// the prompt is not shown and this value is always returned.
    /// To obtain the permission,
    /// the user has to open the app settings and grant it themselves.
    /// [`Helper::open_app_settings`]
    /// and [`Helper::show_open_app_settings_dialog`]
    /// can be used to guide the user there.
    /// This state is entered, for example, when the user taps "Don't ask again"
    /// on the prompt, or when the user denies the same permission twice in a row
    /// on Android 11 (API level 30) or higher.
    /// 
    /// # Support
    /// All Android versions supported by Tauri.
    #[maybe_async]
    pub fn request_read_permission(
        &self, 
        permission: MediaStoreReadPermissionType,
    ) -> Result<MediaStoreReadPermissionState> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().request_read_media_store_permission(permission).await
        }
    }

    /// Checks whether the app has permission to read files in MediaStore.
    /// 
    /// This includes both "partial access" and "full access".
    /// 
    /// See [`MediaStore::request_read_permission`] for details.
    #[maybe_async]
    pub fn check_read_permission(
        &self, 
        permission: MediaStoreReadPermissionType,
    ) -> Result<bool> {
        
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().check_read_media_store_permission(permission).await
        }
    }

    /// Checks whether the app has "partial access" permission to read files in MediaStore.
    /// 
    /// Returns `false` if the app has "full access" permission or does not have access permission.
    /// 
    /// See [`MediaStore::request_read_permission`] for details.
    #[maybe_async]
    pub fn check_partial_read_permission(
        &self,
        permission: MediaStoreReadPermissionType,
    ) -> Result<bool> {
        
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().check_partial_read_media_store_permission(permission).await
        }
    }

    /// Queries the MediaStore to retrieve metadata for a specific file.
    ///
    /// For usage examples, refer to [`MediaStore`] documentation.
    /// 
    /// # Parameters
    /// - `uri`:
    /// URI identifying the file to query.
    /// This URI must be either a URI returned by [`MediaStore::query_files`]
    /// or a URI of a file created using [`PublicStorage`].
    ///
    /// - `projection`: 
    /// Columns to retrieve for each media file, such as file names and image dimensions.
    /// See [MediaColumns](https://developer.android.com/reference/android/provider/MediaStore.MediaColumns),
    /// [FileColumns](https://developer.android.com/reference/android/provider/MediaStore.Files.FileColumns),
    /// [ImageColumns](https://developer.android.com/reference/android/provider/MediaStore.Images.ImageColumns),
    /// [VideoColumns](https://developer.android.com/reference/android/provider/MediaStore.Video.VideoColumns),
    /// [AudioColumns](https://developer.android.com/reference/android/provider/MediaStore.Audio.AudioColumns),
    /// and [DownloadColumns](https://developer.android.com/reference/android/provider/MediaStore.DownloadColumns).
    /// 
    /// # Support
    /// All Android versions supported by Tauri.
    #[maybe_async]
    pub fn query_file(
        &self,
        uri: &FsUri,
        projection: &[MediaStoreColumn<'_>],
    ) -> Result<MediaStoreQueriedFile> {

        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().query_media_store_file(uri, projection).await
        }
    }

    /// Queries the MediaStore to retrieve a list of files.
    /// 
    /// Files that the app does not have permission to access are not included in the list. 
    /// For information about permissions, see [`MediaStore::request_read_permission`].
    /// 
    /// For usage examples, refer to [`MediaStore`] documentation.
    /// 
    /// # Parameters
    /// - `table`: 
    /// MediaStore table to query,
    /// such as image, video, audio, file, download.
    /// The file table is a comprehensive table that includes all files contained in the other tables.
    /// The download table is only available on Android 10 (API level 29) or higher.
    /// 
    /// - `projection`: 
    /// Columns to retrieve for each media file, such as file names and image dimensions.
    /// See [MediaColumns](https://developer.android.com/reference/android/provider/MediaStore.MediaColumns),
    /// [FileColumns](https://developer.android.com/reference/android/provider/MediaStore.Files.FileColumns),
    /// [ImageColumns](https://developer.android.com/reference/android/provider/MediaStore.Images.ImageColumns),
    /// [VideoColumns](https://developer.android.com/reference/android/provider/MediaStore.Video.VideoColumns),
    /// [AudioColumns](https://developer.android.com/reference/android/provider/MediaStore.Audio.AudioColumns),
    /// and [DownloadColumns](https://developer.android.com/reference/android/provider/MediaStore.DownloadColumns).
    /// 
    /// - `options`:
    /// Optional settings.
    /// These options can be used to filter files and implement pagination;
    /// `selection`, `selection_args`, `sort_order`,
    /// `offset`, `limit`, `volume`, `match_pending`, `match_favorite`, `match_trashed`.
    /// See [`MediaStoreQueryFilesOptions`] for a description of each option.
    /// 
    /// # Support
    /// All Android versions supported by Tauri.
    #[maybe_async]
    pub fn query_files(
        &self,
        table: MediaStoreTable,
        projection: &[MediaStoreColumn<'_>],
        options: MediaStoreQueryFilesOptions<'_>,
    ) -> Result<Vec<MediaStoreQueriedFile>> {
        
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().query_media_store_files(
                table, 
                projection, 
                options.volume,
                options.selection, 
                options.selection_args, 
                options.sort_order,
                options.offset, 
                options.limit,
                options.match_pending,
                options.match_favorite,
                options.match_trashed,
            ).await
        }
    }

    #[maybe_async]
    pub fn get_primary_volume(&self) -> Result<Option<StorageVolume>> {
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().get_primary_storage_volume_if_available_for_media_store().await
        }
    }

    #[maybe_async]
    pub fn get_volumes(&self) -> Result<Vec<StorageVolume>> {
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls().get_available_storage_volumes_for_media_store().await
        }
    }
}