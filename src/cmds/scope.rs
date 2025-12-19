
// Based on code from tauri-plugin-fs crate
//
// Source:
// - https://github.com/tauri-apps/plugins-workspace/blob/3d0d2e041bbad9766aebecaeba291a28d8d7bf5c/plugins/fs/build.rs#L20
// - Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// - Licensed under the MIT License or the Apache 2.0 License
#[derive(schemars::JsonSchema, serde::Deserialize)]
#[serde(untagged)]
#[allow(unused)]
pub enum ScopeSchema {

    /// A path that can be accessed by the webview when using the Android fs APIs.
    ///
    /// The pattern can start with a variable that resolves to a system base directory.
    /// The variables are: `$AUDIO`, `$CACHE`, `$CONFIG`, `$DATA`, `$LOCALDATA`, `$DESKTOP`,
    /// `$DOCUMENT`, `$DOWNLOAD`, `$EXE`, `$FONT`, `$HOME`, `$PICTURE`, `$PUBLIC`, `$RUNTIME`,
    /// `$TEMPLATE`, `$VIDEO`, `$RESOURCE`, `$APP`, `$LOG`, `$TEMP`, `$APPCONFIG`, `$APPDATA`,
    /// `$APPLOCALDATA`, `$APPCACHE`, `$APPLOG`.
    Value(std::path::PathBuf),

    Object {
        /// A path that can be accessed by the webview when using the Android fs APIs.
        ///
        /// The pattern can start with a variable that resolves to a system base directory.
        /// The variables are: `$AUDIO`, `$CACHE`, `$CONFIG`, `$DATA`, `$LOCALDATA`, `$DESKTOP`,
        /// `$DOCUMENT`, `$DOWNLOAD`, `$EXE`, `$FONT`, `$HOME`, `$PICTURE`, `$PUBLIC`, `$RUNTIME`,
        /// `$TEMPLATE`, `$VIDEO`, `$RESOURCE`, `$APP`, `$LOG`, `$TEMP`, `$APPCONFIG`, `$APPDATA`,
        /// `$APPLOCALDATA`, `$APPCACHE`, `$APPLOG`.
        path: std::path::PathBuf,
    },
}

#[derive(Debug)]
#[allow(unused)]
pub struct Scope {
    pub path: Option<std::path::PathBuf>,
}