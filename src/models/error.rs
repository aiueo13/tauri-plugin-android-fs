use std::borrow::Cow;
use serde::{ser::Serializer, Serialize};

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct Error {
    inner: InnerError
}

#[allow(unused)]
impl crate::Error {

    pub(crate) const NOT_ANDROID: Self = Self {
        inner: InnerError::Raw(Cow::Borrowed("This plugin is only for Android"))
    };

    pub(crate) fn missing_value(value_name: impl AsRef<str>) -> Self {
        Self::with(format!("missing value: {}", value_name.as_ref()))
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
    SerdeJson(serde_json::Error),

    #[error(transparent)]
    Tauri(tauri::Error),
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
impl_into_err_from_inner!(serde_json::Error, e => crate::Error { inner: InnerError::SerdeJson(e) });
impl_into_err_from_inner!(tauri::Error, e => crate::Error { inner: InnerError::Tauri(e) });

impl<W> From<std::io::IntoInnerError<W>> for crate::Error {
    fn from(e: std::io::IntoInnerError<W>) -> crate::Error {
        crate::Error { inner: InnerError::Io(e.into_error()) }
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