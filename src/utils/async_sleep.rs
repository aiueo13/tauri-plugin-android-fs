use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Condvar, LazyLock, Mutex};
use std::task::{Context, Poll, Waker};
use std::time::{Duration, Instant};


pub async fn sleep(time: Duration) {
    SLEEP_MANAGER.sleep(time).await;
}

static SLEEP_MANAGER: LazyLock<SleepManager> = LazyLock::new(SleepManager::new);


struct SleepManager {
    inner: Arc<(Mutex<ManagerState>, Condvar)>,
}

struct ManagerState {
    tasks: Vec<SleepTask>,
    thread_running: bool,
}

struct SleepTask {
    deadline: Instant,
    shared_state: Arc<Mutex<SharedTaskState>>,
}

struct SharedTaskState {
    completed: bool,
    waker: Option<Waker>,
}

impl SleepManager {

    pub fn new() -> Self {
        Self {
            inner: Arc::new((
                Mutex::new(ManagerState {
                    tasks: Vec::new(),
                    thread_running: false,
                }),
                Condvar::new(),
            )),
        }
    }

    pub fn sleep(&self, time: Duration) -> impl Future<Output = ()> {
        let shared_task_state = Arc::new(Mutex::new(SharedTaskState {
            completed: false,
            waker: None,
        }));

        let task = SleepTask {
            deadline: Instant::now() + time,
            shared_state: shared_task_state.clone(),
        };

        let (lock, cvar) = &*self.inner;
        let mut state = lock.lock().unwrap();
        
        state.tasks.push(task);

        if !state.thread_running {
            state.thread_running = true;
            let inner_clone = self.inner.clone();

            tauri::async_runtime::spawn_blocking(move || {
                let (lock, cvar) = &*inner_clone;
                let mut state = lock.lock().unwrap();

                loop {
                    let now = Instant::now();
                    
                    // 期限切れのタスクを処理する
                    state.tasks.retain(|task| {
                        if task.deadline <= now {
                            let mut ts = task.shared_state.lock().unwrap();
                            ts.completed = true;
                            if let Some(waker) = ts.waker.take() {
                                waker.wake();
                            }
                            false
                        } 
                        else {
                            true
                        }
                    });

                    // タスクがなければスレッドを終了する
                    if state.tasks.is_empty() {
                        state.thread_running = false;
                        break;
                    }

                    // 次に期限を迎えるタスクまで待機する
                    let next_deadline = state.tasks.iter().map(|t| t.deadline).min().unwrap();
                    let now = Instant::now();
                    if now < next_deadline {
                        let wait_time = next_deadline - now;
                        let (new_state, _) = cvar.wait_timeout(state, wait_time).unwrap();
                        state = new_state;
                    }
                }
            });
        }
        else {
            // すでにスレッドが待機中の場合、新しく追加したタスクの期限が
            // 既存のタスクより早い可能性があるので、一度スレッドを起こして再計算させる
            cvar.notify_one();
        }

        SleepFuture { shared_state: shared_task_state }
    }
}

struct SleepFuture {
    shared_state: Arc<Mutex<SharedTaskState>>,
}

impl Future for SleepFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state = self.shared_state.lock().unwrap();
        
        if state.completed {
            Poll::Ready(())
        }
        else {
            state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}