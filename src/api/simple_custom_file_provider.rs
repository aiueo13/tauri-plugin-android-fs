use std::{collections::HashMap, sync::{Arc, LazyLock, Mutex as SyncMutex}};
use tauri::{AppHandle, Manager};
use crate::*;


pub static SIMPLE_CUSTOM_FILE_PROVIDER_NAME: LazyLock<String> = LazyLock::new(|| {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};

    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).expect("could not get random bytes");
    format!("tpafs simple custom file provider: {}", URL_SAFE_NO_PAD.encode(bytes))
});

pub type SimpleCustomFilesState<'a, R> = tauri::State<'a, SimpleCustomFilesStateInner<R>>;
pub type SimpleCustomFilesStateInner<R> = std::sync::Arc::<SimpleCustomFileManager<R>>;

pub fn new_simple_custom_files_state<R: tauri::Runtime>(app: AppHandle<R>) -> SimpleCustomFilesStateInner<R> {
    std::sync::Arc::new(SimpleCustomFileManager::new(app))
}

pub fn take_simple_custom_file_id(uri: &FsUri) -> std::io::Result<i32> {
    #[cfg(target_os = "android")] {
        let (provider_id, path) = api::impls::take_provider_id_and_custom_file_path(&uri.uri)?;
        let expected_provider_id = api::impls::CUSTOM_FILE_PROVIDERS
            .get_provider_id_from_name(&*SIMPLE_CUSTOM_FILE_PROVIDER_NAME)
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "missing provider for the given name"))?;

        if provider_id != expected_provider_id {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "no simple custom file provider"))
        }

        let id = parse_path_to_id(&path)?;
        Ok(id)
    }
    #[cfg(not(target_os = "android"))] {
        Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "unsupported platform"))
    }
}


pub struct SimpleCustomFileManager<R: tauri::Runtime> {
    state: SyncMutex<SimpleCustomFileManagerState>,
    app: tauri::AppHandle<R>
}

struct SimpleCustomFileManagerState {
    files: HashMap<i32, Arc<SimpleCustomFileHandle>>,
    next_id: i32,
}

impl SimpleCustomFileManagerState {

    fn next_id(&mut self) -> i32 {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        id
    }
}

impl<R: tauri::Runtime> SimpleCustomFileManager<R> {

    fn new(app: tauri::AppHandle<R>) -> Self {
        Self {
            app,
            state: SyncMutex::new(SimpleCustomFileManagerState {
                files: HashMap::new(),
                next_id: 0
            })
        }
    }

    pub fn add(&self, 
        name: String,
        mime_type: Option<String>,
        len: Option<u64>,
        last_modified: Option<std::time::SystemTime>,
        open: impl Fn() -> std::io::Result<CustomFile> + Send + Sync + 'static,
    ) -> Result<FsUri> {

        let file = Arc::new(SimpleCustomFileHandle {
            name: Some(name),
            mime_type,
            len,
            last_modified,
            open: Box::new(open)
        });

        let mut locked_state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        let id = locked_state.next_id();
        locked_state.files.insert(id, file);

        self.app.android_fs().build_custom_file_uri(
            &*SIMPLE_CUSTOM_FILE_PROVIDER_NAME,
            id.to_string()
        )
    }

    pub fn close(&self, id: i32) -> bool {
        let mut locked_state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        locked_state.files.remove(&id).is_some()
    }

    fn get(&self, id: i32) -> std::io::Result<Arc<SimpleCustomFileHandle>> {
        let locked_state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        locked_state.files
            .get(&id)
            .map(Arc::clone)
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "file unavailable"))
    }
}

struct SimpleCustomFileHandle {
    pub open: Box<dyn Fn() -> std::io::Result<CustomFile> + Send + Sync + 'static>,
    pub len: Option<u64>,
    pub name: Option<String>,
    pub mime_type: Option<String>,
    pub last_modified: Option<std::time::SystemTime>,
}


pub struct SimpleCustomFileProvider<R: tauri::Runtime> {
    app: tauri::AppHandle<R>
}

impl<R: tauri::Runtime> SimpleCustomFileProvider<R> {

    pub fn new(app: tauri::AppHandle<R>) -> Self {
        Self { app }
    }

    fn get_handle(&self, path: String) -> std::io::Result<Arc<SimpleCustomFileHandle>> {
        let id = parse_path_to_id(&path)?;
        let files: SimpleCustomFilesState<'_, R> = self.app.state();
        let file = files.get(id)?;
        Ok(file)
    }
}

impl<R: tauri::Runtime> CustomFileProvider for SimpleCustomFileProvider<R> {

    fn open(&self, path: String, mode: FileAccessMode) -> std::io::Result<CustomFile> {
        if mode != FileAccessMode::Read {
            return Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "unsupported mode"))?;
        }

        let file = (self.get_handle(path)?.open)()?;
        Ok(file)
    }

    fn get_name(&self, path: String) -> std::io::Result<Option<String>> {
        Ok(self.get_handle(path)?.name.to_owned())
    }

    fn get_len(&self, path: String) -> std::io::Result<Option<u64>> {
        Ok(self.get_handle(path)?.len.clone())
    }

    fn get_mime_type(&self, path: String) -> std::io::Result<Option<String>> {
        Ok(self.get_handle(path)?.mime_type.to_owned())
    }

    fn get_last_modified(&self, path: String) -> std::io::Result<Option<std::time::SystemTime>> {
        Ok(self.get_handle(path)?.last_modified.clone())
    }
}

fn parse_path_to_id(path: &str) -> std::io::Result<i32> {
    let id = path
        .parse()
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid uri format"))?;

    Ok(id)
}