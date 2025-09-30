use anyhow::Result;

use std::collections::HashMap;
use std::sync::atomic::Ordering;
use std::sync::{atomic::AtomicI32, Arc, Mutex};

use cef_ui::{
    Browser, DevToolsMessageObserver, DevToolsMessageObserverCallbacks, DictionaryValue,
    Registration,
};
use log::trace;
use serde::Deserialize;
use tokio::sync::oneshot;
#[derive(Debug)]
struct Response {
    #[allow(dead_code)]
    success: bool,
    data: Vec<u8>,
}

#[derive(Deserialize)]
struct Screenshot {
    data: String,
}

#[derive(Default)]
struct DevToolsState {
    pending_requests: HashMap<i32, oneshot::Sender<Response>>,
}

#[derive(Default)]
struct SharedDevToolsState {
    inner: Mutex<DevToolsState>,
}

impl SharedDevToolsState {
    fn on_result(&self, message_id: i32, success: bool, data: Vec<u8>) {
        let response = Response { success, data };
        if let Some(tx) = self
            .inner
            .lock()
            .unwrap()
            .pending_requests
            .remove(&message_id)
        {
            let _ = tx.send(response);
        }
    }

    fn subscribe(&self, message_id: i32, tx: oneshot::Sender<Response>) {
        let mut state = self.inner.lock().expect("Browser state lock poisoned");
        if state.pending_requests.contains_key(&message_id) {
            trace!(
                "Message ID {} already exists in pending requests",
                message_id
            );
            return;
        }
        state.pending_requests.insert(message_id, tx);
    }
}

pub struct DevTools {
    #[allow(unused)]
    browser: Browser,
    state: Arc<SharedDevToolsState>,
    #[allow(unused)]
    registration: Registration,
    counter: AtomicI32,
}

impl DevTools {
    pub fn new(browser: Browser) -> Self {
        let state = Arc::new(SharedDevToolsState::default());
        let observer = DevToolsMessageObserver::new(DevToolsObserverCallbacks::new(state.clone()));

        let host = browser.get_host().unwrap();
        let registration = host
            .add_dev_tools_message_observer(observer)
            .expect("failed to add DevTools message observer");

        host.execute_dev_tools_method(0, "Page.enable", None)
            .expect("failed to enable Page domain");

        Self {
            browser,
            state,
            registration,
            counter: AtomicI32::new(10),
        }
    }

    pub async fn screenshot(&self) -> Result<String> {
        let response = self.execute_method("Page.captureScreenshot", None).await;

        if !response.success {
            return Err(anyhow::anyhow!(
                "Failed to capture screenshot: {}",
                String::from_utf8_lossy(&response.data)
            ));
        }

        let screenshot = serde_json::from_slice::<Screenshot>(&response.data)?;
        Ok(screenshot.data)
    }

    async fn execute_method(&self, name: &str, params: Option<DictionaryValue>) -> Response {
        let id = self.counter.fetch_add(1, Ordering::Relaxed);
        let (tx, rx) = oneshot::channel();
        self.state.subscribe(id, tx);

        _ = self
            .browser
            .get_host()
            .unwrap()
            .execute_dev_tools_method(id, name, params);

        rx.await.unwrap()
    }
}

struct DevToolsObserverCallbacks {
    state: Arc<SharedDevToolsState>,
}

impl DevToolsObserverCallbacks {
    pub fn new(state: Arc<SharedDevToolsState>) -> Self {
        Self { state }
    }
}

impl DevToolsMessageObserverCallbacks for DevToolsObserverCallbacks {
    fn on_dev_tools_method_result(
        &mut self,
        _: Browser,
        message_id: i32,
        success: bool,
        result: &[u8],
    ) {
        self.state.on_result(message_id, success, result.to_vec());
    }

    fn on_dev_tools_event(&mut self, _: Browser, _: &str, _: &[u8]) {}
}
