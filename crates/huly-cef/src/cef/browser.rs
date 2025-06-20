use anyhow::Result;
use log::error;

use std::sync::{Arc, Mutex};

use cef_ui::{
    BrowserHost, BrowserSettings, CefTask, CefTaskCallbacks, EventFlags, KeyEvent, KeyEventType,
    MouseButtonType, MouseEvent, PaintElementType, ProcessMessage, StringVisitor,
    StringVisitorCallbacks, ThreadId, WindowInfo,
};
use crossbeam_channel::Sender;
use tokio::sync::{mpsc::UnboundedSender, oneshot};

use super::{
    client,
    messages::{LoadStatus, MouseType, TabMessage},
};

/// Maintains the state of a browser instance.
pub struct BrowserState {
    pub last_frame: Option<Vec<u8>>,
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

    pub get_center_oneshot_channel: Option<oneshot::Sender<Result<(i32, i32)>>>,
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

    pub async fn get_element_center(&self, selector: &str) -> Result<(i32, i32)> {
        let (tx, rx) = oneshot::channel::<Result<(i32, i32)>>();

        {
            let mut state = self.state.lock().unwrap();
            state.get_center_oneshot_channel = Some(tx);
        }

        let message = ProcessMessage::new("getElementCenter");
        _ = message
            .get_argument_list()
            .unwrap()
            .unwrap()
            .set_string(0, selector);

        _ = self
            .inner
            .get_main_frame()
            .unwrap()
            .unwrap()
            .send_process_message(cef_ui::ProcessId::Renderer, message);

        rx.await.unwrap()
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

    pub fn set_text(&self, selector: &str, text: &str) {
        let code = format!("document.querySelector('{}').value = '{}';", selector, text);
        log::info!("Executing JavaScript to set text: {}", code);
        _ = self
            .inner
            .get_main_frame()
            .unwrap()
            .unwrap()
            .execute_java_script(&code, "", 0);
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
            last_frame: None,
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
            get_center_oneshot_channel: None,
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
