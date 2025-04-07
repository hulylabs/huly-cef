use std::sync::{Arc, Mutex};

use cef_ui::{
    BrowserHost, BrowserSettings, CefTask, CefTaskCallbacks, EventFlags, KeyEvent, KeyEventType,
    MouseButtonType, MouseEvent, PaintElementType, ThreadId, WindowInfo,
};
use crossbeam_channel::Sender;
use tokio::sync::mpsc::UnboundedSender;

use super::{
    client,
    messages::{CefMessage, MouseType},
};

/// Maintains the state of a browser instance.
pub struct BrowserState {
    /// The width of the browser in pixels.
    pub width: u32,
    /// The height of the browser in pixels.
    pub height: u32,
    /// Whether the browser is active or not.
    pub active: bool,

    pub left_mouse_button_down: bool,
}

pub struct Browser {
    pub inner: cef_ui::Browser,
    pub state: Arc<Mutex<BrowserState>>,
}

impl Browser {
    pub fn new(width: u32, height: u32, sender: UnboundedSender<CefMessage>) -> Self {
        create_browser(width, height, "", sender)
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

    pub fn key_press(&self, character: u16, code: i32, down: bool, ctrl: bool, shift: bool) {
        let event_type = if down {
            KeyEventType::KeyDown
        } else {
            KeyEventType::KeyUp
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
            windows_key_code: code.into(),
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

    pub fn get_url(&self) -> String {
        self.inner
            .get_main_frame()
            .unwrap()
            .unwrap()
            .get_url()
            .unwrap_or_default()
    }

    pub fn get_id(&self) -> i32 {
        self.inner
            .get_identifier()
            .expect("failed to get browser ID")
    }
}

struct CreateBrowserTaskCallback {
    tx: Sender<Browser>,
    width: u32,
    height: u32,
    url: String,
    sender: UnboundedSender<CefMessage>,
}

impl CefTaskCallbacks for CreateBrowserTaskCallback {
    /// Executes the task to create a browser and send it through the channel.
    fn execute(&mut self) {
        let window_info = WindowInfo::new().windowless_rendering_enabled(true);
        let settings = BrowserSettings::new();
        let state = Arc::new(Mutex::new(BrowserState {
            width: self.width,
            height: self.height,
            active: true,
            left_mouse_button_down: false,
        }));
        let client = client::new(state.clone(), self.sender.clone());
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
pub fn create_browser(
    width: u32,
    height: u32,
    url: &str,
    sender: UnboundedSender<CefMessage>,
) -> Browser {
    let (tx, rx) = crossbeam_channel::unbounded::<Browser>();
    let result = cef_ui::post_task(
        ThreadId::UI,
        CefTask::new(CreateBrowserTaskCallback {
            tx,
            width,
            height,
            url: url.to_string(),
            sender,
        }),
    );
    if !result {
        panic!("failed to create a browser in the UI thread");
    }

    rx.recv()
        .expect("failed to receive a CEF browser, created in the UI thread")
}
