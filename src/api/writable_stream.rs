use sync_async::sync_async;
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
#[sync_async(
    use(if_sync) super::impls::SyncWritableStreamImpls as WritableStreamImpls;
    use(if_async) super::impls::AsyncWritableStreamImpls as WritableStreamImpls;
    use(if_sync) super::api_sync::WritableStream;
    use(if_async) super::api_async::WritableStream;
)]
pub struct WritableStream<R: tauri::Runtime> {
    #[cfg(target_os = "android")]
    pub(crate) impls: WritableStreamImpls<R>,

    #[cfg(not(target_os = "android"))]
    #[allow(unused)]
    pub(crate) impls: std::marker::PhantomData<fn() -> R>
}

#[sync_async(
    use(if_async) super::api_async::{AndroidFs, FileOpener, FilePicker, PrivateStorage, PublicStorage};
    use(if_sync) super::api_sync::{AndroidFs, FileOpener, FilePicker, PrivateStorage, PublicStorage};
)]
impl<R: tauri::Runtime> WritableStream<R> {

    /// Converts to a WritableStream for synchronous processing.
    #[always_sync]
    pub fn into_sync(self) -> SyncWritableStream<R> {
        #[cfg(not(target_os = "android"))] {
            // WritableStream を取得する関数は Android 以外だとエラーになる。
            // そのためこれが呼び出されることはない
            panic!("expected on Android")
        }
        #[cfg(target_os = "android")] {
            SyncWritableStream { impls: self.impls.into_sync() }
        }
    }

    /// Converts to a WritableStream for asynchronous processing.
    #[always_sync]
    pub fn into_async(self) -> AsyncWritableStream<R> {
        #[cfg(not(target_os = "android"))] {
            // WritableStream を取得する関数は Android 以外だとエラーになる。
            // そのためこれが呼び出されることはない
            panic!("expected on Android")
        }
        #[cfg(target_os = "android")] {
            AsyncWritableStream { impls: self.impls.into_async() }
        }
    }

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
    #[maybe_async]
    pub fn reflect(self) -> Result<()> {
        #[cfg(not(target_os = "android"))] {
            Err(Error::NOT_ANDROID)
        }
        #[cfg(target_os = "android")] {
            self.impls.reflect().await
        }
    }

    /// [`WritableStream`] is a wrapper around [`std::fs::File`].  
    /// In most cases, it points to the actual target file, but it may also refer to a temporary file.  
    ///
    /// For actual target files, calls [`std::fs::File::sync_all`].  
    /// For temporary files, this function does nothing.  
    #[maybe_async]
    pub fn sync_all(&self) -> std::io::Result<()> {
        #[cfg(not(target_os = "android"))] {
            Err(std::io::Error::new(std::io::ErrorKind::Other, Error::NOT_ANDROID))
        }
        #[cfg(target_os = "android")] {
            self.impls.sync_all().await
        }
    }

    /// [`WritableStream`] is a wrapper around [`std::fs::File`].  
    /// In most cases, it points to the actual target file, but it may also refer to a temporary file.  
    ///
    /// For actual target files, calls [`std::fs::File::sync_data`].  
    /// For temporary files, this function does nothing.  
    #[maybe_async]
    pub fn sync_data(&self) -> std::io::Result<()> {
        #[cfg(not(target_os = "android"))] {
            Err(std::io::Error::new(std::io::ErrorKind::Other, Error::NOT_ANDROID))
        }
        #[cfg(target_os = "android")] {
            self.impls.sync_data().await
        }
    }
}

macro_rules! impl_write {
    ($target:ident) => {

        impl<R: tauri::Runtime> std::io::Write for $target<R> {

            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                #[cfg(not(target_os = "android"))] {
                    Err(std::io::Error::new(std::io::ErrorKind::Other, Error::NOT_ANDROID))
                }
                #[cfg(target_os = "android")] {
                    self.impls.write(buf)
                }
            }

            fn flush(&mut self) -> std::io::Result<()> {
                #[cfg(not(target_os = "android"))] {
                    Err(std::io::Error::new(std::io::ErrorKind::Other, Error::NOT_ANDROID))
                }
                #[cfg(target_os = "android")] {
                    self.impls.flush()
                }
            }

            fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
                #[cfg(not(target_os = "android"))] {
                    Err(std::io::Error::new(std::io::ErrorKind::Other, Error::NOT_ANDROID))
                }
                #[cfg(target_os = "android")] {
                    self.impls.write_all(buf)
                }
            }

            fn write_vectored(&mut self, bufs: &[std::io::IoSlice<'_>]) -> std::io::Result<usize> {
                #[cfg(not(target_os = "android"))] {
                    Err(std::io::Error::new(std::io::ErrorKind::Other, Error::NOT_ANDROID))
                }
                #[cfg(target_os = "android")] {
                    self.impls.write_vectored(bufs)
                }
            }

            fn write_fmt(&mut self, fmt: std::fmt::Arguments<'_>) -> std::io::Result<()> {
                #[cfg(not(target_os = "android"))] {
                    Err(std::io::Error::new(std::io::ErrorKind::Other, Error::NOT_ANDROID))
                }
                #[cfg(target_os = "android")] {
                    self.impls.write_fmt(fmt)
                }
            }
        }
    };
}

impl_write!(AsyncWritableStream);
impl_write!(SyncWritableStream);