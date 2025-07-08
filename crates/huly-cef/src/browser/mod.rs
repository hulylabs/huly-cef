use std::collections::HashMap;

use anyhow::Result;
use crossbeam_channel::Sender;
use log::error;
use serde::{Deserialize, Serialize};

use cef_ui::{
    BrowserHost, BrowserSettings, CefTask, CefTaskCallbacks, EventFlags, KeyEvent, KeyEventType,
    MouseButtonType, MouseEvent, PaintElementType, StringVisitor, ThreadId, WindowInfo,
};

use tokio::sync::{mpsc::UnboundedSender, oneshot};

use crate::{
    browser::state::SharedBrowserState, javascript::GET_CLICKABLE_ELEMENTS_SCRIPT,
    messages::LoadStatus, messages::MouseButton, ClickableElement, TabMessage,
};

mod client;
mod dom;
pub(crate) mod state;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JSMessage {
    ClickableElements(Vec<ClickableElement>),
    ElementCenter { x: i32, y: i32 },
    Clicked(bool),
}

pub struct Browser {
    inner: cef_ui::Browser,
    pub state: state::SharedBrowserState,
    counter: i32,
}

impl Clone for Browser {
    fn clone(&self) -> Self {
        Browser {
            inner: self.inner.clone(),
            state: self.state.clone(),
            counter: self.counter,
        }
    }
}

impl Browser {
    pub fn new(width: u32, height: u32, url: &str) -> Self {
        create_browser(width, height, url)
    }

    pub fn mouse_move(&self, x: i32, y: i32) {
        let mut modifiers = EventFlags::empty();
        if self.state.read(|s| s.left_mouse_button_down) {
            modifiers.insert(EventFlags::LeftMouseButton);
        }

        let event = MouseEvent { x, y, modifiers };

        self.inner
            .get_host()
            .unwrap()
            .send_mouse_move_event(&event, false)
            .expect("failed to send mouse move event");
    }

    pub fn mouse_click(&self, x: i32, y: i32, button: MouseButton, down: bool) {
        if button == MouseButton::Left {
            self.state.update(|state| {
                state.left_mouse_button_down = down;
            });
        }

        let event = MouseEvent {
            x,
            y,
            modifiers: EventFlags::empty(),
        };

        let button = match button {
            MouseButton::Left => MouseButtonType::Left,
            MouseButton::Middle => MouseButtonType::Middle,
            MouseButton::Right => MouseButtonType::Right,
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
            .get_source(StringVisitor::new(dom::DOMVisitor::new(tx)));

        rx.await.unwrap()
    }

    pub async fn screenshot(&self, width: u32, height: u32) -> Vec<u8> {
        let (tx, rx) = oneshot::channel::<Vec<u8>>();
        self.state.update(|state| {
            state.screenshot_width = width;
            state.screenshot_height = height;
            state.screenshot_channel = Some(tx);
        });

        let host = self.inner.get_host().unwrap();
        _ = host.was_resized();
        _ = host.invalidate(PaintElementType::View);
        rx.await.unwrap()
    }

    pub async fn get_clickable_elements(&self) -> Vec<ClickableElement> {
        let msg = self
            .execute_javascript(GET_CLICKABLE_ELEMENTS_SCRIPT, "elements")
            .await;

        match msg {
            Ok(JSMessage::ClickableElements(elements)) => {
                self.state.update(|state| {
                    state.clickable_elements = Some(elements.clone());
                });
                return elements;
            }
            _ => {
                error!("Unexpected JavaScript message body: {:?}", msg);
            }
        }
        vec![]
    }

    // pub async fn wait_until_loaded(&self) -> LoadStatus {
    //     let current_status = self.state.read(|state| state.load_status.clone());
    //     if current_status != LoadStatus::Loading {
    //         return current_status;
    //     }

    //     let (tx, mut rx) = mpsc::unbounded_channel::<TabMessage>();
    //     self.state.update(|state| {
    //         state
    //             .tab_events_subscribers
    //             .insert(TabEventType::LoadStateChanged, tx);
    //     });

    //     // TODO: use timeout here to avoid running indefinitely
    //     while let Some(message) = rx.recv().await {
    //         if let TabMessage::LoadStateChanged { status, .. } = message {
    //             if status != LoadStatus::Loading {
    //                 return status;
    //             }
    //         }
    //     }

    //     LoadStatus::Loading
    // }

    pub async fn click_element(&self, id: i32) {
        let element = self.state.read(|state| {
            state
                .clickable_elements
                .as_ref()
                .and_then(|elements| elements.get(id as usize).cloned())
        });

        if let Some(e) = element {
            let id = e.id;

            let mut script = format!(
                r#"
                let center = {{ }};
                let element = document.querySelector('[data-clickable-id="{id}"]');
                if (element) {{
                    const rect = element.getBoundingClientRect();
                    const x = Math.floor(rect.left + rect.width / 2);
                    const y = Math.floor(rect.top + rect.height / 2);     
                    center = {{ x, y }}

                    element.onclick = function(event) {{
                        element.setAttribute('data-clicked', 'true');
                    }};
                }}
                "#
            );

            let msg = self.execute_javascript(&script, "center").await;

            if let Ok(JSMessage::ElementCenter { x, y }) = msg {
                self.mouse_click(x, y, MouseButton::Left, true);
                std::thread::sleep(std::time::Duration::from_millis(20));
                self.mouse_click(x, y, MouseButton::Left, false);

                script = format!(
                    r#"
                    let clicked = false;
                    let element = document.querySelector('[data-clickable-id="{id}"][data-clicked="true"');
                    if (element) {{
                        clicked = true;
                        element.removeAttribute('data-clicked');
                    }}
                    "#
                );

                let msg = self.execute_javascript(&script, "clicked").await;
                if let Ok(JSMessage::Clicked(clicked)) = msg {
                    if !clicked {
                        self.mouse_click(x, y, MouseButton::Left, true);
                        std::thread::sleep(std::time::Duration::from_millis(20));
                        self.mouse_click(x, y, MouseButton::Left, false);
                    }
                }
            } else {
                error!("unexpected JavaScript message body: {:?}", msg);
            }
        }
    }

    async fn execute_javascript(&self, script: &str, value_to_return: &str) -> Result<JSMessage> {
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

        let (tx, rx) = oneshot::channel::<Result<JSMessage>>();
        self.state.update(|state| {
            state.javascript_messages.insert(id.to_string(), tx);
        });
        rx.await?
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

            clickable_elements: None,

            screenshot_width: 0,
            screenshot_height: 0,
            screenshot_channel: None,

            javascript_messages: HashMap::new(),

            subscribers: HashMap::new(),
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

        self.tx
            .send(Browser {
                inner,
                state: state.clone(),
                counter: 0,
            })
            .expect("failed to send created browser");
    }
}

fn create_browser(width: u32, height: u32, url: &str) -> Browser {
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
