use std::collections::HashMap;

use crossbeam_channel::Sender;

use cef_ui::{
    BrowserHost, BrowserSettings, CefTask, CefTaskCallbacks, PaintElementType, ThreadId, WindowInfo,
};

use log::info;
use tokio::sync::mpsc::UnboundedSender;

use crate::{browser::state::SharedBrowserState, ClickableElement, LoadState, TabMessage};

mod automation;
mod client;
mod devtools;
mod keyboard;
mod mouse;
pub(crate) mod state;

// TODO: add sub structs:
// 1. Navigation
// 2. Graphics
// TODO: make counter atomic
pub struct Browser {
    inner: cef_ui::Browser,
    pub state: state::SharedBrowserState,
    pub mouse: mouse::Mouse,
    pub keyboard: keyboard::Keyboard,
    pub automation: automation::Automation,
    counter: i32,
}

impl Clone for Browser {
    fn clone(&self) -> Self {
        Browser {
            inner: self.inner.clone(),
            state: self.state.clone(),
            mouse: self.mouse.clone(),
            keyboard: self.keyboard.clone(),
            automation: self.automation.clone(),
            counter: self.counter,
        }
    }
}

impl Browser {
    pub fn new(width: u32, height: u32, url: &str) -> Self {
        let (tx, rx) = crossbeam_channel::bounded(1);
        let result = cef_ui::post_task(
            ThreadId::UI,
            CefTask::new(CreateBrowserTaskCallback {
                tx,
                width,
                height,
                url: url.to_string(),
            }),
        );

        if !result {
            panic!("failed to create a browser in the UI thread");
        }

        rx.recv().expect("failed to receive created browser")
    }

    pub fn start_video(&self) {
        self.state.update(|state| {
            state.active = true;
        });

        _ = self.inner.get_host().unwrap().was_hidden(false);
        _ = self.inner.get_host().unwrap().set_focus(true);

        _ = self
            .inner
            .get_host()
            .unwrap()
            .invalidate(PaintElementType::View);
    }

    pub fn stop_video(&self) {
        self.state.update(|state| {
            state.active = false;
        });

        _ = self.inner.get_host().unwrap().was_hidden(true);
    }

    pub fn resize(&self, width: u32, height: u32) {
        self.state.update(|state| {
            state.width = width;
            state.height = height;
        });

        let _ = self.inner.get_host().unwrap().was_resized();
        let _ = self
            .inner
            .get_host()
            .unwrap()
            .invalidate(PaintElementType::View);
    }

    pub fn go_to(&mut self, url: &str) {
        info!("navigating to URL: {}", url);
        self.automation.start_navigation();
        let _ = self.inner.get_main_frame().unwrap().unwrap().load_url(url);
    }

    pub fn go_back(&mut self) {
        self.automation.start_navigation();
        let _ = self.inner.go_back();
    }

    pub fn go_forward(&mut self) {
        self.automation.start_navigation();
        let _ = self.inner.go_forward();
    }

    pub fn reload(&mut self) {
        self.automation.start_navigation();
        let _ = self.inner.reload();
    }

    pub fn close(&self) {
        let _ = self.inner.get_host().unwrap().close_browser(true);
    }

    pub fn get_id(&self) -> i32 {
        self.inner.get_identifier().unwrap()
    }

    pub fn set_focus(&self, focus: bool) {
        let _ = self.inner.get_host().unwrap().set_focus(focus);
    }

    pub fn get_title(&self) -> String {
        self.state.read(|state| state.title.clone())
    }

    pub fn get_url(&self) -> String {
        self.state.read(|state| state.url.clone())
    }

    pub fn get_size(&self) -> (u32, u32) {
        self.state.read(|state| (state.width, state.height))
    }

    pub fn get_load_state(&self) -> LoadState {
        self.state.read(|state| state.load_state.clone())
    }

    pub fn subscribe(&mut self, tx: UnboundedSender<TabMessage>) -> i32 {
        let id = self.counter;
        self.counter += 1;
        self.state.subscribe(id, tx);

        id
    }

    pub fn unsubscribe(&self, id: i32) {
        self.state.unsubscribe(id);
    }
}

struct CreateBrowserTaskCallback {
    tx: Sender<Browser>,
    width: u32,
    height: u32,
    url: String,
}

impl CefTaskCallbacks for CreateBrowserTaskCallback {
    fn execute(&mut self) {
        let window_info = WindowInfo::new().windowless_rendering_enabled(true);
        let settings = BrowserSettings::new().windowless_frame_rate(60);
        let state = SharedBrowserState::new(state::BrowserState {
            title: "".to_string(),
            url: self.url.clone(),
            favicon: None,
            load_state: LoadState::default(),
            cursor: "Pointer".to_string(),
            width: self.width,
            height: self.height,
            active: true,
            left_mouse_button_down: false,

            clickable_elements: None,

            javascript_messages: HashMap::new(),
            subscribers: HashMap::new(),
            single_event_subscribers: HashMap::new(),
        });

        let client = client::new(state.clone());
        let inner = BrowserHost::create_browser_sync(
            &window_info,
            client,
            &self.url,
            &settings,
            None,
            None,
        );

        let mouse = mouse::Mouse::new(inner.clone(), state.clone());
        let keyboard = keyboard::Keyboard::new(inner.clone(), state.clone());
        let automation = automation::Automation::new(inner.clone(), state.clone(), mouse.clone());

        self.tx
            .send(Browser {
                inner,
                state,
                mouse,
                keyboard,
                automation,
                counter: 0,
            })
            .expect("failed to send created browser");
    }
}
