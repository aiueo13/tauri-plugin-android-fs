use serde::{Deserialize, Serialize};


/// The application specific directory.  
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub enum PrivateDir {

    /// The application specific persistent-data directory.  
    /// 
    /// Files stored in this directory are included in [Android Auto Backup](https://developer.android.com/identity/data/autobackup).  
    /// 
    /// The system prevents other apps and user from accessing these locations. 
    /// In cases where the device is rooted or the user has special permissions, the user may be able to access this.   
    ///  
    /// These files will be deleted when the app is uninstalled and may also be deleted at the user’s request.  
    /// 
    /// e.g. `/data/user/0/{app-package-name}/files`
    /// 
    /// <https://developer.android.com/reference/android/content/Context#getFilesDir()>
    Data,

    /// The application specific cache directory.  
    /// 
    /// Files stored in this directory are **not** included in [Android Auto Backup](https://developer.android.com/identity/data/autobackup).  
    /// 
    /// The system prevents other apps and user from accessing these locations. 
    /// In cases where the device is rooted or the user has special permissions, the user may be able to access this.   
    /// 
    /// These files will be deleted when the app is uninstalled and may also be deleted at the user’s request. 
    ///
    /// In addition, the system will automatically delete files in this directory as disk space is needed elsewhere on the device. 
    /// But you should not rely on this. The cache should be explicitly cleared by yourself.
    /// 
    /// e.g. `/data/user/0/{app-package-name}/cache`
    /// 
    /// <https://developer.android.com/reference/android/content/Context#getCacheDir()>
    Cache,

    /// The application specific persistent-data directory.  
    /// 
    /// This is similar to [`PrivateDir::Data`].
    /// But files stored in this directory are **not** included in [Android Auto Backup](https://developer.android.com/identity/data/autobackup).  
    /// 
    /// The system prevents other apps and user from accessing these locations. 
    /// In cases where the device is rooted or the user has special permissions, the user may be able to access this.   
    ///  
    /// These files will be deleted when the app is uninstalled and may also be deleted at the user’s request.  
    /// 
    /// e.g. `/data/user/0/{app-package-name}/no_backup`
    /// 
    /// <https://developer.android.com/reference/android/content/Context#getNoBackupFilesDir()>
    NoBackupData,
}

/// The application specific directory.  
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub enum OutsidePrivateDir {

    /// The application specific persistent-data directory.  
    /// 
    /// These files will be deleted when the app is uninstalled and may also be deleted at the user’s request.  
    Data,
    
    /// The application specific cache directory.  
    /// 
    /// These files will be deleted when the app is uninstalled and may also be deleted at the user’s request. 
    Cache,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub enum PublicDir {
    
    #[serde(untagged)]
    Image(PublicImageDir),

    #[serde(untagged)]
    Video(PublicVideoDir),

    #[serde(untagged)]
    Audio(PublicAudioDir),

    #[serde(untagged)]
    GeneralPurpose(PublicGeneralPurposeDir),
}

/// Directory in which to place images that are available to other applications and users.  
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub enum PublicImageDir {

    /// e.g. `~/Pictures`
    Pictures,

    /// e.g. `~/DCIM`
    DCIM,
}

/// Directory in which to place videos that are available to other applications and users.  
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub enum PublicVideoDir {

	/// e.g. `~/Movies`
	Movies,

	/// e.g. `~/DCIM`
	DCIM,
}

/// Directory in which to place audios that are available to other applications and users.  
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub enum PublicAudioDir {

    /// e.g. `~/Music`
    Music,

    /// e.g. `~/Alarms`
    Alarms,

    /// This is not available on Android 9 (API level 28) and lower.  
    /// 
    /// e.g. `~/Audiobooks`  
    Audiobooks,

    /// e.g. `~/Notifications`
    Notifications,

    /// e.g. `~/Podcasts`
    Podcasts,

    /// e.g. `~/Ringtones`
    Ringtones,

    /// This is not available on Android 11 (API level 30) and lower.  
    /// 
    /// e.g. `~/Recordings`
    Recordings,
}

/// Directory in which to place files that are available to other applications and users.  
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub enum PublicGeneralPurposeDir {

    /// e.g. `~/Documents`
    Documents,

    /// e.g. `~/Download`  
    ///
    /// This is not the plural "Downloads", but the singular "Download".
    /// <https://developer.android.com/reference/android/os/Environment#DIRECTORY_DOWNLOADS>
    Download,
}

impl std::fmt::Display for PublicImageDir {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PublicImageDir::Pictures => write!(f, "Pictures"),
            PublicImageDir::DCIM => write!(f, "DCIM"),
        }
    }
}

impl std::fmt::Display for PublicVideoDir {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PublicVideoDir::Movies => write!(f, "Movies"),
            PublicVideoDir::DCIM => write!(f, "DCIM"),
        }
    }
}

impl std::fmt::Display for PublicAudioDir {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PublicAudioDir::Music => write!(f, "Music"),
            PublicAudioDir::Alarms => write!(f, "Alarms"),
            PublicAudioDir::Audiobooks => write!(f, "Audiobooks"),
            PublicAudioDir::Notifications => write!(f, "Notifications"),
            PublicAudioDir::Podcasts => write!(f, "Podcasts"),
            PublicAudioDir::Ringtones => write!(f, "Ringtones"),
            PublicAudioDir::Recordings => write!(f, "Recordings"),
        }
    }
}

impl std::fmt::Display for PublicGeneralPurposeDir {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PublicGeneralPurposeDir::Documents => write!(f, "Documents"),
            PublicGeneralPurposeDir::Download => write!(f, "Download"),
        }
    }
}

impl std::fmt::Display for PublicDir {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PublicDir::Image(p) => p.fmt(f),
            PublicDir::Video(p) => p.fmt(f),
            PublicDir::Audio(p) => p.fmt(f),
            PublicDir::GeneralPurpose(p) => p.fmt(f),
        }
    }
}

macro_rules! impl_into_pubdir {
    ($target: ident, $wrapper: ident) => {
        impl From<$target> for PublicDir {
            fn from(value: $target) -> Self {
                Self::$wrapper(value)
            }
        }
    };
}
impl_into_pubdir!(PublicImageDir, Image);
impl_into_pubdir!(PublicVideoDir, Video);
impl_into_pubdir!(PublicAudioDir, Audio);
impl_into_pubdir!(PublicGeneralPurposeDir, GeneralPurpose);