use anyhow::Result;
use log::{error, info};

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};

use tokio::{
    sync::{mpsc::UnboundedSender, oneshot, Notify},
    time::error::Elapsed,
};

use crate::{
    browser::{automation::JSMessage, ClickableElement},
    messages::TabMessage,
    LoadState, TabMessageType,
};

type TabMessageCallback = Box<dyn Fn(TabMessage) + Send + Sync>;

pub struct BrowserState {
    pub title: String,
    pub url: String,
    pub favicon: Option<String>,
    pub load_state: LoadState,
    pub navigation_started: bool,
    pub cursor: String,
    pub width: u32,
    pub height: u32,
    pub dpr: f64,
    pub active: bool,
    pub left_mouse_button_down: bool,

    pub clickable_elements: Option<Vec<ClickableElement>>,

    pub javascript_messages: HashMap<String, oneshot::Sender<Result<JSMessage>>>,
    pub subscribers: HashMap<i32, UnboundedSender<TabMessage>>,
    pub single_event_subscribers: HashMap<TabMessageType, TabMessageCallback>,
}

pub struct SharedBrowserState {
    state: Arc<Mutex<BrowserState>>,
    notify: Arc<Notify>,
}

impl SharedBrowserState {
    pub fn new(state: BrowserState) -> Self {
        SharedBrowserState {
            state: Arc::new(Mutex::new(state)),
            notify: Arc::new(Notify::new()),
        }
    }

    pub fn update<T: FnOnce(&mut BrowserState)>(&self, updater: T) {
        let mut state = self.state.lock().expect("Browser state lock poisoned");
        updater(&mut state);
        self.notify.notify_waiters();
    }

    pub fn read<T: FnOnce(&BrowserState) -> R, R>(&self, reader: T) -> R {
        let state = self.state.lock().expect("Browser state lock poisoned");
        reader(&state)
    }

    pub fn update_and_return<T: FnOnce(&mut BrowserState) -> R, R>(&self, updater: T) -> R {
        let mut state = self.state.lock().expect("Browser state lock poisoned");
        let result = updater(&mut state);
        self.notify.notify_waiters();
        result
    }

    pub fn subscribe(&self, id: i32, tx: UnboundedSender<TabMessage>) {
        let mut state = self.state.lock().expect("Browser state lock poisoned");
        state.subscribers.insert(id, tx);
    }

    pub fn unsubscribe(&self, id: i32) {
        let mut state = self.state.lock().expect("Browser state lock poisoned");
        state.subscribers.remove(&id);
    }

    pub fn notify(&self, message: TabMessage) {
        let state = self.state.lock().expect("Browser state lock poisoned");
        for (_, tx) in &state.subscribers {
            if let Err(e) = tx.send(message.clone()) {
                error!("Failed to send message to subscriber: {}", e);
            }
        }

        if let Some(callback) = state.single_event_subscribers.get(&message.event_type()) {
            callback(message);
        }
    }

    pub async fn wait_for<F: Fn(&BrowserState) -> bool>(
        &self,
        condition: F,
        timeout: Duration,
    ) -> Result<(), Elapsed> {
        tokio::time::timeout(timeout, async {
            loop {
                {
                    let state = self.state.lock().expect("Browser state lock poisoned");
                    if condition(&state) {
                        return;
                    }
                }

                self.notify.notified().await;
            }
        })
        .await
    }

    pub fn on(&self, event: TabMessageType, callback: TabMessageCallback) {
        self.update(|s| {
            s.single_event_subscribers.insert(event, callback);
        });
    }
}

impl Clone for SharedBrowserState {
    fn clone(&self) -> Self {
        SharedBrowserState {
            state: self.state.clone(),
            notify: self.notify.clone(),
        }
    }
}

impl Drop for SharedBrowserState {
    fn drop(&mut self) {
        if Arc::strong_count(&self.state) == 1 {
            info!("BrowserState dropped");
        }
    }
}
