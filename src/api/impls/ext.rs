use std::{borrow::Cow, io::{Read, Write}};
use sync_async::sync_async;
use crate::*;
use super::*;


#[sync_async(
    use(if_async) async_utils::{run_blocking, sleep};
    use(if_sync) sync_utils::{run_blocking, sleep};
)]
impl<'a, R: tauri::Runtime> Impls<'a, R> {

    #[always_sync]
    pub fn api_level(&self) -> Result<i32> {
        Ok(self.consts()?.build_version_sdk_int)
    }

    #[always_sync]
    pub fn public_dir_name(&self, dir: impl Into<PublicDir>) -> Result<&'static str> {
        Ok(self.consts()?.public_dir_name(dir)?)
    }

    #[always_sync]
    pub fn temp_dir_path(&self) -> Result<&'static std::path::PathBuf> {
        get_or_init_temp_dir_path(|| {
            let path = self.private_dir_path(PrivateDir::NoBackupData)?
                .join("pluginAndroidFs-tempDir-01K486FKQ2BZSBGFD34RFH9FWJ");

            Ok(path)
        })
    }

    #[maybe_async]
    pub fn remove_all_temp_files(&self) -> Result<()> {
        let path = self.temp_dir_path()?;

        run_blocking(move || {
            let _g = LOCK_FOR_REMOVE_TEMP_FILE.lock();

            match std::fs::remove_dir_all(path) {
                Ok(_) => Ok(()),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
                Err(e) => Err(e.into()),
            }
        }).await
    }

    #[maybe_async]
    pub fn create_new_temp_file(&self) -> Result<(std::fs::File, std::path::PathBuf, FileUri)> {
        let temp_dir_path = self.temp_dir_path()?;

        run_blocking(move || {
            std::mem::drop(LOCK_FOR_REMOVE_TEMP_FILE.lock());

            std::fs::create_dir_all(&temp_dir_path).ok();

            let uid = next_uid_for_temp_file();
            let temp_file_path = temp_dir_path.join(format!("{uid}"));
            let temp_file_uri = FileUri::from_path(&temp_file_path);
            let temp_file = std::fs::File::create_new(&temp_file_path)?;

            Ok((temp_file, temp_file_path, temp_file_uri))
        }).await
    }

    #[maybe_async]
    pub fn get_file_mime_type(&self, uri: &FileUri) -> Result<String> {
        self.get_entry_type(uri).await?.into_file_mime_type_or_err()
    }

    #[maybe_async]
    pub fn get_entry_metadata(&self, uri: &FileUri) -> Result<std::fs::Metadata> {
        let file = self.open_file_readable(uri).await?;
        run_blocking(move || Ok(file.metadata()?)).await
    }

    #[maybe_async]
    pub fn open_file_readable(&self, uri: &FileUri) -> Result<std::fs::File> {
        self.open_file(uri, FileAccessMode::Read).await
    }

    #[maybe_async]
    pub fn open_file_writable(
        &self, 
        uri: &FileUri, 
    ) -> Result<std::fs::File> {

        if self.api_level()? <= api_level::ANDROID_9 {
            // Android 9 以下の場合、w は既存コンテンツを必ず切り捨てる
            #[allow(deprecated)]
            self.open_file(uri, FileAccessMode::Write).await
        }
        else {
            // Android 10 以上の場合、w は既存コンテンツの切り捨てを保証しない。
            // https://issuetracker.google.com/issues/180526528?pli=1
            #[allow(deprecated)]
            const WRITE_TRUNCATE_OR_NOT: FileAccessMode = FileAccessMode::Write;

            // 切り捨ててファイルを開く wt と rwt は全ての file provider が対応しているとは限らない。
            // よってフォールバックを用いてなるべく切り捨てて開けるように試みる。
            let (file, mode) = self.open_file_with_fallback(uri, [
                FileAccessMode::WriteTruncate, 
                FileAccessMode::ReadWriteTruncate,
                WRITE_TRUNCATE_OR_NOT
            ]).await?;

            if mode == WRITE_TRUNCATE_OR_NOT {
                // file provider が既存コンテンツを切り捨てず、
                // かつ書き込むデータ量が元のそれより少ない場合にファイルが壊れる可能性がある。
                // これを避けるため強制的にデータを切り捨てる。
                // ただし file provider の実装によっては set_len は失敗することがあるので最終手段。
                run_blocking(move || {
                    file.set_len(0)?;
                    Ok(file)
                }).await
            }
            else {
                Ok(file)
            }
        }
    }

    #[maybe_async]
    pub fn read_file(&self, uri: &FileUri) -> Result<Vec<u8>> {
        let mut file = self.open_file_readable(uri).await?;
        run_blocking(move || {
            let mut buf = file.metadata().ok()
                .map(|m| m.len() as usize)
                .map(Vec::with_capacity)
                .unwrap_or_else(Vec::new);

            file.read_to_end(&mut buf)?;
            Ok(buf)
        }).await
    }

    #[maybe_async]
    pub fn read_file_to_string(&self, uri: &FileUri) -> Result<String> {
        let mut file = self.open_file_readable(uri).await?;
        run_blocking(move || {
            let mut buf = file.metadata().ok()
                .map(|m| m.len() as usize)
                .map(String::with_capacity)
                .unwrap_or_else(String::new);

            file.read_to_string(&mut buf)?;
            Ok(buf)
        }).await
    }

    #[maybe_async]
    pub fn write_file(
        &self,
        uri: &FileUri, 
        contents: impl AsRef<[u8]>,
    ) -> Result<()> {

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
    pub fn copy_file(&self, src: &FileUri, dest: &FileUri) -> Result<()> {
        let mut src = self.open_file_readable(src).await?;
        let mut dest = self.open_file_writable(dest).await?;
        run_blocking(move || std::io::copy(&mut src, &mut dest).map_err(Into::into)).await?;
        Ok(())
    }

    #[maybe_async]
    pub fn get_file_thumbnail(
        &self, 
        uri: &FileUri,
        preferred_size: Size,
        format: ImageFormat,
    ) -> Result<Option<Vec<u8>>> {

        if let Some(t) = self.get_file_thumbnail_base64(uri, preferred_size, format).await? {
            use base64::engine::Engine;
            return Ok(Some(base64::engine::general_purpose::STANDARD.decode(t)?))
        }
        
        Ok(None)
    }

    #[maybe_async]
    pub fn is_dir(&self, uri: &FileUri) -> Result<bool> {
        if let Some(path) = uri.as_path().map(|p| p.to_path_buf()) {
            return run_blocking(move || Ok(std::fs::metadata(&path)?.is_dir())).await
        }

        Ok(self.get_entry_type(&uri).await?.is_dir())
    }

    #[maybe_async]
    pub fn is_file(&self, uri: &FileUri) -> Result<bool> {
        if let Some(path) = uri.as_path().map(|p| p.to_path_buf()) {
            return run_blocking(move || Ok(std::fs::metadata(&path)?.is_file())).await
        }
        
        Ok(self.get_entry_type(&uri).await?.is_file())
    }

    #[maybe_async]
    pub fn resolve_file_uri(
        &self, 
        dir: &FileUri, 
        relative_path: impl AsRef<std::path::Path>,
        allow_unchecked: bool
    ) -> Result<FileUri> {

        let mut uri = None;
        if let Some(built_uri) = self.try_build_path_uri(dir, relative_path.as_ref())? {
            uri = Some(built_uri);
        }
        else if let Some(built_uri) = self.try_build_saf_external_storage_provider_uri(dir, relative_path.as_ref())? {
            uri = Some(built_uri);
        }

        if let Some(uri) = uri {
            if allow_unchecked || self.is_file(&uri).await? {
               return Ok(uri) 
            }
            return Err(Error::with(format!("This is not a file: {uri:?}")))
        }
        
        self.find_saf_file_uri(dir, relative_path).await
    }

    #[maybe_async]
    pub fn resolve_dir_uri(
        &self,
        dir: &FileUri, 
        relative_path: impl AsRef<std::path::Path>,
        allow_unchecked: bool
    ) -> Result<FileUri> {

        let mut uri = None;
        if let Some(built_uri) = self.try_build_path_uri(dir, relative_path.as_ref())? {
            uri = Some(built_uri);
        }
        else if let Some(built_uri) = self.try_build_saf_external_storage_provider_uri(dir, relative_path.as_ref())? {
            uri = Some(built_uri);
        }
        
        if let Some(uri) = uri {
            if allow_unchecked || self.is_dir(&uri).await? {
               return Ok(uri) 
            }
            return Err(Error::with(format!("This is not a directory: {uri:?}")))
        }
        
        self.find_saf_dir_uri(dir, relative_path).await
    }

    #[always_sync]
    pub fn try_build_path_uri(
        &self,
        dir: &FileUri,
        relative_path: impl AsRef<std::path::Path>
    ) -> Result<Option<FileUri>> {

        let relative_path = validate_relative_path(relative_path.as_ref())?;

        // file:// 形式の URI
        if let Some(path) = dir.as_path() {
            let uri = FileUri::from_path(path.join(relative_path));
            return Ok(Some(uri))
        }

        Ok(None)
    }

    #[always_sync]
    pub fn try_build_saf_external_storage_provider_uri(
        &self,
        dir: &FileUri,
        relative_path: impl AsRef<std::path::Path>
    ) -> Result<Option<FileUri>> {

        let relative_path = validate_relative_path(relative_path.as_ref())?;
        let relative_path = relative_path.to_string_lossy();
        
        if dir.uri.starts_with("content://com.android.externalstorage.documents/tree/") {
            let uri = FileUri {
                document_top_tree_uri: dir.document_top_tree_uri.clone(),
                uri: format!("{}%2F{}", &dir.uri, encode_uri(relative_path))
            };
            return Ok(Some(uri))
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
    pub fn has_storage_permission_for_public_storage(&self) -> Result<bool> {
        if self.is_legacy_storage()? {
            self.has_legacy_storage_permission().await
        }
        else {
            Ok(true)
        }
    }

    #[maybe_async]
    pub fn get_available_storage_volumes_for_public_storage(&self) -> Result<Vec<StorageVolume>> {
        let volumes = self.get_available_storage_volumes().await?
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
    ) -> Result<FileUri> {

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
    ) -> Result<FileUri> {

        let uri = self.create_new_file_in_public_storage(
            volume_id, 
            base_dir, 
            relative_path, 
            mime_type,
            true
        ).await?;

        let mut file = self.open_file_writable(&uri).await?;

        #[if_sync]
        let result = file.write_all(contents.as_ref()).map_err(Into::into);

        #[if_async]
        let result = {
            let contents = upgrade_bytes_ref(contents);
            run_blocking(move || {
                file.write_all(&contents).map_err(Into::into)
            }).await 
        };

        if let Err(err) = result {
            self.remove_file(&uri).await.ok();
            return Err(err)
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
        let tmp_file_uri = self.create_new_file_in_public_storage(
            volume_id,
            base_dir, 
            relative_path.join("TMP-01K3CGCKYSAQ1GHF8JW5FGD4RW"), 
            Some(match base_dir {
                PublicDir::Image(_) => "image/png",
                PublicDir::Audio(_) => "audio/mp3",
                PublicDir::Video(_) => "video/mp4",
                PublicDir::GeneralPurpose(_) => "application/octet-stream"
            }),
            true
        ).await?;

        self.remove_file(&tmp_file_uri).await.ok();
        Ok(())
    }

    #[maybe_async]
    pub fn scan_file_in_public_storage(
        &self,
        uri: &FileUri,
        force: bool,
    ) -> Result<()> {
        
        if !force && api_level::ANDROID_10 <= self.api_level()? {
            return Ok(())
        }

        self.scan_media_store_file(uri).await
    }

    #[maybe_async]
    pub fn scan_file_in_public_storage_for_result(
        &self,
        uri: &FileUri,
        force: bool,
    ) -> Result<()> {
        
        if !force && api_level::ANDROID_10 <= self.api_level()? {
            return Ok(())
        }

        self.scan_media_store_file_for_result(uri).await
    }

    #[maybe_async]
    pub fn scan_file_by_path_in_public_storage(
        &self,
        path: impl AsRef<std::path::Path>,
        mime_type: Option<&str>,
    ) -> Result<FileUri> {

        self.scan_file_to_media_store_by_path(path, mime_type).await
    }

    #[maybe_async]
    pub fn get_file_path_in_public_storage(
        &self,
        uri: &FileUri,
    ) -> Result<std::path::PathBuf> {

        self.get_media_store_file_path(uri).await
    }

    #[maybe_async]
    pub fn set_file_pending_in_public_storage(
        &self,
        uri: &FileUri,
        is_pending: bool
    ) -> Result<()> {

        if api_level::ANDROID_10 <= self.api_level()? {
            return self.set_media_store_file_pending(uri, is_pending).await
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
                let top_dir_path = volume_id.top_dir_path
                    .as_ref()
                    .ok_or_else(|| Error::with("The storage volume is not available for PublicStorage"))?;
                  
                if !self.check_storage_volume_available_by_path(top_dir_path).await? {
                    return Err(Error::with("The storage volume is not currently available"))
                }

                top_dir_path.clone()
            },
            None => {
                self.get_primary_storage_volume_if_available_for_public_storage().await?
                    .ok_or_else(|| Error::with("Primary storage volume is not currently available"))?
                    .id.top_dir_path
                    .ok_or_else(|| Error::with("Primary storage volume is not available for PublicStorage"))?
            }
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
        create_dir_all: bool
    ) -> Result<FileUri> {

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
            uri.push_str(&encode_uri(relative_path_from_volume_root.to_string_lossy()));
      
            FileUri { uri, document_top_tree_uri: None }
        };

        if create_dir_all {
            self.create_dir_all_in_public_storage(
                volume_id, 
                base_dir, 
                relative_path
            ).await.ok();
        }

        Ok(uri)
    }

    #[maybe_async]
    pub fn resolve_root_initial_location(
        &self,
        volume_id: Option<&StorageVolumeId>
    ) -> Result<FileUri> {

        let volume_id = volume_id
            .and_then(|v| v.uid.as_deref())
            .unwrap_or("primary");

        if api_level::ANDROID_10 <= self.api_level()? {
            let base = "content://com.android.externalstorage.documents/root";
            let uri = format!("{base}/{volume_id}");
            Ok(FileUri { uri, document_top_tree_uri: None })
        }
        else {
            let base = "content://com.android.externalstorage.documents/document";
            let uri = format!("{base}/{volume_id}%3A");
            Ok(FileUri { uri, document_top_tree_uri: None })
        }
    }

    #[maybe_async]
    pub fn get_available_storage_volumes_for_app_storage(&self) -> Result<Vec<StorageVolume>> {
        let volumes = self.get_available_storage_volumes().await?
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
        dir: AppDir
    ) -> Result<std::path::PathBuf> {

        if let Some(volume_id) = volume_id {
            let dir_path = volume_id
                .app_dir_path(dir)
                .ok_or_else(|| Error::with("The storage volume has no app-speific directory"))?;
            
            if !self.check_storage_volume_available_by_path(dir_path).await? {
                return Err(Error::with("The storage volume is not currently available"))
            }

            return Ok(dir_path.clone())
        }

        self.get_primary_storage_volume_if_available_for_app_storage().await?
            .ok_or_else(|| Error::with("Primary storage volume is not currently available"))?
            .id.app_dir_path(dir).cloned()
            .ok_or_else(|| Error::with("Primary storage volume has no app-speific directory"))
    }

    #[maybe_async]
    pub fn get_public_media_file_path_in_app_storage(
        &self,
        uri: &FileUri,
    ) -> Result<std::path::PathBuf> {

        self.get_media_store_file_path(uri).await
    }
}

// Tokio の Mutex は async context 内の blocking lock でパニックになるので使わない
static LOCK_FOR_REMOVE_TEMP_FILE: std::sync::LazyLock<std::sync::Mutex<()>> = std::sync::LazyLock::new(|| std::sync::Mutex::new(()));

fn_get_or_init!(get_or_init_temp_dir_path, std::path::PathBuf);

fn next_uid_for_temp_file() -> usize {
    // temp file の寿命はアプリ終了までで、アプリ起動時に全て消される。
    // よって 0 から始まる単調増加の ID でいい。

    use std::sync::atomic::{AtomicUsize, Ordering};
    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    COUNTER.fetch_add(1, Ordering::Relaxed) 
}