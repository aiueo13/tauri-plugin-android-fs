/// A guard that removes the file on drop
pub struct TempFileGuard {
    pub(crate) path: std::path::PathBuf
}

impl Drop for TempFileGuard {

    fn drop(&mut self) {
        std::fs::remove_file(&self.path).ok();
    }
}