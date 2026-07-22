
/// Icon type used for notification UI.
/// 
/// # Serialization
/// Serialized by `serde` as the following TypeScript type:
///
/// ```ts
/// // NOTE: New variants may be added in the future
/// type ProgressNotificationIconType = "App" | "Download" | "Upload" | "Save";
/// ```
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[non_exhaustive]
pub enum ProgressNotificationIcon {
    Download,
    Upload,
    Save,
    App,
}