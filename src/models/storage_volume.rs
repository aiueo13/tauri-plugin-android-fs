use serde::{Deserialize, Serialize};
use crate::*;


#[derive(Debug, Clone, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageVolume {

    /// A user-visible description of the volume.  
    /// This can be determined by the manufacturer and is often localized according to the user’s language.
    ///
    /// e.g.
    /// - `Internal shared storage`
    /// - `SDCARD`
    /// - `SD card`
    /// - `Virtual SD card`
    pub description: String,

    /// Indicates whether this is primary storage volume. 
    /// A device always has one (and one only) primary storage volume. 
    pub is_primary: bool,

    /// Indicates whether this is physically removable.
    /// If `false`, this is device's built-in storage.
    pub is_removable: bool,

    /// Indicates whether thit is stable part of the device.
    /// 
    /// For example, a device’s built-in storage and physical media slots under protective covers are considered stable, 
    /// while USB flash drives connected to handheld devices are not.
    pub is_stable: bool,

    /// Indicates whether this is backed by private user data partition, 
    /// either internal storage or [adopted storage](https://source.android.com/docs/core/storage/adoptable).
    ///
    /// On most recent devices, the primary storage volume will often have this set to `true`.
    pub is_emulated: bool,

    /// Indicates whether this is readonly storage volume.
    ///
    /// e.g. SD card with readonly mode.
    /// 
    /// # Remark
    /// As far as I understand, this should never be `true` 
    /// when either `is_primary` or `is_emulated` is true, 
    /// or when `is_removable` is false, 
    /// but it might not be the case due to any issues or rare cases.
    pub is_readonly: bool,

    pub is_available_for_private_storage: bool,

    pub is_available_for_public_storage: bool,

    pub id: StorageVolumeId
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageVolumeId {
    pub(crate) top_directory_path: Option<std::path::PathBuf>,
    pub(crate) media_store_volume_name: Option<String>,
    pub(crate) private_data_dir_path: Option<std::path::PathBuf>,
    pub(crate) private_cache_dir_path: Option<std::path::PathBuf>,
    pub(crate) uuid: Option<String>,
}

#[allow(unused)]
impl StorageVolumeId {

    pub(crate) fn outside_private_dir_path(&self, dir: OutsidePrivateDir) -> Option<&std::path::PathBuf> {
        match dir {
            OutsidePrivateDir::Data => self.private_data_dir_path.as_ref(),
            OutsidePrivateDir::Cache => self.private_cache_dir_path.as_ref(),
        }
    }
}