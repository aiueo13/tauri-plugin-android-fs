#[cfg(all(target_os = "android", feature = "protocol-content"))]
pub mod protocol_content;

#[cfg(all(target_os = "android", feature = "protocol-thumbnail"))]
pub mod protocol_thumbnail;

#[cfg(all(target_os = "android", any(feature = "protocol-content", feature = "protocol-thumbnail")))]
mod state;

#[cfg(all(target_os = "android", any(feature = "protocol-content", feature = "protocol-thumbnail")))]
mod utils;

#[cfg(all(target_os = "android", any(feature = "protocol-content", feature = "protocol-thumbnail")))]
pub use state::*;

#[cfg(all(target_os = "android", any(feature = "protocol-content", feature = "protocol-thumbnail")))]
pub(super) use utils::*;