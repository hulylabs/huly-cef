use std::sync::{Arc, Mutex};

use cef_ui::{Browser, DevToolsMessageObserver, DevToolsMessageObserverCallbacks, Registration};

#[derive(Default)]
struct DevToolsState {
    page_lifecycle_event: String,
}

#[derive(Default, Clone)]
struct SharedDevToolsState(Arc<Mutex<DevToolsState>>);

impl SharedDevToolsState {
    pub fn notify_event(&self, event: &str, params: &[u8]) {
        if event == "Page.lifecycleEvent" {
            let params: serde_json::Value = serde_json::from_slice(params)
                .expect("failed to parse params of Page.lifecycleEvent");

            dbg!("Page.lifecycleEvent: {:?}", &params);
            let mut state = self.0.lock().unwrap();
            state.page_lifecycle_event = params
                .get("name")
                .expect("missing 'name' in Page.lifecycleEvent params")
                .as_str()
                .expect("name is not a string")
                .to_string();
        }
    }
}

pub struct DevTools {
    // TODO: Check if this references destroyed as expected.
    browser: Browser,
    state: SharedDevToolsState,
    registration: Registration,
    counter: i32,
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

        Self {
            browser,
            state,
            registration,
            counter: 1,
        }
    }

    pub fn wait_until_loaded(&self) {}
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

    fn on_dev_tools_method_result(&mut self, _: Browser, _: i32, _: bool, _: &[u8]) {}

    fn on_dev_tools_event(&mut self, _: Browser, method: &str, params: &[u8]) {
        self.state.notify_event(method, params);
    }

    fn on_dev_tools_agent_attached(&mut self, _: Browser) {}

    fn on_dev_tools_agent_detached(&mut self, _: Browser) {}
}
