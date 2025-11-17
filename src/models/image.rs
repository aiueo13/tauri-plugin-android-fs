use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Deserialize, Serialize)]
pub struct Size {
    pub width: u32,
    pub height: u32
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[non_exhaustive]
pub enum ImageFormat {

    /// - Loss less
    /// - Support transparency
    Png,

    /// - Lossy
    /// - Unsupport transparency
    Jpeg,

    /// - Lossy (**Not loss less**)
    /// - Support transparency
    Webp,

    /// - Lossy
    /// - Unsupport transparency
    JpegWith {

        /// Range is `0.0 ~ 1.0`  
        /// 0.0 means compress for the smallest size.  
        /// 1.0 means compress for max visual quality.  
        quality: f32
    },

    /// - Lossy
    /// - Support transparency
    WebpWith {
        
        /// Range is `0.0 ~ 1.0`  
        /// 0.0 means compress for the smallest size.  
        /// 1.0 means compress for max visual quality.  
        quality: f32
    }
}

impl ImageFormat {

    pub(crate) fn mime_type(&self) -> &'static str {
        match self {
            ImageFormat::Jpeg | ImageFormat::JpegWith { .. } => "image/jpeg",
            ImageFormat::Webp | ImageFormat::WebpWith { .. } => "image/webp",
            ImageFormat::Png => "image/png",
        }
    }
}