#[allow(unused)]
macro_rules! impl_se {
    ($t:item) => {
        #[derive(serde::Serialize)]
        #[serde(rename_all = "camelCase")]
        $t
    };
}

#[allow(unused)]
macro_rules! impl_de {
    ($t:item) => {
        #[derive(serde::Deserialize)]
        #[serde(rename_all = "camelCase")]
        $t
    };
}

mod ext;
mod raw;
mod writable_stream;

use serde::{de::DeserializeOwned, Serialize};
use crate::*;
use sync_async::sync_async;

#[sync_async]
mod util {
    use super::*;

    #[maybe_async]
    pub fn run_blocking<T, F>(task: F) -> Result<T> 
    where 
        T: Send + 'static,
        F: FnOnce() -> Result<T> + Send + 'static,
    {
        #[if_async] {
            tauri::async_runtime::spawn_blocking(task).await?
        }
        #[if_sync] {
            task()
        }
    }

    #[maybe_async]
    pub fn run_blocking_with_io_err<T, F>(task: F) -> std::io::Result<T> 
    where 
        T: Send + 'static,
        F: FnOnce() -> std::io::Result<T> + Send + 'static,
    {
        #[if_async] {
            tauri::async_runtime::spawn_blocking(task)
                .await
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?
        }
        #[if_sync] {
            task()
        }
    }
}


pub use writable_stream::{AsyncWritableStreamImpls, SyncWritableStreamImpls};

#[sync_async]
pub struct Impls<'a, R: tauri::Runtime> {
    pub handle: &'a tauri::plugin::PluginHandle<R>
}

#[sync_async]
impl<'a, R: tauri::Runtime> Impls<'a, R> {

    #[maybe_async]
    pub(super) fn invoke<D: DeserializeOwned>(
        &self,
        command: &str,
        payload: impl Serialize
    ) -> Result<D> {
        
        #[if_sync] {
            self.handle.run_mobile_plugin(command, payload).map_err(Into::into)
        }
        #[if_async] {
            self.handle.run_mobile_plugin_async(command, payload).await.map_err(Into::into)
        }
    }

    #[always_sync]
    pub(super) fn invoke_sync<D: DeserializeOwned>(
        &self,
        command: &str,
        payload: impl Serialize
    ) -> Result<D> {
        
        self.handle.run_mobile_plugin(command, payload).map_err(Into::into)
    }
}