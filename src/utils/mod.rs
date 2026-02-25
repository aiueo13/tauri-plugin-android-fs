#![cfg_attr(not(target_os = "android"), allow(unused))]

use crate::*;


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

pub struct BoundedHashMap<K, V> {
    map: std::collections::HashMap<K, V>,
    order: std::collections::VecDeque<K>,
    bound: usize,
}

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