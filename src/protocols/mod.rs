#![cfg(all(target_os = "android", any(feature = "protocol-content", feature = "protocol-thumbnail")))]

#[cfg(feature = "protocol-content")]
pub mod protocol_content;

#[cfg(feature = "protocol-thumbnail")]
pub mod protocol_thumbnail;

mod state;
mod utils;

pub use state::*;
pub(super) use utils::*;