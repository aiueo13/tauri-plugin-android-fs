use std::io::{Read, Write};
use sync_async::sync_async;
use crate::*;
use super::*;


#[sync_async(
    use(if_async) async_task as task;
    use(if_sync) sync_task as task;
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

    #[allow(unused)]
    #[maybe_async]
    pub fn remove_all_tmp_files(&self) -> Result<()> {
        let path = self.tmp_dir_path()?;

        task::run_blocking(move || {
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

        task::run_blocking(move || {
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
        task::run_blocking(move || Ok(file.metadata()?)).await
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
                task::run_blocking(move || {
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
        task::run_blocking(move || {
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
        task::run_blocking(move || {
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
            task::run_blocking(move ||{
                let result = stream.write_all(&contents);
                Ok((result, stream))
            }).await.expect("should not return err in run_blocking")
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
            task::run_blocking(move || std::io::copy(&mut src, &mut dest).map_err(Into::into)).await?;
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

        Ok(FileUri {
            document_top_tree_uri: dir.document_top_tree_uri.clone(),
            uri: format!("{base_dir}%2F{}", encode_document_id(relative_path))
        })
    }

    #[maybe_async]
    pub fn get_available_storage_volumes_for_public_storage(&self) -> Result<Vec<StorageVolume>> {
        self.requires(api_level::ANDROID_10)?;

        let volumes = self.get_available_storage_volumes().await?
            .into_iter()
            .filter(|v| v.id.media_store_volume_name.is_some())
            .collect::<Vec<_>>();

        Ok(volumes)
    }

    #[maybe_async]
    pub fn get_primary_storage_volume_if_available_for_public_storage(&self) -> Result<Option<StorageVolume>> {
        self.requires(api_level::ANDROID_10)?;

        self.get_primary_storage_volume_if_available()
            .await
            .map(|v| v.filter(|v| v.id.media_store_volume_name.is_some()))
    }

    #[maybe_async]
    pub fn create_dir_all_in_public_storage(
        &self,
        volume_id: Option<&StorageVolumeId>,
        base_dir: impl Into<PublicDir>,
        relative_path: impl AsRef<std::path::Path>, 
    ) -> Result<()> {

        self.requires(api_level::ANDROID_10)?;

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
    pub fn resolve_path(
        &self,
        volume_id: Option<&StorageVolumeId>,
        base_dir: impl Into<PublicDir>,
    ) -> Result<std::path::PathBuf> {

        self.requires(api_level::ANDROID_10)?;

        let mut path = match volume_id {
            Some(volume_id) => {
                let (vn, tp) = volume_id.media_store_volume_name.as_ref()
                    .zip(volume_id.top_directory_path.as_ref())
                    .ok_or_else(|| Error::with("The storage volume is not available for PublicStorage"))?;
                
                if !self.check_media_store_volume_name_available(vn).await? {
                    return Err(Error::with("The storage volume is not currently available"))
                }

                tp.clone()
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
    pub fn resolve_public_storage_initial_location(
        &self,
        volume_id: Option<&StorageVolumeId>,
        base_dir: impl Into<PublicDir>,
        relative_path: impl AsRef<std::path::Path>,
        create_dir_all: bool
    ) -> Result<FileUri> {

        let base_dir = base_dir.into();
            
        let mut uri = self.resolve_public_storage_initial_location_top(volume_id).await?;
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
    pub fn resolve_public_storage_initial_location_top(
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
            .filter(|v| v.id.private_data_dir_path.is_some() || v.id.private_cache_dir_path.is_some())
            .collect::<Vec<_>>();

        Ok(volumes)
    }

    #[maybe_async]
    pub fn get_primary_storage_volume_if_available_for_private_storage(&self) -> Result<Option<StorageVolume>> {
        self.get_primary_storage_volume_if_available()
            .await
            .map(|v| v.filter(|v| v.id.private_data_dir_path.is_some() || v.id.private_cache_dir_path.is_some()))
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


get_or_init!(get_or_init_tmp_dir_path, std::path::PathBuf);

fn next_id_for_tmp_file() -> usize {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    COUNTER.fetch_add(1, Ordering::Relaxed) 
}