use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use cef_ui::{
    Browser, DevToolsMessageObserver, DevToolsMessageObserverCallbacks, DictionaryValue,
    Registration,
};
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;

#[derive(Eq, Hash, PartialEq)]
enum EventType {
    LoadEventFired,
}

enum EventData {
    LoadEventFired,
}

#[derive(Deserialize, Serialize)]
struct LifecycleEvent {
    name: String,
}

#[derive(Default)]
struct DevToolsState {
    load_event_fired: bool,

    subscribers: HashMap<EventType, oneshot::Sender<EventData>>,
}

#[derive(Default, Clone)]
struct SharedDevToolsState(Arc<Mutex<DevToolsState>>);

impl SharedDevToolsState {
    // TODO: we need a way to block this function during adding a subscriber
    pub fn on_event(&self, event: &str, params: &[u8]) {
        let state = self.0.lock().unwrap();
        if event == "Page.lifecycleEvent" {
            let params: LifecycleEvent = serde_json::from_slice(params)
                .expect("failed to parse params of Page.lifecycleEvent");

            if params.name == "init" {
                self.0.lock().unwrap().load_event_fired = false;
            }
        }

        if event == "Page.loadEventFired" {
            self.0.lock().unwrap().load_event_fired = true;
        }
    }

    pub fn subscribe(&self, event_type: EventType) -> oneshot::Receiver<EventData> {
        let (tx, rx) = oneshot::channel();
        self.0.lock().unwrap().subscribers.insert(event_type, tx);
        rx
    }
}

pub struct DevTools {
    browser: Browser,
    state: SharedDevToolsState,
    _registration: Registration,
}

impl DevTools {
    pub fn new(browser: Browser) -> Self {
        let state = SharedDevToolsState::default();
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
            _registration: registration,
        }
    }

    pub async fn wait_until_loaded(&self) {}
}

struct DevToolsObserverCallbacks {
    state: SharedDevToolsState,
}

impl DevToolsObserverCallbacks {
    pub fn new(state: SharedDevToolsState) -> Self {
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
        _message_id: i32,
        _success: bool,
        _result: &[u8],
    ) {
    }

    fn on_dev_tools_event(&mut self, _: Browser, method: &str, params: &[u8]) {
        self.state.on_event(method, params);
    }

    fn on_dev_tools_agent_attached(&mut self, _: Browser) {}

    fn on_dev_tools_agent_detached(&mut self, _: Browser) {}
}
