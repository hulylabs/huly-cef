use std::{io::Cursor, sync::Arc, time::Duration};

use anyhow::Result;

use base64::{prelude::BASE64_STANDARD, Engine};
use cef_ui::{Browser, StringVisitor, StringVisitorCallbacks};
use image::{imageops::FilterType, ImageFormat};
use log::error;
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;

use crate::{
    browser::{devtools::DevTools, mouse::Mouse},
    state::SharedBrowserState,
    ClickableElement, LoadStatus, MouseButton, GET_CLICKABLE_ELEMENTS_SCRIPT,
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
}

impl Clone for Automation {
    fn clone(&self) -> Self {
        Automation {
            browser: self.browser.clone(),
            devtools: self.devtools.clone(),
            mouse: self.mouse.clone(),
            state: self.state.clone(),
        }
    }
}

impl Automation {
    pub fn new(browser: Browser, state: SharedBrowserState, mouse: Mouse) -> Self {
        let devtools = Arc::new(DevTools::new(browser.clone()));

        Automation {
            browser,
            devtools,
            mouse,
            state,
        }
    }

    pub fn start_navigation(&self) {
        self.devtools.start_navigation();
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
        let screenshot = screenshot.resize(width, height, FilterType::Lanczos3);

        let mut cursor = Cursor::new(Vec::new());
        screenshot.write_to(&mut cursor, ImageFormat::Png)?;

        Ok(BASE64_STANDARD.encode(cursor.into_inner()))
    }

    pub async fn wait_until_loaded(&self, url: String) -> Result<(), String> {
        let timeout = Duration::from_secs(10);
        _ = self
            .state
            .wait_for(
                |state| state.load_state.status == LoadStatus::Loaded && state.url == url,
                timeout,
            )
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
        // tokio::time::sleep(Duration::from_secs(5)).await;
        // self.devtools.get_frame_events();
        // Ok(())
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
