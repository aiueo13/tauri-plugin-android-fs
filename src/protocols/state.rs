use crate::*;


pub type ProtocolConfigState<'a> = tauri::State<'a, ProtocolConfigStateInner>;
pub type ProtocolConfigStateInner = std::sync::Arc<ProtocolConfig>;

pub fn new_config_state<R: tauri::Runtime, M: tauri::Manager<R>>(
    config: Option<&config::Config>,
    manager: &M,
) -> ProtocolConfigStateInner {

    std::sync::Arc::new(ProtocolConfig {
        #[cfg(feature = "protocol-thumbnail")]
        enable_thumbnail: config.as_ref().map(|c| c.thumbnail_protocol.enable).unwrap_or(false),

        #[cfg(feature = "protocol-thumbnail")]
        thumbnail_scope: config.as_ref().and_then(|c| tauri::scope::fs::Scope::new(
            manager,
            &c.thumbnail_protocol.scope,
        ).ok()),

        #[cfg(feature = "protocol-content")]
        enable_content: config.as_ref().map(|c| c.content_protocol.enable).unwrap_or(false),

        #[cfg(feature = "protocol-content")]
        content_scope: config.as_ref().and_then(|c| tauri::scope::fs::Scope::new(
            manager,
            &c.content_protocol.scope,
        ).ok()),
    })
}

pub struct ProtocolConfig {
    #[cfg(feature = "protocol-thumbnail")]
    pub thumbnail_scope: Option<tauri::scope::fs::Scope>,

    #[cfg(feature = "protocol-thumbnail")]
    pub enable_thumbnail: bool,

    #[cfg(feature = "protocol-content")]
    pub content_scope: Option<tauri::scope::fs::Scope>,

    #[cfg(feature = "protocol-content")]
    pub enable_content: bool,
}