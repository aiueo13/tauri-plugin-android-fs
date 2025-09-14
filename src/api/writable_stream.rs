use crate::*;


/// A stream for writing to a file on Android.
///
/// Implements [`std::io::Write`], so it can be used for writing.  
/// As with [`std::fs::File`], wrap it with [`std::io::BufWriter`] if buffering is needed.  
///
/// After writing, call [`WritableStream::reflect`] to apply changes.  
///
/// # Inner
/// This is a wrapper around [`std::fs::File`].  
/// In most cases, it points to the actual target file, but it may also refer to a temporary file.  
/// For temporary files, calling [`WritableStream::reflect`] applies the changes to the actual target. 
pub struct WritableStream<R: tauri::Runtime> {
    app: tauri::AppHandle<R>,
    writer: Option<std::fs::File>,
    writer_attr: Option<WriterAttr>,
}

enum WriterAttr {
    ActualTarget,
    TempBuffer {
        writer_path: std::path::PathBuf,
        actual_target_file_uri: FileUri,
    },
}

impl<R: tauri::Runtime> WritableStream<R> {

    #[allow(unused)]
    pub(crate) fn new(
        app: tauri::AppHandle<R>,
        file_uri: FileUri,
        need_write_via_kotlin: bool
    ) -> Result<Self> {

        let api = app.android_fs();
        let (writer, writer_attr) = match need_write_via_kotlin {
            true => {
                let (tmp_file, tmp_file_path) = api.private_storage().create_new_tmp_file()?;
                let attr = WriterAttr::TempBuffer { 
                    writer_path: tmp_file_path, 
                    actual_target_file_uri: file_uri
                };
                (tmp_file, attr)
            },
            false => {
                 let file = api.open_file_writable(&file_uri)?;
                 (file, WriterAttr::ActualTarget)
            }
        };
      
        Ok(Self {
            app,
            writer: Some(writer),
            writer_attr: Some(writer_attr),
        })
    }
}

impl<R: tauri::Runtime> WritableStream<R> {

    /// [`WritableStream`] is a wrapper around [`std::fs::File`].
    /// In most cases, it points to the actual target file, but it may also refer to a temporary file.
    ///
    /// For actual target files, this function does nothing.
    ///
    /// For temporary files, calling this function applies the changes to the actual target, and remove the temporary file.
    /// This may take as long as the write operation or even longer.
    /// Note that if the file is located on cloud storage or similar, the function returns
    /// without waiting for the uploading to complete.
    /// 
    /// If not called explicitly, the same process is performed asynchronously on drop, 
    /// and no error is returned. 
    pub fn reflect(mut self) -> Result<()> {
        let Some(writer) = self.writer.take() else {
            return Ok(())
        };
        let Some(writer_attr) = self.writer_attr.take() else {
            return Ok(())
        };

        if let WriterAttr::TempBuffer { 
            writer_path, 
            actual_target_file_uri, 
        } = writer_attr {

            // 反映されるまで待機する
            let result1 = writer.sync_data();
            // copy を行う前にファイルを閉じる
            std::mem::drop(writer);

            let result2 = self.app
                .android_fs()
                .copy_via_kotlin(&(writer_path.clone().into()), &actual_target_file_uri, None);

            let _ = std::fs::remove_file(&writer_path);

            result1?;
            result2?;
        }

        Ok(())
    }

    /// [`WritableStream`] is a wrapper around [`std::fs::File`].  
    /// In most cases, it points to the actual target file, but it may also refer to a temporary file.  
    ///
    /// For actual target files, calls [`std::fs::File::sync_all`].  
    /// For temporary files, this function does nothing.  
    pub fn sync_all(&mut self) -> std::io::Result<()> {
        let Some(writer) = self.writer.as_mut() else {
            return Ok(())
        };
        let Some(writer_attr) = self.writer_attr.as_ref() else {
            return Ok(())
        };
        
        if let WriterAttr::ActualTarget = writer_attr {
            writer.sync_all()?;
        }
        Ok(())
    }

    /// [`WritableStream`] is a wrapper around [`std::fs::File`].  
    /// In most cases, it points to the actual target file, but it may also refer to a temporary file.  
    ///
    /// For actual target files, calls [`std::fs::File::sync_data`].  
    /// For temporary files, this function does nothing.  
    pub fn sync_data(&mut self) -> std::io::Result<()> {
        let Some(writer) = self.writer.as_mut() else {
            return Ok(())
        };
        let Some(writer_attr) = self.writer_attr.as_ref() else {
            return Ok(())
        };
        
        if let WriterAttr::ActualTarget = writer_attr {
            writer.sync_data()?;
        }
        Ok(())
    }
}

impl<R: tauri::Runtime> std::io::Write for WritableStream<R> {

    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self.writer.as_mut() {
            Some(w) => w.write(buf),
            None => Err(std::io::Error::new(std::io::ErrorKind::Other, "writer missing")),
        }
    }

    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        match self.writer.as_mut() {
            Some(w) => w.write_all(buf),
            None => Err(std::io::Error::new(std::io::ErrorKind::Other, "writer missing")),
        }
    }

    fn write_vectored(&mut self, bufs: &[std::io::IoSlice<'_>]) -> std::io::Result<usize> {
        match self.writer.as_mut() {
            Some(w) => w.write_vectored(bufs),
            None => Err(std::io::Error::new(std::io::ErrorKind::Other, "writer missing")),
        }
    }

    fn write_fmt(&mut self, fmt: std::fmt::Arguments<'_>) -> std::io::Result<()> {
        match self.writer.as_mut() {
            Some(w) => w.write_fmt(fmt),
            None => Err(std::io::Error::new(std::io::ErrorKind::Other, "writer missing")),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self.writer.as_mut() {
            Some(w) => w.flush(),
            None => Err(std::io::Error::new(std::io::ErrorKind::Other, "writer missing")),
        }
    }
}

impl<R: tauri::Runtime> std::ops::Drop for WritableStream<R> {

    fn drop(&mut self) {
        // reflect が行われた場合、以下で return される
        let Some(writer) = self.writer.take() else {
            return
        };
        let Some(writer_attr) = self.writer_attr.take() else {
            return
        };

        // reflect が行われなかった場合、保険として reflect と同じ処理を行う。
        // ただし drop 内ではエラーの伝搬も panic も行えない。
        // よって std::io::BufWriter の drop 実装と同じようにエラーは握りつぶす。

        if let WriterAttr::TempBuffer { 
            writer_path, 
            actual_target_file_uri, 
        } = writer_attr {

            let app = self.app.clone();
            let src = writer;
            let (src_uri, src_path) = (writer_path.clone().into(), writer_path);
            let dest_uri = actual_target_file_uri.clone();

            // 時間がかかるので別スレッドに委託する
            tauri::async_runtime::spawn_blocking(move || {
                // 反映されるまで待機する
                let _ = src.sync_data();
                // copy を行う前にファイルを閉じる
                std::mem::drop(src);

                let _ = app.android_fs().copy_via_kotlin(&src_uri, &dest_uri, None);
                let _ = std::fs::remove_file(src_path);
            });
        }
    }
}