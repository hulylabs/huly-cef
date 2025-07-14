use std::sync::{Arc, Mutex};

use cef_ui::{
    Browser, DevToolsMessageObserver, DevToolsMessageObserverCallbacks, DictionaryValue,
    Registration,
};
use log::info;
use serde::{Deserialize, Serialize};
use tokio::sync::Notify;

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

#[derive(Default)]
struct DevToolsState {
    load_fired: bool,
    network_idle_fired: bool,
}

#[derive(Default)]
struct SharedDevToolsState {
    inner: Mutex<DevToolsState>,
    notify: Notify,
}

impl SharedDevToolsState {
    pub fn on_event(&self, event: Event) {
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

    pub async fn wait_until<P: Fn(&DevToolsState) -> bool>(&self, predicate: P) {
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
        }
    }

    pub async fn wait_until_loaded(&self) {
        self.state
            .wait_until(|s| s.load_fired && s.network_idle_fired)
            .await;
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
        _message_id: i32,
        _success: bool,
        _result: &[u8],
    ) {
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
