use serde::{Deserialize, Serialize};


/// Filters for VisualMediaPicker.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub enum VisualMediaTarget<'a> {

    /// Allow only images to be selected.  
    ImageOnly,

    /// Allow only videos to be selected.  
    VideoOnly,

    /// Allow only images and videos to be selected.  
    ImageAndVideo,

    /// Allow only images or videos of specified single Mime type to be selected.  
    ImageOrVideo {
        mime_type: &'a str
    }
}