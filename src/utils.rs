#[cfg(target_os = "android")]
pub fn encode_document_id(input: impl AsRef<str>) -> String {
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
pub fn validate_relative_path(path: &std::path::Path) -> crate::Result<&std::path::Path> {
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

// Tokio crate ver. 1.47.1 (MIT License) の src/tokio/util/as_ref.rs を元にしたコード
// Code: https://docs.rs/tokio/1.47.1/src/tokio/util/as_ref.rs.html
// MIT License: https://spdx.org/licenses/MIT
#[cfg(target_os = "android")]
pub fn upgrade_bytes_ref<B: AsRef<[u8]>>(buf: B) -> Vec<u8> {

    // Tokio crate ver. 1.47.1 (MIT License) の src/tokio/util/typeid.rs を元にしたコード
    // Code: https://docs.rs/tokio/1.47.1/src/tokio/util/typeid.rs.html
    // MIT License: https://spdx.org/licenses/MIT
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

    // Tokio crate ver. 1.47.1 (MIT License) の src/tokio/util/typeid.rs を元にしたコード
    // Code: https://docs.rs/tokio/1.47.1/src/tokio/util/typeid.rs.html
    // MIT License: https://spdx.org/licenses/MIT
    // 
    // SAFETY: this function does not compare lifetimes. Values returned as `Ok`
    // may have their lifetimes extended.
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