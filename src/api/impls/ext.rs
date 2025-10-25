use std::{borrow::Cow, io::{Read, Write}};
use sync_async::sync_async;
use crate::*;
use super::*;


#[sync_async(
    use(if_async) async_utils::{run_blocking, run_blocking_with_io_err, sleep};
    use(if_sync) sync_utils::{run_blocking, run_blocking_with_io_err, sleep};
)]
impl<'a, R: tauri::Runtime> Impls<'a, R> {

    #[always_sync]
    pub fn api_level(&self) -> Result<i32> {
        Ok(self.consts()?.build_version_sdk_int)
    }

    #[always_sync]
    pub fn requires(&self, api_level: i32) -> Result<()> {
        let current = self.api_level()?;
        if api_level <= current {
            return Ok(())
        }

        Err(Error::with(format!("requires Android API level {api_level} or higher. but: {current}")))
    }

    #[always_sync]
    pub fn tmp_dir_path(&self) -> Result<&'static std::path::PathBuf> {
        get_or_init_tmp_dir_path(|| {
            let path = self.internal_private_dir_path(PrivateDir::Cache)?
                .join("pluginAndroidFs-tmpDir-01K486FKQ2BZSBGFD34RFH9FWJ");

            Ok(path)
        })
    }

    #[maybe_async]
    pub fn remove_all_tmp_files(&self) -> Result<()> {
        let path = self.tmp_dir_path()?;

        run_blocking(move || {
            match std::fs::remove_dir_all(path) {
                Ok(_) => Ok(()),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
                Err(e) => Err(e.into()),
            }
        }).await
    }

    #[maybe_async]
    pub fn create_new_tmp_file(&self) -> Result<(std::fs::File, std::path::PathBuf)> {
        let tmp_dir_path = self.tmp_dir_path()?;

        run_blocking(move || {
            std::fs::create_dir_all(&tmp_dir_path).ok();

            let uid = next_id_for_tmp_file();
            let tmp_file_path = tmp_dir_path.join(format!("{uid}"));
            let tmp_file = std::fs::File::create_new(&tmp_file_path)?;

            Ok((tmp_file, tmp_file_path))
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

        #[allow(deprecated)]
        const WRITE_TRUNCATE_OR_NOT: FileAccessMode = FileAccessMode::Write;

        // Android 9 以下の場合、w は既存コンテンツを切り捨てる
        if self.api_level()? <= api_level::ANDROID_9 {
            self.open_file(uri, WRITE_TRUNCATE_OR_NOT).await
        }
        // Android 10 以上の場合、w は既存コンテンツの切り捨てを保証しない。
        // そのため切り捨ててファイルを開くには wt を用いる必要があるが、
        // wt は全ての file provider が対応しているとは限らないため、
        // フォールバックを用いてなるべく多くの状況に対応する。
        // https://issuetracker.google.com/issues/180526528?pli=1
        else {
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
    pub fn write_file_auto(
        &self,
        uri: &FileUri, 
        contents: impl AsRef<[u8]>,
    ) -> Result<()> {

        let need_write_via_kotlin = self.need_write_file_via_kotlin(uri).await?;
        self.write_file(uri, contents, need_write_via_kotlin).await
    }

    #[maybe_async]
    pub fn write_file_via_kotlin(
        &self,
        uri: &FileUri, 
        contents: impl AsRef<[u8]>,
    ) -> Result<()> {

        let need_write_via_kotlin = true;
        self.write_file(uri, contents, need_write_via_kotlin).await
    }

    #[maybe_async]
    pub fn write_file(
        &self,
        uri: &FileUri, 
        contents: impl AsRef<[u8]>,
        need_write_via_kotlin: bool,
    ) -> Result<()> {

        let mut stream = self.create_writable_stream(uri, need_write_via_kotlin).await?;

        #[if_sync]
        let result = stream.write_all(contents.as_ref());

        #[if_async]
        let (result, stream) = {
            let contents = upgrade_bytes_ref(contents);
            run_blocking(move ||{
                // run_blocking内ではエラーを返さないようにする。
                let result = stream.write_all(&contents);
                Ok((result, stream))
            }).await? 
            // 起こることはないと思うが、
            // tokio の spwan_blocking でエラーになった場合は早期 return する。
            // reflect や dispose_without_reflct は最悪呼び出されなくてもいい。
        };

        match result {
            Ok(_) => stream.reflect().await,
            Err(e) => {
                stream.dispose_without_reflect().await.ok();
                Err(e.into())
            },
        }
    }

    #[maybe_async]
    pub fn copy_file(&self, src: &FileUri, dest: &FileUri) -> Result<()> {
        if self.need_write_file_via_kotlin(dest).await? {
            self.copy_file_via_kotlin(src, dest, None).await?;
        }
        else {
            // std::io::copy は std::fs::File 同士のコピーの場合、最適化が働く可能性がある。
            // そのため WritableStream は用いない。
            let mut src = self.open_file_readable(src).await?;
            let mut dest = self.open_file_writable(dest).await?;
            run_blocking(move || std::io::copy(&mut src, &mut dest).map_err(Into::into)).await?;
        }
        Ok(())
    }

    #[maybe_async]
    pub fn need_write_file_via_kotlin(&self, uri: &FileUri) -> Result<bool> {
        // - https://issuetracker.google.com/issues/200201777
        // - https://stackoverflow.com/questions/51015513/fileoutputstream-writes-0-bytes-to-google-drive
        // - https://stackoverflow.com/questions/51490194/file-written-using-action-create-document-is-empty-on-google-drive-but-not-local
        // - https://community.latenode.com/t/csv-export-to-google-drive-results-in-empty-file-but-local-storage-works-fine 
        // 
        // Intent.ACTION_OPEN_DOCUMENT や Intent.ACTION_CREATE_DOCUMENT などの SAF で
        // 取得した Google Drive のファイルに対して生の FD を用いて書き込んだ場合、
        // それが反映されず空のファイルのみが残ることがある。
        // これの対処法として Context.openOutputStream から得た OutputStream で書き込んだ後
        // flush 関数を使うことで反映させることができる。
        // このプラグインでは Context.openAssetFileDescriptor から FD を取得して操作しているが
        // これはハック的な手法ではなく公式の doc でも SAF の例として用いられている手法であるため
        // この動作は仕様ではなく GoogleDrive 側のバグだと考えていいと思う。
        // 
        // また Web を調べたが GoogleDrive 以外でこのような問題が起こるのは見つけれなかった。
        // 実際、試した限りでは DropBox で書き込んだものが普通に反映された。
        // もしかしたら他のクラウドストレージアプリでは起こるかもしれないが、
        // それは仕様ではなく FileProvider 側のバグ？だと思うのでこちら側ではコストを考え
        // ホワイトリスト方式ではなくブラックリスト方式を用いて判定する。
        
        const TARGET_URI_PREFIXES: &'static [&'static str] = &[
            "content://com.google.android.apps.docs", // Google drive
        ];

        Ok(TARGET_URI_PREFIXES.iter().any(|prefix| uri.uri.starts_with(prefix)))
    }

    #[maybe_async]
    pub fn try_resolve_file_uri(
        &self, 
        dir: &FileUri, 
        relative_path: impl AsRef<std::path::Path>
    ) -> Result<FileUri> {

        let uri = self.resolve_entry_uri_unvalidated(dir, relative_path).await?;         

        if !self.get_entry_type(&uri).await?.is_file() {
            return Err(crate::Error::with(format!("This is not a file: {uri:?}")))
        }
        Ok(uri)
    }

    #[maybe_async]
    pub fn try_resolve_dir_uri(
        &self,
        dir: &FileUri, 
        relative_path: impl AsRef<std::path::Path>
    ) -> Result<FileUri> {

        let uri = self.resolve_entry_uri_unvalidated(dir, relative_path).await?;
            
        if !self.get_entry_type(&uri).await?.is_dir() {
            return Err(crate::Error::with(format!("This is not a directory: {uri:?}")))
        }
        Ok(uri)
    }

    #[maybe_async]
    pub fn resolve_entry_uri_unvalidated(
        &self, 
        dir: &FileUri, 
        relative_path: impl AsRef<std::path::Path>
    ) -> Result<FileUri> {

        let base_dir = &dir.uri;
        let relative_path = validate_relative_path(relative_path.as_ref())?;
        let relative_path = relative_path.to_string_lossy();

        if relative_path.is_empty() {
            return Ok(dir.clone())
        }
        if let Some(path) = dir.as_path() {
            return Ok(FileUri::from_path(path.join(relative_path.as_ref())))
        }

        Ok(FileUri {
            document_top_tree_uri: dir.document_top_tree_uri.clone(),
            uri: format!("{base_dir}%2F{}", encode_document_id(relative_path))
        })
    }

    #[maybe_async]
    pub fn request_storage_permission_for_public_storage(&self) -> Result<bool> {
        if api_level::ANDROID_10 <= self.api_level()? {
            return Ok(true)
        }
        
        self.request_legacy_storage_permission().await
    }

    #[maybe_async]
    pub fn has_storage_permission_for_public_storage(&self) -> Result<bool> {
        if api_level::ANDROID_10 <= self.api_level()? {
            return Ok(true)
        }
        
        self.has_legacy_storage_permission().await
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
    pub fn create_new_file_in_public_store(
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
    pub fn write_new_file_in_public_store(
        &self,
        volume_id: Option<&StorageVolumeId>,
        base_dir: impl Into<PublicDir>,
        relative_path: impl AsRef<std::path::Path>, 
        mime_type: Option<&str>,
        contents: impl AsRef<[u8]>,
    ) -> Result<FileUri> {

        let uri = self.create_new_file_in_public_store(
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
            run_blocking(move ||{
                // run_blocking内ではエラーを返さないようにする。
                file.write_all(&contents).map_err(Into::into)
            }).await 
        };

        if let Err(err) = result {
            self.remove_file(&uri).await.ok();
            return Err(err)
        }

        self.set_file_pending_in_public_storage(&uri, false).await?;
        self.scan_file_in_public_storage(&uri).await?;
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
        let tmp_file_uri = self.create_new_file_in_public_store(
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
    ) -> Result<()> {
        
        if api_level::ANDROID_11 <= self.api_level()? {
            return Ok(())
        }

        self.scan_media_store_file(uri).await
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
    pub fn resolve_path_in_public_storage(
        &self,
        volume_id: Option<&StorageVolumeId>,
        base_dir: impl Into<PublicDir>,
    ) -> Result<std::path::PathBuf> {

        let mut path = match volume_id {
            Some(volume_id) => {
                let path = volume_id.top_directory_path
                    .as_ref()
                    .ok_or_else(|| Error::with("The storage volume is not available for PublicStorage"))?;
                  
                if !self.check_storage_volume_available_by_path(path).await? {
                    return Err(Error::with("The storage volume is not currently available"))
                }

                path.clone()
            },
            None => {
                self.get_primary_storage_volume_if_available_for_public_storage().await?
                    .and_then(|v| v.id.top_directory_path)
                    .ok_or_else(|| Error::with("Primary storage volume is not currently available"))?
            }
        };

        path.push(self.consts()?.public_dir_name(base_dir)?);
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
            
        let mut uri = self.resolve_initial_location_top_in_public_storage(volume_id).await?;
        uri.uri.push_str(self.consts()?.public_dir_name(base_dir)?);

        let relative_path = validate_relative_path(relative_path.as_ref())?;
        let relative_path = relative_path.to_string_lossy();
        if !relative_path.is_empty() {
            uri.uri.push_str("%2F");
            uri.uri.push_str(&encode_document_id(relative_path.as_ref()));
        }

        if create_dir_all {
            self.create_dir_all_in_public_storage(
                volume_id, 
                base_dir, 
                relative_path.as_ref()
            ).await.ok();
        }

        Ok(uri)
    }

    #[maybe_async]
    pub fn resolve_initial_location_top_in_public_storage(
        &self,
        volume_id: Option<&StorageVolumeId>
    ) -> Result<FileUri> {

        let volume_id = volume_id
            .and_then(|v| v.uuid.as_deref())
            .unwrap_or("primary");

        Ok(FileUri {
            uri: format!("content://com.android.externalstorage.documents/document/{volume_id}%3A"),
            document_top_tree_uri: None 
        })
    }

    #[maybe_async]
    pub fn get_available_storage_volumes_for_private_storage(&self) -> Result<Vec<StorageVolume>> {
        let volumes = self.get_available_storage_volumes().await?
            .into_iter()
            .filter(|v| v.is_available_for_private_storage)
            .collect::<Vec<_>>();

        Ok(volumes)
    }

    #[maybe_async]
    pub fn get_primary_storage_volume_if_available_for_private_storage(&self) -> Result<Option<StorageVolume>> {
        self.get_primary_storage_volume_if_available()
            .await
            .map(|v| v.filter(|v| v.is_available_for_private_storage))
    }

    #[maybe_async]
    pub fn resolve_outside_private_dir_path(
        &self, 
        volume_id: Option<&StorageVolumeId>,
        dir: OutsidePrivateDir
    ) -> Result<std::path::PathBuf> {

        if let Some(volume_id) = volume_id {
            let dir_path = volume_id
                .outside_private_dir_path(dir)
                .ok_or_else(|| Error::with("The storage volume has no app-speific directory"))?;
            
            if !self.check_storage_volume_available_by_path(dir_path).await? {
                return Err(Error::with("The storage volume is not currently available"))
            }

            return Ok(dir_path.clone())
        }

        self.get_primary_storage_volume_if_available_for_private_storage().await?
            .and_then(|v| v.id.outside_private_dir_path(dir).map(Clone::clone))
            .ok_or_else(|| Error::with("Primary storage volume is not currently available"))
    }
}


fn_get_or_init!(get_or_init_tmp_dir_path, std::path::PathBuf);

fn next_id_for_tmp_file() -> usize {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    COUNTER.fetch_add(1, Ordering::Relaxed) 
}