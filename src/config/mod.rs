#![allow(unused)]

#[derive(serde::Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Config {

    #[serde(default)]
    pub thumbnail_protocol: ThumbnailProtocolConfig,

    #[serde(default)]
    pub content_protocol: ContentProtocolConfig,
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

    /*
    /// Configuration for the internal cache used by the content protocol.
    ///
    /// If not specified, no cache is applied.
    #[serde(default)]
    pub cache: ContentProtocolCacheConfig,
    */
}

/* 
#[derive(serde::Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct ContentProtocolCacheConfig {

    /// Maximum number of files stored in the cache.  
    /// This is a cache of file descriptors and metadata; the actual file data is not covered.  
    ///
    /// Eviction is managed using an LRU (Least Recently Used) cache policy.
    ///
    /// If not specified, the number of cached files is unbounded.
    #[serde(default)]
    pub max_files: Option<usize>,

    /// Time-to-live (TTL) for each cached entry in milliseconds.
    ///
    /// If not specified, cached entries do not expire based on time.
    #[serde(default)]
    pub ttl_ms: Option<u64>,
}*/