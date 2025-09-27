#[allow(unused)]
use std::io::{Read as _, Write as _};
use crate::*;


/// ***Root API***  
/// 
/// # Examples
/// ```
/// fn example(app: &tauri::AppHandle) {
///     use tauri_plugin_android_fs::AndroidFsExt as _;
/// 
///     let api = app.android_fs();
/// }
/// ```
pub struct AndroidFs<R: tauri::Runtime> {
    #[allow(unused)]
    pub(crate) app: tauri::AppHandle<R>, 

    #[cfg(target_os = "android")]
    pub(crate) api: tauri::plugin::PluginHandle<R>, 

    #[cfg(target_os = "android")]
    pub(crate) intent_lock: std::sync::Mutex<()>,
}

impl<R: tauri::Runtime> AndroidFs<R> {

    pub(crate) fn new<C: serde::de::DeserializeOwned>(
        app: tauri::AppHandle<R>,
        api: tauri::plugin::PluginApi<R, C>,
    ) -> crate::Result<Self> {

        #[cfg(target_os = "android")] {
            Ok(Self {
                api: api.register_android_plugin("com.plugin.android_fs", "AndroidFsPlugin")?, 
                app,
                intent_lock: std::sync::Mutex::new(())
            })
        }
        
        #[cfg(not(target_os = "android"))] {
            Ok(Self { app })
        }
    }
}

impl<R: tauri::Runtime> AndroidFs<R> {

    /// API of file storage intended for the app's use only.
    pub fn private_storage(&self) -> PrivateStorage<'_, R> {
        PrivateStorage(self)
    }

    /// API of file storage that is available to other applications and users.
    pub fn public_storage(&self) -> PublicStorage<'_, R> {
        PublicStorage(self)
    }

    /// API of file/dir picker.
    pub fn file_picker(&self) -> FilePicker<'_, R> {
        FilePicker(self)
    }

    /// API of opening file/dir with other apps.
    pub fn file_opener(&self) -> FileOpener<'_, R> {
        FileOpener(self)
    }

    /// Get the file or directory name.  
    /// 
    /// # Args
    /// - ***uri*** :  
    /// Target URI.  
    /// Must be **readable**.
    /// 
    /// # Support
    /// All Android version.
    pub fn get_name(&self, uri: &FileUri) -> crate::Result<String> {
        on_android!({
            impl_se!(struct Req<'a> { uri: &'a FileUri });
            impl_de!(struct Res { name: String });

            self.api
                .run_mobile_plugin::<Res>("getName", Req { uri })
                .map(|v| v.name)
                .map_err(Into::into)
        })
    }

    /// Queries the provider to get the MIME type.
    ///
    /// For files in [`PrivateStorage`], the MIME type is determined from the file extension.  
    /// In most other cases, it uses the MIME type that was associated with the file when it was created.  
    /// If the MIME type is unknown or unset, it falls back to `"application/octet-stream"`.  
    /// 
    /// If the target is a directory, an error will occur.  
    /// To check whether the target is a file or a directory, use [`AndroidFs::get_type`].  
    /// 
    /// # Args
    /// - ***uri*** :  
    /// Target file URI.  
    /// Must be **readable**.
    /// 
    /// # Support
    /// All Android version.
    pub fn get_mime_type(&self, uri: &FileUri) -> crate::Result<String> {
        on_android!({
            self.get_type(uri)?.into_file_mime_type_or_err()
        })
    }

    /// Gets the entry type.
    ///
    /// If the target is a directory, returns [`EntryType::Dir`].
    ///
    /// If the target is a file, returns [`EntryType::File { mime_type }`](EntryType::File).  
    /// For files in [`PrivateStorage`], the MIME type is determined from the file extension.  
    /// In most other cases, it uses the MIME type that was associated with the file when it was created.  
    /// If the MIME type is unknown or unset, it falls back to `"application/octet-stream"`.  
    /// 
    /// # Args
    /// - ***uri*** :  
    /// Target URI.  
    /// Must be **readable**.
    /// 
    /// # Support
    /// All Android version.
    pub fn get_type(&self, uri: &FileUri) -> crate::Result<EntryType> {
        on_android!({
            impl_se!(struct Req<'a> { uri: &'a FileUri });
            impl_de!(struct Res { value: Option<String> });

            self.api
                .run_mobile_plugin::<Res>("getMimeType", Req { uri })
                .map(|v| match v.value {
                    Some(mime_type) => EntryType::File { mime_type },
                    None => EntryType::Dir,
                })
                .map_err(Into::into)
        })
    }

    /// Queries the file system to get information about a file, directory.
    /// 
    /// # Args
    /// - ***uri*** :  
    /// Target URI.  
    /// Must be **readable**.
    /// 
    /// # Note
    /// This uses [`AndroidFs::open_file`] internally.
    /// 
    /// # Support
    /// All Android version.
    pub fn get_metadata(&self, uri: &FileUri) -> crate::Result<std::fs::Metadata> {
        on_android!({
            let file = self.open_file_readable(uri)?;
            Ok(file.metadata()?)
        })
    }

    /// Open the file in **readable** mode. 
    /// 
    /// # Note
    /// If the target is a file on cloud storage or otherwise not physically present on the device,
    /// the file provider may downloads the entire contents, and then opens it. 
    /// As a result, this processing may take longer than with regular local files.
    /// And files might be a pair of pipe or socket for streaming data.
    /// 
    /// # Args
    /// - ***uri*** :  
    /// Target file URI.  
    /// This need to be **readable**.
    /// 
    /// # Support
    /// All Android version.
    pub fn open_file_readable(&self, uri: &FileUri) -> Result<std::fs::File> {
        self.open_file(uri, FileAccessMode::Read)
    }

    /// Open the file in **writable** mode.  
    /// This truncates the existing contents.  
    /// 
    /// # Note
    /// For file provider of some cloud storage, 
    /// writing by file descriptor like std::fs may not correctoly notify and reflect changes. 
    /// If you need to write to such files, use [`AndroidFs::open_writable_stream`].
    /// It will fall back to Kotlin API as needed.
    /// And you can check by [`AndroidFs::need_write_via_kotlin`].
    /// 
    /// # Args
    /// - ***uri*** :  
    /// Target file URI.  
    /// This need to be **writable**.
    /// 
    /// # Support
    /// All Android version.
    pub fn open_file_writable(
        &self, 
        uri: &FileUri, 
    ) -> crate::Result<std::fs::File> {

        on_android!(#[allow(deprecated)] {
            // Android 9 以下の場合、w は既存コンテンツを切り捨てる
            if self.api_level()? <= api_level::ANDROID_9 {
                self.open_file(uri, FileAccessMode::Write)
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
                    FileAccessMode::Write
                ])?;

                if mode == FileAccessMode::Write {
                    // file provider が既存コンテンツを切り捨てず、
                    // かつ書き込むデータ量が元のそれより少ない場合にファイルが壊れる可能性がある。
                    // これを避けるため強制的にデータを切り捨てる。
                    // file provider の実装によっては set_len は失敗することがあるので最終手段。
                    file.set_len(0)?;
                }

                Ok(file)
            }
        })
    }

    /// Open the file in the specified mode.  
    /// 
    /// # Note
    /// If the target is a file on cloud storage or otherwise not physically present on the device,
    /// the file provider may downloads the entire contents, and then opens it. 
    /// As a result, this processing may take longer than with regular local files.
    /// And files might be a pair of pipe or socket for streaming data.
    /// 
    /// When writing to a file with this function,
    /// pay attention to the following points:
    /// 
    /// 1. **File reflection**:  
    /// For file provider of some cloud storage, 
    /// writing by file descriptor like std::fs may not correctoly notify and reflect changes. 
    /// If you need to write to such files, use [`AndroidFs::open_writable_stream`].
    /// It will fall back to Kotlin API as needed.
    /// And you can check by [`AndroidFs::need_write_via_kotlin`].
    /// 
    /// 2. **File mode restrictions**:  
    /// Files provided by third-party apps may not support writable modes other than
    /// [`FileAccessMode::Write`]. However, [`FileAccessMode::Write`] does not guarantee
    /// that existing contents will always be truncated.  
    /// As a result, if the new contents are shorter than the original, the file may
    /// become corrupted. To avoid this, consider using
    /// [`AndroidFs::open_file_writable`] or [`AndroidFs::open_writable_stream`], which
    /// ensure that existing contents are truncated and also automatically apply the
    /// maximum possible fallbacks.  
    /// <https://issuetracker.google.com/issues/180526528>
    /// 
    /// # Args
    /// - ***uri*** :  
    /// Target file URI.  
    /// This must have corresponding permissions (read, write, or both) for the specified ***mode***.
    /// 
    /// - ***mode*** :  
    /// Indicates how the file is opened and the permissions granted. 
    /// The only ones that can be expected to work in all cases are [`FileAccessMode::Write`] and [`FileAccessMode::Read`].
    /// Because files hosted by third-party apps may not support others.
    /// 
    /// # Support
    /// All Android version.
    pub fn open_file(&self, uri: &FileUri, mode: FileAccessMode) -> crate::Result<std::fs::File> {
        on_android!({
            impl_se!(struct Req<'a> { uri: &'a FileUri, mode: &'a str });
            impl_de!(struct Res { fd: std::os::fd::RawFd });
    
            let mode = mode.to_mode();

            self.api
                .run_mobile_plugin::<Res>("getFileDescriptor", Req { uri, mode })
                .map(|v| {
                    use std::os::fd::FromRawFd;
                    unsafe { std::fs::File::from_raw_fd(v.fd) }
                })
                .map_err(Into::into)
        })
    }
 
    /// For detailed documentation and notes, see [`AndroidFs::open_file`].  
    ///
    /// The modes specified in ***candidate_modes*** are tried in order.  
    /// If the file can be opened, this returns the file along with the mode used.  
    /// If all attempts fail, an error is returned.  
    pub fn open_file_with_fallback(
        &self, 
        uri: &FileUri, 
        candidate_modes: impl IntoIterator<Item = FileAccessMode>
    ) -> crate::Result<(std::fs::File, FileAccessMode)> {

        on_android!({
            impl_se!(struct Req<'a> { uri: &'a FileUri, modes: Vec<&'a str> });
            impl_de!(struct Res { fd: std::os::fd::RawFd, mode: String });
    
            let modes = candidate_modes.into_iter().map(|m| m.to_mode()).collect::<Vec<_>>();

            if modes.is_empty() {
                return Err(Error::with("candidate_modes must not be empty"));
            }

            self.api
                .run_mobile_plugin::<Res>("getFileDescriptorWithFallback", Req { uri, modes })
                .map_err(Into::into)
                .and_then(|v| FileAccessMode::from_mode(&v.mode).map(|m| (v.fd, m)))
                .map(|(fd, mode)| {
                    let file = {
                        use std::os::fd::FromRawFd;
                        unsafe { std::fs::File::from_raw_fd(fd) }
                    };
                    (file, mode)
                })
        })
    }

    /// Opens a stream for writing to the specified file.  
    /// This truncates the existing contents.  
    /// 
    /// # Usage
    /// [`WritableStream`] implements [`std::io::Write`], so it can be used for writing.  
    /// As with [`std::fs::File`], wrap it with [`std::io::BufWriter`] if buffering is needed.  
    ///
    /// After writing, call [`WritableStream::reflect`].  
    /// 
    /// # Note
    /// The behavior depends on [`AndroidFs::need_write_via_kotlin`].  
    /// If it is `false`, this behaves like [`AndroidFs::open_file_writable`].  
    /// If it is `true`, this behaves like [`AndroidFs::open_writable_stream_via_kotlin`].  
    /// 
    /// # Args
    /// - ***uri*** :  
    /// Target file URI.  
    /// This need to be **writable**.
    /// 
    /// # Support
    /// All Android version.
    pub fn open_writable_stream(
        &self,
        uri: &FileUri
    ) -> Result<WritableStream<R>> {

        on_android!({
            let need_write_via_kotlin = self.need_write_via_kotlin(uri)?;
            WritableStream::new(self.app.clone(), uri.clone(), need_write_via_kotlin)
        })
    }

    /// Opens a writable stream to the specified file.  
    /// This truncates the existing contents.  
    /// 
    /// This function always writes content via the Kotlin API.
    /// But this takes several times longer compared.  
    /// [`AndroidFs::open_writable_stream`] automatically falls back to this function depending on [`AndroidFs::need_write_via_kotlin`].  
    /// 
    /// # Usage
    /// [`WritableStream`] implements [`std::io::Write`], so it can be used for writing.  
    /// As with [`std::fs::File`], wrap it with [`std::io::BufWriter`] if buffering is needed.  
    ///
    /// After writing, call [`WritableStream::reflect`].
    /// 
    /// # Args
    /// - ***uri*** :  
    /// Target file URI.  
    /// This need to be **writable**.
    /// 
    /// # Support
    /// All Android version.
    pub fn open_writable_stream_via_kotlin(
        &self,
        uri: &FileUri
    ) -> Result<WritableStream<R>> {

        on_android!({
            let need_write_via_kotlin = true;
            WritableStream::new(self.app.clone(), uri.clone(), need_write_via_kotlin)
        })
    }

    /// Reads the entire contents of a file into a bytes vector.  
    /// 
    /// # Args
    /// - ***uri*** :  
    /// Target file URI.    
    /// Must be **readable**.
    /// 
    /// # Support
    /// All Android version.
    pub fn read(&self, uri: &FileUri) -> crate::Result<Vec<u8>> {
        on_android!({
            let mut file = self.open_file_readable(uri)?;
            let mut buf = file.metadata().ok()
                .map(|m| m.len() as usize)
                .map(Vec::with_capacity)
                .unwrap_or_else(Vec::new);

            file.read_to_end(&mut buf)?;
            Ok(buf)
        })
    }

    /// Reads the entire contents of a file into a string.  
    /// 
    /// # Args
    /// - ***uri*** :  
    /// Target file URI.  
    /// Must be **readable**.
    /// 
    /// # Support
    /// All Android version.
    pub fn read_to_string(&self, uri: &FileUri) -> crate::Result<String> {
        on_android!({
            let mut file = self.open_file_readable(uri)?;
            let mut buf = file.metadata().ok()
                .map(|m| m.len() as usize)
                .map(String::with_capacity)
                .unwrap_or_else(String::new);
    
            file.read_to_string(&mut buf)?;
            Ok(buf)
        })
    }

    /// Writes a slice as the entire contents of a file.  
    /// This function will entirely replace its contents if it does exist.    
    /// 
    /// # Note
    /// The behavior depends on [`AndroidFs::need_write_via_kotlin`].  
    /// If it is `false`, this uses [`std::fs::File::write_all`].  
    /// If it is `true`, this uses [`AndroidFs::write_via_kotlin`].  
    /// 
    /// # Args
    /// - ***uri*** :  
    /// Target file URI.  
    /// Must be **writable**.
    /// 
    /// # Support
    /// All Android version.
    pub fn write(&self, uri: &FileUri, contents: impl AsRef<[u8]>) -> crate::Result<()> {
        on_android!({
            let mut stream = self.open_writable_stream(uri)?;
            stream.write_all(contents.as_ref())?;
            stream.reflect()?;
            Ok(())
        })
    }

    /// Writes a slice as the entire contents of a file.  
    /// This function will entirely replace its contents if it does exist.    
    /// 
    /// This function always writes content via the Kotlin API.
    /// But this takes several times longer compared.   
    /// [`AndroidFs::write`] automatically falls back to this function depending on [`AndroidFs::need_write_via_kotlin`].  
    /// 
    /// # Support
    /// All Android version.
    pub fn write_via_kotlin(
        &self, 
        uri: &FileUri,
        contents: impl AsRef<[u8]>
    ) -> crate::Result<()> {

        on_android!({
            let mut stream = self.open_writable_stream_via_kotlin(uri)?;
            stream.write_all(contents.as_ref())?;
            stream.reflect()?;
            Ok(())
        })
    }

    /// Copies the contents of the source file to the destination.  
    /// If the destination already has contents, they are truncated before writing the source contents.  
    /// 
    /// # Note
    /// The behavior depends on [`AndroidFs::need_write_via_kotlin`].  
    /// If it is `false`, this uses [`std::io::copy`] with [`std::fs::File`].  
    /// If it is `true`, this uses [`AndroidFs::copy_via_kotlin`].  
    /// 
    /// # Args
    /// - ***src*** :  
    /// The URI of source file.   
    /// Must be **readable**.
    /// 
    /// - ***dest*** :  
    /// The URI of destination file.  
    /// Must be **writable**.
    /// 
    /// # Support
    /// All Android version.
    pub fn copy(&self, src: &FileUri, dest: &FileUri) -> crate::Result<()> {
        on_android!({
            if self.need_write_via_kotlin(dest)? {
                self.copy_via_kotlin(src, dest, None)?;
            }
            else {
                // std::io::copy は std::fs::File 同士のコピーの場合、最適化が働く可能性がある。
                // そのため self.open_writable_stream は用いない。
                let src = &mut self.open_file_readable(src)?;
                let dest = &mut self.open_file_writable(dest)?;
                std::io::copy(src, dest)?;
            }
            Ok(())
        })
    }

    /// Copies the contents of src file to dest.  
    /// If dest already has contents, it is truncated before write src contents.  
    /// 
    /// This function always writes content via the Kotlin API.  
    /// [`AndroidFs::copy`] automatically falls back to this function depending on [`AndroidFs::need_write_via_kotlin`].   
    /// 
    /// # Args
    /// - ***src*** :  
    /// The URI of source file.   
    /// Must be **readable**.
    /// 
    /// - ***dest*** :  
    /// The URI of destination file.  
    /// Must be **writable**.
    /// 
    /// - ***buffer_size***:  
    /// The size of the buffer, in bytes, to use during the copy process on Kotlin.  
    /// If `None`, [`DEFAULT_BUFFER_SIZE`](https://kotlinlang.org/api/core/kotlin-stdlib/kotlin.io/-d-e-f-a-u-l-t_-b-u-f-f-e-r_-s-i-z-e.html) is used. 
    /// At least, when I checked, it was 8 KB.  
    /// If zero, this causes error.
    /// 
    /// # Support
    /// All Android version.
    pub fn copy_via_kotlin(
        &self, 
        src: &FileUri, 
        dest: &FileUri,
        buffer_size: Option<u32>,
    ) -> crate::Result<()> {

        on_android!({
            impl_se!(struct Req<'a> { src: &'a FileUri, dest: &'a FileUri, buffer_size: Option<u32> });
            impl_de!(struct Res;);

            if buffer_size.is_some_and(|s| s <= 0) {
                return Err(Error { msg: "buffer_size must be non zero".into() })
            }

            self.api
                .run_mobile_plugin::<Res>("copyFile", Req { src, dest, buffer_size })
                .map(|_| ())
                .map_err(Into::into)
        })
    }

    /// Determines whether the file must be written via the Kotlin API rather than through a file descriptor.
    /// 
    /// In the case of a file that physically exists on the device, this will always return false.
    /// This is intended for special cases, such as some cloud storage.
    /// 
    /// # Support
    /// All Android version.
    pub fn need_write_via_kotlin(&self, uri: &FileUri) -> crate::Result<bool> {
        on_android!({
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
            // それは仕様ではなく FileProvider 側のバグ？だと思うのでこちら側ではコストを考え対処療法のみを行う。
            // つまりホワイトリスト方式ではなくブラックリスト方式を用いて判定する。
            //
            // 
            // 未来の自分用: 
            // Context.openOutputStream は内部で Context.openAssetFileDescriptor を使っている。
            // その関数が返す ParcelFileDescriptor の releaseResources 関数が怪しい。
            // 787行目: https://android.googlesource.com/platform/frameworks/base/+/refs/heads/android10-mainline-media-release/core/java/android/os/ParcelFileDescriptor.java?utm_source=chatgpt.com%2F%2F%2F
            // releaseResources に関して、doc では FD が閉じられる際に呼ばれるフック関数であり 
            // FileProvider がリソースを解放するためのフック関数だと書いているが、
            // これに Google Drive はファイル更新のトリガーを実装しているのかもしれない。
            // Rust 側に渡す生の FD を取得する際に detachFd 関数を呼び出しているが、
            // これは内部で releaseResources を呼び出しているため
            // まだ書き込んでない空のファイルが完了済みのものとしてマークされたのかも？
            // ただこれはソースコードを見て思いついた妄想なので要検証。
            // TODO: 時間がある時にリフレクションで呼び出して検証し、もしそうだったら issue を建てたい。

            const TARGET_URI_PREFIXES: &'static [&'static str] = &[
                "content://com.google.android.apps.docs", // Google drive
            ];

            Ok(TARGET_URI_PREFIXES.iter().any(|prefix| uri.uri.starts_with(prefix)))
        })
    }

    /// Renames a file or directory to a new name, and return new URI.  
    /// Even if the names conflict, the existing file will not be overwritten.  
    /// 
    /// Note that when files or folders (and their descendants) are renamed, their URIs will change, and any previously granted permissions will be lost.
    /// In other words, this function returns a new URI without any permissions.
    /// However, for files created in PublicStorage, the URI remains unchanged even after such operations, and all permissions are retained.
    /// In this, this function returns the same URI as original URI.
    ///
    /// # Args
    /// - ***uri*** :  
    /// URI of target entry.  
    /// 
    /// - ***new_name*** :  
    /// New name of target entry. 
    /// This include extension if use.  
    /// The behaviour in the same name already exists depends on the file provider.  
    /// In the case of e.g. [`PublicStorage`], the suffix (e.g. `(1)`) is added to this name.  
    /// In the case of files hosted by other applications, errors may occur.  
    /// But at least, the existing file will not be overwritten.  
    /// The system may sanitize these strings as needed, so those strings may not be used as it is.
    /// 
    /// # Support
    /// All Android version.
    pub fn rename(&self, uri: &FileUri, new_name: impl AsRef<str>) -> crate::Result<FileUri> {
        on_android!({
            impl_se!(struct Req<'a> { uri: &'a FileUri, new_name: &'a str });

            let new_name = new_name.as_ref();

            self.api
                .run_mobile_plugin::<FileUri>("rename", Req { uri, new_name })
                .map_err(Into::into)
        })
    }

    /// Remove the file.
    /// 
    /// # Args
    /// - ***uri*** :  
    /// Target file URI.  
    /// Must be **writable**, at least. But even if it is, 
    /// removing may not be possible in some cases. 
    /// For details, refer to the documentation of the function that provided the URI.  
    /// If not file, an error will occur.
    /// 
    /// # Support
    /// All Android version.
    pub fn remove_file(&self, uri: &FileUri) -> crate::Result<()> {
        on_android!({
            impl_se!(struct Req<'a> { uri: &'a FileUri });
            impl_de!(struct Res;);
    
            self.api
                .run_mobile_plugin::<Res>("deleteFile", Req { uri })
                .map(|_| ())
                .map_err(Into::into)
        })
    }

    /// Remove the **empty** directory.
    /// 
    /// # Args
    /// - ***uri*** :  
    /// Target directory URI.  
    /// Must be **writable**.  
    /// If not empty directory, an error will occur.
    /// 
    /// # Support
    /// All Android version.
    pub fn remove_dir(&self, uri: &FileUri) -> crate::Result<()> {
        on_android!({
            impl_se!(struct Req<'a> { uri: &'a FileUri });
            impl_de!(struct Res;);
        
            self.api
                .run_mobile_plugin::<Res>("deleteEmptyDir", Req { uri })
                .map(|_| ())
                .map_err(Into::into)
        })
    }

    /// Removes a directory and all its contents. Use carefully!
    /// 
    /// # Args
    /// - ***uri*** :  
    /// Target directory URI.  
    /// Must be **writable**.  
    /// If not directory, an error will occur.
    /// 
    /// # Support
    /// All Android version.
    pub fn remove_dir_all(&self, uri: &FileUri) -> crate::Result<()> {
        on_android!({
            impl_se!(struct Req<'a> { uri: &'a FileUri });
            impl_de!(struct Res;);
        
            self.api
                .run_mobile_plugin::<Res>("deleteDirAll", Req { uri })
                .map(|_| ())
                .map_err(Into::into)
        })
    }

    /// Build a URI of an **existing** file located at the relative path from the specified directory.   
    /// Error occurs, if the file does not exist.  
    /// 
    /// The permissions and validity period of the returned URI depend on the origin directory 
    /// (e.g., the top directory selected by [`FilePicker::pick_dir`]) 
    /// 
    /// # Note
    /// For [`AndroidFs::create_new_file`] and etc, the system may sanitize path strings as needed, so those strings may not be used as it is.
    /// However, this function does not perform any sanitization, so the same ***relative_path*** may still fail.
    /// 
    /// # Args
    /// - ***uri*** :  
    /// Base directory URI.  
    /// Must be **readable**.  
    /// 
    /// - ***relative_path*** :
    /// Relative path from base directory.
    /// 
    /// # Support
    /// All Android version.
    pub fn try_resolve_file_uri(
        &self, 
        dir: &FileUri, 
        relative_path: impl AsRef<std::path::Path>
    ) -> crate::Result<FileUri> {

        on_android!({
            #[allow(deprecated)]
            let uri = self.resolve_uri(dir, relative_path)?;         

            if !self.get_type(&uri)?.is_file() {
                return Err(crate::Error::with(format!("This is not a file: {uri:?}")))
            }
            Ok(uri)
        })
    }

    /// Build a URI of an **existing** directory located at the relative path from the specified directory.   
    /// Error occurs, if the directory does not exist.  
    /// 
    /// The permissions and validity period of the returned URI depend on the origin directory 
    /// (e.g., the top directory selected by [`FilePicker::pick_dir`]) 
    /// 
    /// # Note
    /// For [`AndroidFs::create_new_file`] and etc, the system may sanitize path strings as needed, so those strings may not be used as it is.
    /// However, this function does not perform any sanitization, so the same ***relative_path*** may still fail.
    /// 
    /// # Args
    /// - ***uri*** :  
    /// Base directory URI.  
    /// Must be **readable**.  
    /// 
    /// - ***relative_path*** :
    /// Relative path from base directory.
    /// 
    /// # Support
    /// All Android version.
    pub fn try_resolve_dir_uri(
        &self,
        dir: &FileUri, 
        relative_path: impl AsRef<std::path::Path>
    ) -> crate::Result<FileUri> {

        on_android!({
            #[allow(deprecated)]
            let uri = self.resolve_uri(dir, relative_path)?;
            
            if !self.get_type(&uri)?.is_dir() {
                return Err(crate::Error::with(format!("This is not a directory: {uri:?}")))
            }
            Ok(uri)
        })
    }

    /// Build a URI of an entry located at the relative path from the specified directory.   
    /// 
    /// This function does **not** create any entries; it only constructs the URI.
    /// 
    /// This function does not perform checks on the arguments or the returned URI.  
    /// Even if the dir argument refers to a file, no error occurs (and no panic either).
    /// Instead, it simply returns an invalid URI that will cause errors if used with other functions.  
    /// 
    /// If you need check, consider using [`AndroidFs::try_resolve_file_uri`] or [`AndroidFs::try_resolve_dir_uri`] instead. 
    /// Or use this with [`AndroidFs::get_mime_type`].
    /// 
    /// The permissions and validity period of the returned URI depend on the origin directory 
    /// (e.g., the top directory selected by [`FilePicker::pick_dir`]) 
    /// 
    /// # Note
    /// For [`PublicStorage::create_new_file`] and etc, the system may sanitize path strings as needed, so those strings may not be used as it is.
    /// However, this function does not perform any sanitization, so the same ***relative_path*** may still fail.
    /// 
    /// # Args
    /// - ***uri*** :  
    /// Base directory URI.  
    /// Must be **readable**.  
    /// 
    /// - ***relative_path*** :
    /// Relative path from base directory.
    /// 
    /// # Support
    /// All Android version.
    #[deprecated = "Use AndroidFs::try_resolve_file_uri or AndroidFs::try_resolve_dir_uri instead"]
    pub fn resolve_uri(
        &self, 
        dir: &FileUri, 
        relative_path: impl AsRef<std::path::Path>
    ) -> crate::Result<FileUri> {

        on_android!({
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
        })
    }

    /// See [`AndroidFs::get_thumbnail_to`] for descriptions.  
    /// 
    /// If thumbnail does not wrote to dest, return false.
    pub fn get_thumbnail_to(
        &self, 
        src: &FileUri,
        dest: &FileUri,
        preferred_size: Size,
        format: ImageFormat,
    ) -> crate::Result<bool> {

        on_android!({
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
        
            self.api
                .run_mobile_plugin::<Res>("getThumbnail", Req { src, dest, format, quality, width, height })
                .map(|v| v.value)
                .map_err(Into::into)
        })
    }

    /// Query the provider to get a file thumbnail.  
    /// If thumbnail does not exist it, return None.
    /// 
    /// Note this does not cache. Please do it in your part if need.  
    /// 
    /// # Args
    /// - ***uri*** :  
    /// Targe file uri.  
    /// Thumbnail availablty depends on the file provider.  
    /// In general, images and videos are available.  
    /// For files in [`PrivateStorage`], 
    /// the file type must match the filename extension.  
    /// 
    /// - ***preferred_size*** :  
    /// Optimal thumbnail size desired.  
    /// This may return a thumbnail of a different size, 
    /// but never more than double the requested size. 
    /// In any case, the aspect ratio is maintained.
    /// 
    /// - ***format*** :  
    /// Thumbnail image format.  
    /// [`ImageFormat::Jpeg`] is recommended. 
    /// If you need transparency, use others.
    /// 
    /// # Support
    /// All Android version.
    pub fn get_thumbnail(
        &self,
        uri: &FileUri,
        preferred_size: Size,
        format: ImageFormat,
    ) -> crate::Result<Option<Vec<u8>>> {

        on_android!({
            let (tmp_file, tmp_file_path) = self.private_storage().create_new_tmp_file()?;
            std::mem::drop(tmp_file);

            let result = self.get_thumbnail_to(uri, &(&tmp_file_path).into(), preferred_size, format)
                .and_then(|ok| {
                    if ok {
                        std::fs::read(&tmp_file_path)
                            .map(Some)
                            .map_err(Into::into)
                    }
                    else {
                        Ok(None)
                    }
                });

            let _ = std::fs::remove_file(&tmp_file_path);

            result
        })
    }

    /// Creates a new empty file in the specified location and returns a URI.   
    /// 
    /// The permissions and validity period of the returned URIs depend on the origin directory 
    /// (e.g., the top directory selected by [`FilePicker::pick_dir`]) 
    /// 
    /// # Args  
    /// - ***dir*** :  
    /// The URI of the base directory.  
    /// Must be **read-write**.
    ///  
    /// - ***relative_path*** :  
    /// The file path relative to the base directory.  
    /// Any missing subdirectories in the specified path will be created automatically.  
    /// If a file with the same name already exists, 
    /// the system append a sequential number to ensure uniqueness.  
    /// If no extension is present, 
    /// the system may infer one from ***mime_type*** and may append it to the file name. 
    /// But this append-extension operation depends on the model and version.  
    /// The system may sanitize these strings as needed, so those strings may not be used as it is.
    ///  
    /// - ***mime_type*** :  
    /// The MIME type of the file to be created.  
    /// If this is None, MIME type is inferred from the extension of ***relative_path***
    /// and if that fails, `application/octet-stream` is used.  
    ///  
    /// # Support
    /// All Android version.
    pub fn create_new_file(
        &self,
        dir: &FileUri, 
        relative_path: impl AsRef<std::path::Path>, 
        mime_type: Option<&str>
    ) -> crate::Result<FileUri> {

        on_android!({
            impl_se!(struct Req<'a> { dir: &'a FileUri, mime_type: Option<&'a str>, relative_path: &'a str });
        
            let relative_path = validate_relative_path(relative_path.as_ref())?;
            let relative_path = relative_path.to_string_lossy();
                
            self.api
                .run_mobile_plugin::<FileUri>("createFile", Req { dir, mime_type, relative_path: relative_path.as_ref() })
                .map_err(Into::into)
        })
    }

    /// Recursively create a directory and all of its parent components if they are missing,
    /// then return the URI.  
    /// If it already exists, do nothing and just return the direcotry uri.
    /// 
    /// [`AndroidFs::create_new_file`] does this automatically, so there is no need to use it together.
    /// 
    /// # Args  
    /// - ***dir*** :  
    /// The URI of the base directory.  
    /// Must be **read-write**.
    ///  
    /// - ***relative_path*** :  
    /// The directory path relative to the base directory.    
    /// The system may sanitize these strings as needed, so those strings may not be used as it is.
    ///  
    /// # Support
    /// All Android version.
    pub fn create_dir_all(
        &self,
        dir: &FileUri, 
        relative_path: impl AsRef<std::path::Path>, 
    ) -> Result<FileUri> {

        on_android!({
            impl_se!(struct Req<'a> { dir: &'a FileUri,relative_path: &'a str });
        
            let relative_path = validate_relative_path(relative_path.as_ref())?;
            let relative_path = relative_path.to_string_lossy();
                
            self.api
                .run_mobile_plugin::<FileUri>("createDirAll", Req { dir, relative_path: relative_path.as_ref() })
                .map_err(Into::into)
        })
    }

    /// Returns the child files and directories of the specified directory.  
    /// The order of the entries is not guaranteed.  
    /// 
    /// The permissions and validity period of the returned URIs depend on the origin directory 
    /// (e.g., the top directory selected by [`FilePicker::pick_dir`])  
    /// 
    /// This retrieves all metadata including `uri`, `name`, `last_modified`, `len`, and `mime_type`. 
    /// If only specific information is needed, 
    /// using [`AndroidFs::read_dir_with_options`] will improve performance.
    /// 
    /// # Note
    /// The returned type is an iterator, but the file system call is not executed lazily.  
    /// Instead, all data is retrieved at once.  
    /// For directories containing thousands or even tens of thousands of entries,  
    /// this function may take several seconds to complete.  
    /// The returned iterator itself is low-cost, as it only performs lightweight data formatting.
    /// 
    /// # Args
    /// - ***uri*** :  
    /// Target directory URI.  
    /// Must be **readable**.
    /// 
    /// # Support
    /// All Android version.
    pub fn read_dir(&self, uri: &FileUri) -> crate::Result<impl Iterator<Item = Entry>> {
        let entries = self.read_dir_with_options(uri, EntryOptions::ALL)?
            .map(Entry::try_from)
            .filter_map(Result::ok);
        
        Ok(entries)
    }

    /// Returns the child files and directories of the specified directory.  
    /// The order of the entries is not guaranteed.  
    /// 
    /// The permissions and validity period of the returned URIs depend on the origin directory 
    /// (e.g., the top directory selected by [`FilePicker::pick_dir`])  
    /// 
    /// # Note
    /// The returned type is an iterator, but the file system call is not executed lazily.  
    /// Instead, all data is retrieved at once.  
    /// For directories containing thousands or even tens of thousands of entries,  
    /// this function may take several seconds to complete.  
    /// The returned iterator itself is low-cost, as it only performs lightweight data formatting.
    /// 
    /// # Args
    /// - ***uri*** :  
    /// Target directory URI.  
    /// Must be **readable**.
    /// 
    /// # Support
    /// All Android version.
    pub fn read_dir_with_options(
        &self, 
        uri: &FileUri, 
        options: EntryOptions
    ) -> Result<impl Iterator<Item = OptionalEntry>> {
        
        on_android!(std::iter::Empty::<_>, {
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
    
            self.api
                .run_mobile_plugin::<Res>("readDir", Req { uri, options })
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
                .map_err(Into::into)
        })
    }

    /// Take persistent permission to access the file, directory and its descendants.  
    /// This is a prolongation of an already acquired permission, not the acquisition of a new one.  
    /// 
    /// This works by just calling, without displaying any confirmation to the user.
    /// 
    /// Note that [there is a limit to the total number of URI that can be made persistent by this function.](https://stackoverflow.com/questions/71099575/should-i-release-persistableuripermission-when-a-new-storage-location-is-chosen/71100621#71100621)  
    /// Therefore, it is recommended to relinquish the unnecessary persisted URI by [`AndroidFs::release_persisted_uri_permission`] or [`AndroidFs::release_all_persisted_uri_permissions`].  
    /// Persisted permissions may be relinquished by other apps, user, or by moving/removing entries.
    /// So check by [`AndroidFs::check_persisted_uri_permission`].  
    /// And you can retrieve the list of persisted uris using [`AndroidFs::get_all_persisted_uri_permissions`].
    /// 
    /// # Args
    /// - **uri** :  
    /// URI of the target file or directory.   
    /// This must be a URI taken from following :  
    ///     - [`FilePicker::pick_files`]  
    ///     - [`FilePicker::pick_file`]  
    ///     - [`FilePicker::pick_visual_medias`]  
    ///     - [`FilePicker::pick_visual_media`]  
    ///     - [`FilePicker::pick_dir`]  
    ///     - [`FilePicker::save_file`]  
    ///     - [`AndroidFs::try_resolve_file_uri`], [`AndroidFs::try_resolve_dir_uri`], [`AndroidFs::resolve_uri`], [`AndroidFs::read_dir`], [`AndroidFs::create_new_file`], [`AndroidFs::create_dir_all`] :  
    ///     If use URI from thoese fucntions, the permissions of the origin directory URI is persisted, not a entry iteself by this function. 
    ///     Because the permissions and validity period of the descendant entry URIs depend on the origin directory.   
    /// 
    /// # Support
    /// All Android version. 
    pub fn take_persistable_uri_permission(&self, uri: &FileUri) -> crate::Result<()> {
        on_android!({
            impl_se!(struct Req<'a> { uri: &'a FileUri });
            impl_de!(struct Res;);

            self.api
                .run_mobile_plugin::<Res>("takePersistableUriPermission", Req { uri })
                .map(|_| ())
                .map_err(Into::into)
        })
    }

    /// Check a persisted URI permission grant by [`AndroidFs::take_persistable_uri_permission`].  
    /// Returns false if there are only non-persistent permissions or no permissions.
    /// 
    /// # Args
    /// - **uri** :  
    /// URI of the target file or directory.  
    /// This must be a URI taken from following :  
    ///     - [`FilePicker::pick_files`]  
    ///     - [`FilePicker::pick_file`]  
    ///     - [`FilePicker::pick_visual_medias`]  
    ///     - [`FilePicker::pick_visual_media`]  
    ///     - [`FilePicker::pick_dir`]  
    ///     - [`FilePicker::save_file`]  
    ///     - [`AndroidFs::try_resolve_file_uri`], [`AndroidFs::try_resolve_dir_uri`], [`AndroidFs::resolve_uri`], [`AndroidFs::read_dir`], [`AndroidFs::create_new_file`], [`AndroidFs::create_dir_all`] :  
    ///     If use URI from thoese fucntions, the permissions of the origin directory URI is checked, not a entry iteself by this function. 
    ///     Because the permissions and validity period of the descendant entry URIs depend on the origin directory.   
    /// 
    /// - **mode** :  
    /// The mode of permission you want to check.  
    /// 
    /// # Support
    /// All Android version.
    pub fn check_persisted_uri_permission(&self, uri: &FileUri, mode: PersistableAccessMode) -> crate::Result<bool> {
        on_android!({
            impl_se!(struct Req<'a> { uri: &'a FileUri, mode: PersistableAccessMode });
            impl_de!(struct Res { value: bool });

            self.api
                .run_mobile_plugin::<Res>("checkPersistedUriPermission", Req { uri, mode })
                .map(|v| v.value)
                .map_err(Into::into)
        })
    }

    /// Return list of all persisted URIs that have been persisted by [`AndroidFs::take_persistable_uri_permission`] and currently valid.   
    /// 
    /// # Support
    /// All Android version.
    pub fn get_all_persisted_uri_permissions(&self) -> crate::Result<impl Iterator<Item = PersistedUriPermission>> {
        on_android!(std::iter::Empty::<_>, {
            impl_de!(struct Obj { uri: FileUri, r: bool, w: bool, d: bool });
            impl_de!(struct Res { items: Vec<Obj> });
    
            self.api
                .run_mobile_plugin::<Res>("getAllPersistedUriPermissions", "")
                .map(|v| v.items.into_iter())
                .map(|v| v.map(|v| {
                    let (uri, can_read, can_write) = (v.uri, v.r, v.w);
                    match v.d {
                        true => PersistedUriPermission::Dir { uri, can_read, can_write },
                        false => PersistedUriPermission::File { uri, can_read, can_write }
                    }
                }))
                .map_err(Into::into)
        })
    }

    /// Relinquish a persisted URI permission grant by [`AndroidFs::take_persistable_uri_permission`].   
    /// 
    /// # Args
    /// - ***uri*** :  
    /// URI of the target file or directory.  
    ///
    /// # Support
    /// All Android version.
    pub fn release_persisted_uri_permission(&self, uri: &FileUri) -> crate::Result<()> {
        on_android!({
            impl_se!(struct Req<'a> { uri: &'a FileUri });
            impl_de!(struct Res;);

            self.api
                .run_mobile_plugin::<Res>("releasePersistedUriPermission", Req { uri })
                .map(|_| ())
                .map_err(Into::into)
        })
    }

    /// Relinquish a all persisted uri permission grants by [`AndroidFs::take_persistable_uri_permission`].  
    /// 
    /// # Support
    /// All Android version.
    pub fn release_all_persisted_uri_permissions(&self) -> crate::Result<()> {
        on_android!({
            impl_de!(struct Res);

            self.api
                .run_mobile_plugin::<Res>("releaseAllPersistedUriPermissions", "")
                .map(|_| ())
                .map_err(Into::into)
        })
    }

    /// Verify whether this plugin is available.  
    /// 
    /// On Android, this returns true.  
    /// On other platforms, this returns false.  
    pub fn is_available(&self) -> bool {
        cfg!(target_os = "android")
    }

    /// Get the api level of this Android device.
    /// 
    /// The correspondence table between API levels and Android versions can be found following.  
    /// <https://developer.android.com/guide/topics/manifest/uses-sdk-element#api-level-table>
    /// 
    /// If you want the constant value of the API level from an Android version, there is the [`api_level`] module.
    /// 
    /// # Table
    /// | Android version  | API Level |
    /// |------------------|-----------|
    /// | 16.0             | 36        |
    /// | 15.0             | 35        |
    /// | 14.0             | 34        |
    /// | 13.0             | 33        |
    /// | 12L              | 32        |
    /// | 12.0             | 31        |
    /// | 11.0             | 30        |
    /// | 10.0             | 29        |
    /// | 9.0              | 28        |
    /// | 8.1              | 27        |
    /// | 8.0              | 26        |
    /// | 7.1 - 7.1.2      | 25        |
    /// | 7.0              | 24        |
    /// 
    /// Tauri does not support Android versions below 7.
    pub fn api_level(&self) -> Result<i32> {
        Ok(self.consts()?.build_version_sdk_int)
    }
}


#[allow(unused)]
impl<R: tauri::Runtime> AndroidFs<R> {

    pub(crate) fn check_media_store_volume_name_available(
        &self,
        media_store_volume_name: impl AsRef<str>,
    ) -> Result<bool> {

        on_android!({
            impl_se!(struct Req<'a> { media_store_volume_name: &'a str });
            impl_de!(struct Res { value: bool });
            
            let media_store_volume_name = media_store_volume_name.as_ref();
            
            self.api
                .run_mobile_plugin::<Res>("checkMediaStoreVolumeNameAvailable", Req { media_store_volume_name })
                .map(|v| v.value)
                .map_err(Into::into)
        })
    }

    pub(crate) fn check_storage_volume_available_by_path(
        &self,
        path: impl AsRef<std::path::Path>,
    ) -> Result<bool> {

        on_android!({
            impl_se!(struct Req<'a> { path: &'a std::path::Path });
            impl_de!(struct Res { value: bool });

            let path = path.as_ref();

            self.api
                .run_mobile_plugin::<Res>("checkStorageVolumeAvailableByPath", Req { path })
                .map(|v| v.value)
                .map_err(Into::into)
        })
    }

    pub(crate) fn get_available_storage_volumes(&self) -> Result<Vec<StorageVolume>> {
        on_android!({
            impl_de!(struct Res { volumes: Vec<StorageVolume> });

            let mut volumes = self.api
                .run_mobile_plugin::<Res>("getAvailableStorageVolumes", "")
                .map(|v| v.volumes)?;

            // primary volume を先頭にする。他はそのままの順序
            volumes.sort_by(|a, b| b.is_primary.cmp(&a.is_primary));

            Ok(volumes)
        })
    }

    pub(crate) fn get_primary_storage_volume_if_available(&self) -> Result<Option<StorageVolume>> {
        on_android!({
            impl_de!(struct Res { volume: Option<StorageVolume> });

            self.api
                .run_mobile_plugin::<Res>("getPrimaryStorageVolumeIfAvailable", "")
                .map(|v| v.volume)
                .map_err(Into::into)
        })
    }

    pub(crate) fn consts(&self) -> Result<&Consts> {
        on_android!({
            static CONSTS: std::sync::OnceLock<Consts> = std::sync::OnceLock::new();

            if CONSTS.get().is_none() {
                let _ = CONSTS.set(
                    self.api.run_mobile_plugin::<Consts>("getConsts", "")?
                );
            }
            let consts = CONSTS.get().expect("Should call 'set' before 'get'");

            Ok(consts)
        })
    }
}

/// アプリ起動中に変更されることのない値
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(unused)]
pub(crate) struct Consts {
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

#[allow(unused)]
impl Consts {

    pub(crate) fn public_dir_name(&self, dir: impl Into<PublicDir>) -> Result<&str> {
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