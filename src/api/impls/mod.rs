#![allow(unused)]

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

mod ext;
mod raw;

use serde::{de::DeserializeOwned, Serialize};
use std::collections::{HashMap, VecDeque};
use std::hash::Hash;
use std::borrow::Borrow;
use crate::*;
use sync_async::sync_async;


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


#[sync_async]
mod utils {
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
    pub fn sleep(duration: std::time::Duration) -> Result<()> {
        #[if_async] {
            // NOTE:
            // tokio の sleep は使わない。
            // Tauri はデベロッパーが独自の Tokio runtime を設定できるので
            // time が有効になってない Tokio runtime が使われることでパニックになる可能性がある。
            tauri::async_runtime::spawn_blocking(move || std::thread::sleep(duration)).await?;
            Ok(())
        }
        #[if_sync] {
            std::thread::sleep(duration);
            Ok(())
        }
    }
}