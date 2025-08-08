use anyhow::Result;
use tokio::time::error::Elapsed;

use std::collections::HashMap;
use std::sync::atomic::Ordering;
use std::sync::{atomic::AtomicI32, Arc, Mutex};
use std::time::Duration;

use cef_ui::{
    Browser, DevToolsMessageObserver, DevToolsMessageObserverCallbacks, DictionaryValue,
    Registration,
};
use log::info;
use serde::Deserialize;
use tokio::sync::{oneshot, Notify};

#[derive(Debug, Clone)]
pub enum LifecycleEventType {
    Init,
    DOMContentLoaded,
    Load,
    NetworkAlmostIdle,
    NetworkIdle,
    InteractiveTime,
}

pub enum Event {
    PageLifecycleEvent {
        frame_id: String,
        loader_id: String,
        name: LifecycleEventType,
    },
    PageFrameNavigated {
        frame_id: String,
        is_main: bool,
        url: String,
    },
}

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
    frame_events: HashMap<String, Vec<LifecycleEventType>>,
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
            Event::PageLifecycleEvent { name, frame_id, .. } => {
                if state.frame_events.contains_key(frame_id) {
                    state.frame_events.entry(frame_id.clone()).and_modify(|e| {
                        e.push(name.clone());
                    });
                } else {
                    info!(
                        "                               state doesn't have an entry for frame_id: {}",
                        frame_id
                    );
                    if matches!(name, LifecycleEventType::Init) {
                        info!(
                        "                               init event received. Creating new entry for frame_id: {}",
                            frame_id
                        );
                        state
                            .frame_events
                            .insert(frame_id.clone(), vec![name.clone()]);
                    }
                }
            }

            Event::PageFrameNavigated { .. } => {}
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

    async fn wait_until<P: Fn(&DevToolsState) -> bool>(
        &self,
        predicate: P,
        timeout: Duration,
    ) -> Result<(), Elapsed> {
        tokio::time::timeout(timeout, async {
            loop {
                {
                    let state = self.inner.lock().expect("Browser state lock poisoned");
                    if predicate(&state) {
                        return;
                    }
                }
                self.notify.notified().await;
            }
        })
        .await
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
            counter: AtomicI32::new(10),
        }
    }

    pub fn start_navigation(&self) {
        self.state.inner.lock().unwrap().frame_events.clear();
    }

    pub async fn get_main_frame_id(&self) {
        let resp = self.execute_method("Page.getFrameTree", None).await;
        let frame_tree: serde_json::Value =
            serde_json::from_slice(&resp.data).expect("failed to parse Page.getFrameTree response");

        info!(
            "Frame tree: {}",
            serde_json::to_string_pretty(&frame_tree).unwrap()
        );
    }

    pub fn get_frame_events(&self) {
        info!("==============================");
        {
            let state = self.state.inner.lock().unwrap();

            for entry in state.frame_events.iter() {
                info!("Frame ID: {}", entry.0);
                for event in entry.1 {
                    info!("     Event: {:?}", event);
                }
            }
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

        self.browser
            .get_host()
            .unwrap()
            .execute_dev_tools_method(id, name, params);

        rx.await.unwrap()
    }
}

#[derive(Deserialize)]
struct FrameNavigated {
    frame: Frame,
}

#[derive(Deserialize)]
struct Frame {
    id: String,
    #[serde(rename = "parentId")]
    parent_id: Option<String>,
    url: String,
}

#[derive(Deserialize)]
struct LifecycleEvent {
    name: String,
    #[serde(rename = "frameId")]
    frame_id: String,
    #[serde(rename = "loaderId")]
    loader_id: String,
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

    fn on_dev_tools_event(&mut self, _: Browser, event: &str, params: &[u8]) {
        if event == "Page.lifecycleEvent" {
            let params: LifecycleEvent = serde_json::from_slice(params)
                .expect("failed to parse params of Page.lifecycleEvent");

            info!(
                "                               Lifecycle event: {} for frame: {}",
                params.name, params.frame_id
            );

            let name = match params.name.as_str() {
                "init" => LifecycleEventType::Init,
                "DOMContentLoaded" => LifecycleEventType::DOMContentLoaded,
                "load" => LifecycleEventType::Load,
                "networkIdle" => LifecycleEventType::NetworkIdle,
                "networkAlmostIdle" => LifecycleEventType::NetworkAlmostIdle,
                "InteractiveTime" => LifecycleEventType::InteractiveTime,
                _ => {
                    return;
                }
            };
            let frame_id = params.frame_id;
            let loader_id = params.loader_id;

            self.state.on_event(Event::PageLifecycleEvent {
                name,
                frame_id,
                loader_id,
            });
        }

        if event == "Page.frameNavigated" {
            let params = serde_json::from_slice::<FrameNavigated>(params)
                .expect("failed to parse params of Page.frameNavigated");

            if params.frame.parent_id.is_none() {
                info!(
                    "                               Main frame navigated: id={}, url={}",
                    params.frame.id, params.frame.url
                );

                self.state.on_event(Event::PageFrameNavigated {
                    frame_id: params.frame.id,
                    is_main: true,
                    url: params.frame.url,
                });
            }
        }
    }
}
