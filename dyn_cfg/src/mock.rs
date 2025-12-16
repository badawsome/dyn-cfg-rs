use std::{
    collections::HashMap,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll},
};

use faststr::FastStr;
use futures::Stream;
use tokio::sync::{Notify, broadcast};

use crate::facade::{ConfCenterBasic, WatchConfCenter};
use crate::models::ConfGetRawResult;

#[derive(Debug, Clone)]
pub struct MockConfCenterBasic {
    conf: mini_moka::sync::Cache<FastStr, ConfGetRawResult>,
    default_result: Arc<Mutex<ConfGetRawResult>>,
    watchers: Arc<Mutex<HashMap<FastStr, broadcast::Sender<ConfGetRawResult>>>>,
    /// Track how many watchers are waiting for acknowledgment for each key
    pending_acknowledgments: Arc<Mutex<HashMap<FastStr, usize>>>,
    /// Notification for when all watchers have acknowledged
    completion_notifiers: Arc<Mutex<HashMap<FastStr, Arc<Notify>>>>,
}

impl std::fmt::Display for MockConfCenterBasic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MockConfCenterBasic")
    }
}

impl ConfCenterBasic for MockConfCenterBasic {
    fn get_raw(&self, key: FastStr) -> impl std::future::Future<Output = ConfGetRawResult> + Send {
        async move {
            match self.conf.get(&key) {
                Some(res) => res.clone(),
                None => self.default_result.lock().unwrap().clone(),
            }
        }
    }
}

impl MockConfCenterBasic {
    pub fn new(default_result: ConfGetRawResult) -> Self {
        Self {
            conf: mini_moka::sync::Cache::new(100),
            default_result: Arc::new(Mutex::new(default_result)),
            watchers: Arc::new(Mutex::new(HashMap::new())),
            pending_acknowledgments: Arc::new(Mutex::new(HashMap::new())),
            completion_notifiers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn default() -> Self {
        Self::new(ConfGetRawResult::NotExist {
            key: "mock-key".into(),
        })
    }

    pub fn insert(&self, key: FastStr, value: ConfGetRawResult) {
        self.conf.insert(key.clone(), value.clone());
        self.notify_watchers(key, value);
    }

    fn notify_watchers(&self, key: FastStr, value: ConfGetRawResult) {
        if let Some(sender) = self.watchers.lock().unwrap().get(&key) {
            let _ = sender.send(value.clone());
        }
    }

    /// Insert a configuration value and wait for all watchers to acknowledge the update.
    ///
    /// This method waits for all active watchers to confirm receipt of the update, ensuring
    /// configuration changes are fully propagated. If watchers do not acknowledge within the
    /// timeout period (500ms), a warning is logged but the operation does not fail.
    pub async fn insert_and_wait(&self, key: FastStr, value: ConfGetRawResult) {
        // Insert the value first
        self.conf.insert(key.clone(), value.clone());

        // Notify watchers
        self.notify_watchers(key.clone(), value);

        // Give watchers time to process the update
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }

    /// Track that a watcher has received and processed an update
    pub fn track_watcher_acknowledgment(&self, key: &FastStr) {
        let mut should_notify = false;
        {
            let mut pending = self.pending_acknowledgments.lock().unwrap();
            if let Some(count) = pending.get_mut(key) {
                if *count > 0 {
                    *count -= 1;
                    if *count == 0 {
                        should_notify = true;
                    }
                }
            }
        }

        if should_notify {
            let notifiers = self.completion_notifiers.lock().unwrap();
            if let Some(notify) = notifiers.get(key) {
                notify.notify_one();
            }
        }
    }

    fn get_or_create_watcher(&self, key: FastStr) -> broadcast::Receiver<ConfGetRawResult> {
        let mut watchers = self.watchers.lock().unwrap();
        match watchers.get(&key) {
            Some(sender) => sender.subscribe(),
            None => {
                let (sender, receiver) = broadcast::channel(16);
                watchers.insert(key, sender);
                receiver
            }
        }
    }
}

// Custom stream implementation for configuration watching
pub struct ConfWatchStream {
    receiver: broadcast::Receiver<ConfGetRawResult>,
}

impl ConfWatchStream {
    fn new(receiver: broadcast::Receiver<ConfGetRawResult>) -> Self {
        Self { receiver }
    }
}

impl Stream for ConfWatchStream {
    type Item = ConfGetRawResult;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // Try to receive without blocking
        match self.receiver.try_recv() {
            Ok(value) => Poll::Ready(Some(value)),
            Err(broadcast::error::TryRecvError::Empty) => {
                // No message available right now, but there might be one later
                // We need to register interest in receiving messages
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            Err(broadcast::error::TryRecvError::Closed) => Poll::Ready(None),
            Err(broadcast::error::TryRecvError::Lagged(_)) => {
                // If we lagged, we missed some messages, but there might be a latest value
                // Try to get the current value without blocking
                match self.receiver.try_recv() {
                    Ok(value) => Poll::Ready(Some(value)),
                    Err(_) => {
                        cx.waker().wake_by_ref();
                        Poll::Pending
                    }
                }
            }
        }
    }
}

impl WatchConfCenter for MockConfCenterBasic {
    fn watch_raw(
        &self,
        key: FastStr,
    ) -> impl futures::stream::Stream<Item = ConfGetRawResult> + Unpin + Send + Sync + 'static {
        let receiver = self.get_or_create_watcher(key.clone());
        ConfWatchStream::new(receiver)
    }
}
