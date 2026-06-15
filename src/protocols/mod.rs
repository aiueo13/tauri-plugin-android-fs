#![cfg(all(target_os = "android", any(feature = "protocol_content", feature = "protocol_thumbnail")))]

#[cfg(feature = "protocol_content")]
pub mod protocol_content;

#[cfg(feature = "protocol_thumbnail")]
pub mod protocol_thumbnail;

mod state;
mod utils;

pub use state::*;
pub(super) use utils::*;