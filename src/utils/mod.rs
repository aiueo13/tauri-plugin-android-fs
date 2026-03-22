#[allow(unused_imports)]
use crate::*;

#[cfg(target_os = "android")]
mod async_sleep;


#[cfg(target_os = "android")]
#[sync_async::sync_async]
pub mod utils {
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
    pub fn sleep(time: std::time::Duration) {
        #[if_async] {
            // NOTE:
            // tokio の sleep は使わない。
            // Tauri はデベロッパーが独自の Tokio runtime を設定できるので
            // time が有効になってない Tokio runtime が使われることでパニックになる可能性がある。
            async_sleep::sleep(time).await;
        }
        #[if_sync] {
            std::thread::sleep(time);
        }
    }
}

#[derive(serde::Deserialize)]
#[serde(untagged)]
#[cfg_attr(not(target_os = "android"), allow(unused))]
pub enum AfsUriOrFsPath {
    AfsUri(FileUri),
    FsPath(tauri_plugin_fs::FilePath),
}

#[cfg(target_os = "android")]
impl AfsUriOrFsPath {

    pub fn try_into_content_or_safe_file_scheme_uri(self) -> Result<FileUri> {
        match self {
            AfsUriOrFsPath::AfsUri(uri) => {
                if !uri.is_content_scheme() {
                    return Err(Error::invalid_uri_scheme(&uri.uri))
                }
                Ok(uri)
            },
            AfsUriOrFsPath::FsPath(path) => {
                match path {
                    tauri_plugin_fs::FilePath::Path(path) => {
                        Ok(FileUri::from_path(tauri::path::SafePathBuf::new(path)?))
                    },
                    tauri_plugin_fs::FilePath::Url(url) => {
                        if url.scheme() != "content" {
                            return Err(Error::invalid_uri_scheme(url))
                        }
                        Ok(FileUri::from_uri(url))
                    }
                }
            },
        }
    }

    pub fn try_into_content_uri(self) -> Result<FileUri> {
        match self {
            AfsUriOrFsPath::AfsUri(uri) => {
                if !uri.is_content_scheme() {
                    return Err(Error::invalid_uri_scheme(&uri.uri))
                }
                Ok(uri)
            },
            AfsUriOrFsPath::FsPath(path) => {
                match path {
                    tauri_plugin_fs::FilePath::Path(_) => {
                        Err(Error::with("invalid value: expected a content-scheme URI"))
                    },
                    tauri_plugin_fs::FilePath::Url(url) => {
                        if url.scheme() != "content" {
                            return Err(Error::invalid_uri_scheme(url))
                        }
                        Ok(FileUri::from_uri(url))
                    }
                }
            },
        }
    }
}

pub fn encode_android_uri_component(input: impl AsRef<str>) -> String {
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

#[cfg(target_os = "android")]
pub fn range_to_offset_and_len(range: impl std::ops::RangeBounds<u64>) -> (u128, Option<u128>) {
    use std::ops::Bound::{Included, Excluded, Unbounded};

    let offset = match range.start_bound() {
        Included(&v) => v as u128,
        Excluded(&v) => v as u128 + 1,
        Unbounded => 0,
    };
    let len = match range.end_bound() {
        Included(&v) => Some((v as u128 + 1).saturating_sub(offset)),
        Excluded(&v) => Some((v as u128).saturating_sub(offset)),
        Unbounded => None,
    };
    (offset, len)
}

#[cfg(target_os = "android")]
pub fn saturate_u128_to_u64(val: u128) -> u64 {
    u128::min(val, u64::MAX as u128) as u64
}

#[cfg(target_os = "android")]
pub fn validate_relative_path(path: &std::path::Path) -> Result<&std::path::Path> {
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
#[cfg(target_os = "android")]
pub fn upgrade_bytes_ref<B: AsRef<[u8]>>(buf: B) -> Vec<u8> {

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

#[cfg(target_os = "android")]
pub struct BoundedHashMap<K, V> {
    map: std::collections::HashMap<K, V>,
    order: std::collections::VecDeque<K>,
    bound: usize,
}

#[cfg(target_os = "android")]
impl<K: Eq + std::hash::Hash + Clone, V> BoundedHashMap<K, V> {

    pub fn with_bound(bound: usize) -> Self {
        Self {
            map: std::collections::HashMap::new(),
            order: std::collections::VecDeque::new(),
            bound,
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        // キーが既にあるなら重複を避けるために一度削除
        if self.map.contains_key(&key) {
            self.order.retain(|k| k != &key);
        }

        self.map.insert(key.clone(), value);
        self.order.push_back(key);

        // 容量超過時、最古の要素を削除
        if self.bound < self.map.len() {
            if let Some(oldest_key) = self.order.pop_front() {
                self.map.remove(&oldest_key);
            }
        }
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&V> 
    where 
        Q: ?Sized + std::hash::Hash + Eq,
        K: std::borrow::Borrow<Q>
    {
        self.map.get(key)
    }
}