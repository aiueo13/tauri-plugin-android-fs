#![allow(unused)]

#[derive(serde::Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Config {

    #[serde(default)]
    pub file_provider: FileProviderConfig,

    #[serde(default)]
    pub thumbnail_protocol: ThumbnailProtocolConfig,

    #[serde(default)]
    pub content_protocol: ContentProtocolConfig,
}

#[derive(serde::Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FileProviderConfig {

    /// The access scope for the file provider.
    #[serde(default)]
    pub scope: tauri::utils::config::FsScope,

    /// Enables the file provider.
    #[serde(default)]
    pub enable: bool,
}

#[derive(serde::Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ThumbnailProtocolConfig {

    /// The access scope for the thumbnail protocol.
    #[serde(default)]
    pub scope: tauri::utils::config::FsScope,

    /// Enables the thumbnail protocol.
    #[serde(default)]
    pub enable: bool,
}

#[derive(serde::Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentProtocolConfig {

    /// The access scope for the content protocol.
    #[serde(default)]
    pub scope: tauri::utils::config::FsScope,

    /// Enables the content protocol.
    #[serde(default)]
    pub enable: bool,
}