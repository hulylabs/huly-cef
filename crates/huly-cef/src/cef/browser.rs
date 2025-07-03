use anyhow::Result;
use log::error;
use serde::{Deserialize, Serialize};

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use cef_ui::{
    BrowserHost, BrowserSettings, CefTask, CefTaskCallbacks, EventFlags, KeyEvent, KeyEventType,
    MouseButtonType, MouseEvent, PaintElementType, StringVisitor, StringVisitorCallbacks, ThreadId,
    WindowInfo,
};
use crossbeam_channel::Sender;
use tokio::sync::{
    mpsc::{self, UnboundedSender},
    oneshot,
};

use crate::javascript::GET_CLICKABLE_ELEMENTS_SCRIPT;

use super::{
    client,
    messages::{LoadStatus, MouseType, TabEventType, TabMessage},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClickableElement {
    pub id: i32,
    pub tag: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JavaScriptMessage {
    ClickableElements(Vec<ClickableElement>),
    ElementCenter { x: i32, y: i32 },
}

/// Maintains the state of a browser instance.
pub struct BrowserState {
    pub title: String,
    pub url: String,
    pub favicon: Option<String>,
    pub load_status: LoadStatus,
    pub can_go_back: bool,
    pub can_go_forward: bool,
    pub error_code: i32,
    pub error_text: String,
    pub cursor: String,
    pub width: u32,
    pub height: u32,
    pub active: bool,
    pub left_mouse_button_down: bool,

    pub clickable_elements: Option<Vec<ClickableElement>>,

    pub screenshot_width: u32,
    pub screenshot_height: u32,
    pub screenshot_channel: Option<oneshot::Sender<Vec<u8>>>,

    pub tab_events_subscribers: HashMap<TabEventType, UnboundedSender<TabMessage>>,
    pub javascript_messages: HashMap<String, oneshot::Sender<Result<JavaScriptMessage>>>,
}

pub struct Browser {
    inner: cef_ui::Browser,
    pub state: Arc<Mutex<BrowserState>>,
}

impl Clone for Browser {
    fn clone(&self) -> Self {
        Browser {
            inner: self.inner.clone(),
            state: Arc::clone(&self.state),
        }
    }
}

impl Browser {
    pub fn new(width: u32, height: u32, url: &str, sender: UnboundedSender<TabMessage>) -> Self {
        create_browser(width, height, url, sender)
    }

    pub fn mouse_move(&self, x: i32, y: i32) {
        let mut modifiers = EventFlags::empty();
        if self.state.lock().unwrap().left_mouse_button_down {
            modifiers.insert(EventFlags::LeftMouseButton);
        }

        let event = MouseEvent { x, y, modifiers };

        self.inner
            .get_host()
            .unwrap()
            .send_mouse_move_event(&event, false)
            .expect("failed to send mouse move event");
    }

    pub fn mouse_click(&self, x: i32, y: i32, button: MouseType, down: bool) {
        if button == MouseType::Left {
            let mut state = self.state.lock().unwrap();
            state.left_mouse_button_down = down;
        }

        let event = MouseEvent {
            x,
            y,
            modifiers: EventFlags::empty(),
        };

        let button = match button {
            MouseType::Left => MouseButtonType::Left,
            MouseType::Middle => MouseButtonType::Middle,
            MouseType::Right => MouseButtonType::Right,
        };

        self.inner
            .get_host()
            .unwrap()
            .send_mouse_click_event(&event, button, !down, 1)
            .expect("failed to send mouse click event");
    }

    pub fn mouse_wheel(&self, x: i32, y: i32, dx: i32, dy: i32) {
        let event = MouseEvent {
            x,
            y,
            modifiers: EventFlags::empty(),
        };
        self.inner
            .get_host()
            .unwrap()
            .send_mouse_wheel_event(&event, dx, -dy)
            .expect("failed to send mouse wheel event");
    }

    pub fn key_press(
        &self,
        character: u16,
        windowscode: i32,
        code: i32,
        down: bool,
        ctrl: bool,
        shift: bool,
    ) {
        let mut event_type = KeyEventType::KeyUp;
        if down {
            event_type = KeyEventType::KeyDown;
        };

        let mut modifiers = EventFlags::empty();
        if ctrl {
            modifiers = modifiers.union(EventFlags::ControlDown);
        }
        if shift {
            modifiers = modifiers.union(EventFlags::ShiftDown);
        }
        let mut event = KeyEvent {
            event_type,
            modifiers,
            windows_key_code: windowscode.into(),
            native_key_code: code,
            is_system_key: false,
            character,
            unmodified_character: character,
            focus_on_editable_field: false,
        };

        _ = self.inner.get_host().unwrap().send_key_event(event.clone());

        if event_type == KeyEventType::KeyDown && character != 0 {
            event.event_type = KeyEventType::Char;
            _ = self.inner.get_host().unwrap().send_key_event(event);
        }
    }

    pub fn char(&self, character: u16) {
        let event = KeyEvent {
            event_type: KeyEventType::Char,
            modifiers: EventFlags::empty(),
            windows_key_code: 0.into(),
            native_key_code: 0,
            is_system_key: false,
            character,
            unmodified_character: character,
            focus_on_editable_field: false,
        };

        _ = self.inner.get_host().unwrap().send_key_event(event);
    }

    pub fn start_video(&self) {
        let mut state = self.state.lock().unwrap();
        state.active = true;

        _ = self.inner.get_host().unwrap().was_hidden(false);
        _ = self.inner.get_host().unwrap().set_focus(true);

        _ = self
            .inner
            .get_host()
            .unwrap()
            .invalidate(PaintElementType::View);
    }

    pub fn stop_video(&self) {
        let mut state = self.state.lock().unwrap();
        state.active = false;

        _ = self.inner.get_host().unwrap().was_hidden(true);
    }

    pub fn resize(&self, width: u32, height: u32) {
        let mut state = self.state.lock().unwrap();
        state.width = width;
        state.height = height;

        let _ = self.inner.get_host().unwrap().was_resized();
        let _ = self
            .inner
            .get_host()
            .unwrap()
            .invalidate(PaintElementType::View);
    }

    pub fn go_to(&self, url: &str) {
        let _ = self.inner.get_main_frame().unwrap().unwrap().load_url(url);
    }

    pub fn go_back(&self) {
        let _ = self.inner.go_back();
    }

    pub fn go_forward(&self) {
        let _ = self.inner.go_forward();
    }

    pub fn reload(&self) {
        let _ = self.inner.reload();
    }

    pub fn close(&self) {
        let _ = self.inner.get_host().unwrap().close_browser(true);
    }

    pub fn get_id(&self) -> i32 {
        self.inner
            .get_identifier()
            .expect("failed to get browser ID")
    }

    pub fn set_focus(&self, focus: bool) {
        let _ = self.inner.get_host().unwrap().set_focus(focus);
    }

    pub async fn get_dom(&self) -> String {
        let (tx, rx) = oneshot::channel::<String>();
        _ = self
            .inner
            .get_main_frame()
            .unwrap()
            .unwrap()
            .get_source(StringVisitor::new(DOMVisitor::new(tx)));

        rx.await.unwrap()
    }

    pub async fn screenshot(&self, width: u32, height: u32) -> Vec<u8> {
        let (tx, rx) = oneshot::channel::<Vec<u8>>();

        {
            let mut state = self.state.lock().unwrap();
            state.screenshot_width = width;
            state.screenshot_height = height;
            state.screenshot_channel = Some(tx);
        }

        _ = self.inner.get_host().unwrap().was_resized();
        rx.await.unwrap()
    }

    pub async fn get_clickable_elements(&self) -> Vec<ClickableElement> {
        let rx = self.execute_javascript(GET_CLICKABLE_ELEMENTS_SCRIPT, "response");
        let msg = rx.await.unwrap();

        if let Ok(msg) = msg {
            match msg {
                JavaScriptMessage::ClickableElements(elements) => {
                    let mut state = self.state.lock().unwrap();
                    state.clickable_elements = Some(elements.clone());
                    return elements;
                }
                _ => {
                    error!("Unexpected JavaScript message body: {:?}", msg);
                }
            }
        }
        vec![]
    }

    pub async fn wait_until_loaded(&self) -> LoadStatus {
        {
            let state = self.state.lock().unwrap();
            if state.load_status != LoadStatus::Loading {
                return state.load_status.clone();
            }
        }

        let (tx, mut rx) = mpsc::unbounded_channel::<TabMessage>();
        {
            let mut state = self.state.lock().unwrap();
            state
                .tab_events_subscribers
                .insert(TabEventType::LoadStateChanged, tx);
        }

        // TODO: use timeout here to avoid running indefinitely
        while let Some(message) = rx.recv().await {
            if let TabMessage::LoadStateChanged { status, .. } = message {
                if status != LoadStatus::Loading {
                    return status;
                }
            }
        }

        LoadStatus::Loading
    }

    pub async fn click_element(&self, id: i32) {
        let element = {
            let state = self.state.lock().unwrap();
            state
                .clickable_elements
                .as_ref()
                .and_then(|elements| elements.get(id as usize).cloned())
        };

        if let Some(e) = element {
            let id = e.id;

            let script = format!(
                r#"
                let response = {{ }};
                let element = document.querySelector('[data-clickable-id="{id}"]');
                if (element) {{
                    const rect = element.getBoundingClientRect();
                    const x = Math.floor(rect.left + rect.width / 2);
                    const y = Math.floor(rect.top + rect.height / 2);     
                    response = {{ x, y }}
                }}
                "#
            );

            let rx = self.execute_javascript(&script, "response");
            let center = rx.await.unwrap();

            match center {
                Ok(JavaScriptMessage::ElementCenter { x, y }) => {
                    self.mouse_click(x, y, MouseType::Left, true);
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    self.mouse_click(x, y, MouseType::Left, false);
                }
                _ => {
                    error!("unexpected JavaScript message body: {:?}", center);
                }
            }
        }
    }

    fn execute_javascript(
        &self,
        script: &str,
        value_to_return: &str,
    ) -> oneshot::Receiver<Result<JavaScriptMessage>> {
        let id = self.get_id();
        let script = format!(
            r#"{{
                {script}
                sendMessage({{
                    id: "{}",
                    message: JSON.stringify({value_to_return}),
                }});
            }}"#,
            id.clone()
        );

        _ = self
            .inner
            .get_main_frame()
            .unwrap()
            .unwrap()
            .execute_java_script(&script, "", 0);

        let (tx, rx) = oneshot::channel::<Result<JavaScriptMessage>>();
        {
            let mut state = self.state.lock().unwrap();
            state.javascript_messages.insert(id.to_string(), tx);
        }
        rx
    }
}

struct DOMVisitor {
    tx: Option<oneshot::Sender<String>>,
}

impl DOMVisitor {
    pub fn new(tx: oneshot::Sender<String>) -> Self {
        DOMVisitor { tx: Some(tx) }
    }
}

impl StringVisitorCallbacks for DOMVisitor {
    fn visit(&mut self, string: &str) {
        if let Err(e) = self.tx.take().unwrap().send(string.to_string()) {
            error!("failed to get DOM: {}", e);
        }
    }
}

struct CreateBrowserTaskCallback {
    tx: Sender<Browser>,
    width: u32,
    height: u32,
    url: String,
    event_channel: UnboundedSender<TabMessage>,
}

impl CefTaskCallbacks for CreateBrowserTaskCallback {
    /// Executes the task to create a browser and send it through the channel.
    fn execute(&mut self) {
        let window_info = WindowInfo::new().windowless_rendering_enabled(true);
        let settings = BrowserSettings::new().windowless_frame_rate(60);
        let state = Arc::new(Mutex::new(BrowserState {
            title: "".to_string(),
            url: self.url.clone(),
            favicon: None,
            load_status: LoadStatus::Loading,
            can_go_back: false,
            can_go_forward: false,
            error_code: 0,
            error_text: "".to_string(),
            cursor: "Pointer".to_string(),
            width: self.width,
            height: self.height,
            active: true,
            left_mouse_button_down: false,
            tab_events_subscribers: HashMap::new(),

            clickable_elements: None,
            javascript_messages: HashMap::new(),
            screenshot_width: 0,
            screenshot_height: 0,
            screenshot_channel: None,
        }));

        let client = client::new(state.clone(), self.event_channel.clone());
        let inner = BrowserHost::create_browser_sync(
            &window_info,
            client,
            &self.url,
            &settings,
            None,
            None,
        );

        self.tx
            .send(Browser { inner, state })
            .expect("failed to send a browser");
    }
}

/// Creates a new browser instance.
///
/// # Parameters
///
/// - `width`: The width of the browser.
/// - `height`: The height of the browser.
/// - `url`: The URL to load in the browser.
/// - `sender`: A channel for CEF messages.
///
/// # Returns
///
/// A new instance of a CEF browser.
///
/// # Panics
///
/// This function will panic if it fails to create a browser in the UI thread.
fn create_browser(
    width: u32,
    height: u32,
    url: &str,
    event_channel: UnboundedSender<TabMessage>,
) -> Browser {
    // TODO: use oneshot
    let (tx, rx) = crossbeam_channel::unbounded::<Browser>();
    let result = cef_ui::post_task(
        ThreadId::UI,
        CefTask::new(CreateBrowserTaskCallback {
            tx,
            width,
            height,
            url: url.to_string(),
            event_channel,
        }),
    );

    if !result {
        panic!("failed to create a browser in the UI thread");
    }

    rx.recv()
        .expect("failed to receive a CEF browser, created in the UI thread")
}
