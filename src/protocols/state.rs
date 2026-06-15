use crate::*;


pub type ProtocolConfigState<'a> = tauri::State<'a, ProtocolConfigStateInner>;
pub type ProtocolConfigStateInner = std::sync::Arc<ProtocolsConfig>;

pub fn new_config_state<R: tauri::Runtime, M: tauri::Manager<R>>(
    config: Option<&config::Config>,
    manager: &M,
) -> ProtocolConfigStateInner {

    std::sync::Arc::new(ProtocolsConfig {
        #[cfg(feature = "protocol_thumbnail")]
        thumbnail: ThumbnailProtocolConfig { 
            scope: config.as_ref().and_then(|c| tauri::scope::fs::Scope::new(
                manager,
                &c.thumbnail_protocol.scope,
            ).ok()),
            enable: config.as_ref().map(|c| c.thumbnail_protocol.enable).unwrap_or(false),
        },
        #[cfg(feature = "protocol_content")]
        content: ContentProtocolConfig { 
            scope: config.as_ref().and_then(|c| tauri::scope::fs::Scope::new(
                manager,
                &c.content_protocol.scope,
            ).ok()), 
            enable: config.as_ref().map(|c| c.content_protocol.enable).unwrap_or(false),
        },
    })
}

pub struct ProtocolsConfig {
    #[cfg(feature = "protocol_thumbnail")]
    pub thumbnail: ThumbnailProtocolConfig,

    #[cfg(feature = "protocol_content")]
    pub content: ContentProtocolConfig,
}

#[cfg(feature = "protocol_content")]
pub struct ContentProtocolConfig {
    pub scope: Option<tauri::scope::fs::Scope>,
    pub enable: bool,
}

#[cfg(feature = "protocol_thumbnail")]
pub struct ThumbnailProtocolConfig {
    pub scope: Option<tauri::scope::fs::Scope>,
    pub enable: bool,
}