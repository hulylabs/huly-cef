use std::{
    io::Cursor,
    sync::{Arc, Mutex},
    time::Duration,
};

use anyhow::Result;

use base64::{prelude::BASE64_STANDARD, Engine};
use cef_ui::{Browser, ProcessId, StringVisitor};
use image::{imageops::FilterType, ImageFormat};
use tokio::sync::{oneshot, Notify};

use crate::{
    application::ipc::{self, ResponseBody},
    browser::{
        automation::{devtools::DevTools, dom::DOMVisitor},
        mouse::Mouse,
    },
    state::SharedBrowserState,
    ClickableElement, LoadState, LoadStatus, MouseButton, TabMessage, TabMessageType,
};

mod devtools;
mod dom;

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

        rx.await.expect("failed to get the page source")
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

    pub async fn get_clickable_elements(&self) -> Result<Vec<ClickableElement>, String> {
        let response = self
            .send_request(ipc::RequestBody::GetClickableElements)
            .await?;
        let ipc::ResponseBody::ClickableElements(elements) = response else {
            return Err(format!("unexpected response: {:?}", response));
        };

        self.clickable_elements
            .lock()
            .unwrap()
            .replace(elements.clone());
        Ok(elements)
    }

    pub async fn click_element(&self, id: i32) -> Result<(), String> {
        let Some(element) = self
            .clickable_elements
            .lock()
            .unwrap()
            .as_ref()
            .and_then(|elements| elements.get(id as usize).cloned())
        else {
            return Err(format!("Element not found: {}", id));
        };

        let request = ipc::RequestBody::GetElementCenter {
            selector: format!("[data-clickable-id={}]", element.id),
        };

        let response = self.send_request(request).await?;
        let ResponseBody::ElementCenter { x, y } = response else {
            return Err(format!("unexpected response: {:?}", response));
        };

        self.mouse.click(x, y, MouseButton::Left, true);
        std::thread::sleep(std::time::Duration::from_millis(20));
        self.mouse.click(x, y, MouseButton::Left, false);
        std::thread::sleep(std::time::Duration::from_millis(1000));

        let request = ipc::RequestBody::CheckElementClicked {
            selector: format!("[data-clickable-id={}]", element.id),
        };

        let response = self.send_request(request).await?;
        let ResponseBody::Clicked(clicked) = response else {
            return Err(format!("unexpected response: {:?}", response));
        };

        if !clicked {
            self.mouse.click(x, y, MouseButton::Left, true);
            std::thread::sleep(std::time::Duration::from_millis(20));
            self.mouse.click(x, y, MouseButton::Left, false);
        }

        Ok(())
    }

    async fn send_request(&self, body: ipc::RequestBody) -> Result<ipc::ResponseBody, String> {
        let id = uuid::Uuid::new_v4().to_string();
        let request = ipc::Request {
            id: id.clone(),
            body,
        };

        let frame = self.browser.get_main_frame().unwrap().unwrap();
        frame
            .send_process_message(ProcessId::Renderer, request.into())
            .expect("failed to send IPC message");

        let (tx, rx) = oneshot::channel::<ipc::Response>();
        self.state.update(|state| {
            state.ipc_messages.insert(id.to_string(), tx);
        });

        let response = rx.await.expect("failed to receive IPC response");

        Ok(response.body)
    }
}
