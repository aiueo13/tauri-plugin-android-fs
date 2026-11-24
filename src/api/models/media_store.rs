use std::borrow::Cow;
use serde::{Deserialize, Serialize};
use super::*;


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MediaStoreReadPermissionType {
    Image,
    Video,
    ImageAndVideo,
    Audio,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MediaStoreReadPermissionState {

    /// The permission has been granted.
    /// 
    /// This state has two sub-states:
    /// 
    /// - **Full access**:
    /// all files of the target media can be accessed.
    /// Requesting the permission again in this state does not show the prompt,
    /// and `Granted` is always returned.
    /// 
    /// - **Partial access**:
    /// only the files selected by the user can be accessed.
    /// Requesting the permission again in this state shows the prompt,
    /// letting the user choose which files to allow.
    /// [`MediaStore::check_partial_read_permission`](crate::api::api_async::MediaStore::check_partial_read_permission) can be used to check
    /// whether the app is currently in this state.
    ///
    /// Note that audio does not support partial access;
    /// only full access is available for it.
    Granted,

    /// The permission request was denied,
    /// but the prompt can be shown again on the next request
    /// unlike [`PermanentlyDenied`](Self::PermanentlyDenied).
    Denied,

    /// The permission request was denied.
    /// 
    /// Unlike [`Denied`](Self::Denied), requesting the permission has no effect in this state:
    /// due to an Android limitation,
    /// the prompt is not shown and this value is always returned.
    /// To obtain the permission,
    /// the user has to open the app settings and grant it themselves.
    /// [`Helper::open_app_settings`](crate::api::api_async::Helper::open_app_settings) 
    /// and [`Helper::show_open_app_settings_dialog`](crate::api::api_async::Helper::show_open_app_settings_dialog)
    /// can be used to guide the user there.
    /// 
    /// This state is entered, for example, when the user taps "Don't ask again"
    /// on the prompt, or when the user denies the same permission twice in a row
    /// on Android 11 (API level 30) or higher.
    PermanentlyDenied,
}

impl MediaStoreReadPermissionState {

    pub fn is_granted(&self) -> bool {
        matches!(self, Self::Granted)
    }

    pub fn is_permanently_denied(&self) -> bool {
        matches!(self, Self::PermanentlyDenied)
    }

    // is_denied は PermanentlyDenied の存在でややこしくなるので提供しない。
    // `!self.is_granted()` の方が語弊がなく明示的。
}

#[non_exhaustive]
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum MediaStoreColumn<'a> {
    String { name: Cow<'a, str>, min_api_level: Option<i32> }, 
    Integer { name: Cow<'a, str>, min_api_level: Option<i32> }, 
    Float { name: Cow<'a, str>, min_api_level: Option<i32> }, 
    Blob { name: Cow<'a, str>, min_api_level: Option<i32> }, 
}

impl<'a> MediaStoreColumn<'a> {

    pub fn string(column_name: impl Into<Cow<'a, str>>) -> Self {
        Self::String { name: column_name.into(), min_api_level: None }
    }

    pub fn integer(column_name: impl Into<Cow<'a, str>>) -> Self {
        Self::Integer { name: column_name.into(), min_api_level: None }
    }

    pub fn float(column_name: impl Into<Cow<'a, str>>) -> Self {
        Self::Float { name: column_name.into(), min_api_level: None }
    }

    pub fn blob(column_name: impl Into<Cow<'a, str>>) -> Self {
        Self::Blob { name: column_name.into(), min_api_level: None }
    }

    /// Specifies that this column should only be queried on Android versions with an API level
    /// equal to or higher than the specified minimum API level. 
    /// 
    /// On Android versions below the minimum API level, 
    /// the corresponding column value is treated as null.
    pub fn since_api_level(self, min_api_level: i32) -> Self {
        let min_api_level = Some(min_api_level);
        match self {
            Self::String { name, .. } => Self::String { name, min_api_level },
            Self::Integer { name, .. } => Self::Integer { name, min_api_level },
            Self::Float { name, .. } => Self::Float { name, min_api_level },
            Self::Blob { name, .. } => Self::Blob { name, min_api_level },
        }
    }

    pub fn index_of(
        column_name: impl AsRef<str>,
        projection: &[MediaStoreColumn<'_>],
    ) -> Result<usize> {

        let column_name = column_name.as_ref();
        for i in 0..projection.len() {
            let column = &projection[i];
            if column.name() == column_name {
                return Ok(i)
            }
        }

        Err(Error::with(format!("missing column: {column_name}")))
    }

    pub(crate) fn name<'b>(&'b self) -> &'b str {
        match self {
            Self::String { name, .. } => name,
            Self::Integer { name, .. } => name,
            Self::Float { name, .. } => name,
            Self::Blob { name, .. } => name,
        }
    }

    pub(crate) fn type_name(&self) -> &'static str {
        match self {
            Self::String { .. } => "string",
            Self::Integer { .. } => "integer",
            Self::Float { .. } => "float",
            Self::Blob { .. } => "blob",
        }
    }

    pub(crate) fn min_api_level(&self) -> i32 {
        match self {
            Self::String { min_api_level, .. } => min_api_level.unwrap_or(1),
            Self::Integer { min_api_level, .. } => min_api_level.unwrap_or(1),
            Self::Float { min_api_level, .. } => min_api_level.unwrap_or(1),
            Self::Blob { min_api_level, .. } => min_api_level.unwrap_or(1),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MediaStoreStorageVolume<'a> {
    All,
    Primary,
    Specific(&'a StorageVolumeId),
}

impl<'a> MediaStoreStorageVolume<'a> {

    pub(crate) fn as_str_for_cmd_arg(&self) -> String {
        match self {
            MediaStoreStorageVolume::All => "all".into(),
            MediaStoreStorageVolume::Primary => "primary".into(),
            MediaStoreStorageVolume::Specific(storage_volume_id) => {
                if let Some(volume_name) = &storage_volume_id.media_store_volume_name {
                    format!("specific:{volume_name}")
                }
                else {
                    MediaStoreStorageVolume::Primary.as_str_for_cmd_arg()
                }
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MediaStoreTable {
    File,
    Image,
    Video,
    Audio,
    Download,
}

impl MediaStoreTable {

    pub(crate) fn as_str_for_cmd_arg(&self) -> &'static str {
        match self {
            MediaStoreTable::File => "file",
            MediaStoreTable::Image => "image",
            MediaStoreTable::Video => "video",
            MediaStoreTable::Audio => "audio",
            MediaStoreTable::Download => "download",
        }
    }
}

#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MediaStoreMatchMode {

    /// Includes items that match the specified criteria.
    Include,

    /// Excludes items that match the specified criteria.
    Exclude,

    /// Includes only items that match the specified criteria.
    Only,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MediaStoreQueryFilesOptions<'a> {

    /// SQL WHERE clause used to filter the queried files.
    /// 
    /// User-supplied values must not be directly included in the selection string, 
    /// as doing so can expose the application to SQL injection attacks.
    /// Instead, ? placeholders should be used to represent values in the selection string, 
    /// with the corresponding values provided through `selection_args`.
    pub selection: Option<&'a str>,

    /// Arguments to replace ? placeholders in `selection`.
    pub selection_args: Option<&'a [String]>,

    /// SQL ORDER BY clause used to specify the sort order of the queried files.
    pub sort_order: Option<&'a str>,

    /// Number of matching files to skip before returning results.
    pub offset: Option<u64>,

    /// Maximum number of files to return.
    pub limit: Option<u64>,

    /// Storage volume to query (e.g., internal storage, an SD card, or a USB drive).
    /// 
    /// If `None` is specified, [`MediaStoreStorageVolume::All`] is used.
    pub volume: Option<MediaStoreStorageVolume<'a>>,

    /// Option specifying how files with pending status are matched when querying the MediaStore.
    /// 
    /// If `None` is specified, [`MediaStoreMatchMode::Exclude`] is used.
    /// 
    /// This parameter is supported on Android 11 (API level 30) or higher.
    /// If unsupported, it is ignored.
    pub match_pending: Option<MediaStoreMatchMode>,

    /// Option specifying how files with favorite status are matched when querying the MediaStore.
    /// 
    /// If `None` is specified, [`MediaStoreMatchMode::Include`] is used.
    /// 
    /// This parameter is supported on Android 11 (API level 30) or higher.
    /// If unsupported, it is ignored.
    pub match_favorite: Option<MediaStoreMatchMode>,

    /// Option specifying how files with trashed status are matched when querying the MediaStore.
    /// 
    /// If `None` is specified, [`MediaStoreMatchMode::Exclude`] is used.
    /// 
    /// This parameter is supported on Android 11 (API level 30) or higher.
    /// If unsupported, it is ignored.
    pub match_trashed: Option<MediaStoreMatchMode>,
}

/// File metadata returned by a MediaStore query.
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaStoreQueriedFile {

    /// The values of the queried columns.
    ///
    /// The values are stored in the same order as the columns specified
    /// in the projection parameter of the query.
    /// If a column value is null,
    /// the corresponding value is [`serde_json::Value::Null`].
    /// 
    /// Instead of using this value directly, 
    /// functions such as [`MediaStoreQueriedFile::get_integer`] and [`MediaStoreQueriedFile::get_string`]
    /// can be used to retrieve a column value by specifying the column index.
    pub column_values: Vec<serde_json::Value>,

    /// The MediaStore content URI of the file.
    pub uri: FsUri,
}

impl MediaStoreQueriedFile {

    pub fn get_required_string(&self, column_index: usize) -> Result<String> {
        self.get_string(column_index).ok_or_else(|| Error::with("Missing or invalid MediaStore column"))
    }

    pub fn get_string(&self, column_index: usize) -> Option<String> {
        self.column_values
            .get(column_index)
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned)
    }

    pub fn get_required_integer(&self, column_index: usize) -> Result<i64> {
        self.get_integer(column_index).ok_or_else(|| Error::with("Missing or invalid MediaStore column"))
    }

    pub fn get_integer(&self, column_index: usize) -> Option<i64> {
        self.column_values
            .get(column_index)
            .and_then(serde_json::Value::as_i64)
    }

    pub fn get_required_integer_as_usize(&self, column_index: usize) -> Result<usize> {
        self.get_integer_as_usize(column_index).ok_or_else(|| Error::with("Missing or invalid MediaStore column"))
    }

    pub fn get_integer_as_usize(&self, column_index: usize) -> Option<usize> {
        self.column_values
            .get(column_index)
            .and_then(serde_json::Value::as_u64)
            .and_then(|v| usize::try_from(v).ok())
    }

    pub fn get_required_integer_as_isize(&self, column_index: usize) -> Result<isize> {
        self.get_integer_as_isize(column_index).ok_or_else(|| Error::with("Missing or invalid MediaStore column"))
    }

    pub fn get_integer_as_isize(&self, column_index: usize) -> Option<isize> {
        self.column_values
            .get(column_index)
            .and_then(serde_json::Value::as_i64)
            .and_then(|v| isize::try_from(v).ok())
    }

    pub fn get_required_integer_as_u64(&self, column_index: usize) -> Result<u64> {
        self.get_integer_as_u64(column_index).ok_or_else(|| Error::with("Missing or invalid MediaStore column"))
    }

    pub fn get_integer_as_u64(&self, column_index: usize) -> Option<u64> {
        self.column_values
            .get(column_index)
            .and_then(serde_json::Value::as_u64)
    }

    pub fn get_required_integer_as_u32(&self, column_index: usize) -> Result<u32> {
        self.get_integer_as_u32(column_index).ok_or_else(|| Error::with("Missing or invalid MediaStore column"))
    }

    pub fn get_integer_as_u32(&self, column_index: usize) -> Option<u32> {
        self.column_values
            .get(column_index)
            .and_then(serde_json::Value::as_u64)
            .and_then(|value| u32::try_from(value).ok())
    }

    pub fn get_required_integer_as_i32(&self, column_index: usize) -> Result<i32> {
        self.get_integer_as_i32(column_index).ok_or_else(|| Error::with("Missing or invalid MediaStore column"))
    }

    pub fn get_integer_as_i32(&self, column_index: usize) -> Option<i32> {
        self.column_values
            .get(column_index)
            .and_then(serde_json::Value::as_i64)
            .and_then(|value| i32::try_from(value).ok())
    }

    pub fn get_required_float_as_f32(&self, column_index: usize) -> Result<f32> {
        self.get_float_as_f32(column_index).ok_or_else(|| Error::with("Missing or invalid MediaStore column"))
    }

    pub fn get_float_as_f32(&self, column_index: usize) -> Option<f32> {
        self.column_values
            .get(column_index)
            .and_then(serde_json::Value::as_f64)
            .map(|value| value as f32)
    }

    pub fn get_required_float(&self, column_index: usize) -> Result<f64> {
        self.get_float(column_index).ok_or_else(|| Error::with("Missing or invalid MediaStore column"))
    }

    pub fn get_float(&self, column_index: usize) -> Option<f64> {
        self.column_values
            .get(column_index)
            .and_then(serde_json::Value::as_f64)
    }


    // TODO: メジャーアップデートで削除する
    #[deprecated = "Use `get_required_integer_as_i32` instead."]
    pub fn get_required_i32(&self, column_index: usize) -> Result<i32> {
        self.get_required_integer_as_i32(column_index)
    }
}