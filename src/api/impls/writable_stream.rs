use std::sync::Arc;
use sync_async::sync_async;
use crate::*;
use super::*;


#[sync_async(
    use(if_sync) SyncWritableStreamImpls as WritableStreamImpls;
    use(if_async) AsyncWritableStreamImpls as WritableStreamImpls;
)]
impl<'a, R: tauri::Runtime> Impls<'a, R> {

    #[maybe_async]
    pub fn create_writable_stream_auto(
        &self,
        output_uri: &FileUri,
    ) -> Result<WritableStreamImpls<R>> {

        let need_write_via_kotlin = self.need_write_file_via_kotlin(output_uri).await?;
        self.create_writable_stream(output_uri, need_write_via_kotlin).await
    }

    #[maybe_async]
    pub fn create_writable_stream_via_kotlin(
        &self,
        output_uri: &FileUri,
    ) -> Result<WritableStreamImpls<R>> {

        let need_write_via_kotlin = true;
        self.create_writable_stream(output_uri, need_write_via_kotlin).await
    }

    #[maybe_async]
    pub fn create_writable_stream(
        &self,
        output_uri: &FileUri,
        need_write_via_kotlin: bool
    ) -> Result<WritableStreamImpls<R>> {

        let (output, output_attr) = match need_write_via_kotlin {
            true => {
                let (tmp_file, tmp_file_path) = self.create_new_tmp_file().await?;
                let output = tmp_file;
                let output_attr = OutputAttr::TempBuffer { 
                    output_path: tmp_file_path, 
                    actual_target_uri: output_uri.clone(),
                };
                (output, output_attr)
            },
            false => {
                let output = self.open_file_writable(&output_uri).await?;
                let output_attr = OutputAttr::ActualTarget;
                (output, output_attr)
            }
        };
        
        let inner = WritableStreamInner {
            handle: self.handle.clone(),
            output: Some(std::sync::Arc::new(output)),
            output_attr: Some(std::sync::Arc::new(output_attr)),
        };

        Ok(WritableStreamImpls { inner })
    }
}


#[sync_async]
pub struct WritableStreamImpls<R: tauri::Runtime> {
    inner: WritableStreamInner<R>
}

struct WritableStreamInner<R: tauri::Runtime> {
    handle: tauri::plugin::PluginHandle<R>,
    output: Option<Arc<std::fs::File>>,
    output_attr: Option<Arc<OutputAttr>>,
}

#[derive(Clone)]
enum OutputAttr {
    ActualTarget,
    TempBuffer {
        output_path: std::path::PathBuf,
        actual_target_uri: FileUri,
    },
}


#[sync_async(
    use(if_async) async_utils::run_blocking;
    use(if_sync) sync_utils::run_blocking;
    use(if_async) AsyncImpls as Impls;
    use(if_sync) SyncImpls as Impls;
)]
impl<R: tauri::Runtime> WritableStreamImpls<R> {

    #[always_sync]
    pub fn into_sync(self) -> SyncWritableStreamImpls<R> {
        SyncWritableStreamImpls { inner: self.inner }
    }

    #[always_sync]
    pub fn into_async(self) -> AsyncWritableStreamImpls<R> {
        AsyncWritableStreamImpls { inner: self.inner }
    }

    #[maybe_async]
    pub fn sync_all(&self) -> std::io::Result<()> {
        let Some(output) = self.inner.output.as_ref() else {
            return Ok(())
        };
        let Some(output_attr) = self.inner.output_attr.as_ref() else {
            return Ok(())
        };
        
        if let OutputAttr::ActualTarget = output_attr.as_ref() {
            let output = Arc::clone(output);
            run_blocking(move || output.sync_all().map_err(Into::into)).await?;
        }
        Ok(())
    }

    #[maybe_async]
    pub fn sync_data(&self) -> std::io::Result<()> {
        let Some(output) = self.inner.output.as_ref() else {
            return Ok(())
        };
        let Some(output_attr) = self.inner.output_attr.as_ref() else {
            return Ok(())
        };
        
        if let OutputAttr::ActualTarget = output_attr.as_ref() {
            let output = Arc::clone(output);
            run_blocking(move || output.sync_data().map_err(Into::into)).await?;
        }
        Ok(())
    }

    #[maybe_async]
    pub fn reflect(
        mut self,
    ) -> Result<()> {

        let Some(output) = (&mut self.inner.output).take() else {
            return Ok(())
        };
        let Some(output_attr) = (&mut self.inner.output_attr).take() else {
            return Ok(())
        };

        if let OutputAttr::TempBuffer { 
            output_path, 
            actual_target_uri, 
        } = Arc::try_unwrap(output_attr).unwrap_or_else(|arc| (*arc).clone()) {

            // コピーする前にファイルデータを反映させてファイルを閉じる
            let result1 = run_blocking(move || {
                let result = output.sync_data().map_err(Into::into);
                std::mem::drop(output);
                result
            }).await;

            let impls = Impls { handle: &self.inner.handle };

            let result2 = impls.copy_file_via_kotlin(
                &(output_path.clone().into()), 
                &actual_target_uri, 
                None
            ).await;

            let result3 = run_blocking(move || 
                std::fs::remove_file(output_path).map_err(Into::into)
            ).await;

            result1?;
            result2?;
            result3?;
        }

        Ok(())
    }

    #[maybe_async]
    pub fn dispose_without_reflect(
        mut self
    ) -> Result<()> {

        let Some(output) = (&mut self.inner.output).take() else {
            return Ok(())
        };
        let Some(output_attr) = (&mut self.inner.output_attr).take() else {
            return Ok(())
        };

        std::mem::drop(output);

        if let OutputAttr::TempBuffer { output_path, .. } = output_attr.as_ref() {
            let tmp_file_path = output_path.clone();
            run_blocking(move || std::fs::remove_file(tmp_file_path).map_err(Into::into)).await?;
        }

        Ok(())
    }
}

impl<R: tauri::Runtime> Drop for WritableStreamInner<R> {

    fn drop(&mut self) {
        let Some(output) = (&mut self.output).take() else {
            return
        };
        let Some(output_attr) = (&mut self.output_attr).take() else {
            return
        };

        if let OutputAttr::TempBuffer { 
            output_path,
            actual_target_uri,
        } = Arc::try_unwrap(output_attr).unwrap_or_else(|arc| (*arc).clone()) {

            let handle = self.handle.clone();
                
            tauri::async_runtime::spawn_blocking(move || {
                // コピーする前にファイルデータを反映させてファイルを閉じる
                output.sync_data().ok();
                std::mem::drop(output);
                    
                let impls = SyncImpls { handle: &handle };
                impls.copy_file_via_kotlin(
                    &(output_path.clone().into()), 
                    &actual_target_uri, 
                    None
                ).ok();

                std::fs::remove_file(output_path).ok();
            });
        }
    }
} 

macro_rules! impl_write {
    ($target:ident) => {

        impl<R: tauri::Runtime> std::io::Write for $target<R> {

            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                if let Some(output) = self.inner.output.as_mut() {
                    return output.write(buf)
                }
                Err(std::io::Error::new(std::io::ErrorKind::Other, "missing writer"))
            }

            fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
                if let Some(output) = self.inner.output.as_mut() {
                    return output.write_all(buf)
                }
                Err(std::io::Error::new(std::io::ErrorKind::Other, "missing writer"))
            }

            fn write_fmt(&mut self, fmt: std::fmt::Arguments<'_>) -> std::io::Result<()> {
                if let Some(output) = self.inner.output.as_mut() {
                    return output.write_fmt(fmt)
                }
                Err(std::io::Error::new(std::io::ErrorKind::Other, "Missing writer"))
            }

            fn write_vectored(&mut self, bufs: &[std::io::IoSlice<'_>]) -> std::io::Result<usize> {
                if let Some(output) = self.inner.output.as_mut() {
                    return output.write_vectored(bufs)
                }
                Err(std::io::Error::new(std::io::ErrorKind::Other, "Missing writer"))
            }
    
            fn flush(&mut self) -> std::io::Result<()> {
                if let Some(output) = self.inner.output.as_mut() {
                    return output.flush()
                }
                Err(std::io::Error::new(std::io::ErrorKind::Other, "Missing writer"))
            }
        }
    };
}

impl_write!(AsyncWritableStreamImpls);
impl_write!(SyncWritableStreamImpls);