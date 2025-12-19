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

fn encode_uri(input: impl AsRef<str>) -> String {
    // https://developer.android.com/reference/android/net/Uri.html#encode(java.lang.String)
    
    const SAFE: &percent_encoding::AsciiSet = &percent_encoding::NON_ALPHANUMERIC
        .remove(b'_')
        .remove(b'-')
        .remove(b'!')
        .remove(b'.')
        .remove(b'~')
        .remove(b'\'')
        .remove(b'(')
        .remove(b')')
        .remove(b'*');

    percent_encoding::utf8_percent_encode(input.as_ref(), SAFE).to_string()
}

fn validate_relative_path(path: &std::path::Path) -> Result<&std::path::Path> {
    for component in path.components() {
        use std::path::Component::*;
        
        match component {
            RootDir => return Err(crate::Error::with("must not start with root directory")),
            ParentDir => return Err(crate::Error::with("must not contain parent directory, i.e., '..'")),
            CurDir => return Err(crate::Error::with("must not contain current directory, i.e., '.'")),
            Prefix(_) => (),
            Normal(_) => (),
        }
    }

    Ok(path)
}

// Based on code from Tokio crate ver. 1.47.1
//
// Source:
// - https://docs.rs/tokio/1.47.1/src/tokio/util/as_ref.rs.html
// - Copyright (c) Tokio Contributors
// - Licensed under the MIT License
fn upgrade_bytes_ref<B: AsRef<[u8]>>(buf: B) -> Vec<u8> {

    // Based on code from Tokio crate ver. 1.47.1
    //
    // Source:
    // - https://docs.rs/tokio/1.47.1/src/tokio/util/typeid.rs.html
    // - Copyright (c) Tokio Contributors
    // - Licensed under the MIT License
    fn nonstatic_typeid<T>() -> std::any::TypeId
        where
            T: ?Sized,
    {
        trait NonStaticAny {
            fn get_type_id(&self) -> std::any::TypeId
            where
                Self: 'static;
        }

        impl<T: ?Sized> NonStaticAny for std::marker::PhantomData<T> {
            #[inline(always)]
            fn get_type_id(&self) -> std::any::TypeId
                where
                Self: 'static,
            {
                std::any::TypeId::of::<T>()
            }
        }

        let phantom_data = std::marker::PhantomData::<T>;
        NonStaticAny::get_type_id(unsafe {
            std::mem::transmute::<&dyn NonStaticAny, &(dyn NonStaticAny + 'static)>(&phantom_data)
        })
    }

    // Based on code from Tokio crate ver. 1.47.1
    //
    // Source:
    // - https://docs.rs/tokio/1.47.1/src/tokio/util/typeid.rs.html
    // - Copyright (c) Tokio Contributors
    // - Licensed under the MIT License
    unsafe fn try_transmute<Src, Target: 'static>(x: Src) -> std::result::Result<Target, Src> {
        if nonstatic_typeid::<Src>() == std::any::TypeId::of::<Target>() {
            let x = std::mem::ManuallyDrop::new(x);
            Ok(std::mem::transmute_copy::<Src, Target>(&x))
        } 
        else {
            Err(x)
        }
    }

    let buf = match unsafe { try_transmute::<B, Vec<u8>>(buf) } {
        Ok(vec) => return vec,
        Err(original_buf) => original_buf,
    };

    let buf = match unsafe { try_transmute::<B, String>(buf) } {
        Ok(string) => return string.into_bytes(),
        Err(original_buf) => original_buf,
    };

    buf.as_ref().to_owned()
}