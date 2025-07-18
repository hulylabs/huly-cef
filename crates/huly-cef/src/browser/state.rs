use anyhow::Result;
use log::{error, info};

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use tokio::sync::{mpsc::UnboundedSender, oneshot};

use crate::{
    browser::{automation::JSMessage, ClickableElement},
    messages::TabMessage,
    LoadState,
};

pub enum RenderMode {
    Stream,
    Screenshot,
}

pub struct BrowserState {
    pub title: String,
    pub url: String,
    pub favicon: Option<String>,
    pub load_state: LoadState,
    pub cursor: String,
    pub width: u32,
    pub height: u32,
    pub active: bool,
    pub left_mouse_button_down: bool,

    pub render_mode: RenderMode,

    pub clickable_elements: Option<Vec<ClickableElement>>,

    pub javascript_messages: HashMap<String, oneshot::Sender<Result<JSMessage>>>,
    pub subscribers: HashMap<i32, UnboundedSender<TabMessage>>,
}

pub struct SharedBrowserState(Arc<Mutex<BrowserState>>);

impl SharedBrowserState {
    pub fn new(state: BrowserState) -> Self {
        SharedBrowserState(Arc::new(Mutex::new(state)))
    }

    pub fn update<T: FnOnce(&mut BrowserState)>(&self, updater: T) {
        let mut state = self.0.lock().expect("Browser state lock poisoned");
        updater(&mut state);
    }

    pub fn read<T: FnOnce(&BrowserState) -> R, R>(&self, reader: T) -> R {
        let state = self.0.lock().expect("Browser state lock poisoned");
        reader(&state)
    }

    pub fn update_and_return<T: FnOnce(&mut BrowserState) -> R, R>(&self, updater: T) -> R {
        let mut state = self.0.lock().expect("Browser state lock poisoned");
        updater(&mut state)
    }

    pub fn subscribe(&self, id: i32, tx: UnboundedSender<TabMessage>) {
        let mut state = self.0.lock().expect("Browser state lock poisoned");
        state.subscribers.insert(id, tx);
    }

    pub fn unsubscribe(&self, id: i32) {
        let mut state = self.0.lock().expect("Browser state lock poisoned");
        state.subscribers.remove(&id);
    }

    pub fn notify(&self, message: TabMessage) {
        let state = self.0.lock().expect("Browser state lock poisoned");
        for (_, tx) in &state.subscribers {
            if let Err(e) = tx.send(message.clone()) {
                error!("Failed to send message to subscriber: {}", e);
            }
        }
    }
}

impl Clone for SharedBrowserState {
    fn clone(&self) -> Self {
        SharedBrowserState(self.0.clone())
    }
}

impl Drop for SharedBrowserState {
    fn drop(&mut self) {
        if Arc::strong_count(&self.0) == 1 {
            info!("BrowserState dropped");
        }
    }
}
