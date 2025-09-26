use std::borrow::Cow;
use serde::{ser::Serializer, Serialize};

#[derive(Debug, Clone, thiserror::Error)]
#[error("{msg}")]
pub struct Error {
    pub(crate) msg: Cow<'static, str>
}

impl Error {

    #[allow(unused)]
    pub(crate) fn with(msg: impl Into<Cow<'static, str>>) -> Self {
        Self { msg: msg.into() }
    }
}

#[cfg(target_os = "android")]
impl From<tauri::plugin::mobile::PluginInvokeError> for crate::Error {

    fn from(value: tauri::plugin::mobile::PluginInvokeError) -> Self {
        Self { msg: Cow::Owned(value.to_string())}
    }
}

impl From<std::io::Error> for crate::Error {

    fn from(value: std::io::Error) -> Self {
        Self { msg: Cow::Owned(value.to_string())}
    }
}

impl<W> From<std::io::IntoInnerError<W>> for crate::Error {

    fn from(value: std::io::IntoInnerError<W>) -> Self {
        Self { msg: Cow::Owned(value.error().to_string())}
    }
}

impl From<serde_json::Error> for crate::Error {

    fn from(value: serde_json::Error) -> Self {
        Self { msg: Cow::Owned(value.to_string())}
    }
}

impl From<tauri::Error> for crate::Error {

    fn from(value: tauri::Error) -> Self {
        Self { msg: Cow::Owned(value.to_string())}
    }
}

impl Serialize for crate::Error {

    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.msg)
    }
}