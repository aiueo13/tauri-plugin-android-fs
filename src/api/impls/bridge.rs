use std::any::Any;
use jni::{JNIEnv, objects::{JByteArray, JClass, JObject, JString, JThrowable, JValue, ReleaseMode}, sys::{jint, jintArray, jlong}};
use crate::{FileAccessMode, api::{CUSTOM_FILE_PROVIDERS, impls::{CustomFileDescriptor, take_provider_id_and_custom_file_path}}, utils::panic_message};


pub const PLUGIN_PACKAGE: &str = "okayu.tauri.plugin.android.fs";
pub const PLUGIN_MAIN_CLASS: &str = "AndroidFsPlugin";

const CUSTOM_FILE_ERRNO_EXCEPTION_CLASS: &str = "okayu/tauri/plugin/android/fs/CustomFileErrnoException";


#[no_mangle]
extern "system" fn Java_okayu_tauri_plugin_android_fs_CustomFileProviderBridge_getMimeType<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    uri: JString<'l>,
) -> JString<'l> {

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> std::io::Result<_> {
        let uri = env.get_string(&uri)
            .expect("uri instance should be java.lang.String")
            .to_string_lossy()
            .to_string();

        let (provider_id, path) = take_provider_id_and_custom_file_path(&uri)?;

        let mime_type = CUSTOM_FILE_PROVIDERS
            .use_provider(provider_id, |provider| {
                provider.get_mime_type(path)
            })??
            .map(|m| env.new_string(m).expect("failed to create new string"))
            .unwrap_or_else(|| JObject::null().into());

        Ok(mime_type)
    }));

    match result {
        Ok(Ok(mime_type)) => mime_type,
        Ok(Err(err)) => {
            throw_io_exception(env, err);
            JObject::null().into()
        },
        Err(panic) => {
            throw_for_panic(env, panic);
            JObject::null().into()
        },
    }
}

#[no_mangle]
extern "system" fn Java_okayu_tauri_plugin_android_fs_CustomFileProviderBridge_getName<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    uri: JString<'l>,
) -> JString<'l> {

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> std::io::Result<_> {
        let uri = env.get_string(&uri)
            .expect("uri instance should be java.lang.String")
            .to_string_lossy()
            .to_string();

        let (provider_id, path) = take_provider_id_and_custom_file_path(&uri)?;

        let name = CUSTOM_FILE_PROVIDERS
            .use_provider(provider_id, |provider| {
                provider.get_name(path)
            })??
            .map(|m| env.new_string(m).expect("failed to create new string"))
            .unwrap_or_else(|| JObject::null().into());

        Ok(name)
    }));

    match result {
        Ok(Ok(name)) => name,
        Ok(Err(err)) => {
            throw_io_exception(env, err);
            JObject::null().into()
        },
        Err(panic) => {
            throw_for_panic(env, panic);
            JObject::null().into()
        },
    }
}

#[no_mangle]
extern "system" fn Java_okayu_tauri_plugin_android_fs_CustomFileProviderBridge_getLen<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    uri: JString<'l>,
) -> jlong {

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> std::io::Result<_> {
        let uri = env.get_string(&uri)
            .expect("uri instance should be java.lang.String")
            .to_string_lossy()
            .to_string();

        let (provider_id, path) = take_provider_id_and_custom_file_path(&uri)?;

        let len = CUSTOM_FILE_PROVIDERS
            .use_provider(provider_id, |provider| {
                provider.get_len(path)
            })??
            .map(|l| l.try_into().unwrap_or(-1))
            .unwrap_or(-1);

        Ok(len)
    }));

    match result {
        Ok(Ok(len)) => len,
        Ok(Err(err)) => {
            throw_io_exception(env, err);
            -1
        },
        Err(panic) => {
            throw_for_panic(env, panic);
            -1
        },
    }
}

#[no_mangle]
extern "system" fn Java_okayu_tauri_plugin_android_fs_CustomFileProviderBridge_getLastModified<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    uri: JString<'l>,
) -> jlong {

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> std::io::Result<_> {
        let uri = env.get_string(&uri)
            .expect("uri instance should be java.lang.String")
            .to_string_lossy()
            .to_string();

        let (provider_id, path) = take_provider_id_and_custom_file_path(&uri)?;

        let modified = CUSTOM_FILE_PROVIDERS
            .use_provider(provider_id, |provider| {
                provider.get_last_modified(path)
            })??
            .and_then(|l| l.duration_since(std::time::UNIX_EPOCH).ok())
            .and_then(|l| l.as_millis().try_into().ok())
            .unwrap_or(-1);

        Ok(modified)
    }));

    match result {
        Ok(Ok(modified)) => modified,
        Ok(Err(err)) => {
            throw_io_exception(env, err);
            -1
        },
        Err(panic) => {
            throw_for_panic(env, panic);
            -1
        },
    }
}

#[no_mangle]
extern "system" fn Java_okayu_tauri_plugin_android_fs_CustomFileProviderBridge_delete<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    uri: JString<'l>
) {

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> std::io::Result<_> {
        let uri = env.get_string(&uri)
            .expect("uri instance should be java.lang.String")
            .to_string_lossy()
            .to_string();

        let (provider_id, path) = take_provider_id_and_custom_file_path(&uri)?;

        CUSTOM_FILE_PROVIDERS.use_provider(provider_id, |provider| {
            provider.remove(path)
        })?
    }));

    match result {
        Ok(Ok(())) => (),
        Ok(Err(err)) => {
            if err.kind() == std::io::ErrorKind::Unsupported {
                throw_unsupported_operation_exception(env, err);
            }
            else {
                throw_io_exception(env, err);
            }
            Default::default()
        },
        Err(panic) => {
            throw_for_panic(env, panic);
            Default::default()
        },
    }
}

#[no_mangle]
extern "system" fn Java_okayu_tauri_plugin_android_fs_CustomFileProviderBridge_open<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    uri: JString<'l>,
    mode: JString<'l>,
) -> jintArray {

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> std::io::Result<_> {
        let uri = env.get_string(&uri)
            .expect("uri instance should be java.lang.String")
            .to_string_lossy()
            .to_string();

        let mode = env.get_string(&mode)
            .expect("mode instance should be java.lang.String")
            .to_string_lossy()
            .to_string();

        let (provider_id, path) = take_provider_id_and_custom_file_path(&uri)?;

        let mode = FileAccessMode::from_mode(&mode)
            .map_err(|_| std::io::Error::other("invalid mode"))?;

        let value = match CUSTOM_FILE_PROVIDERS.open_file(provider_id, path, mode)? {
            CustomFileDescriptor::File { file } => {
                use std::os::fd::IntoRawFd;
                let fd = file.into_raw_fd();
                [0, fd]
            },
            CustomFileDescriptor::CustomFile { id } => {
                let custom_file_id = id;
                [1, custom_file_id]
            },
        };

        let array = env.new_int_array(2).expect("failed to create new int array");
        env.set_int_array_region(&array, 0, &value).expect("failed to set array region");
        Ok(array.into_raw())
    }));

    match result {
        Ok(Ok(file_value)) => file_value,
        Ok(Err(err)) => {
            throw_io_exception(env, err);
            Default::default()
        },
        Err(panic) => {
            throw_for_panic(env, panic);
            Default::default()
        },
    }
}

#[no_mangle]
extern "system" fn Java_okayu_tauri_plugin_android_fs_CustomFileBridge_close<'l>(
    env: JNIEnv<'l>,
    _class: JClass<'l>,
    file_id: jint,
) {

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        CUSTOM_FILE_PROVIDERS.close_custom_file(file_id);
    }));

    match result {
        Ok(()) => (),
        Err(panic) => {
            throw_for_panic(env, panic);
            ()
        },
    }
}

/// # SAFETY
/// buf は有効な Java byte 配列へのローカル参照である。
/// 実行中に他の Rust thread や Java thread から buf にアクセスされない。
/// 同じ buf に対して、この関数が同時に複数回呼ばれない。
#[no_mangle]
unsafe extern "system" fn Java_okayu_tauri_plugin_android_fs_CustomFileBridge_readAt<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    file_id: jint,
    offset: jlong,
    len: jint,
    buf: JByteArray,
) -> jint {

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> std::io::Result<_> {
        let offset: u64 = offset
            .try_into()
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid offset range"))?;

        let len: usize = len
            .try_into()
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid len range"))?;

        // SAFETY:
        // 存在期間中に他の Rust thread や Java thread から同じ配列にアクセスされず、
        // 同じ配列に対する AutoElements が複数存在しない。
        let mut buf_array = env
            .get_array_elements(&buf, ReleaseMode::CopyBack)
            .map_err(|err| std::io::Error::other(err))?;

        let buf: &mut [u8] = bytemuck::cast_slice_mut(&mut buf_array);

        if buf.len() < len {
            return Err(std::io::Error::from_raw_os_error(libc::EINVAL));
        }

        let buf = &mut buf[..len];
        
        let nread = CUSTOM_FILE_PROVIDERS.use_custom_file(file_id, |file| -> std::io::Result<_> {
            // ProxyFileDescriptorCallback.onRead は EOF に到達しない限り
            // len まで正確に読み込む必要がある。
            let mut total_n = 0;
            loop {
                let buf = &mut buf[total_n..];
                if buf.is_empty() {
                    break;
                }

                match file.read_at(buf, offset + total_n as u64) {
                    Ok(0) => break,
                    Ok(n) => total_n += n,
                    Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                    Err(e) => return Err(e),
                }
            }

            Ok(total_n)
        })??;

        let nread: jint = nread
            .try_into()
            .map_err(|_| std::io::Error::other("invalid nread range"))?;

        Ok(nread)
    }));

    match result {
        Ok(Ok(nread)) => nread,
        Ok(Err(err)) => {
            throw_custom_file_errno_exception(env, err);
            -1
        },
        Err(panic) => {
            throw_for_panic(env, panic);
            -1
        },
    }
}

/// # SAFETY
/// buf は有効な Java byte 配列へのローカル参照である。
/// 実行中に他の Rust thread や Java thread から buf にアクセスされない。
/// 同じ buf に対して、この関数が同時に複数回呼ばれない。
#[no_mangle]
unsafe extern "system" fn Java_okayu_tauri_plugin_android_fs_CustomFileBridge_writeAt<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    file_id: jint,
    offset: jlong,
    len: jint,
    buf: JByteArray,
) -> jint {

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> std::io::Result<_> {
        let offset: u64 = offset
            .try_into()
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid offset range"))?;

        let len: usize = len
            .try_into()
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid len range"))?;

        // SAFETY:
        // 存在期間中に他の Rust thread や Java thread から同じ配列にアクセスされず、
        // 同じ配列に対する AutoElements が複数存在しない。
        let buf_array = env
            .get_array_elements(&buf, ReleaseMode::NoCopyBack)
            .map_err(|err| std::io::Error::other(err))?;

        let buf: &[u8] = bytemuck::cast_slice(&buf_array);

        if buf.len() < len {
            return Err(std::io::Error::from_raw_os_error(libc::EINVAL));
        }

        let buf = &buf[..len];
        
        let nwrite = CUSTOM_FILE_PROVIDERS.use_custom_file(file_id, |file| -> std::io::Result<_> {
            let mut total_n = 0;
            loop {
                let buf = &buf[total_n..];
                if buf.is_empty() {
                    break;
                }

                match file.write_at(buf, offset + total_n as u64) {
                    Ok(0) => return Err(std::io::Error::new(std::io::ErrorKind::WriteZero, "unexpected zero write")),
                    Ok(n) => total_n += n,
                    Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                    Err(e) => return Err(e),
                }
            }
            Ok(total_n)
        })??;

        let nwrite: jint = nwrite
            .try_into()
            .map_err(|_| std::io::Error::other("invalid nwrite range"))?;

        Ok(nwrite)
    }));

    match result {
        Ok(Ok(nwrite)) => nwrite,
        Ok(Err(err)) => {
            throw_custom_file_errno_exception(env, err);
            -1
        },
        Err(panic) => {
            throw_for_panic(env, panic);
            -1
        },
    }
}

#[no_mangle]
extern "system" fn Java_okayu_tauri_plugin_android_fs_CustomFileBridge_len<'l>(
    env: JNIEnv<'l>,
    _class: JClass<'l>,
    file_id: jint,
) -> jlong {

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> std::io::Result<_> {
        let len = CUSTOM_FILE_PROVIDERS.use_custom_file(file_id, |file| {
            file.len()
        })??;

        let len = len
            .try_into()
            .map_err(|_| std::io::Error::other("too big len"))?;

        Ok(len)
    }));

    match result {
        Ok(Ok(len)) => len,
        Ok(Err(err)) => {
            throw_custom_file_errno_exception(env, err);
            -1
        },
        Err(panic) => {
            throw_for_panic(env, panic);
            -1
        },
    }
}

#[no_mangle]
extern "system" fn Java_okayu_tauri_plugin_android_fs_CustomFileBridge_fsync<'l>(
    env: JNIEnv<'l>,
    _class: JClass<'l>,
    file_id: jint,
) {

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> std::io::Result<_> {
        CUSTOM_FILE_PROVIDERS.use_custom_file(file_id, |file| {
            file.fsync()
        })?
    }));

    match result {
        Ok(Ok(())) => (),
        Ok(Err(err)) => {
            throw_custom_file_errno_exception(env, err);
            ()
        },
        Err(panic) => {
            throw_for_panic(env, panic);
            ()
        },
    }
}


fn throw_io_exception<'local>(
    mut env: JNIEnv<'local>,
    err: std::io::Error
) {

    if env.exception_check().unwrap_or(true) {
        return;
    }

    if errno_from_std_io_err(&err) == libc::ENOENT {
        let _ = env.throw_new("java/io/FileNotFoundException", err.to_string());
    }
    else {
        let _ = env.throw_new("java/io/IOException", err.to_string());
    }
}

fn throw_unsupported_operation_exception<'local>(
    mut env: JNIEnv<'local>,
    err: impl std::error::Error
) {

    if env.exception_check().unwrap_or(true) {
        return;
    }

    let _ = env.throw_new("java/lang/UnsupportedOperationException ", err.to_string());
}

fn throw_for_panic<'local>(
    mut env: JNIEnv<'local>,
    panic_payload: Box<dyn Any + Send + 'static>
) {

    if env.exception_check().unwrap_or(true) {
        return;
    }

    let message = match panic_message(panic_payload) {
        Some(message) => format!("rust panicked: {message}"),
        None => format!("rust panicked"),
    };

    let _ = env.throw_new("java/lang/IllegalStateException", message);
}

fn throw_custom_file_errno_exception<'local>(
    mut env: JNIEnv<'local>,
    err: std::io::Error,
) {

    if env.exception_check().unwrap_or(true) {
        return;
    }

    let result = (|| -> jni::errors::Result<()> {
        let msg = env.new_string(err.to_string())?;
        let errno = errno_from_std_io_err(&err);

        let exception = env.new_object(
            CUSTOM_FILE_ERRNO_EXCEPTION_CLASS,
            "(Ljava/lang/String;I)V",
            &[JValue::Object(&msg), JValue::Int(errno)],
        )?;
        let _ = env.throw(JThrowable::from(exception));
        Ok(())
    })();

    if let Err(e) = result {
        if !env.exception_check().unwrap_or(true) {
            let _ = env.throw_new("java/lang/Error", format!("Failed to resolve CustomFileErrnoException: {e}"));
        }
    }
}

fn errno_from_std_io_err(err: &std::io::Error) -> i32 {
    use std::io::ErrorKind as K;

    if let Some(errno) = err.raw_os_error() {
        return errno;
    }

    match err.kind() {
        K::NotFound => libc::ENOENT,
        K::PermissionDenied => libc::EACCES,
        K::ConnectionRefused => libc::ECONNREFUSED,
        K::ConnectionReset => libc::ECONNRESET,
        K::HostUnreachable => libc::EHOSTUNREACH,
        K::NetworkUnreachable => libc::ENETUNREACH,
        K::ConnectionAborted => libc::ECONNABORTED,
        K::NotConnected => libc::ENOTCONN,
        K::AddrInUse => libc::EADDRINUSE,
        K::AddrNotAvailable => libc::EADDRNOTAVAIL,
        K::NetworkDown => libc::ENETDOWN,
        K::BrokenPipe => libc::EPIPE,
        K::AlreadyExists => libc::EEXIST,
        K::WouldBlock => libc::EAGAIN, // Linux/Android では EWOULDBLOCK == EAGAIN
        K::NotADirectory => libc::ENOTDIR,
        K::IsADirectory => libc::EISDIR,
        K::DirectoryNotEmpty => libc::ENOTEMPTY,
        K::ReadOnlyFilesystem => libc::EROFS,
        K::StaleNetworkFileHandle => libc::ESTALE,
        K::InvalidInput => libc::EINVAL,
        K::InvalidData => libc::EILSEQ,
        K::TimedOut => libc::ETIMEDOUT,
        K::WriteZero => libc::EIO,
        K::StorageFull => libc::ENOSPC,
        K::NotSeekable => libc::ESPIPE,
        K::QuotaExceeded => libc::EDQUOT,
        K::FileTooLarge => libc::EFBIG,
        K::ResourceBusy => libc::EBUSY,
        K::ExecutableFileBusy => libc::ETXTBSY,
        K::Deadlock => libc::EDEADLK,
        K::CrossesDevices => libc::EXDEV,
        K::TooManyLinks => libc::EMLINK,
        K::InvalidFilename => libc::ENAMETOOLONG,
        K::ArgumentListTooLong => libc::E2BIG,
        K::Interrupted => libc::EINTR,
        K::Unsupported => libc::ENOSYS,
        K::UnexpectedEof => libc::EIO,
        K::OutOfMemory => libc::ENOMEM,
        K::Other => libc::EIO,
        _ => libc::EIO,
    }
}