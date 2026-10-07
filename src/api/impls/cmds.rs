macro_rules! impl_se {
    ($t:item) => {
        #[derive(serde::Serialize)]
        #[serde(rename_all = "camelCase")]
        $t
    };
}

macro_rules! impl_de {
    ($t:item) => {
        #[derive(serde::Deserialize)]
        #[serde(rename_all = "camelCase")]
        $t
    };
}

macro_rules! fn_get_or_init {
    ($name:ident, $T:ty) => {
        fn $name(init: impl FnOnce() -> Result<$T>) -> Result<&'static $T> {
            static VALUE: std::sync::OnceLock<$T> = std::sync::OnceLock::new();

            Ok(match VALUE.get() {
                Some(value) => value,
                None => {
                    VALUE.set(init()?).ok();
                    VALUE.get().expect("Should call 'set' before 'get'")
                }
            })
        }
    };
}

use super::CUSTOM_FILE_PROVIDERS;
use crate::*;
use serde::{de::DeserializeOwned, Serialize};
use std::io::{Read, Write};
use std::sync::LazyLock;
use sync_async::sync_async;

#[sync_async]
pub struct Impls<'a, R: tauri::Runtime> {
    pub handle: &'a tauri::plugin::PluginHandle<R>,
}

#[sync_async]
impl<'a, R: tauri::Runtime> Impls<'a, R> {

    #[maybe_async]
    pub(super) fn invoke<D: DeserializeOwned>(
        &self,
        command: &str,
        payload: impl Serialize,
    ) -> Result<D> {

        #[if_sync] {
            self.handle
                .run_mobile_plugin(command, payload)
                .map_err(Into::into)
        }
        #[if_async] {
            self.handle
                .run_mobile_plugin_async(command, payload)
                .await
                .map_err(Into::into)
        }
    }

    #[always_sync]
    pub(super) fn invoke_sync<D: DeserializeOwned>(
        &self,
        command: &str,
        payload: impl Serialize,
    ) -> Result<D> {

        self.handle
            .run_mobile_plugin(command, payload)
            .map_err(Into::into)
    }
}

pub fn take_provider_id_and_custom_file_path(
    uri: impl AsRef<str>,
) -> std::io::Result<(i32, String)> {

    let mut uri = uri.as_ref();
    if !uri.starts_with("content://") {
        return Err(std::io::Error::other("invalid uri scheme"));
    }
    uri = uri.trim_start_matches("content://");

    let mut components = uri.splitn(4, "/");
    components.next();
    let app_process_id = components.next().ok_or_else(|| std::io::Error::other("invalid uri"))?;
    let provider_id = components.next().ok_or_else(|| std::io::Error::other("invalid uri"))?;
    let encoded_path = components.next().ok_or_else(|| std::io::Error::other("invalid uri"))?;

    // 古い URI を防ぐ
    if app_process_id != &*APP_PROCESS_INSTANCE_ID {
        return Err(std::io::Error::other("invalid app process id"));
    }

    let provider_id = provider_id
        .parse()
        .map_err(|_| std::io::Error::other("invalid provider id format"))?;

    let path = decode_android_uri_component(encoded_path);
    Ok((provider_id, path))
}

#[sync_async(
    use(if_async) async_utils::{run_blocking, sleep};
    use(if_sync) sync_utils::{run_blocking, sleep};
)]
impl<'a, R: tauri::Runtime> Impls<'a, R> {

    #[always_sync]
    pub fn build_custom_file_uri(
        &self,
        provider_name: impl AsRef<str>,
        path: impl AsRef<str>,
    ) -> Result<FsUri> {

        if self.api_level()? < api_level::ANDROID_8 {
            return Err(Error::with("unsupported Android 7 or lower"));
        }

        let provider_name = provider_name.as_ref();
        let Some(file_provider_id) = CUSTOM_FILE_PROVIDERS.get_provider_id_from_name(provider_name) else {
            return Err(Error::with(format!("provider not found: {provider_name}")));
        };

        let app_id = &self.consts()?.app_id;
        let app_process_instance_id = APP_PROCESS_INSTANCE_ID.as_str();
        let encoded_path = encode_android_uri_component(path);

        Ok(FsUri {
            uri: format!("content://{app_id}.tpafs-custom-file-provider/{app_process_instance_id}/{file_provider_id}/{encoded_path}"),
            document_top_tree_uri: None,
        })
    }

    #[always_sync]
    pub fn get_uri_for_file_path(
        &self,
        path: impl AsRef<std::path::Path>,
        mime_type: Option<&str>,
    ) -> Result<FsUri> {

        if self.api_level()? < api_level::ANDROID_8 {
            return Err(Error::with("unsupported Android 7 or lower"));
        }

        let path = path.as_ref();
        let mime_type = match mime_type {
            Some(mime_type) => Some(mime_type.to_string()),
            None => {
                if let Some(ext) = path.extension().map(|e| e.to_string_lossy()) {
                    let impls = SyncImpls { handle: &self.handle };
                    impls.get_mime_type_from_extension(ext)?
                }
                else {
                    None
                }
            }
        };
        let path = path.to_string_lossy();
        let mime_type = mime_type.as_deref().unwrap_or("application/octet-stream");

        self.build_custom_file_uri(
            &*FILE_PROVIDER_NAME, 
            format!("{path}/TpafsMimeType={mime_type}")
        )
    }

    #[maybe_async]
    pub fn get_entry_name(&self, uri: &FsUri) -> Result<String> {
        impl_se!(struct Req<'a> {
            uri: &'a FsUri,
        });
        impl_de!(struct Res {
            name: String,
        });

        self.invoke::<Res>("getName", Req { uri })
            .await
            .map(|v| v.name)
    }

    #[maybe_async]
    pub fn get_entry_type(&self, uri: &FsUri) -> Result<EntryType> {
        impl_se!(
            struct Req<'a> {
                uri: &'a FsUri,
            }
        );
        impl_de!(
            struct Res {
                value: Option<String>,
            }
        );

        self.invoke::<Res>("getMimeType", Req { uri })
            .await
            .map(|v| match v.value {
                Some(mime_type) => EntryType::File { mime_type },
                None => EntryType::Dir,
            })
    }

    #[maybe_async]
    pub fn get_entry_info(&self, uri: &FsUri) -> Result<Entry> {
        impl_se!(
            struct Req<'a> {
                uri: &'a FsUri,
            }
        );
        impl_de!(
            struct Res {
                uri: FsUri,
                mime_type: Option<String>,
                name: String,
                last_modified: i64,
                len: Option<i64>,
            }
        );

        let v = self.invoke::<Res>("getMetadata", Req { uri }).await?;

        Ok(match v.mime_type {
            // ファイルの時は必ず Some(mime_type) になり、
            // フォルダの時にのみ None になる。
            Some(mime_type) => Entry::File {
                uri: v.uri,
                name: v.name,
                last_modified: std::time::UNIX_EPOCH
                    + std::time::Duration::from_millis(i64::max(0, v.last_modified) as u64),
                len: i64::max(0, v.len.ok_or_else(|| Error::missing_value("len"))?) as u64,
                mime_type,
            },
            None => Entry::Dir {
                uri: v.uri,
                name: v.name,
                last_modified: std::time::UNIX_EPOCH
                    + std::time::Duration::from_millis(i64::max(0, v.last_modified) as u64),
            },
        })
    }

    #[maybe_async]
    pub fn get_file_len(&self, uri: &FsUri) -> Result<u64> {
        impl_se!(
            struct Req<'a> {
                uri: &'a FsUri,
            }
        );
        impl_de!(
            struct Res {
                len: i64,
            }
        );

        self.invoke::<Res>("getLen", Req { uri })
            .await
            .map(|v| i64::max(0, v.len) as u64)
    }

    #[maybe_async]
    #[allow(unused)]
    pub fn get_file_resource_for_content_protocol(
        &self,
        uri: &FsUri,
    ) -> Result<(std::fs::File, Option<String>, Option<u64>)> {

        impl_se!(
            struct Req<'a> {
                uri: &'a FsUri,
            }
        );
        impl_de!(
            struct Res {
                fd: std::os::fd::RawFd,
                mime_type: Option<String>,
                len: Option<u64>,
            }
        );

        self.invoke::<Res>("getFileResourceForContentProtocol", Req { uri })
            .await
            .map(|r| {
                let file = unsafe {
                    use std::os::fd::FromRawFd;
                    std::fs::File::from_raw_fd(r.fd)
                };
                (file, r.mime_type, r.len)
            })
    }

    #[maybe_async]
    pub fn open_file(&self, uri: &FsUri, mode: FileAccessMode) -> Result<std::fs::File> {
        impl_se!(
            struct Req<'a> {
                uri: &'a FsUri,
                mode: &'a str,
            }
        );
        impl_de!(
            struct Res {
                fd: std::os::fd::RawFd,
            }
        );

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
        uri: &FsUri,
        candidate_modes: impl IntoIterator<Item = FileAccessMode>,
    ) -> Result<(std::fs::File, FileAccessMode)> {

        impl_se!(
            struct Req<'a> {
                uri: &'a FsUri,
                modes: Vec<&'a str>,
            }
        );
        impl_de!(
            struct Res {
                fd: std::os::fd::RawFd,
                mode: String,
            }
        );

        let modes = candidate_modes
            .into_iter()
            .map(|m| m.to_mode())
            .collect::<Vec<_>>();

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
    pub fn rename_entry(&self, uri: &FsUri, new_name: impl AsRef<str>) -> Result<FsUri> {
        impl_se!(
            struct Req<'a> {
                uri: &'a FsUri,
                new_name: &'a str,
            }
        );

        let new_name = new_name.as_ref();

        self.invoke::<FsUri>("rename", Req { uri, new_name }).await
    }

    #[maybe_async]
    pub fn remove_file(&self, uri: &FsUri) -> Result<()> {
        impl_se!(
            struct Req<'a> {
                uri: &'a FsUri,
            }
        );
        impl_de!(
            struct Res;
        );

        self.invoke::<Res>("deleteFile", Req { uri })
            .await
            .map(|_| ())
    }

    #[maybe_async]
    pub fn remove_dir_if_empty(&self, uri: &FsUri) -> Result<()> {
        impl_se!(
            struct Req<'a> {
                uri: &'a FsUri,
            }
        );
        impl_de!(
            struct Res;
        );

        self.invoke::<Res>("deleteEmptyDir", Req { uri })
            .await
            .map(|_| ())
    }

    #[maybe_async]
    pub fn remove_dir_all(&self, uri: &FsUri) -> Result<()> {
        impl_se!(struct Req<'a> {
            uri: &'a FsUri,
        });
        impl_de!(struct Res;);

        self.invoke::<Res>("deleteDirAll", Req { uri })
            .await
            .map(|_| ())
    }

    #[maybe_async]
    pub fn create_new_file(
        &self,
        dir: &FsUri,
        relative_path: impl AsRef<std::path::Path>,
        mime_type: Option<&str>,
    ) -> Result<FsUri> {

        impl_se!(
            struct Req<'a> {
                dir: &'a FsUri,
                mime_type: Option<&'a str>,
                relative_path: &'a str,
            }
        );

        let relative_path = validate_relative_path(relative_path.as_ref())?;
        let relative_path = relative_path.to_string_lossy();

        self.invoke::<FsUri>(
            "createFile",
            Req {
                dir,
                mime_type,
                relative_path: relative_path.as_ref(),
            },
        )
        .await
    }

    #[maybe_async]
    pub fn create_new_file_and_return_relative_path(
        &self,
        dir: &FsUri,
        relative_path: impl AsRef<std::path::Path>,
        mime_type: Option<&str>,
    ) -> Result<(FsUri, std::path::PathBuf)> {

        impl_se!(
            struct Req<'a> {
                dir: &'a FsUri,
                mime_type: Option<&'a str>,
                relative_path: &'a str,
            }
        );
        impl_de!(
            struct Res {
                uri: FsUri,
                relative_path: std::path::PathBuf,
            }
        );

        let relative_path = validate_relative_path(relative_path.as_ref())?;
        let relative_path = relative_path.to_string_lossy();

        self.invoke::<Res>(
            "createFileAndReturnRelativePath",
            Req {
                dir,
                mime_type,
                relative_path: relative_path.as_ref(),
            },
        )
        .await
        .map(|v| (v.uri, v.relative_path))
    }

    #[maybe_async]
    pub fn create_dir_all(
        &self,
        dir: &FsUri,
        relative_path: impl AsRef<std::path::Path>,
    ) -> Result<FsUri> {

        impl_se!(
            struct Req<'a> {
                dir: &'a FsUri,
                relative_path: &'a str,
            }
        );

        let relative_path = validate_relative_path(relative_path.as_ref())?;
        let relative_path = relative_path.to_string_lossy();

        self.invoke::<FsUri>(
            "createDirAll",
            Req {
                dir,
                relative_path: relative_path.as_ref(),
            },
        )
        .await
    }

    #[maybe_async]
    pub fn create_dir_all_and_return_relative_path(
        &self,
        dir: &FsUri,
        relative_path: impl AsRef<std::path::Path>,
    ) -> Result<(FsUri, std::path::PathBuf)> {

        impl_se!(
            struct Req<'a> {
                dir: &'a FsUri,
                relative_path: &'a str,
            }
        );
        impl_de!(
            struct Res {
                uri: FsUri,
                relative_path: std::path::PathBuf,
            }
        );

        let relative_path = validate_relative_path(relative_path.as_ref())?;
        let relative_path = relative_path.to_string_lossy();

        self.invoke::<Res>(
            "createDirAllAndReturnRelativePath",
            Req {
                dir,
                relative_path: relative_path.as_ref(),
            },
        )
        .await
        .map(|v| (v.uri, v.relative_path))
    }

    #[maybe_async]
    pub fn create_new_dir(
        &self,
        dir: &FsUri,
        relative_path: impl AsRef<std::path::Path>,
    ) -> Result<FsUri> {

        impl_se!(
            struct Req<'a> {
                dir: &'a FsUri,
                relative_path: &'a str,
            }
        );

        let relative_path = validate_relative_path(relative_path.as_ref())?;
        let relative_path = relative_path.to_string_lossy();

        self.invoke::<FsUri>(
            "createNewDir",
            Req {
                dir,
                relative_path: relative_path.as_ref(),
            },
        )
        .await
    }

    #[maybe_async]
    pub fn create_new_dir_and_return_relative_path(
        &self,
        dir: &FsUri,
        relative_path: impl AsRef<std::path::Path>,
    ) -> Result<(FsUri, std::path::PathBuf)> {

        impl_se!(struct Req<'a> {
            dir: &'a FsUri,
            relative_path: &'a str,
        });
        impl_de!(struct Res {
            uri: FsUri,
            relative_path: std::path::PathBuf,
        });

        let relative_path = validate_relative_path(relative_path.as_ref())?;
        let relative_path = relative_path.to_string_lossy();

        self.invoke::<Res>(
            "createNewDirAndReturnRelativePath",
            Req {
                dir,
                relative_path: relative_path.as_ref(),
            },
        )
        .await
        .map(|v| (v.uri, v.relative_path))
    }

    #[maybe_async]
    pub fn read_dir(
        &self,
        uri: &FsUri,
        options: EntryOptions,
        range: impl std::ops::RangeBounds<u64>,
    ) -> Result<impl Iterator<Item = OptionalEntry>> {

        impl_se!(struct Req<'a> {
            uri: &'a FsUri,
            options: Ops,
            offset: &'a str,
            limit: Option<&'a str>,
        });
        // ファイルかフォルダかを知るために mime_type は常に取得する。
        impl_se!(struct Ops {
            uri: bool,
            name: bool,
            last_modified: bool,
            len: bool,
        });
        impl_de!(struct Res {
            entries: Vec<Obj>,
        });
        impl_de!(struct Obj {
            uri: Option<FsUri>,
            mime_type: Option<String>,
            name: Option<String>,
            last_modified: Option<i64>,
            len: Option<i64>,
        });

        let map_entry = move |v: Obj| -> OptionalEntry {
            let map_time = |millis: i64| -> std::time::SystemTime {
                use std::time::{Duration, UNIX_EPOCH};

                UNIX_EPOCH + Duration::from_millis(i64::max(0, millis) as u64)
            };

            // ファイルの時は必ず Some(mime_type) になり、
            // フォルダの時にのみ None になる。
            match v.mime_type {
                Some(mime_type) => OptionalEntry::File {
                    uri: v.uri,
                    name: v.name,
                    last_modified: v.last_modified.map(map_time),
                    len: v.len.map(|i| i64::max(0, i) as u64),
                    mime_type: if options.mime_type {
                        Some(mime_type)
                    }
                    else {
                        None
                    },
                },
                None => OptionalEntry::Dir {
                    uri: v.uri,
                    name: v.name,
                    last_modified: v.last_modified.map(map_time),
                },
            }
        };

        let (offset, limit) = range_to_offset_and_len(range);
        let entries;
        // range の要素は u64 で表されているため offset が u64::MAX を超えた場合は
        // 表現できない範囲として要素ゼロとして扱う。
        if limit == Some(0) || (u64::MAX as u128) < offset {
            entries = Vec::new();
        }
        else {
            let offset = saturate_u128_to_u64(offset);
            let limit = limit.map(saturate_u128_to_u64);
            let options = Ops {
                uri: options.uri,
                name: options.name,
                last_modified: options.last_modified,
                len: options.len,
            };

            entries = self
                .invoke::<Res>(
                    "readDir",
                    Req {
                        uri,
                        options,
                        offset: &offset.to_string(),
                        limit: limit.map(|l| l.to_string()).as_deref(),
                    },
                )
                .await?
                .entries
        }

        Ok(entries.into_iter().map(map_entry))
    }

    #[maybe_async]
    pub fn get_file_thumbnail_to_file(
        &self,
        src: &FsUri,
        dest: &FsUri,
        preferred_size: Size,
        format: ImageFormat,
    ) -> Result<bool> {

        impl_se!(
            struct Req<'a> {
                src: &'a FsUri,
                dest: &'a FsUri,
                format: &'a str,
                quality: u8,
                width: u32,
                height: u32,
            }
        );
        impl_de!(
            struct Res {
                value: bool,
            }
        );

        let (quality, format) = format.to_quality_and_format_str();
        let quality = (quality * 100.0).clamp(0.0, 100.0) as u8;
        let Size { width, height } = preferred_size;

        self.invoke::<Res>(
            "getThumbnailToFile",
            Req {
                src,
                dest,
                format,
                quality,
                width,
                height,
            },
        )
        .await
        .map(|v| v.value)
    }

    #[maybe_async]
    pub fn get_file_thumbnail_base64(
        &self,
        uri: &FsUri,
        preferred_size: Size,
        format: ImageFormat,
    ) -> Result<Option<String>> {

        impl_se!(
            struct Req<'a> {
                uri: &'a FsUri,
                format: &'a str,
                quality: u8,
                width: u32,
                height: u32,
            }
        );
        impl_de!(
            struct Res {
                bytes: Option<String>,
            }
        );

        let (quality, format) = format.to_quality_and_format_str();
        let quality = (quality * 100.0).clamp(0.0, 100.0) as u8;
        let Size { width, height } = preferred_size;

        let Some(thumbnail) = self
            .invoke::<Res>(
                "getThumbnail",
                Req {
                    uri,
                    format,
                    quality,
                    width,
                    height,
                },
            )
            .await
            .map(|v| v.bytes)? else {

            return Ok(None);
        };
        if thumbnail.is_empty() {
            return Ok(None);
        }

        Ok(Some(thumbnail))
    }

    #[maybe_async]
    #[allow(unused)]
    pub fn check_media_store_volume_name_available(
        &self,
        media_store_volume_name: impl AsRef<str>,
    ) -> Result<bool> {

        impl_se!(
            struct Req<'a> {
                media_store_volume_name: &'a str,
            }
        );
        impl_de!(
            struct Res {
                value: bool,
            }
        );

        let media_store_volume_name = media_store_volume_name.as_ref();

        self.invoke::<Res>(
            "checkMediaStoreVolumeNameAvailable",
            Req {
                media_store_volume_name,
            },
        )
        .await
        .map(|v| v.value)
    }

    #[maybe_async]
    pub fn check_storage_volume_available_by_path(
        &self,
        path: impl AsRef<std::path::Path>,
    ) -> Result<bool> {

        impl_se!(
            struct Req<'a> {
                path: &'a std::path::Path,
            }
        );
        impl_de!(
            struct Res {
                value: bool,
            }
        );

        let path = path.as_ref();

        self.invoke::<Res>("checkStorageVolumeAvailableByPath", Req { path })
            .await
            .map(|v| v.value)
    }

    #[maybe_async]
    pub fn get_available_storage_volumes(&self) -> Result<Vec<StorageVolume>> {
        impl_de!(
            struct Res {
                volumes: Vec<StorageVolume>,
            }
        );

        let mut volumes = self
            .invoke::<Res>("getAvailableStorageVolumes", "")
            .await
            .map(|v| v.volumes)?;

        // primary volume を先頭にする。他はそのままの順序
        volumes.sort_by(|a, b| b.is_primary.cmp(&a.is_primary));

        Ok(volumes)
    }

    #[maybe_async]
    pub fn get_primary_storage_volume_if_available(&self) -> Result<Option<StorageVolume>> {
        impl_de!(
            struct Res {
                volume: Option<StorageVolume>,
            }
        );

        self.invoke::<Res>("getPrimaryStorageVolumeIfAvailable", "")
            .await
            .map(|v| v.volume)
    }

    #[always_sync]
    pub fn consts(&self) -> Result<&'static Consts> {
        get_or_init_const(|| self.invoke_sync::<Consts>("getConsts", ""))
    }

    #[always_sync]
    pub fn private_dir_path(&self, dir: PrivateDir) -> Result<&'static std::path::PathBuf> {
        let paths = get_or_init_private_dir_paths(|| {
            self.invoke_sync::<PrivateDirPaths>("getPrivateBaseDirAbsolutePaths", "")
        })?;

        Ok(match dir {
            PrivateDir::Data => &paths.data,
            PrivateDir::Cache => &paths.cache,
            PrivateDir::NoBackupData => &paths.no_backup_data,
        })
    }

    #[maybe_async]
    pub fn set_media_store_file_pending(&self, uri: &FsUri, is_pending: bool) -> Result<()> {
        impl_se!(
            struct Req<'a> {
                uri: &'a FsUri,
                pending: bool,
            }
        );
        impl_de!(
            struct Res;
        );

        self.invoke::<Res>(
            "setMediaStoreFilePending",
            Req {
                uri,
                pending: is_pending,
            },
        )
        .await
        .map(|_| ())
    }

    #[maybe_async]
    pub fn create_new_media_store_file(
        &self,
        volume_id: Option<&StorageVolumeId>,
        base_dir: impl Into<PublicDir>,
        relative_path: impl AsRef<std::path::Path>,
        mime_type: Option<&str>,
        is_pending: bool,
    ) -> Result<FsUri> {

        impl_se!(
            struct Req<'a> {
                volume_name: Option<&'a str>,
                relative_path: std::path::PathBuf,
                mime_type: Option<&'a str>,
                pending: bool,
            }
        );
        impl_de!(
            struct Res {
                uri: FsUri,
            }
        );

        let consts = self.consts()?;
        let relative_path = {
            let mut p = std::path::PathBuf::new();
            p.push(consts.public_dir_name(base_dir)?);
            p.push(validate_relative_path(relative_path.as_ref())?);
            p
        };

        let volume_name = volume_id.and_then(|v| v.media_store_volume_name.as_deref());

        self.invoke::<Res>(
            "createNewMediaStoreFile",
            Req {
                volume_name,
                relative_path,
                mime_type,
                pending: is_pending,
            },
        )
        .await
        .map(|v| v.uri)
    }

    #[maybe_async]
    pub fn show_pick_file_dialog(
        &self,
        initial_location: Option<&FsUri>,
        mime_types: &[&str],
        multiple: bool,
        local_only: bool,
    ) -> Result<Vec<FsUri>> {

        impl_se!(
            struct Req<'a> {
                mime_types: &'a [&'a str],
                multiple: bool,
                initial_location: Option<&'a FsUri>,
                local_only: bool,
            }
        );
        impl_de!(
            struct Res {
                uris: Vec<FsUri>,
            }
        );

        let result = self
            .invoke::<Res>(
                "showOpenFileDialog",
                Req {
                    mime_types,
                    multiple,
                    initial_location,
                    local_only,
                },
            )
            .await
            .map(|v| v.uris);

        // kotlin から他のアプリを呼び出した後にすぐに frontend 側に戻ると、
        // その frontend 側の関数の呼び出しがなぜか終了しないことが偶にある。
        // よって遅延を強制的に追加してこれを回避する。
        // https://github.com/aiueo13/tauri-plugin-android-fs/issues/1
        sleep(std::time::Duration::from_millis(200)).await;

        result
    }

    #[maybe_async]
    pub fn show_pick_visual_media_dialog(
        &self,
        target: VisualMediaTarget<'_>,
        multiple: bool,
        local_only: bool,
    ) -> Result<Vec<FsUri>> {

        impl_se!(
            struct Req<'a> {
                multiple: bool,
                target: &'a str,
                local_only: bool,
            }
        );
        impl_de!(
            struct Res {
                uris: Vec<FsUri>,
            }
        );

        let target = match target {
            VisualMediaTarget::ImageOnly => "image/*",
            VisualMediaTarget::VideoOnly => "video/*",
            VisualMediaTarget::ImageAndVideo => "*/*",
            VisualMediaTarget::ImageOrVideo { mime_type } => {
                let is_image_or_video =
                    mime_type.starts_with("image/") || mime_type.starts_with("video/");
                if !is_image_or_video {
                    return Err(Error::with(format!(
                        "mime_type must be an image or a video, but {mime_type}"
                    )));
                }

                mime_type
            }
        };

        let result = self
            .invoke::<Res>(
                "showOpenVisualMediaDialog",
                Req {
                    multiple,
                    target,
                    local_only,
                },
            )
            .await
            .map(|v| v.uris);

        // show_pick_file_dialog 内のコメントを参照
        sleep(std::time::Duration::from_millis(200)).await;

        result
    }

    #[maybe_async]
    pub fn show_pick_content_dialog(
        &self,
        mime_types: &[&str],
        multiple: bool,
    ) -> Result<Vec<FsUri>> {

        impl_se!(
            struct Req<'a> {
                mime_types: &'a [&'a str],
                multiple: bool,
            }
        );
        impl_de!(
            struct Res {
                uris: Vec<FsUri>,
            }
        );

        let result = self
            .invoke::<Res>(
                "showOpenContentDialog",
                Req {
                    mime_types,
                    multiple,
                },
            )
            .await
            .map(|v| v.uris);

        // show_pick_file_dialog 内のコメントを参照
        sleep(std::time::Duration::from_millis(200)).await;

        result
    }

    #[maybe_async]
    pub fn show_pick_dir_dialog(
        &self,
        initial_location: Option<&FsUri>,
        local_only: bool,
    ) -> Result<Option<FsUri>> {

        impl_se!(
            struct Req<'a> {
                initial_location: Option<&'a FsUri>,
                local_only: bool,
            }
        );
        impl_de!(
            struct Res {
                uri: Option<FsUri>,
            }
        );

        let result = self
            .invoke::<Res>(
                "showManageDirDialog",
                Req {
                    initial_location,
                    local_only,
                },
            )
            .await
            .map(|v| v.uri);

        // show_pick_file_dialog 内のコメントを参照
        sleep(std::time::Duration::from_millis(200)).await;

        result
    }

    #[maybe_async]
    pub fn show_save_file_dialog(
        &self,
        initial_location: Option<&FsUri>,
        initial_file_name: impl AsRef<str>,
        mime_type: Option<&str>,
        local_only: bool,
    ) -> Result<Option<FsUri>> {

        impl_se!(
            struct Req<'a> {
                initial_file_name: &'a str,
                mime_type: Option<&'a str>,
                initial_location: Option<&'a FsUri>,
                local_only: bool,
            }
        );
        impl_de!(
            struct Res {
                uri: Option<FsUri>,
            }
        );

        let initial_file_name = initial_file_name.as_ref();

        let result = self
            .invoke::<Res>(
                "showSaveFileDialog",
                Req {
                    local_only,
                    initial_file_name,
                    mime_type,
                    initial_location,
                },
            )
            .await
            .map(|v| v.uri);

        // show_pick_file_dialog 内のコメントを参照
        sleep(std::time::Duration::from_millis(200)).await;

        result
    }

    #[maybe_async]
    pub fn is_visual_media_picker_available(&self) -> Result<bool> {
        impl_de!(
            struct Res {
                value: bool,
            }
        );

        self.invoke::<Res>("isVisualMediaDialogAvailable", "")
            .await
            .map(|v| v.value)
    }

    #[maybe_async]
    pub fn show_share_file_app_chooser<'b>(
        &self,
        uris: impl IntoIterator<Item = &'b FsUri>,
    ) -> Result<()> {

        impl_se!(
            struct Req<'a> {
                uris: Vec<&'a FsUri>,
            }
        );
        impl_de!(
            struct Res;
        );

        let uris = uris.into_iter().collect::<Vec<_>>();

        let result = self
            .invoke::<Res>("shareFiles", Req { uris })
            .await
            .map(|_| ());

        // show_pick_file_dialog 内のコメントを参照
        sleep(std::time::Duration::from_millis(200)).await;

        result
    }

    #[maybe_async]
    pub fn show_open_file_app_chooser(&self, uri: &FsUri) -> Result<()> {
        impl_se!(
            struct Req<'a> {
                uri: &'a FsUri,
            }
        );
        impl_de!(
            struct Res;
        );

        let result = self
            .invoke::<Res>("viewFile", Req { uri })
            .await
            .map(|_| ());

        // show_pick_file_dialog 内のコメントを参照
        sleep(std::time::Duration::from_millis(200)).await;

        result
    }

    #[maybe_async]
    pub fn show_open_dir_app_chooser(&self, uri: &FsUri) -> Result<()> {
        impl_se!(
            struct Req<'a> {
                uri: &'a FsUri,
            }
        );
        impl_de!(
            struct Res;
        );

        let result = self.invoke::<Res>("viewDir", Req { uri }).await.map(|_| ());

        // show_pick_file_dialog 内のコメントを参照
        sleep(std::time::Duration::from_millis(200)).await;

        result
    }

    #[maybe_async]
    pub fn show_edit_file_app_chooser(&self, uri: &FsUri) -> Result<()> {
        impl_se!(
            struct Req<'a> {
                uri: &'a FsUri,
            }
        );
        impl_de!(
            struct Res;
        );

        let result = self
            .invoke::<Res>("editFile", Req { uri })
            .await
            .map(|_| ());

        // show_pick_file_dialog 内のコメントを参照
        sleep(std::time::Duration::from_millis(200)).await;

        result
    }

    #[maybe_async]
    pub fn request_legacy_storage_permission(&self) -> Result<bool> {
        impl_de!(
            struct Res {
                granted: bool,
                prompted: bool,
            }
        );

        let result = self
            .invoke::<Res>("requestLegacyStoragePermission", ())
            .await?;

        if result.prompted {
            // show_pick_file_dialog 内のコメントを参照
            sleep(std::time::Duration::from_millis(200)).await;
        }

        Ok(result.granted)
    }

    #[maybe_async]
    pub fn check_legacy_storage_permission(&self) -> Result<bool> {
        impl_de!(
            struct Res {
                granted: bool,
            }
        );

        self.invoke::<Res>("checkLegacyStoragePermission", ())
            .await
            .map(|res| res.granted)
    }

    #[always_sync]
    pub fn is_legacy_storage(&self) -> Result<bool> {
        let is_legacy_storage = get_or_init_is_legacy_storage(move || {
            impl_de!(
                struct Res {
                    value: bool,
                }
            );

            self.invoke_sync::<Res>("isLegacyStorage", ())
                .map(|res| res.value)
        })?;

        Ok(*is_legacy_storage)
    }

    #[maybe_async]
    pub fn find_saf_file_uri(
        &self,
        parent_uri: &FsUri,
        relative_path: impl AsRef<std::path::Path>,
    ) -> Result<FsUri> {

        impl_se!(
            struct Req<'a> {
                parent_uri: &'a FsUri,
                relative_path: &'a std::path::Path,
            }
        );

        let relative_path = validate_relative_path(relative_path.as_ref())?;

        self.invoke::<FsUri>(
            "findSafFileUri",
            Req {
                parent_uri,
                relative_path,
            },
        )
        .await
    }

    #[maybe_async]
    pub fn find_saf_dir_uri(
        &self,
        parent_uri: &FsUri,
        relative_path: impl AsRef<std::path::Path>,
    ) -> Result<FsUri> {

        impl_se!(
            struct Req<'a> {
                parent_uri: &'a FsUri,
                relative_path: &'a std::path::Path,
            }
        );

        let relative_path = validate_relative_path(relative_path.as_ref())?;

        self.invoke::<FsUri>(
            "findSafDirUri",
            Req {
                parent_uri,
                relative_path,
            },
        )
        .await
    }

    #[maybe_async]
    pub fn scan_media_store_file(&self, uri: &FsUri) -> Result<()> {
        impl_se!(
            struct Req<'a> {
                uri: &'a FsUri,
            }
        );

        self.invoke::<()>("scanMediaStoreFile", Req { uri }).await
    }

    #[maybe_async]
    pub fn scan_media_store_file_for_result(&self, uri: &FsUri) -> Result<()> {
        impl_se!(
            struct Req<'a> {
                uri: &'a FsUri,
            }
        );

        self.invoke::<()>("scanMediaStoreFileForResult", Req { uri })
            .await
    }

    #[maybe_async]
    pub fn scan_file_to_media_store_by_path(
        &self,
        path: impl AsRef<std::path::Path>,
        mime_type: Option<&str>,
    ) -> Result<FsUri> {

        impl_se!(
            struct Req<'a> {
                path: &'a std::path::Path,
                mime_type: Option<&'a str>,
            }
        );
        impl_de!(
            struct Res {
                uri: FsUri,
            }
        );

        let path = path.as_ref();

        self.invoke::<Res>("scanFileToMediaStoreByPath", Req { path, mime_type })
            .await
            .map(|res| res.uri)
    }

    #[maybe_async]
    pub fn get_media_store_file_path(&self, uri: &FsUri) -> Result<std::path::PathBuf> {
        impl_se!(
            struct Req<'a> {
                uri: &'a FsUri,
            }
        );
        impl_de!(
            struct Res {
                path: std::path::PathBuf,
            }
        );

        self.invoke::<Res>("getMediaStoreFileAbsolutePath", Req { uri })
            .await
            .map(|v| v.path)
    }

    #[maybe_async]
    pub fn check_picker_uri_permission(
        &self,
        uri: &FsUri,
        permission: UriPermission,
    ) -> Result<bool> {

        impl_se!(
            struct Req<'a> {
                uri: &'a FsUri,
            }
        );
        impl_de!(
            struct Res {
                can_write: bool,
                can_read: bool,
            }
        );

        let p = self
            .invoke::<Res>("getPickerUriPermission", Req { uri })
            .await?;

        Ok(match permission {
            UriPermission::Read => p.can_read,
            UriPermission::Write => p.can_write,
            UriPermission::ReadAndWrite => p.can_read && p.can_write,
            UriPermission::ReadOrWrite => p.can_read || p.can_write,
        })
    }

    #[maybe_async]
    pub fn check_persisted_picker_uri_permission(
        &self,
        uri: &FsUri,
        permission: UriPermission,
    ) -> Result<bool> {

        impl_se!(
            struct Req<'a> {
                uri: &'a FsUri,
            }
        );
        impl_de!(
            struct Res {
                can_write: bool,
                can_read: bool,
            }
        );

        let p = self
            .invoke::<Res>("getPersistedPickerUriPermission", Req { uri })
            .await?;

        Ok(match permission {
            UriPermission::Read => p.can_read,
            UriPermission::Write => p.can_write,
            UriPermission::ReadAndWrite => p.can_read && p.can_write,
            UriPermission::ReadOrWrite => p.can_read || p.can_write,
        })
    }

    #[maybe_async]
    pub fn persist_picker_uri_permission(&self, uri: &FsUri) -> Result<()> {
        impl_se!(
            struct Req<'a> {
                uri: &'a FsUri,
            }
        );

        self.invoke::<()>("persistPickerUriPermission", Req { uri })
            .await
    }

    #[maybe_async]
    pub fn release_persisted_picker_uri_permission(&self, uri: &FsUri) -> Result<bool> {
        impl_se!(
            struct Req<'a> {
                uri: &'a FsUri,
            }
        );
        impl_de!(
            struct Res {
                is_released: bool,
            }
        );

        self.invoke::<Res>("releasePersistedPickerUriPermission", Req { uri })
            .await
            .map(|res| res.is_released)
    }

    #[maybe_async]
    pub fn release_all_persisted_picker_uri_permissions(&self) -> Result<()> {
        self.invoke::<()>("releaseAllPersistedPickerUriPermissions", ())
            .await
    }

    #[maybe_async]
    pub fn get_all_persisted_picker_uri_permissions(
        &self,
    ) -> Result<impl Iterator<Item = PersistedUriPermissionState>> {

        impl_de!(
            struct Obj {
                uri: FsUri,
                r: bool,
                w: bool,
                d: bool,
            }
        );
        impl_de!(
            struct Res {
                items: Vec<Obj>,
            }
        );

        self.invoke::<Res>("getAllPersistedPickerUriPermissions", ())
            .await
            .map(|v| v.items.into_iter())
            .map(|v| {
                v.map(|v| {
                    let (uri, can_read, can_write) = (v.uri, v.r, v.w);
                    match v.d {
                        true => PersistedUriPermissionState::Dir {
                            uri,
                            can_read,
                            can_write,
                        },
                        false => PersistedUriPermissionState::File {
                            uri,
                            can_read,
                            can_write,
                        },
                    }
                })
            })
    }

    #[maybe_async]
    pub fn get_mime_type_from_extension(&self, ext: impl AsRef<str>) -> Result<Option<String>> {
        let ext = ext.as_ref().trim_start_matches('.').to_lowercase();

        if let Some(mime_type) = MAP_OF_EXT_AND_MIME_TYPE
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(&ext) {

            return Ok(mime_type.to_owned());
        }

        let mime_type = {
            impl_se!(struct Req<'a> {
                extension: &'a str,
            });
            impl_de!(struct Res {
                mime_type: Option<String>,
            });

            self.invoke::<Res>("getMimeTypeFromExtension", Req { extension: &ext })
                .await
                .map(|res| res.mime_type)?
        };

        MAP_OF_EXT_AND_MIME_TYPE
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(ext.to_string(), mime_type.to_owned());

        Ok(mime_type)
    }

    #[maybe_async]
    pub fn start_progress_notification(
        &self,
        icon_type: ProgressNotificationIcon,
        title: Option<&str>,
        text: Option<&str>,
        sub_text: Option<&str>,
        progress: Option<i32>,
        progress_max: Option<i32>,
    ) -> Result<i32> {

        impl_se!(struct Req<'a> {
            icon_type: ProgressNotificationIcon,
            title: Option<&'a str>,
            text: Option<&'a str>,
            sub_text: Option<&'a str>,
            progress: Option<i32>,
            progress_max: Option<i32>,
        });
        impl_de!(struct Res {
            id: i32,
        });

        self.invoke::<Res>(
            "startProgressNotification",
            Req {
                icon_type,
                title,
                text,
                sub_text,
                progress,
                progress_max,
            },
        )
        .await
        .map(|v| v.id)
    }

    #[maybe_async]
    pub fn update_progress_notification(
        &self,
        id: i32,
        icon_type: ProgressNotificationIcon,
        title: Option<&str>,
        text: Option<&str>,
        sub_text: Option<&str>,
        progress: Option<i32>,
        progress_max: Option<i32>,
    ) -> Result<()> {

        impl_se!(struct Req<'a> {
            id: i32,
            icon_type: ProgressNotificationIcon,
            title: Option<&'a str>,
            text: Option<&'a str>,
            sub_text: Option<&'a str>,
            progress: Option<i32>,
            progress_max: Option<i32>,
        });

        self.invoke::<()>(
            "updateProgressNotification",
            Req {
                id,
                icon_type,
                title,
                text,
                sub_text,
                progress,
                progress_max,
            },
        )
        .await
    }

    #[maybe_async]
    pub fn finish_progress_notification(
        &self,
        id: i32,
        icon_type: ProgressNotificationIcon,
        title: Option<&str>,
        text: Option<&str>,
        sub_text: Option<&str>,
        share_src: Option<&FsUri>,
        error: bool,
    ) -> Result<()> {

        impl_se!(
            struct Req<'a> {
                id: i32,
                icon_type: ProgressNotificationIcon,
                title: Option<&'a str>,
                text: Option<&'a str>,
                sub_text: Option<&'a str>,
                error: bool,
                share_src: Option<&'a FsUri>,
            }
        );

        self.invoke::<()>(
            "finishProgressNotification",
            Req {
                id,
                icon_type,
                title,
                text,
                sub_text,
                error,
                share_src,
            },
        )
        .await
    }

    #[maybe_async]
    pub fn cancel_notification(&self, id: i32) -> Result<()> {
        impl_se!(
            struct Req {
                id: i32,
            }
        );

        self.invoke::<()>("cancelNotification", Req { id }).await
    }

    #[maybe_async]
    pub fn cancel_all_notifications(&self) -> Result<()> {
        self.invoke::<()>("cancelAllNotifications", ()).await
    }

    #[maybe_async]
    pub fn request_notification_permission(&self) -> Result<bool> {
        impl_de!(
            struct Res {
                granted: bool,
                prompted: bool,
            }
        );

        let result = self
            .invoke::<Res>("requestNotificationPermission", ())
            .await?;

        if result.prompted {
            // show_pick_file_dialog 内のコメントを参照
            sleep(std::time::Duration::from_millis(200)).await;
        }

        Ok(result.granted)
    }

    #[maybe_async]
    pub fn check_notification_permission(&self) -> Result<bool> {
        impl_de!(
            struct Res {
                granted: bool,
            }
        );

        self.invoke::<Res>("hasNotificationPermission", ())
            .await
            .map(|res| res.granted)
    }

    #[always_sync]
    pub fn api_level(&self) -> Result<i32> {
        Ok(self.consts()?.build_version_sdk_int)
    }

    #[always_sync]
    pub fn public_dir_name(&self, dir: impl Into<PublicDir>) -> Result<&'static str> {
        Ok(self.consts()?.public_dir_name(dir)?)
    }

    #[maybe_async]
    pub fn get_file_mime_type(&self, uri: &FsUri) -> Result<String> {
        self.get_entry_type(uri).await?.into_file_mime_type_or_err()
    }

    #[maybe_async]
    pub fn open_file_readable(&self, uri: &FsUri) -> Result<std::fs::File> {
        self.open_file(uri, FileAccessMode::Read).await
    }

    #[maybe_async]
    pub fn open_file_writable(&self, uri: &FsUri) -> Result<std::fs::File> {
        // w は既存コンテンツの切り捨てを保証しない。
        #[allow(deprecated)]
        const WRITE_TRUNCATE_OR_NOT: FileAccessMode = FileAccessMode::Write;

        // 切り捨ててファイルを開く wt と rwt は全ての file provider が対応しているとは限らない。
        // よってフォールバックを用いてなるべく切り捨てて開けるように試みる。
        let (file, mode) = self.open_file_with_fallback(uri, [
            FileAccessMode::WriteTruncate,
            FileAccessMode::ReadWriteTruncate,
            WRITE_TRUNCATE_OR_NOT,
        ]).await?;

        if mode == WRITE_TRUNCATE_OR_NOT {
            // file provider が既存コンテンツを切り捨てず、
            // かつ書き込むデータ量が元のそれより少ない場合にファイルが壊れる可能性がある。
            // これを避けるため強制的にデータを切り捨てる。
            // ただし file provider の実装によっては set_len は失敗することがあるので最終手段。
            run_blocking(move || {
                if file.metadata().ok().is_some_and(|m| m.len() == 0) {
                    return Ok(file)
                }

                file.set_len(0)?;
                Ok(file)
            })
            .await
        }
        else {
            Ok(file)
        }
    }

    #[maybe_async]
    pub fn read_file(&self, uri: &FsUri) -> Result<Vec<u8>> {
        let mut file = self.open_file_readable(uri).await?;
        run_blocking(move || {
            let mut buf = file
                .metadata()
                .ok()
                .map(|m| m.len() as usize)
                .map(Vec::with_capacity)
                .unwrap_or_else(Vec::new);

            file.read_to_end(&mut buf)?;
            Ok(buf)
        })
        .await
    }

    #[maybe_async]
    pub fn read_file_to_string(&self, uri: &FsUri) -> Result<String> {
        let mut file = self.open_file_readable(uri).await?;
        run_blocking(move || {
            let mut buf = file
                .metadata()
                .ok()
                .map(|m| m.len() as usize)
                .map(String::with_capacity)
                .unwrap_or_else(String::new);

            file.read_to_string(&mut buf)?;
            Ok(buf)
        })
        .await
    }

    #[maybe_async]
    pub fn write_file(&self, uri: &FsUri, contents: impl AsRef<[u8]>) -> Result<()> {
        let mut file = self.open_file_writable(uri).await?;

        #[if_sync] {
            file.write_all(contents.as_ref())?;
        }
        #[if_async] {
            let contents = upgrade_bytes_ref(contents);
            run_blocking(move || file.write_all(&contents).map_err(Into::into)).await?;
        }
        Ok(())
    }

    #[maybe_async]
    pub fn copy_file(&self, src: &FsUri, dest: &FsUri) -> Result<()> {
        let mut src = self.open_file_readable(src).await?;
        let mut dest = self.open_file_writable(dest).await?;
        run_blocking(move || std::io::copy(&mut src, &mut dest).map_err(Into::into)).await?;
        Ok(())
    }

    #[maybe_async]
    pub fn get_file_thumbnail(
        &self,
        uri: &FsUri,
        preferred_size: Size,
        format: ImageFormat,
    ) -> Result<Option<Vec<u8>>> {

        if let Some(t) = self
            .get_file_thumbnail_base64(uri, preferred_size, format)
            .await?
        {
            use base64::engine::Engine;
            return Ok(Some(base64::engine::general_purpose::STANDARD.decode(t)?));
        }

        Ok(None)
    }

    #[maybe_async]
    pub fn is_dir(&self, uri: &FsUri) -> Result<bool> {
        if let Some(path) = uri.to_path() {
            return run_blocking(move || Ok(std::fs::metadata(&path)?.is_dir())).await;
        }

        Ok(self.get_entry_type(&uri).await?.is_dir())
    }

    #[maybe_async]
    pub fn is_file(&self, uri: &FsUri) -> Result<bool> {
        if let Some(path) = uri.to_path() {
            return run_blocking(move || Ok(std::fs::metadata(&path)?.is_file())).await;
        }

        Ok(self.get_entry_type(&uri).await?.is_file())
    }

    #[maybe_async]
    pub fn resolve_file_uri(
        &self,
        dir: &FsUri,
        relative_path: impl AsRef<std::path::Path>,
        allow_unchecked: bool,
    ) -> Result<FsUri> {

        let mut uri = None;
        if let Some(built_uri) = self.try_build_path_uri(dir, relative_path.as_ref())? {
            uri = Some(built_uri);
        } 
        else if let Some(built_uri) = self.try_build_saf_external_storage_provider_uri(dir, relative_path.as_ref())? {
            uri = Some(built_uri);
        }

        if let Some(uri) = uri {
            if allow_unchecked || self.is_file(&uri).await? {
                return Ok(uri);
            }
            return Err(Error::with(format!("not a file: {uri:?}")));
        }

        self.find_saf_file_uri(dir, relative_path).await
    }

    #[maybe_async]
    pub fn resolve_dir_uri(
        &self,
        dir: &FsUri,
        relative_path: impl AsRef<std::path::Path>,
        allow_unchecked: bool,
    ) -> Result<FsUri> {

        let mut uri = None;
        if let Some(built_uri) = self.try_build_path_uri(dir, relative_path.as_ref())? {
            uri = Some(built_uri);
        }
        else if let Some(built_uri) = self.try_build_saf_external_storage_provider_uri(dir, relative_path.as_ref())? {
            uri = Some(built_uri);
        }

        if let Some(uri) = uri {
            if allow_unchecked || self.is_dir(&uri).await? {
                return Ok(uri);
            }
            return Err(Error::with(format!("not a directory: {uri:?}")));
        }

        self.find_saf_dir_uri(dir, relative_path).await
    }

    #[always_sync]
    pub fn try_build_path_uri(
        &self,
        dir: &FsUri,
        relative_path: impl AsRef<std::path::Path>,
    ) -> Result<Option<FsUri>> {

        let relative_path = validate_relative_path(relative_path.as_ref())?;

        // file:// 形式の URI
        if let Some(path) = dir.to_path() {
            let uri = FsUri::from_path(path.join(relative_path));
            return Ok(Some(uri));
        }

        Ok(None)
    }

    #[always_sync]
    pub fn try_build_saf_external_storage_provider_uri(
        &self,
        dir: &FsUri,
        relative_path: impl AsRef<std::path::Path>,
    ) -> Result<Option<FsUri>> {

        let relative_path = validate_relative_path(relative_path.as_ref())?;
        let relative_path = relative_path.to_string_lossy();

        if dir.uri.starts_with("content://com.android.externalstorage.documents/tree/") {
            let uri = FsUri {
                document_top_tree_uri: dir.document_top_tree_uri.clone(),
                uri: format!(
                    "{}%2F{}",
                    &dir.uri,
                    encode_android_uri_component(relative_path)
                ),
            };

            return Ok(Some(uri));
        }

        Ok(None)
    }

    #[maybe_async]
    pub fn request_storage_permission_for_public_storage(&self) -> Result<bool> {
        if self.is_legacy_storage()? {
            self.request_legacy_storage_permission().await
        }
        else {
            Ok(true)
        }
    }

    #[maybe_async]
    pub fn check_storage_permission_for_public_storage(&self) -> Result<bool> {
        if self.is_legacy_storage()? {
            self.check_legacy_storage_permission().await
        }
        else {
            Ok(true)
        }
    }

    #[maybe_async]
    pub fn get_available_storage_volumes_for_public_storage(&self) -> Result<Vec<StorageVolume>> {
        let volumes = self
            .get_available_storage_volumes()
            .await?
            .into_iter()
            .filter(|v| v.is_available_for_public_storage)
            .collect::<Vec<_>>();

        Ok(volumes)
    }

    #[maybe_async]
    pub fn get_primary_storage_volume_if_available_for_public_storage(&self) -> Result<Option<StorageVolume>> {
        self.get_primary_storage_volume_if_available()
            .await
            .map(|v| v.filter(|v| v.is_available_for_public_storage))
    }

    #[maybe_async]
    pub fn create_new_file_in_public_storage(
        &self,
        volume_id: Option<&StorageVolumeId>,
        base_dir: impl Into<PublicDir>,
        relative_path: impl AsRef<std::path::Path>,
        mime_type: Option<&str>,
        is_pending: bool,
    ) -> Result<FsUri> {

        self.create_new_media_store_file(volume_id, base_dir, relative_path, mime_type, is_pending).await
    }

    #[maybe_async]
    pub fn write_new_file_in_public_storage(
        &self,
        volume_id: Option<&StorageVolumeId>,
        base_dir: impl Into<PublicDir>,
        relative_path: impl AsRef<std::path::Path>,
        mime_type: Option<&str>,
        contents: impl AsRef<[u8]>,
    ) -> Result<FsUri> {

        let uri = self
            .create_new_file_in_public_storage(volume_id, base_dir, relative_path, mime_type, true)
            .await?;

        let mut file = self.open_file_writable(&uri).await?;

        #[if_sync]
        let result = file.write_all(contents.as_ref()).map_err(Into::into);

        #[if_async]
        let result = {
            let contents = upgrade_bytes_ref(contents);
            run_blocking(move || file.write_all(&contents).map_err(Into::into)).await
        };

        if let Err(err) = result {
            self.remove_file(&uri).await.ok();
            return Err(err);
        }

        self.set_file_pending_in_public_storage(&uri, false).await?;
        self.scan_file_in_public_storage(&uri, false).await?;
        Ok(uri)
    }

    #[maybe_async]
    pub fn create_dir_all_in_public_storage(
        &self,
        volume_id: Option<&StorageVolumeId>,
        base_dir: impl Into<PublicDir>,
        relative_path: impl AsRef<std::path::Path>,
    ) -> Result<()> {

        let relative_path = validate_relative_path(relative_path.as_ref())?;
        let base_dir = base_dir.into();
        let tmp_file_uri = self
            .create_new_file_in_public_storage(
                volume_id,
                base_dir,
                relative_path.join("TMP-01K3CGCKYSAQ1GHF8JW5FGD4RW"),
                Some(match base_dir {
                    PublicDir::Image(_) => "image/png",
                    PublicDir::Audio(_) => "audio/mp3",
                    PublicDir::Video(_) => "video/mp4",
                    PublicDir::GeneralPurpose(_) => "application/octet-stream",
                }),
                true,
            )
            .await?;

        self.remove_file(&tmp_file_uri).await.ok();
        Ok(())
    }

    #[maybe_async]
    pub fn scan_file_in_public_storage(&self, uri: &FsUri, force: bool) -> Result<()> {
        if !force && api_level::ANDROID_11 <= self.api_level()? {
            return Ok(());
        }

        self.scan_media_store_file(uri).await
    }

    #[maybe_async]
    pub fn scan_file_in_public_storage_for_result(&self, uri: &FsUri, force: bool) -> Result<()> {
        if !force && api_level::ANDROID_11 <= self.api_level()? {
            return Ok(());
        }

        self.scan_media_store_file_for_result(uri).await
    }

    #[maybe_async]
    pub fn scan_file_by_path_in_public_storage(
        &self,
        path: impl AsRef<std::path::Path>,
        mime_type: Option<&str>,
    ) -> Result<FsUri> {

        self.scan_file_to_media_store_by_path(path, mime_type).await
    }

    #[maybe_async]
    pub fn get_file_path_in_public_storage(&self, uri: &FsUri) -> Result<std::path::PathBuf> {
        self.get_media_store_file_path(uri).await
    }

    #[maybe_async]
    pub fn set_file_pending_in_public_storage(&self, uri: &FsUri, is_pending: bool) -> Result<()> {
        if api_level::ANDROID_11 <= self.api_level()? {
            return self.set_media_store_file_pending(uri, is_pending).await;
        }

        Ok(())
    }

    #[maybe_async]
    pub fn resolve_dir_path_in_public_storage(
        &self,
        volume_id: Option<&StorageVolumeId>,
        base_dir: impl Into<PublicDir>,
    ) -> Result<std::path::PathBuf> {

        let mut path = match volume_id {
            Some(volume_id) => {
                let top_dir_path = volume_id.top_dir_path.as_ref().ok_or_else(|| {
                    Error::with("The storage volume is not available for PublicStorage")
                })?;

                if !self.check_storage_volume_available_by_path(top_dir_path).await? {
                    return Err(Error::with("The storage volume is not currently available"));
                }

                top_dir_path.clone()
            }
            None => self
                .get_primary_storage_volume_if_available_for_public_storage()
                .await?
                .ok_or_else(|| Error::with("Primary storage volume is not currently available"))?
                .id
                .top_dir_path
                .ok_or_else(|| {
                    Error::with("Primary storage volume is not available for PublicStorage")
                })?,
        };

        path.push(self.public_dir_name(base_dir)?);
        Ok(path)
    }

    #[maybe_async]
    pub fn resolve_initial_location_in_public_storage(
        &self,
        volume_id: Option<&StorageVolumeId>,
        base_dir: impl Into<PublicDir>,
        relative_path: impl AsRef<std::path::Path>,
        create_dir_all: bool,
    ) -> Result<FsUri> {

        let base_dir = base_dir.into();
        let relative_path = validate_relative_path(relative_path.as_ref())?;
        let uri = {
            let volume_id = volume_id
                .and_then(|v| v.uid.as_deref())
                .unwrap_or("primary");

            let mut relative_path_from_volume_root = std::path::PathBuf::new();
            relative_path_from_volume_root.push(self.public_dir_name(base_dir)?);
            relative_path_from_volume_root.push(relative_path);

            let mut uri = String::from("content://com.android.externalstorage.documents/document/");
            uri.push_str(volume_id);
            uri.push_str("%3A");
            uri.push_str(&encode_android_uri_component(relative_path_from_volume_root.to_string_lossy()));

            FsUri {
                uri,
                document_top_tree_uri: None,
            }
        };

        if create_dir_all {
            self.create_dir_all_in_public_storage(volume_id, base_dir, relative_path)
                .await
                .ok();
        }

        Ok(uri)
    }

    #[maybe_async]
    pub fn resolve_root_initial_location(
        &self,
        volume_id: Option<&StorageVolumeId>,
    ) -> Result<FsUri> {

        let volume_id = volume_id
            .and_then(|v| v.uid.as_deref())
            .unwrap_or("primary");

        if api_level::ANDROID_10 <= self.api_level()? {
            let base = "content://com.android.externalstorage.documents/root";
            let uri = format!("{base}/{volume_id}");
            Ok(FsUri {
                uri,
                document_top_tree_uri: None,
            })
        }
        else {
            let base = "content://com.android.externalstorage.documents/document";
            let uri = format!("{base}/{volume_id}%3A");
            Ok(FsUri {
                uri,
                document_top_tree_uri: None,
            })
        }
    }

    #[maybe_async]
    pub fn get_available_storage_volumes_for_app_storage(&self) -> Result<Vec<StorageVolume>> {
        let volumes = self
            .get_available_storage_volumes()
            .await?
            .into_iter()
            .filter(|v| v.is_available_for_app_storage)
            .collect::<Vec<_>>();

        Ok(volumes)
    }

    #[maybe_async]
    pub fn get_primary_storage_volume_if_available_for_app_storage(&self) -> Result<Option<StorageVolume>> {
        self.get_primary_storage_volume_if_available()
            .await
            .map(|v| v.filter(|v| v.is_available_for_app_storage))
    }

    #[maybe_async]
    pub fn resolve_dir_path_in_app_storage(
        &self,
        volume_id: Option<&StorageVolumeId>,
        dir: AppDir,
    ) -> Result<std::path::PathBuf> {

        if let Some(volume_id) = volume_id {
            let dir_path = volume_id
                .app_dir_path(dir)
                .ok_or_else(|| Error::with("The storage volume has no app-specific directory"))?;

            if !self
                .check_storage_volume_available_by_path(dir_path)
                .await? {
                return Err(Error::with("The storage volume is not currently available"));
            }

            return Ok(dir_path.clone());
        }

        self.get_primary_storage_volume_if_available_for_app_storage()
            .await?
            .ok_or_else(|| Error::with("Primary storage volume is not currently available"))?
            .id
            .app_dir_path(dir)
            .cloned()
            .ok_or_else(|| Error::with("Primary storage volume has no app-specific directory"))
    }

    #[maybe_async]
    pub fn get_public_media_file_path_in_app_storage(
        &self,
        uri: &FsUri,
    ) -> Result<std::path::PathBuf> {

        self.get_media_store_file_path(uri).await
    }
}

fn_get_or_init!(get_or_init_is_legacy_storage, bool);
fn_get_or_init!(get_or_init_const, Consts);
fn_get_or_init!(get_or_init_private_dir_paths, PrivateDirPaths);

static MAP_OF_EXT_AND_MIME_TYPE: std::sync::LazyLock<std::sync::Mutex<BoundedHashMap<String, Option<String>>>> = std::sync::LazyLock::new(|| std::sync::Mutex::new(BoundedHashMap::with_bound(1000)));

static APP_PROCESS_INSTANCE_ID: LazyLock<String> = LazyLock::new(|| {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
    use std::time::SystemTime;

    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes[..16]).expect("could not get random bytes");

    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    bytes[16..].copy_from_slice(&timestamp.to_be_bytes());

    URL_SAFE_NO_PAD.encode(bytes)
});

pub static FILE_PROVIDER_NAME: LazyLock<String> = LazyLock::new(|| {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};

    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).expect("could not get random bytes");
    format!("tpafs file provider: {}", URL_SAFE_NO_PAD.encode(bytes))
});

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct PrivateDirPaths {
    data: std::path::PathBuf,
    cache: std::path::PathBuf,
    no_backup_data: std::path::PathBuf,
}

/// アプリ起動中に変更されることのない値
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(unused)]
pub struct Consts {
    pub app_id: String,
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
                PublicVideoDir::Pictures => &self.env_dir_pictures,
            },
            PublicDir::Audio(dir) => match dir {
                PublicAudioDir::Music => &self.env_dir_music,
                PublicAudioDir::Alarms => &self.env_dir_alarms,
                PublicAudioDir::Notifications => &self.env_dir_notifications,
                PublicAudioDir::Podcasts => &self.env_dir_podcasts,
                PublicAudioDir::Ringtones => &self.env_dir_ringtones,
                PublicAudioDir::Recordings => self.env_dir_recordings.as_ref().ok_or_else(|| Error::with("requires Android 12 (API level 31) or higher"))?,
                PublicAudioDir::Audiobooks => self.env_dir_audiobooks.as_ref().ok_or_else(|| Error::with("requires Android 10 (API level 29) or higher"))?,
            },
            PublicDir::GeneralPurpose(dir) => match dir {
                PublicGeneralPurposeDir::Documents => &self.env_dir_documents,
                PublicGeneralPurposeDir::Download => &self.env_dir_download,
            },
        })
    }
}
