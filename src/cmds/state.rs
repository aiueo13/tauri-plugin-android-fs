#![cfg_attr(not(target_os = "android"), allow(unused))]

use crate::*;
use tauri::Manager as _;


pub type FileWriterResourcesState<'a, R> = PluginResourcesState<'a, R, FileWriterStateMarker>;
pub type FileWriterResourcesStateInner<R> = PluginResourcesStateInner<R, FileWriterStateMarker>;

pub struct FileWriterStateMarker;

pub fn new_file_writer_resources_state<R: tauri::Runtime>(app: tauri::AppHandle<R>) -> FileWriterResourcesStateInner<R> {
    std::sync::Arc::new(PluginResources::new(app))
}


pub type FileStreamResourcesState<'a, R> = PluginResourcesState<'a, R, FileStreamStateMarker>;
pub type FileStreamResourcesStateInner<R> = PluginResourcesStateInner<R, FileStreamStateMarker>;

pub struct FileStreamStateMarker;

pub fn new_file_stream_resources_state<R: tauri::Runtime>(app: tauri::AppHandle<R>) -> FileStreamResourcesStateInner<R> {
    std::sync::Arc::new(PluginResources::new(app))
}


pub type PluginResourcesState<'a, R, K> = tauri::State<'a, PluginResourcesStateInner<R, K>>;

pub type PluginResourcesStateInner<R, K> = std::sync::Arc::<PluginResources<R, K>>;

pub struct PluginResources<R: tauri::Runtime, K> {
    list: std::sync::Mutex<std::collections::HashSet<tauri::ResourceId>>,
    app: tauri::AppHandle<R>,
    _marker: std::marker::PhantomData<K>,
}

impl<R: tauri::Runtime, K> PluginResources<R, K> {

    fn new(app: tauri::AppHandle<R>) -> Self {
        Self {
            list: std::sync::Mutex::new(std::collections::HashSet::new()),
            app,
            _marker: Default::default()
        }
    }

    pub fn add<T: Sync + Send + 'static>(&self, r: T) -> Result<tauri::ResourceId> {
        let id = self.app.resources_table().add(PluginResource::new(r));
        self.list.lock()?.insert(id);
        Ok(id)
    }

    pub fn get<T: Sync + Send + 'static>(&self, id: tauri::ResourceId) -> Result<std::sync::Arc<T>> {
        let r = self.app.resources_table().get::<PluginResource<T>>(id)?;
        Ok(std::sync::Arc::clone(&r.resource))
    }

    pub fn close(&self, id: tauri::ResourceId) -> Result<()> {  
        self.list.lock()?.remove(&id);
        
        let mut rt = self.app.resources_table();
        if rt.has(id) {
            rt.close(id)?;
        }

        Ok(())
    }

    pub fn close_all(&self) -> Result<()> {
        let ids = self.list.lock()?
            .drain()
            .collect::<Vec<tauri::ResourceId>>();

        let mut rt = self.app.resources_table();
        for id in ids {
            if rt.has(id) {
                rt.close(id)?;
            }
        }

        Ok(())
    }

    pub fn count(&self) -> Result<usize> {
        let ids = self.list.lock()?
            .iter()
            .cloned()
            .collect::<Vec<tauri::ResourceId>>();

        let mut count = 0;
        let rt = self.app.resources_table();
        for id in ids {
            if rt.has(id) {
                count += 1;
            }
        }

        Ok(count)
    }
}

struct PluginResource<T> {
    resource: std::sync::Arc<T>
}

impl<T> PluginResource<T> {

    fn new(resource: T) -> Self {
        Self { resource: std::sync::Arc::new(resource) }
    }
}

impl<T: Sync + Send + 'static> tauri::Resource for PluginResource<T> {}