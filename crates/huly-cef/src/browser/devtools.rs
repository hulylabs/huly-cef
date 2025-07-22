use anyhow::Result;

use std::collections::HashMap;
use std::sync::atomic::Ordering;
use std::sync::{atomic::AtomicI32, Arc, Mutex};
use std::time::Duration;

use cef_ui::{
    Browser, DevToolsMessageObserver, DevToolsMessageObserverCallbacks, DictionaryValue,
    Registration,
};
use log::info;
use serde::{Deserialize, Serialize};
use tokio::sync::{oneshot, Notify};

#[derive(Debug)]
enum LifecycleEventType {
    Init,
    Load,
    NetworkAlmostIdle,
    NetworkIdle,
}

#[derive(Debug, strum::Display)]
enum Event {
    PageLifecycleEvent(LifecycleEventType),
}

struct Response {
    success: bool,
    data: Vec<u8>,
}

#[derive(Deserialize)]
struct Screenshot {
    data: String,
}

#[derive(Default)]
struct DevToolsState {
    load_fired: bool,
    network_idle_fired: bool,

    pending_requests: HashMap<i32, oneshot::Sender<Response>>,
}

#[derive(Default)]
struct SharedDevToolsState {
    inner: Mutex<DevToolsState>,
    notify: Notify,
}

impl SharedDevToolsState {
    fn on_event(&self, event: Event) {
        self.update_state(&event);
        self.notify.notify_waiters();
    }

    fn update_state(&self, event: &Event) {
        let mut state = self.inner.lock().unwrap();
        match event {
            Event::PageLifecycleEvent(name) => match name {
                LifecycleEventType::Init => {
                    state.network_idle_fired = false;
                    state.load_fired = false;
                }
                LifecycleEventType::Load => state.load_fired = true,
                LifecycleEventType::NetworkAlmostIdle | LifecycleEventType::NetworkIdle => {
                    if state.load_fired {
                        state.network_idle_fired = true;
                    }
                }
            },
        }
    }

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
            info!(
                "Message ID {} already exists in pending requests",
                message_id
            );
            return;
        }

        state.pending_requests.insert(message_id, tx);
    }

    async fn wait_until<P: Fn(&DevToolsState) -> bool>(&self, predicate: P) {
        loop {
            {
                let state = self.inner.lock().expect("Browser state lock poisoned");
                if predicate(&state) {
                    return;
                }
            }
            self.notify.notified().await;
        }
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

        let params = DictionaryValue::new();
        _ = params.set_bool("enabled", true);
        _ = host.execute_dev_tools_method(1, "Page.setLifecycleEventsEnabled", Some(params));

        Self {
            browser,
            state,
            registration,
            counter: AtomicI32::new(0),
        }
    }

    pub async fn wait_until_loaded(&self, time_to_wait: Duration) {
        let result = tokio::time::timeout(
            time_to_wait,
            self.state
                .wait_until(|s| s.load_fired && s.network_idle_fired),
        )
        .await;

        if result.is_err() {
            info!(
                "Timeout while waiting for page to load ({} sec)",
                time_to_wait.as_secs()
            );
        }
    }

    pub async fn screenshot(&self) -> Result<String> {
        let id = self.counter.fetch_add(1, Ordering::Relaxed);
        let (tx, rx) = oneshot::channel();
        self.state.subscribe(id, tx);

        self.browser.get_host().unwrap().execute_dev_tools_method(
            id,
            "Page.captureScreenshot",
            None,
        )?;

        let devtools_response = rx.await?;
        let screenshot = serde_json::from_slice::<Screenshot>(&devtools_response.data)?;
        Ok(screenshot.data)
    }
}

#[derive(Deserialize, Serialize)]
struct LifecycleEvent {
    name: String,
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
    fn on_dev_tools_message(&mut self, _: Browser, _: &[u8]) -> bool {
        false
    }

    fn on_dev_tools_method_result(
        &mut self,
        _: Browser,
        message_id: i32,
        success: bool,
        result: &[u8],
    ) {
        self.state.on_result(message_id, success, result.to_vec());
    }

    fn on_dev_tools_event(&mut self, _: Browser, event: &str, params: &[u8]) {
        if event == "Page.lifecycleEvent" {
            let params: LifecycleEvent = serde_json::from_slice(params)
                .expect("failed to parse params of Page.lifecycleEvent");

            match params.name.as_str() {
                "init" => self
                    .state
                    .on_event(Event::PageLifecycleEvent(LifecycleEventType::Init)),
                "load" => self
                    .state
                    .on_event(Event::PageLifecycleEvent(LifecycleEventType::Load)),
                "networkIdle" => self
                    .state
                    .on_event(Event::PageLifecycleEvent(LifecycleEventType::NetworkIdle)),
                "networkAlmostIdle" => self.state.on_event(Event::PageLifecycleEvent(
                    LifecycleEventType::NetworkAlmostIdle,
                )),
                _ => {}
            }
        }
    }

    fn on_dev_tools_agent_attached(&mut self, _: Browser) {}

    fn on_dev_tools_agent_detached(&mut self, _: Browser) {}
}
