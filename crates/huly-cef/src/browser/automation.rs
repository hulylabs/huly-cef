use std::sync::Arc;

use anyhow::Result;

use cef_ui::{Browser, StringVisitor, StringVisitorCallbacks};
use log::error;
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;

use crate::{
    browser::{devtools::DevTools, mouse::Mouse},
    state::{RenderMode, SharedBrowserState},
    ClickableElement, MouseButton, GET_CLICKABLE_ELEMENTS_SCRIPT,
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
        let (w, h) = self.state.read(|s| (s.width, s.height));
        self.state.update(|s| {
            s.render_mode = RenderMode::Screenshot;
            s.width = width;
            s.height = height;
        });

        // TODO: add render_mode checks in render handler and also size checks
        _ = self.browser.get_host().unwrap().was_resized();
        let screenshot = self.devtools.screenshot().await;

        self.state.update(|s| {
            s.render_mode = RenderMode::Stream;
            s.width = w;
            s.height = h;
        });

        screenshot
    }

    pub async fn wait_until_loaded(&self) {
        self.devtools.wait_until_loaded().await;
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
