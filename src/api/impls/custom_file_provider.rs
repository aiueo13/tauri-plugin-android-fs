use std::{collections::HashMap, sync::{Arc, LazyLock, Mutex as SyncMutex}};
use crate::{CustomFileCallback, CustomFileProvider, CustomFileRepr, FileAccessMode};


pub static CUSTOM_FILE_PROVIDERS: LazyLock<CustomFileProviderManager> = LazyLock::new(|| {
    CustomFileProviderManager::new()
});


pub struct CustomFileProviderManager {
    state: SyncMutex<CustomFileProviderManagerState>
}

struct CustomFileProviderManagerState {
    providers: Vec<(i32, String, Arc<dyn Fn() -> Arc<dyn CustomFileProvider> + Send + Sync + 'static>)>,
    custom_files: HashMap<i32, Arc<SyncMutex<Box<dyn CustomFileCallback>>>>,
    next_provider_id: i32,
    next_custom_file_id: i32,
}

impl CustomFileProviderManagerState {

    fn next_custom_file_id(&mut self) -> i32 {
        let id = self.next_custom_file_id;
        self.next_custom_file_id += 1;
        id
    }

    fn next_provider_id(&mut self) -> i32 {
        let id = self.next_provider_id;
        self.next_provider_id += 1;
        id
    }
}

impl CustomFileProviderManager {

    fn new() -> Self {
        Self {
            state: SyncMutex::new(CustomFileProviderManagerState {
                providers: Vec::new(),
                custom_files: HashMap::new(),
                next_provider_id: 0,
                next_custom_file_id: 0,
            })
        }
    }
}

impl CustomFileProviderManager {

    pub fn add_providers<F: Fn() -> Arc<dyn CustomFileProvider> + Send + Sync + 'static>(
        &self,
        providers: impl IntoIterator<Item = (String, F)>
    ) {

        let mut locked_state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        for (provider_name, provider) in providers {
            let provider_id = locked_state.next_provider_id();
            locked_state.providers.push((provider_id, provider_name, Arc::new(provider)));
        }
    }

    pub fn use_provider<R>(
        &self,
        provider_id: i32,
        f: impl FnOnce(&Arc<dyn CustomFileProvider>) -> R
    ) -> std::io::Result<R> {

        let provider = {
            let locked_state = self.state.lock().unwrap_or_else(|e| e.into_inner());
            locked_state.providers
                .iter()
                .find(|p| p.0 == provider_id)
                .map(|p| Arc::clone(&p.2))
                .ok_or_else(|| std::io::Error::other("missing provider"))?
        };

        let provider = (provider)();
        Ok(f(&provider))
    }

    pub fn get_provider_id_from_name(&self, provider_name: impl AsRef<str>) -> Option<i32> {
        let provider_name = provider_name.as_ref();
        let locked_state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        locked_state.providers
            .iter()
            .find(|p| &p.1 == provider_name)
            .map(|p| p.0)
    }
}

pub enum CustomFileDescriptor {
    CustomFile {
        id: i32,
    },
    File {
        file: std::fs::File
    }
}

impl CustomFileProviderManager {

    pub fn open_file(
        &self, 
        provider_id: i32, 
        path: String, 
        mode: FileAccessMode
    ) -> std::io::Result<CustomFileDescriptor> {

        let file = self.use_provider(provider_id, |p| p.open(path, mode))??;

        match file.repr {
            CustomFileRepr::Custom(callback) => {
                let mut locked_state = self.state.lock().unwrap_or_else(|e| e.into_inner());
                let id = locked_state.next_custom_file_id();
                locked_state.custom_files.insert(id, Arc::new(SyncMutex::new(callback)));
                Ok(CustomFileDescriptor::CustomFile { id })
            },
            CustomFileRepr::Raw(file) => {
                Ok(CustomFileDescriptor::File { file })
            },
        }
    }

    pub fn use_custom_file<R>(
        &self,
        file_id: i32,
        f: impl FnOnce(&mut dyn CustomFileCallback) -> R
    ) -> std::io::Result<R> {

        let file = {
            let locked_state = self.state.lock().unwrap_or_else(|e| e.into_inner());
            locked_state.custom_files
                .get(&file_id)
                .map(Arc::clone)
                .ok_or_else(|| std::io::Error::other("missing file"))?
        };

        let mut file = file.lock().expect("custom file callback should not be poisoned");
        Ok(f(file.as_mut()))
    }

    pub fn close_custom_file(&self, file_id: i32) {
        let mut locked_state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        locked_state.custom_files.remove(&file_id);
    }
}