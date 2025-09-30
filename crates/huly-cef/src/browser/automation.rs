use std::{
    io::Cursor,
    sync::{Arc, Mutex},
    time::Duration,
};

use anyhow::Result;

use base64::{prelude::BASE64_STANDARD, Engine};
use cef_ui::{Browser, StringVisitor, StringVisitorCallbacks};
use image::{imageops::FilterType, ImageFormat};
use log::error;
use serde::{Deserialize, Serialize};
use tokio::sync::{oneshot, Notify};

use crate::{
    browser::{devtools::DevTools, mouse::Mouse},
    state::SharedBrowserState,
    ClickableElement, LoadState, LoadStatus, MouseButton, TabMessage, TabMessageType,
    GET_CLICKABLE_ELEMENTS_SCRIPT,
};

pub struct DOMVisitor {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JSMessage {
    ClickableElements(Vec<ClickableElement>),
    ElementCenter { x: i32, y: i32 },
    Clicked(bool),
}

pub struct Automation {
    browser: Browser,
    devtools: Arc<DevTools>,
    mouse: Mouse,
    state: SharedBrowserState,
    load_states: Arc<Mutex<Vec<LoadState>>>,
    clickable_elements: Arc<Mutex<Option<Vec<ClickableElement>>>>,
    notify: Arc<Notify>,
}

impl Clone for Automation {
    fn clone(&self) -> Self {
        Automation {
            browser: self.browser.clone(),
            devtools: self.devtools.clone(),
            mouse: self.mouse.clone(),
            state: self.state.clone(),
            clickable_elements: self.clickable_elements.clone(),
            load_states: self.load_states.clone(),
            notify: self.notify.clone(),
        }
    }
}

impl Automation {
    pub fn new(browser: Browser, state: SharedBrowserState, mouse: Mouse) -> Self {
        let devtools = Arc::new(DevTools::new(browser.clone()));

        let notify = Arc::new(Notify::new());
        let notify_clone = notify.clone();

        let load_states = Arc::new(Mutex::new(Vec::new()));
        let load_states_clone = load_states.clone();
        state.on(
            TabMessageType::LoadState,
            Box::new(move |message| match message {
                TabMessage::LoadState(load_state) => {
                    load_states_clone.lock().unwrap().push(load_state);
                    notify_clone.notify_waiters();
                }
                _ => {}
            }),
        );

        let clickable_elements = Arc::default();

        Automation {
            browser,
            devtools,
            mouse,
            state,
            load_states,
            clickable_elements,
            notify,
        }
    }

    pub fn start_navigation(&mut self) {
        self.load_states.lock().unwrap().clear();
    }

    pub async fn get_dom(&self) -> String {
        let (tx, rx) = oneshot::channel::<String>();
        _ = self
            .browser
            .get_main_frame()
            .unwrap()
            .unwrap()
            .get_source(StringVisitor::new(DOMVisitor::new(tx)));

        rx.await.unwrap()
    }

    pub async fn screenshot(&self, width: u32, height: u32) -> Result<String> {
        let screenshot = self.devtools.screenshot().await?;
        let screenshot = BASE64_STANDARD.decode(screenshot)?;
        let screenshot = image::load_from_memory(&screenshot)?;
        let screenshot = screenshot.resize_exact(width, height, FilterType::Lanczos3);

        let mut cursor = Cursor::new(Vec::new());
        screenshot.write_to(&mut cursor, ImageFormat::Png)?;

        Ok(BASE64_STANDARD.encode(cursor.into_inner()))
    }

    pub async fn wait_until_loaded(&mut self) -> Result<(), String> {
        let timeout = Duration::from_secs(30);
        _ = tokio::time::timeout(timeout, async {
            loop {
                {
                    let load_states = self.load_states.lock().unwrap();
                    if let Some(last_state) = load_states.last() {
                        if last_state.status == LoadStatus::Loaded {
                            return;
                        }
                    }
                }
                self.notify.notified().await;
            }
        })
        .await;

        let load_state = self.state.read(|state| state.load_state.clone());
        match load_state.status {
            LoadStatus::Loaded => Ok(()),
            LoadStatus::Loading => Err(format!(
                "Page is still loading after {} seconds",
                timeout.as_secs()
            )),
            LoadStatus::LoadError => Err(load_state.error_message),
        }
    }

    pub async fn get_clickable_elements(&self) -> Vec<ClickableElement> {
        let msg = self
            .execute_javascript(GET_CLICKABLE_ELEMENTS_SCRIPT, "elements")
            .await;

        match msg {
            Ok(JSMessage::ClickableElements(elements)) => {
                self.clickable_elements
                    .lock()
                    .unwrap()
                    .replace(elements.clone());
                return elements;
            }
            _ => {
                error!("Unexpected JavaScript message body: {:?}", msg);
            }
        }
        vec![]
    }

    pub async fn click_element(&self, id: i32) {
        let element = self
            .clickable_elements
            .lock()
            .unwrap()
            .as_ref()
            .and_then(|elements| elements.get(id as usize).cloned());
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
                self.mouse.click(x, y, MouseButton::Left, true);
                std::thread::sleep(std::time::Duration::from_millis(20));
                self.mouse.click(x, y, MouseButton::Left, false);
                std::thread::sleep(std::time::Duration::from_millis(1000));

                script = format!(
                    r#"
                    let clicked = false;
                    let element = document.querySelector('[data-clickable-id="{id}"]');
                    if (element && element.hasAttribute('data-clicked')) {{
                        clicked = true;
                        element.removeAttribute('data-clicked');
                    }} else if (!element) {{
                        clicked = true;
                    }}
                "#
                );

                let msg = self.execute_javascript(&script, "clicked").await;
                if let Ok(JSMessage::Clicked(clicked)) = msg {
                    if !clicked {
                        self.mouse.click(x, y, MouseButton::Left, true);
                        std::thread::sleep(std::time::Duration::from_millis(20));
                        self.mouse.click(x, y, MouseButton::Left, false);
                    }
                }
            } else {
                error!("unexpected JavaScript message body: {:?}", msg);
            }
        }
    }

    async fn execute_javascript(&self, script: &str, value_to_return: &str) -> Result<JSMessage> {
        // TODO: We need to wait for context to be initialized before executing JavaScript. It's done in render process.
        let id = uuid::Uuid::new_v4().to_string();
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
            .browser
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
}
