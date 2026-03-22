use sync_async::sync_async;
use crate::*;
use super::*;


#[sync_async]
pub struct ProgressNotificationGuard<R: tauri::Runtime> {
    #[cfg(target_os = "android")]
    inner: Inner<R>,

    #[cfg(not(target_os = "android"))]
    inner: std::marker::PhantomData<fn() -> R>,
}

#[cfg(target_os = "android")]
#[sync_async(
    use(if_sync) impls::SyncImpls as Impls;
    use(if_async) impls::AsyncImpls as Impls;
)]
impl<R: tauri::Runtime> ProgressNotificationGuard<R> {

    #[maybe_async]
    pub(crate) fn start_new_notification(
        icon: ProgressNotificationIcon,
        title: Option<String>,
        text: Option<String>,
        sub_text: Option<String>,
        progress: Option<u64>,
        progress_max: Option<u64>,
        handle: tauri::plugin::PluginHandle<R>,
    ) -> Result<Self> {

        let impls = Impls { handle: &handle };

        let (n_progress, n_progress_max) = normalize_progress_and_max(progress, progress_max);
        let id = impls
            .start_progress_notification(
                icon, 
                title.as_deref(), 
                text.as_deref(), 
                sub_text.as_deref(),
                n_progress, 
                n_progress_max
            )
            .await?;

        Ok(Self {
            inner: Inner {
                current_state: std::sync::Mutex::new(CurrentState {
                    title: title.clone(),
                    text,
                    sub_text,
                    progress,
                    progress_max
                }),
                drop_behavior: std::sync::Mutex::new(DropBehavior { 
                    title: None,
                    text: None,
                    sub_text: None,
                    error: true,
                }),
                update_throttler: Throttler::with_delay(std::time::Duration::from_millis(1100)),
                id,
                icon,
                handle,
                is_finished: false,
            }
        })
    }

    #[always_sync]
    fn impls(&self) -> Impls<'_, R> {
        Impls { handle: &self.inner.handle }
    }
}

#[sync_async(
    use(if_async) api_async::{AndroidFs, Utils};
    use(if_sync) api_sync::{AndroidFs, Utils};
)]
impl<R: tauri::Runtime> ProgressNotificationGuard<R> {

    #[always_sync]
    pub fn into_sync(self) -> SyncProgressNotificationGuard<R> {
        SyncProgressNotificationGuard { inner: self.inner }
    }

    #[always_sync]
    pub fn into_async(self) -> AsyncProgressNotificationGuard<R> {
        AsyncProgressNotificationGuard { inner: self.inner }
    }

    #[always_sync]
    pub fn title(&self) -> Option<String> {
        #[cfg(not(target_os = "android"))] {
            None
        }
        #[cfg(target_os = "android")] {
            self.inner.lock_current_state().title.clone()
        }
    }

    #[always_sync]
    pub fn text(&self) -> Option<String> {
        #[cfg(not(target_os = "android"))] {
            None
        }
        #[cfg(target_os = "android")] {
            self.inner.lock_current_state().text.clone()
        }
    }

    #[always_sync]
    pub fn sub_text(&self) -> Option<String> {
        #[cfg(not(target_os = "android"))] {
            None
        }
        #[cfg(target_os = "android")] {
            self.inner.lock_current_state().sub_text.clone()
        }
    }

    #[always_sync]
    pub fn progress(&self) -> Option<u64> {
        #[cfg(not(target_os = "android"))] {
            None
        }
        #[cfg(target_os = "android")] {
            self.inner.lock_current_state().progress
        }
    }

    #[always_sync]
    pub fn progress_max(&self) -> Option<u64> {
        #[cfg(not(target_os = "android"))] {
            None
        }
        #[cfg(target_os = "android")] {
            self.inner.lock_current_state().progress_max
        }
    }

    #[maybe_async]
    pub fn update_progress_by(&self, addend: u64) -> Result<()> {
        #[cfg(not(target_os = "android"))] {
            Ok(())
        }
        #[cfg(target_os = "android")] {
            {
                let mut state = self.inner.lock_current_state();
                state.progress = Some(state.progress.unwrap_or(0).saturating_add(addend));
            }

            self.update_notification().await?;
            Ok(())
        }
    }

    #[maybe_async]
    pub fn update_progress(&self, progress: Option<u64>) -> Result<()> {
        #[cfg(not(target_os = "android"))] {
            Ok(())
        }
        #[cfg(target_os = "android")] {
            {
                let mut state = self.inner.lock_current_state();
                state.progress = progress;
            }

            self.update_notification().await?;
            Ok(())
        }
    }

    #[maybe_async]
    pub fn update_progress_max(&self, progress_max: Option<u64>) -> Result<()> {
        #[cfg(not(target_os = "android"))] {
            Ok(())
        }
        #[cfg(target_os = "android")] {
            {
                let mut state = self.inner.lock_current_state();
                state.progress_max = progress_max;
            }

            self.update_notification().await?;
            Ok(())
        }
    }

    #[maybe_async]
    pub fn update_title(&self, title: Option<&str>) -> Result<()> {
        #[cfg(not(target_os = "android"))] {
            Ok(())
        }
        #[cfg(target_os = "android")] {
            {
                let mut state = self.inner.lock_current_state();
                state.title = title.map(|s| s.to_string());
            }

            self.update_notification().await?;
            Ok(())
        }
    }

    #[maybe_async]
    pub fn update_text(&self, text: Option<&str>) -> Result<()> {
        #[cfg(not(target_os = "android"))] {
            Ok(())
        }
        #[cfg(target_os = "android")] {
           {
                let mut state = self.inner.lock_current_state();
                state.text = text.map(|s| s.to_string());
            }

            self.update_notification().await?;
            Ok(())
        }
    }

    #[maybe_async]
    pub fn update_sub_text(&self, sub_text: Option<&str>) -> Result<()> {
        #[cfg(not(target_os = "android"))] {
            Ok(())
        }
        #[cfg(target_os = "android")] {
            {
                let mut state = self.inner.lock_current_state();
                state.sub_text = sub_text.map(|s| s.to_string());
            };

            self.update_notification().await?;
            Ok(())
        }
    }

    #[maybe_async]
    pub fn update(
        &self,
        title: Option<&str>,
        text: Option<&str>,
        sub_text: Option<&str>,
        progress: Option<u64>,
        progress_max: Option<u64>,
    ) -> Result<()> {

        #[cfg(not(target_os = "android"))] {
            Ok(())
        }
        #[cfg(target_os = "android")] {
            {
                let mut state = self.inner.lock_current_state();
                *state = CurrentState {
                    title: title.map(|s| s.to_string()),
                    text: text.map(|s| s.to_string()),
                    sub_text: sub_text.map(|s| s.to_string()),
                    progress,
                    progress_max
                };
            }

            self.update_notification().await?;
            Ok(())
        }
    }

    #[always_sync]
    pub fn set_drop_behavior_to_complete(
        &self,
        title: Option<&str>,
        text: Option<&str>,
        sub_text: Option<&str>,
    ) {

        #[cfg(target_os = "android")] {
            let title = title.map(|s| s.to_string());
            let text = text.map(|s| s.to_string());
            let sub_text = sub_text.map(|s| s.to_string());

            *self.inner.lock_drop_behavior() = DropBehavior { 
                title: Some(Box::new(move || title)),
                text: Some(Box::new(move || text)),
                sub_text: Some(Box::new(move || sub_text)),
                error: false,
            };
        }
    }

    #[always_sync]
    pub fn set_drop_behavior_to_complete_with(
        &self,
        title: impl 'static + Send + FnOnce() -> Option<String>,
        text: impl 'static + Send + FnOnce() -> Option<String>,
        sub_text: impl 'static + Send + FnOnce() -> Option<String>,
    ) {
        
        #[cfg(target_os = "android")] {
            *self.inner.lock_drop_behavior() = DropBehavior { 
                title: Some(Box::new(title)),
                text: Some(Box::new(text)),
                sub_text: Some(Box::new(sub_text)),
                error: false,
            };
        }
    }

    #[always_sync]
    pub fn set_drop_behavior_to_fail(
        &self,
        title: Option<&str>,
        text: Option<&str>,
        sub_text: Option<&str>
    ) {

        #[cfg(target_os = "android")] {
            let title = title.map(|s| s.to_string());
            let text = text.map(|s| s.to_string());
            let sub_text = sub_text.map(|s| s.to_string());

            *self.inner.lock_drop_behavior() = DropBehavior { 
                title: Some(Box::new(move || title)),
                text: Some(Box::new(move || text)),
                sub_text: Some(Box::new(move || sub_text)),
                error: true,
            };
        }
    }

    #[always_sync]
    pub fn set_drop_behavior_to_fail_with(
        &self,
        title: impl 'static + Send + FnOnce() -> Option<String>,
        text: impl 'static + Send + FnOnce() -> Option<String>,
        sub_text: impl 'static + Send + FnOnce() -> Option<String>,
    ) {
        
        #[cfg(target_os = "android")] {
            *self.inner.lock_drop_behavior() = DropBehavior { 
                title: Some(Box::new(title)),
                text: Some(Box::new(text)),
                sub_text: Some(Box::new(sub_text)),
                error: true,
            };
        }
    }

    #[maybe_async]
    pub fn complete(
        self,
        title: Option<&str>,
        text: Option<&str>,
        sub_text: Option<&str>,
    ) -> Result<()> {

        #[cfg(not(target_os = "android"))] {
            Ok(())
        }
        #[cfg(target_os = "android")] {
            self.finish_notification(title, text, sub_text, false).await
        }
    }

    #[maybe_async]
    pub fn fail(
        self, 
        title: Option<&str>, 
        text: Option<&str>,
        sub_text: Option<&str>,
    ) -> Result<()> {

        #[cfg(not(target_os = "android"))] {
            Ok(())
        }
        #[cfg(target_os = "android")] { 
            self.finish_notification(title, text, sub_text, true).await
        }
    }
    
    
    #[cfg(target_os = "android")] 
    #[maybe_async]
    fn finish_notification(
        mut self,
        title: Option<&str>, 
        text: Option<&str>,
        sub_text: Option<&str>,
        error: bool
    ) -> Result<()> {

        self.impls().finish_progress_notification(
            self.inner.id, 
            self.inner.icon,
            title,
            text,
            sub_text,
            error
        ).await?;
            
        self.inner.is_finished = true;
        Ok(())
    }

    #[cfg(target_os = "android")] 
    #[maybe_async]
    fn update_notification(&self) -> Result<()> {
        if self.inner.update_throttler.check_and_mark() {
            let state = self.inner.lock_current_state().clone();
            let (progress, progress_max) = normalize_progress_and_max(state.progress, state.progress_max);
            
            self.impls().update_progress_notification(
                self.inner.id,
                self.inner.icon, 
                state.title.as_deref(),
                state.text.as_deref(), 
                state.sub_text.as_deref(), 
                progress,
                progress_max,
            ).await?;
        }

        Ok(())
    }
}


#[cfg(target_os = "android")]
struct Inner<R: tauri::Runtime> {
    id: i32,
    icon: ProgressNotificationIcon,
    is_finished: bool,
    drop_behavior: std::sync::Mutex<DropBehavior>,
    current_state: std::sync::Mutex<CurrentState>,
    update_throttler: Throttler,
    handle: tauri::plugin::PluginHandle<R>,
}

#[cfg(target_os = "android")]
struct DropBehavior {
    title: Option<Box<dyn Send + 'static + FnOnce() -> Option<String>>>,
    text: Option<Box<dyn Send + 'static + FnOnce() -> Option<String>>>,
    sub_text: Option<Box<dyn Send + 'static + FnOnce() -> Option<String>>>,
    error: bool,
}

#[cfg(target_os = "android")]
#[derive(Clone)]
struct CurrentState {
    title: Option<String>,
    text: Option<String>,
    sub_text: Option<String>,
    progress: Option<u64>,
    progress_max: Option<u64>,
}

#[cfg(target_os = "android")]
impl<R: tauri::Runtime> Inner<R> {

    fn lock_drop_behavior<'a>(&'a self) -> std::sync::MutexGuard<'a, DropBehavior> {
        self.drop_behavior.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn lock_current_state<'a>(&'a self) -> std::sync::MutexGuard<'a, CurrentState> {
        self.current_state.lock().unwrap_or_else(|e| e.into_inner())
    }
}

#[cfg(target_os = "android")]
impl<R: tauri::Runtime> Drop for Inner<R> {

    fn drop(&mut self) {
        if self.is_finished {
            return
        }

        let handle = self.handle.clone();
        let id = self.id;
        let icon = self.icon;
        let (error, title, text, sub_text) = {
            let mut d = self.lock_drop_behavior();
            (d.error, d.title.take(), d.text.take(), d.sub_text.take())
        };

        tauri::async_runtime::spawn(async move {
            let impls = impls::AsyncImpls { handle: &handle };
            impls.finish_progress_notification(
                id, 
                icon,
                title.and_then(|f| f()).as_deref(),
                text.and_then(|f| f()).as_deref(),
                sub_text.and_then(|f| f()).as_deref(),
                error
            ).await.ok();
        });
    }
}

#[cfg(target_os = "android")]
struct Throttler {
    next: std::sync::Mutex<std::time::Instant>,
    interval: std::time::Duration,
}

#[cfg(target_os = "android")]
impl Throttler {

    pub fn with_delay(interval: std::time::Duration) -> Self {
        Self {
            next: std::sync::Mutex::new(std::time::Instant::now() + interval),
            interval,
        }
    }

    pub fn check_and_mark(&self) -> bool {
        let mut next = self.next.lock().unwrap_or_else(|e| e.into_inner());
        let now = std::time::Instant::now();
        
        if now < *next {
            return false
        }

        *next = now + self.interval;
        true
    }
}

#[cfg(target_os = "android")]
fn normalize_progress_and_max(
    progress: Option<u64>,
    progress_max: Option<u64>,
) -> (Option<i32>, Option<i32>) {

    let Some((progress, progress_max)) = Option::zip(progress, progress_max) else {
        return (None, None)
    };
    
    const PROGRESS_MAX: i32 = 100_000;

    if progress_max == 0 {
        return (Some(0), Some(0)); 
    }

    let ratio = progress as f64 / progress_max as f64;
    let scaled_progress = (ratio * PROGRESS_MAX as f64) as i32;

    (Some(i32::min(scaled_progress, PROGRESS_MAX)), Some(PROGRESS_MAX))
}