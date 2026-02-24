use std::borrow::Cow;
use serde::{ser::Serializer, Serialize};

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct Error {
    inner: InnerError
}

#[allow(unused)]
impl crate::Error {

    pub(crate) const NOT_ANDROID: Self = Self::from_static_str(
        "unsupported platform; only Android is supported"
    );

    pub(crate) fn missing_value(value_name: impl AsRef<str>) -> Self {
        Self::with(format!("missing value: {}", value_name.as_ref()))
    }

    pub(crate) fn invalid_type(type_name: impl AsRef<str>) -> Self {
        Self::with(format!("invalid type for {}", type_name.as_ref()))
    }

    pub(crate) fn invalid_value(value_name: impl AsRef<str>) -> Self {
        Self::with(format!("invalid value {}", value_name.as_ref()))
    }

    pub(crate) const fn from_static_str(msg: &'static str) -> Self {
        Self { inner: InnerError::Raw(Cow::Borrowed(msg)), }
    }

    pub fn with(msg: impl Into<Cow<'static, str>>) -> Self {
        Self { inner: InnerError::Raw(msg.into()) }
    }
}

impl From<crate::Error> for std::io::Error {

    fn from(e: crate::Error) -> std::io::Error {
        match e.inner {
            InnerError::Io(e) => e,
            e => std::io::Error::new(std::io::ErrorKind::Other, e)
        }
    }
}

impl From<crate::Error> for tauri::Error {

    fn from(e: crate::Error) -> tauri::Error {
        match e.inner {
            InnerError::Tauri(e) => e,
            InnerError::Io(e) => tauri::Error::Io(e),

            #[cfg(target_os = "android")]
            InnerError::PluginInvoke(e) => tauri::Error::PluginInvoke(e),

            e => tauri::Error::Anyhow(e.into()),
        }
    }
}


#[derive(Debug, thiserror::Error)]
enum InnerError {
    #[error("{0}")]
    Raw(Cow<'static, str>),

    #[cfg(target_os = "android")]
    #[error(transparent)]
    PluginInvoke(tauri::plugin::mobile::PluginInvokeError),

    #[cfg(target_os = "android")]
    #[error(transparent)]
    Base64Decode(base64::DecodeError),

    #[error(transparent)]
    Io(std::io::Error),

    #[error(transparent)]
    ParseInt(std::num::ParseIntError),

    #[error(transparent)]
    SerdeJson(serde_json::Error),

    #[error(transparent)]
    Tauri(tauri::Error),

    #[error(transparent)]
    TauriHttpHeaderToStr(tauri::http::header::ToStrError),

    #[error(transparent)]
    TauriPluginFs(tauri_plugin_fs::Error),

    #[error(transparent)]
    StdSystemTime(std::time::SystemTimeError),

    #[error(transparent)]
    Utf8Error(std::str::Utf8Error),
}

macro_rules! impl_into_err_from_inner {
    ($from:ty, $e:pat => $a:expr) => {
        impl From<$from> for crate::Error {
            fn from($e: $from) -> crate::Error {
                $a
            }
        }
    };
}

#[cfg(target_os = "android")]
impl_into_err_from_inner!(tauri::plugin::mobile::PluginInvokeError, e => crate::Error { inner: InnerError::PluginInvoke(e) });

#[cfg(target_os = "android")]
impl_into_err_from_inner!(base64::DecodeError, e => crate::Error { inner: InnerError::Base64Decode(e) });

impl_into_err_from_inner!(std::io::Error, e => crate::Error { inner: InnerError::Io(e) });
impl_into_err_from_inner!(std::num::ParseIntError, e => crate::Error { inner: InnerError::ParseInt(e) });
impl_into_err_from_inner!(serde_json::Error, e => crate::Error { inner: InnerError::SerdeJson(e) });
impl_into_err_from_inner!(tauri::Error, e => crate::Error { inner: InnerError::Tauri(e) });
impl_into_err_from_inner!(tauri::http::header::ToStrError, e => crate::Error { inner: InnerError::TauriHttpHeaderToStr(e) });
impl_into_err_from_inner!(tauri_plugin_fs::Error, e => crate::Error { inner: InnerError::TauriPluginFs(e) });
impl_into_err_from_inner!(std::time::SystemTimeError, e => crate::Error { inner: InnerError::StdSystemTime(e) });
impl_into_err_from_inner!(std::str::Utf8Error, e => crate::Error { inner: InnerError::Utf8Error(e) });

impl<W> From<std::io::IntoInnerError<W>> for crate::Error {
    fn from(e: std::io::IntoInnerError<W>) -> crate::Error {
        crate::Error { inner: InnerError::Io(e.into_error()) }
    }
}

impl<T> From<std::sync::PoisonError<T>> for crate::Error {
    fn from(_: std::sync::PoisonError<T>) -> crate::Error {
        crate::Error::with("thread poisoned")
    }
}

impl Serialize for crate::Error {

    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match &self.inner {
            InnerError::Raw(msg) => serializer.serialize_str(&msg),
            e => serializer.serialize_str(&e.to_string())
        }
    }
}