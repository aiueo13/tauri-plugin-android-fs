use sync_async::sync_async;
use crate::*;
use super::*;


#[sync_async]
impl<'a, R: tauri::Runtime> Impls<'a, R> {

    #[maybe_async]
    pub fn get_entry_name(&self, uri: &FileUri) -> Result<String> {
        impl_se!(struct Req<'a> { uri: &'a FileUri });
        impl_de!(struct Res { name: String });

        self.invoke::<Res>("getName", Req { uri })
            .await
            .map(|v| v.name)
    }

    #[maybe_async]
    pub fn get_entry_type(&self, uri: &FileUri) -> Result<EntryType> {
        impl_se!(struct Req<'a> { uri: &'a FileUri });
        impl_de!(struct Res { value: Option<String> });

        self.invoke::<Res>("getMimeType", Req { uri })
            .await
            .map(|v| match v.value {
                Some(mime_type) => EntryType::File { mime_type },
                None => EntryType::Dir,
            })
    }

    #[maybe_async]
    pub fn open_file(&self, uri: &FileUri, mode: FileAccessMode) -> Result<std::fs::File> {
        impl_se!(struct Req<'a> { uri: &'a FileUri, mode: &'a str });
        impl_de!(struct Res { fd: std::os::fd::RawFd });
    
        let mode = mode.to_mode();

        self.invoke::<Res>("getFileDescriptor", Req { uri, mode })
            .await
            .map(|v| {
                use std::os::fd::FromRawFd;
                unsafe { std::fs::File::from_raw_fd(v.fd) }
            })
    }

    #[maybe_async]
    pub fn open_file_with_fallback(
        &self, 
        uri: &FileUri, 
        candidate_modes: impl IntoIterator<Item = FileAccessMode>
    ) -> Result<(std::fs::File, FileAccessMode)> {

        impl_se!(struct Req<'a> { uri: &'a FileUri, modes: Vec<&'a str> });
        impl_de!(struct Res { fd: std::os::fd::RawFd, mode: String });
    
        let modes = candidate_modes.into_iter().map(|m| m.to_mode()).collect::<Vec<_>>();

        if modes.is_empty() {
            return Err(Error::with("candidate_modes must not be empty"));
        }

        self.invoke::<Res>("getFileDescriptorWithFallback", Req { uri, modes })
            .await
            .and_then(|v| FileAccessMode::from_mode(&v.mode).map(|m| (v.fd, m)))
            .map(|(fd, mode)| {
                let file = {
                    use std::os::fd::FromRawFd;
                    unsafe { std::fs::File::from_raw_fd(fd) }
                };
                (file, mode)
            })
    }

    #[maybe_async]
    pub fn copy_file_via_kotlin(
        &self, 
        src: &FileUri, 
        dest: &FileUri,
        buffer_size: Option<u32>,
    ) -> Result<()> {

        impl_se!(struct Req<'a> { src: &'a FileUri, dest: &'a FileUri, buffer_size: Option<u32> });
        impl_de!(struct Res;);

        if buffer_size.is_some_and(|s| s <= 0) {
            return Err(Error { msg: "buffer_size must be non zero".into() })
        }

        self.invoke::<Res>("copyFile", Req { src, dest, buffer_size })
            .await
            .map(|_| ())
            .map_err(Into::into)
    }

    #[maybe_async]
    pub fn rename_entry(&self, uri: &FileUri, new_name: impl AsRef<str>) -> Result<FileUri> {
        impl_se!(struct Req<'a> { uri: &'a FileUri, new_name: &'a str });

        let new_name = new_name.as_ref();

        self.invoke::<FileUri>("rename", Req { uri, new_name })
            .await
    }

    #[maybe_async]
    pub fn remove_file(&self, uri: &FileUri) -> Result<()> {
        impl_se!(struct Req<'a> { uri: &'a FileUri });
        impl_de!(struct Res;);
    
        self.invoke::<Res>("deleteFile", Req { uri })
            .await
            .map(|_| ())
    }

    #[maybe_async]
    pub fn remove_dir_if_empty(&self, uri: &FileUri) -> Result<()> {
        impl_se!(struct Req<'a> { uri: &'a FileUri });
        impl_de!(struct Res;);
        
        self.invoke::<Res>("deleteEmptyDir", Req { uri })
            .await
            .map(|_| ())
    }

    #[maybe_async]
    pub fn remove_dir_all(&self, uri: &FileUri) -> Result<()> {
        impl_se!(struct Req<'a> { uri: &'a FileUri });
        impl_de!(struct Res;);
        
        self.invoke::<Res>("deleteDirAll", Req { uri })
            .await
            .map(|_| ())
    }

    #[maybe_async]
    pub fn create_new_file(
        &self,
        dir: &FileUri, 
        relative_path: impl AsRef<std::path::Path>, 
        mime_type: Option<&str>
    ) -> Result<FileUri> {

        impl_se!(struct Req<'a> { dir: &'a FileUri, mime_type: Option<&'a str>, relative_path: &'a str });
        
        let relative_path = validate_relative_path(relative_path.as_ref())?;
        let relative_path = relative_path.to_string_lossy();
                
        self.invoke::<FileUri>("createFile", Req { dir, mime_type, relative_path: relative_path.as_ref() })
            .await
    }

    #[maybe_async]
    pub fn create_dir_all(
        &self,
        dir: &FileUri, 
        relative_path: impl AsRef<std::path::Path>, 
    ) -> Result<FileUri> {

        impl_se!(struct Req<'a> { dir: &'a FileUri,relative_path: &'a str });
        
        let relative_path = validate_relative_path(relative_path.as_ref())?;
        let relative_path = relative_path.to_string_lossy();
                
        self.invoke::<FileUri>("createDirAll", Req { dir, relative_path: relative_path.as_ref() })
            .await
    }

    #[maybe_async]
    pub fn read_dir_with_options(
        &self, 
        uri: &FileUri, 
        options: EntryOptions
    ) -> Result<impl Iterator<Item = OptionalEntry>> {
        
        impl_se!(struct Req<'a> { uri: &'a FileUri, options: Ops });
        impl_de!(struct Obj {
            uri: Option<FileUri>,
            mime_type: Option<String>,
            name: Option<String>,
            last_modified: Option<i64>,
            len: Option<i64>, 
        });
        impl_de!(struct Res { entries: Vec<Obj> });

        // OptionalEntry { mime_type } の値に関わらず
        // ファイルかフォルダかを知るために mime_type は常に使用する。
        impl_se!(struct Ops {
            uri: bool,
            name: bool,
            last_modified: bool,
            len: bool,
        });

        let need_mt = options.mime_type;
        let options = Ops {
            uri: options.uri,
            name: options.name,
            last_modified: options.last_modified,
            len: options.len,
        };

        use std::time::{UNIX_EPOCH, Duration};
    
        self.invoke::<Res>("readDir", Req { uri, options })
            .await
            .map(|v| v.entries.into_iter())
            .map(move |v| v.map(move |v| match v.mime_type {
                // ファイルの時は必ず Some(mime_type) になり、
                // フォルダの時にのみ None になる。
                Some(mime_type) => OptionalEntry::File {
                    uri: v.uri,
                    name: v.name,
                    last_modified: v.last_modified.map(|i| UNIX_EPOCH + Duration::from_millis(i as u64)),
                    len: v.len.map(|i| i as u64),
                    mime_type: if need_mt { Some(mime_type) } else { None },
                },
                None => OptionalEntry::Dir {
                    uri: v.uri,
                    name: v.name,
                    last_modified: v.last_modified.map(|i| UNIX_EPOCH + Duration::from_millis(i as u64)),
                }
            }))
    }

    #[maybe_async]
    pub fn take_persistable_uri_permission(&self, uri: &FileUri) -> Result<()> {
        impl_se!(struct Req<'a> { uri: &'a FileUri });
        impl_de!(struct Res;);

        self.invoke::<Res>("takePersistableUriPermission", Req { uri })
            .await
            .map(|_| ())
    }

    #[maybe_async]
    pub fn check_persisted_uri_permission(
        &self, 
        uri: &FileUri, 
        mode: PersistableAccessMode
    ) -> Result<bool> {
        
        impl_se!(struct Req<'a> { uri: &'a FileUri, mode: PersistableAccessMode });
        impl_de!(struct Res { value: bool });

        self.invoke::<Res>("checkPersistedUriPermission", Req { uri, mode })
            .await
            .map(|v| v.value)
    }

    #[maybe_async]
    pub fn get_all_persisted_uri_permissions(&self) -> Result<impl Iterator<Item = PersistedUriPermission>> {
        impl_de!(struct Obj { uri: FileUri, r: bool, w: bool, d: bool });
        impl_de!(struct Res { items: Vec<Obj> });
    
        self.invoke::<Res>("getAllPersistedUriPermissions", "")
            .await
            .map(|v| v.items.into_iter())
            .map(|v| v.map(|v| {
                let (uri, can_read, can_write) = (v.uri, v.r, v.w);
                match v.d {
                    true => PersistedUriPermission::Dir { uri, can_read, can_write },
                    false => PersistedUriPermission::File { uri, can_read, can_write }
                }
            }))
    }

    #[maybe_async]
    pub fn release_persisted_uri_permission(&self, uri: &FileUri) -> Result<()> {
        impl_se!(struct Req<'a> { uri: &'a FileUri });
        impl_de!(struct Res;);

        self.invoke::<Res>("releasePersistedUriPermission", Req { uri })
            .await
            .map(|_| ())
    }

    #[maybe_async]
    pub fn release_all_persisted_uri_permissions(&self) -> Result<()> {
        impl_de!(struct Res;);

        self.invoke::<Res>("releaseAllPersistedUriPermissions", "")
            .await
            .map(|_| ())
    }

    #[maybe_async]
    pub fn get_file_thumbnail_to_file(
        &self, 
        src: &FileUri,
        dest: &FileUri,
        preferred_size: Size,
        format: ImageFormat,
    ) -> Result<bool> {

        impl_se!(struct Req<'a> {
            src: &'a FileUri, 
            dest: &'a FileUri,
            format: &'a str,
            quality: u8,
            width: u32,
            height: u32,
        });
        impl_de!(struct Res { value: bool });

        let (quality, format) = match format {
            ImageFormat::Png => (1.0, "Png"),
            ImageFormat::Jpeg => (0.75, "Jpeg"),
            ImageFormat::Webp => (0.7, "Webp"),
            ImageFormat::JpegWith { quality } => (quality, "Jpeg"),
            ImageFormat::WebpWith { quality } => (quality, "Webp"),
        };
        let quality = (quality * 100.0).clamp(0.0, 100.0) as u8;
        let Size { width, height } = preferred_size;
        
        self.invoke::<Res>("getThumbnailToFile", Req { src, dest, format, quality, width, height })
            .await   
            .map(|v| v.value)
    }

    #[maybe_async]
    pub fn get_file_thumbnail_in_memory(
        &self, 
        uri: &FileUri,
        preferred_size: Size,
        format: ImageFormat,
    ) -> Result<Option<Vec<u8>>> {

        impl_se!(struct Req<'a> {
            uri: &'a FileUri, 
            format: &'a str,
            quality: u8,
            width: u32,
            height: u32,
        });
        impl_de!(struct Res { bytes: Option<String> });

        let (quality, format) = match format {
            ImageFormat::Png => (1.0, "Png"),
            ImageFormat::Jpeg => (0.75, "Jpeg"),
            ImageFormat::Webp => (0.7, "Webp"),
            ImageFormat::JpegWith { quality } => (quality, "Jpeg"),
            ImageFormat::WebpWith { quality } => (quality, "Webp"),
        };
        let quality = (quality * 100.0).clamp(0.0, 100.0) as u8;
        let Size { width, height } = preferred_size;
        
        let Some(bytes) = self.invoke::<Res>("getThumbnail", Req { uri, format, quality, width, height })
            .await
            .map(|v| v.bytes)? else {
                    
            return Ok(None)
        };
        if bytes.is_empty() {
            return Ok(None)
        }

        use base64::engine::Engine;
        let bytes = base64::engine::general_purpose::STANDARD.decode(bytes)?;
        Ok(Some(bytes))
    }

    #[maybe_async]
    pub fn check_media_store_volume_name_available(
        &self,
        media_store_volume_name: impl AsRef<str>,
    ) -> Result<bool> {

        impl_se!(struct Req<'a> { media_store_volume_name: &'a str });
        impl_de!(struct Res { value: bool });
            
        let media_store_volume_name = media_store_volume_name.as_ref();
            
        self.invoke::<Res>("checkMediaStoreVolumeNameAvailable", Req { media_store_volume_name })
            .await
            .map(|v| v.value)
    }

    #[maybe_async]
    pub fn check_storage_volume_available_by_path(
        &self,
        path: impl AsRef<std::path::Path>,
    ) -> Result<bool> {

        impl_se!(struct Req<'a> { path: &'a std::path::Path });
        impl_de!(struct Res { value: bool });

        let path = path.as_ref();

        self.invoke::<Res>("checkStorageVolumeAvailableByPath", Req { path })
            .await
            .map(|v| v.value)
    }

    #[maybe_async]
    pub fn get_available_storage_volumes(&self) -> Result<Vec<StorageVolume>> {
        impl_de!(struct Res { volumes: Vec<StorageVolume> });

        let mut volumes = self.invoke::<Res>("getAvailableStorageVolumes", "")
            .await
            .map(|v| v.volumes)?;

        // primary volume を先頭にする。他はそのままの順序
        volumes.sort_by(|a, b| b.is_primary.cmp(&a.is_primary));

        Ok(volumes)
    }

    #[maybe_async]
    pub fn get_primary_storage_volume_if_available(&self) -> Result<Option<StorageVolume>> {
        impl_de!(struct Res { volume: Option<StorageVolume> });

        self.invoke::<Res>("getPrimaryStorageVolumeIfAvailable", "")
            .await
            .map(|v| v.volume)
    }

    #[always_sync]
    pub fn consts(&self) -> Result<&'static Consts> {
        get_or_init_const(|| self.invoke_sync::<Consts>("getConsts", ""))
    }

    #[always_sync]
    pub fn internal_private_dir_path(
        &self, 
        dir: PrivateDir
    ) -> Result<&'static std::path::PathBuf> {

        let paths = get_or_init_internal_private_dir_paths(
            || self.invoke_sync::<InternalPrivateDirPaths>("getPrivateBaseDirAbsolutePaths", "")
        )?;

        Ok(match dir {
            PrivateDir::Data => &paths.data,
            PrivateDir::Cache => &paths.cache,
            PrivateDir::NoBackupData => &paths.no_backup_data,
        })
    }

    #[maybe_async]
    pub fn set_file_pending_in_public_storage(
        &self,
        uri: &FileUri,
        is_pending: bool
    ) -> Result<()> {

        impl_se!(struct Req<'a> { uri: &'a FileUri, pending: bool });
        impl_de!(struct Res;);

        self.invoke::<Res>("setMediaStoreFilePending", Req { uri, pending: is_pending })
            .await
            .map(|_| ())
    }

    #[maybe_async]
    pub fn create_new_file_in_public_storage(
        &self,
        volume_id: Option<&StorageVolumeId>,
        base_dir: impl Into<PublicDir>,
        relative_path: impl AsRef<std::path::Path>, 
        mime_type: Option<&str>,
        is_pending: bool,
    ) -> Result<FileUri> {

        impl_se!(struct Req<'a> { 
            media_store_volume_name: &'a str, 
            relative_path: std::path::PathBuf, 
            mime_type: Option<&'a str>,
            pending: bool
        });
        impl_de!(struct Res { uri: FileUri });

        self.requires(api_level::ANDROID_10)?;

        let consts = self.consts()?;
        let relative_path = {
            let mut p = std::path::PathBuf::new();
            p.push(consts.public_dir_name(base_dir)?);
            p.push(validate_relative_path(relative_path.as_ref())?);
            p
        };
        let media_store_volume_name = volume_id
            .map(|v| v.media_store_volume_name.as_ref())
            .unwrap_or(consts.media_store_primary_volume_name.as_ref())
            .ok_or_else(|| Error::with("The storage volume is not available for PublicStorage"))?;

        self.invoke::<Res>("createNewMediaStoreFile", Req {
                media_store_volume_name, 
                relative_path,
                mime_type,
                pending: is_pending
            })
            .await
            .map(|v| v.uri)
    }

    #[maybe_async]
    pub fn show_pick_file_dialog(
        &self,
        initial_location: Option<&FileUri>,
        mime_types: &[&str],
        multiple: bool,
    ) -> Result<Vec<FileUri>> {

        impl_se!(struct Req<'a> { 
            mime_types: &'a [&'a str],
                multiple: bool,
                initial_location: Option<&'a FileUri>
        });
        impl_de!(struct Res { uris: Vec<FileUri> });
    
        let result = self.invoke::<Res>("showOpenFileDialog", Req { mime_types, multiple, initial_location })
            .await
            .map(|v| v.uris);

        // intent からの結果を取得してからすぐ frontend 側に戻ると
        // その frontend 側の関数の呼び出しがなぜか終了しないことが偶にある。
        // よって遅延を強制的に追加してこれを回避する。
        // https://github.com/aiueo13/tauri-plugin-android-fs/issues/1
        // 
        // TODO: 
        // - sleep await を用いる
        // - kotlin側で invoke.resolve ではなく channel を用いて結果を返すことを試してみる
        std::thread::sleep(std::time::Duration::from_millis(200));

        result
    }

    #[maybe_async]
    pub fn show_pick_visual_media_dialog(
        &self,
        target: VisualMediaTarget<'_>,
        multiple: bool,
    ) -> Result<Vec<FileUri>> {

        impl_se!(struct Req<'a> { multiple: bool, target: &'a str });
        impl_de!(struct Res { uris: Vec<FileUri> });

        let target = match target {
            VisualMediaTarget::ImageOnly => "image/*",
            VisualMediaTarget::VideoOnly => "video/*",
            VisualMediaTarget::ImageAndVideo => "*/*",
            VisualMediaTarget::ImageOrVideo { mime_type } => {
                let is_image_or_video = mime_type.starts_with("image/") || mime_type.starts_with("video/");
                if !is_image_or_video {
                    return Err(Error::with(format!("mime_type must be an image or a video, but {mime_type}")))
                }
                    
                mime_type
            }
        };
    
        let result = self.invoke::<Res>("showOpenVisualMediaDialog", Req { multiple, target })
            .await
            .map(|v| v.uris);

        // show_pick_file_dialog 内のコメントを参照
        std::thread::sleep(std::time::Duration::from_millis(200));

        result
    }

    #[maybe_async]
    pub fn show_pick_content_dialog(
        &self,
        mime_types: &[&str],
        multiple: bool
    ) -> Result<Vec<FileUri>> {

        impl_se!(struct Req<'a> { mime_types: &'a [&'a str], multiple: bool });
        impl_de!(struct Res { uris: Vec<FileUri> });

        let result = self.invoke::<Res>("showOpenContentDialog", Req { mime_types, multiple })
            .await
            .map(|v| v.uris);

        // show_pick_file_dialog 内のコメントを参照
        std::thread::sleep(std::time::Duration::from_millis(200));

        result
    }

    #[maybe_async]
    pub fn show_pick_dir_dialog(
        &self,
        initial_location: Option<&FileUri>,
    ) -> Result<Option<FileUri>> {

        impl_se!(struct Req<'a> { initial_location: Option<&'a FileUri> });
        impl_de!(struct Res { uri: Option<FileUri> });

        let result = self.invoke::<Res>("showManageDirDialog", Req { initial_location })
            .await
            .map(|v| v.uri);

        // show_pick_file_dialog 内のコメントを参照
        std::thread::sleep(std::time::Duration::from_millis(200));

        result
    }

    #[maybe_async]
    pub fn show_save_file_dialog(
        &self,
        initial_location: Option<&FileUri>,
        initial_file_name: impl AsRef<str>,
        mime_type: Option<&str>,
    ) -> Result<Option<FileUri>> {
        
        impl_se!(struct Req<'a> {
            initial_file_name: &'a str, 
            mime_type: Option<&'a str>, 
            initial_location: Option<&'a FileUri> 
        });
        impl_de!(struct Res { uri: Option<FileUri> });
    
        let initial_file_name = initial_file_name.as_ref();
        
        let result = self.invoke::<Res>("showSaveFileDialog", Req { initial_file_name, mime_type, initial_location })
            .await
            .map(|v| v.uri);

        // show_pick_file_dialog 内のコメントを参照
        std::thread::sleep(std::time::Duration::from_millis(200));

        result
    }

    #[maybe_async]
    pub fn is_visual_media_picker_available(&self) -> Result<bool> {
        impl_de!(struct Res { value: bool });

        self.invoke::<Res>("isVisualMediaDialogAvailable", "")
            .await
            .map(|v| v.value)
    }

    #[maybe_async]
    pub fn show_share_file_app_chooser<'b>(
        &self, 
        uris: impl IntoIterator<Item = &'b FileUri>, 
    ) -> Result<()> {

        impl_se!(struct Req<'a> { uris: Vec<&'a FileUri>, common_mime_type: Option<&'a str>, use_app_chooser: bool, exclude_self_from_app_chooser: bool });
        impl_de!(struct Res;);

        let use_app_chooser = true;
        let exclude_self_from_app_chooser = true;
        let common_mime_type = None;
        let uris = uris.into_iter().collect::<Vec<_>>();

        self.invoke::<Res>("shareFiles", Req { uris, common_mime_type, use_app_chooser, exclude_self_from_app_chooser })
            .await
            .map(|_| ())
    }

    #[maybe_async]
    pub fn show_open_file_app_chooser(
        &self, 
        uri: &FileUri,
    ) -> Result<()> {

        impl_se!(struct Req<'a> { uri: &'a FileUri, mime_type: Option<&'a str>, use_app_chooser: bool, exclude_self_from_app_chooser: bool });
        impl_de!(struct Res;);

        let use_app_chooser = true;
        let exclude_self_from_app_chooser = true;
        let mime_type = None;
    
        self.invoke::<Res>("viewFile", Req { uri, mime_type, use_app_chooser, exclude_self_from_app_chooser })
            .await
            .map(|_| ())
    }

    #[maybe_async]
    pub fn show_open_dir_app_chooser(
        &self, 
        uri: &FileUri,
    ) -> Result<()> {

        impl_se!(struct Req<'a> { uri: &'a FileUri, use_app_chooser: bool, exclude_self_from_app_chooser: bool });
        impl_de!(struct Res;);

        let use_app_chooser = true;
        let exclude_self_from_app_chooser = true;
    
        self.invoke::<Res>("viewDir", Req { uri, use_app_chooser, exclude_self_from_app_chooser })
            .await
            .map(|_| ())
    }

    #[maybe_async]
    pub fn show_edit_file_app_chooser(
        &self, 
        uri: &FileUri,
    ) -> Result<()> {

        impl_se!(struct Req<'a> { uri: &'a FileUri, mime_type: Option<&'a str>, use_app_chooser: bool, exclude_self_from_app_chooser: bool });
        impl_de!(struct Res;);

        let use_app_chooser = true;
        let exclude_self_from_app_chooser = true;
        let mime_type = None;
    
        self.invoke::<Res>("editFile", Req { uri, mime_type, use_app_chooser, exclude_self_from_app_chooser })
            .await
            .map(|_| ())
    }
}

get_or_init!(get_or_init_const, Consts);
get_or_init!(get_or_init_internal_private_dir_paths, InternalPrivateDirPaths);

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct InternalPrivateDirPaths {
    data: std::path::PathBuf, 
    cache: std::path::PathBuf, 
    no_backup_data: std::path::PathBuf, 
}

/// アプリ起動中に変更されることのない値
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Consts {
    pub build_version_sdk_int: i32,

    /// Android 10 (API level 29) 以上で有効
    pub media_store_primary_volume_name: Option<String>,

    pub env_dir_pictures: String,
    pub env_dir_dcim: String,
    pub env_dir_movies: String,
    pub env_dir_music: String,
    pub env_dir_alarms: String,
    pub env_dir_notifications: String,
    pub env_dir_podcasts: String,
    pub env_dir_ringtones: String,
    pub env_dir_documents: String,
    pub env_dir_download: String,

    /// Android 10 (API level 29) 以上で有効
    pub env_dir_audiobooks: Option<String>,

    /// Android 12 (API level 31) 以上で有効
    pub env_dir_recordings: Option<String>,
}

impl Consts {

    pub fn public_dir_name(&self, dir: impl Into<PublicDir>) -> Result<&str> {
        Ok(match dir.into() {
            PublicDir::Image(dir) => match dir {
                PublicImageDir::Pictures => &self.env_dir_pictures,
                PublicImageDir::DCIM => &self.env_dir_dcim,
            },
            PublicDir::Video(dir) => match dir {
                PublicVideoDir::Movies => &self.env_dir_movies,
                PublicVideoDir::DCIM => &self.env_dir_dcim,
            },
            PublicDir::Audio(dir) => match dir  {
                PublicAudioDir::Music => &self.env_dir_music,
                PublicAudioDir::Alarms => &self.env_dir_alarms,
                PublicAudioDir::Notifications => &self.env_dir_notifications,
                PublicAudioDir::Podcasts => &self.env_dir_podcasts,
                PublicAudioDir::Ringtones => &self.env_dir_ringtones,
                PublicAudioDir::Recordings => self.env_dir_recordings.as_ref().ok_or_else(|| Error { msg: "requires API level 31 or higher".into() })?,
                PublicAudioDir::Audiobooks => self.env_dir_audiobooks.as_ref().ok_or_else(|| Error { msg: "requires API level 29 or higher".into() })?,
            },
            PublicDir::GeneralPurpose(dir) => match dir {
                PublicGeneralPurposeDir::Documents => &self.env_dir_documents,
                PublicGeneralPurposeDir::Download => &self.env_dir_download,
            }
        })
    }
}