use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use cef_ui::{
    Browser, DevToolsMessageObserver, DevToolsMessageObserverCallbacks, DictionaryValue,
    Registration,
};
use log::info;
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;

#[derive(Debug, strum::Display)]
enum Event {
    LoadEventFired,
    LifecycleEvent(String),
}

#[derive(Default)]
struct DevToolsState {
    subscribers: HashMap<String, oneshot::Sender<Event>>,

    load_event_fired: bool,
}

#[derive(Default, Clone)]
struct SharedDevToolsState(Arc<Mutex<DevToolsState>>);

impl SharedDevToolsState {
    pub fn on_event(&self, event: Event) {
        self.update_state(&event);
        self.send_event(event);
    }

    fn update_state(&self, event: &Event) {
        let mut state = self.0.lock().unwrap();
        match event {
            Event::LoadEventFired => {
                info!("Page.loadEventFired");
                state.load_event_fired = true;
            }
            Event::LifecycleEvent(name) => {
                if name == "init" {
                    state.load_event_fired = false;
                }
                info!("Page.lifecycleEvent: {}", name);
            }
        }
    }

    fn send_event(&self, event: Event) {
        let mut state = self.0.lock().unwrap();
        if let Some(subscriber) = state.subscribers.remove(&event.to_string()) {
            let _ = subscriber.send(event);
        }
    }

    pub fn subscribe(&self, event_type: String) -> oneshot::Receiver<Event> {
        let mut state = self.0.lock().unwrap();
        let (tx, rx) = oneshot::channel();

        if state.load_event_fired {
            info!("event {} already fired, sending immediately", event_type);
            tx.send(Event::LoadEventFired).unwrap();
        } else {
            info!("subscribing to event: {}", event_type);
            state.subscribers.insert(event_type, tx);
        }
        rx
    }
}

pub struct DevTools {
    #[allow(unused)]
    browser: Browser,
    state: SharedDevToolsState,
    #[allow(unused)]
    registration: Registration,
}

impl Clone for DevTools {
    fn clone(&self) -> Self {
        DevTools {
            browser: self.browser.clone(),
            state: self.state.clone(),
            registration: self.registration.clone(),
        }
    }
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
            registration,
        }
    }

    pub async fn wait_until_loaded(&self) {
        self.state
            .subscribe(Event::LoadEventFired.to_string())
            .await
            .unwrap();
    }
}

#[derive(Deserialize, Serialize)]
struct LifecycleEvent {
    name: String,
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

    fn on_dev_tools_event(&mut self, _: Browser, event: &str, params: &[u8]) {
        if event == "Page.lifecycleEvent" {
            let params: LifecycleEvent = serde_json::from_slice(params)
                .expect("failed to parse params of Page.lifecycleEvent");

            self.state.on_event(Event::LifecycleEvent(params.name));
        }

        if event == "Page.loadEventFired" {
            self.state.on_event(Event::LoadEventFired);
        }
    }

    fn on_dev_tools_agent_attached(&mut self, _: Browser) {}

    fn on_dev_tools_agent_detached(&mut self, _: Browser) {}
}
