use std::{
    fs::read_to_string,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use base64::Engine;
use futures::{SinkExt, StreamExt};
use huly_cef::{
    browser::Browser,
    messages::{BrowserMessage, BrowserMessageType, ServerMessage, ServerMessageType},
};
use image::ImageEncoder;
use log::{error, info};
use tokio::net::TcpStream;
use tokio_tungstenite::WebSocketStream;

use crate::server::tab;

use super::ServerState;

pub async fn handle(state: Arc<Mutex<ServerState>>, mut websocket: WebSocketStream<TcpStream>) {
    while let Some(msg) = websocket.next().await {
        let msg = match msg {
            Ok(msg) => msg,
            Err(e) => {
                error!("failed to read a message: {:?}", e);
                continue;
            }
        };

        if msg.is_close() {
            break;
        }

        info!("received message: {:?}", msg);

        let msg = match serde_json::from_slice::<BrowserMessage>(&msg.into_data()) {
            Ok(msg) => msg,
            Err(e) => {
                error!("failed to deserialize message: {:?}", e);
                continue;
            }
        };

        let tab = state.lock().unwrap().tabs.get(&msg.tab_id).cloned();
        let mut resp = None;
        match (msg.body, tab) {
            (BrowserMessageType::Close, _) => break,
            (BrowserMessageType::RestoreSession, _) => resp = Some(restore_session(&state)),
            (BrowserMessageType::OpenTab(url), _) => {
                // TODO: pass wait_unti_loaded as a parameter
                resp = Some(open_tab(&state, &url, true).await)
            }
            (BrowserMessageType::CloseTab, _) => close_tab(&state, msg.tab_id),
            (BrowserMessageType::GetTabs, _) => resp = Some(get_tabs(&state)),
            (BrowserMessageType::Resize { width, height }, _) => resize(&state, width, height),
            (BrowserMessageType::Screenshot { width, height }, Some(tab)) => {
                resp = Some(ServerMessageType::Screenshot(
                    get_screenshot(tab, width, height).await,
                ));
            }
            (BrowserMessageType::GoTo { url }, Some(tab)) => tab.go_to(&url),
            (BrowserMessageType::MouseMove { x, y }, Some(tab)) => tab.mouse_move(x, y),
            (BrowserMessageType::MouseClick { x, y, button, down }, Some(tab)) => {
                tab.mouse_click(x, y, button, down)
            }
            (BrowserMessageType::MouseWheel { x, y, dx, dy }, Some(tab)) => {
                tab.mouse_wheel(x, y, dx, dy)
            }
            (
                BrowserMessageType::KeyPress {
                    character,
                    code,
                    windowscode,
                    down,
                    ctrl,
                    shift,
                },
                Some(tab),
            ) => tab.key_press(character, windowscode, code, down, ctrl, shift),
            (BrowserMessageType::Char(char), Some(tab)) => tab.char(char),
            (BrowserMessageType::StopVideo, Some(tab)) => tab.stop_video(),
            (BrowserMessageType::StartVideo, Some(tab)) => tab.start_video(),
            (BrowserMessageType::Reload, Some(tab)) => tab.reload(),
            (BrowserMessageType::GoBack, Some(tab)) => tab.go_back(),
            (BrowserMessageType::GoForward, Some(tab)) => tab.go_forward(),
            (BrowserMessageType::SetFocus(focus), Some(tab)) => tab.set_focus(focus),
            (BrowserMessageType::GetDOM, Some(tab)) => {
                resp = Some(ServerMessageType::DOM(tab.get_dom().await));
            }
            (BrowserMessageType::GetClickableElements, Some(tab)) => {
                info!("Get clickable elements for tab {}", msg.tab_id);
                let elements = tab.get_clickable_elements().await;
                info!("Found {} clickable elements", elements.len());
                resp = Some(ServerMessageType::ClickableElements(elements));
            }
            (BrowserMessageType::ClickElement(id), Some(tab)) => {
                tab.click_element(id);
            }
            (BrowserMessageType::SetText { selector, text }, Some(tab)) => {
                tab.set_text(&selector, &text);
            }
            (_, None) => {
                error!("tab with id {} not found", msg.tab_id);
                continue;
            }
        }

        if let Some(resp) = resp {
            let resp = ServerMessage {
                id: msg.id,
                tab_id: msg.tab_id,
                body: resp,
            };

            let resp = serde_json::to_string(&resp).expect("failed to serialize response message");
            websocket
                .send(tungstenite::Message::Text(resp.into()))
                .await
                .expect("failed to send session message");
        }
    }
}

fn close(state: Arc<Mutex<ServerState>>) {
    let state = state.lock().unwrap();
    let tabs: Vec<String> = state
        .tabs
        .values()
        .map(|tab| tab.state.lock().unwrap().url.clone())
        .collect();
    save_session(&state.cache_path, &tabs);

    for (_, tab) in state.tabs.iter() {
        tab.close();
    }
}

fn save_session(cache_path: &str, tabs: &[String]) {
    let session_file_path = PathBuf::from(cache_path).join("session.json");
    info!(
        "saving session to cache path: {}",
        session_file_path.display()
    );
    if let Err(error) = serde_json::to_string(tabs)
        .and_then(|content| Ok(std::fs::write(session_file_path, content)))
    {
        error!("failed to save session file: {:?}", error);
    }
}

fn restore_session(state: &Arc<Mutex<ServerState>>) -> ServerMessageType {
    let state = state.lock().unwrap();

    let session_file_path = PathBuf::from(state.cache_path.clone()).join("session.json");
    info!(
        "restoring session from cache path: {}",
        session_file_path.display()
    );
    let vec = read_to_string(session_file_path)
        .and_then(|content| serde_json::from_str::<Vec<String>>(&content).map_err(|e| e.into()))
        .unwrap_or_else(|error| {
            error!("failed to read session file: {:?}", error);
            Vec::new()
        });

    ServerMessageType::Session(vec)
}

async fn open_tab(
    state: &Arc<Mutex<ServerState>>,
    url: &str,
    wait_until_loaded: bool,
) -> ServerMessageType {
    let (width, height) = {
        let state = state.lock().unwrap();
        state.size
    };
    let tab = tab::create(state.clone(), width, height, url);
    let id = tab.get_id();
    {
        let mut state = state.lock().unwrap();
        state.tabs.insert(id, tab.clone());
    }

    if wait_until_loaded {
        tab.wait_until_loaded().await;
    }

    ServerMessageType::Tab(id)
}

fn close_tab(state: &Arc<Mutex<ServerState>>, id: i32) {
    let tab = state.lock().unwrap().tabs.remove(&id);
    if let Some(tab) = tab {
        tab.close();
    } else {
        error!("tab with id {} not found", id);
    }
}

fn get_tabs(state: &Arc<Mutex<ServerState>>) -> ServerMessageType {
    let ids = {
        let state = state.lock().unwrap();
        state
            .tabs
            .values()
            .map(|tab| tab.get_id().clone())
            .collect::<Vec<_>>()
    };

    ServerMessageType::Tabs(ids)
}

fn resize(state: &Arc<Mutex<ServerState>>, width: u32, height: u32) {
    let mut state = state.lock().unwrap();
    state.size = (width, height);
    state.tabs.iter().for_each(|t| t.1.resize(width, height));
}

async fn get_screenshot(tab: Browser, width: u32, height: u32) -> String {
    let data = tab.screenshot(width, height).await;

    let mut bytes: Vec<u8> = Vec::new();
    {
        let encoder = image::codecs::png::PngEncoder::new(&mut bytes);
        encoder
            .write_image(&data, width, height, image::ExtendedColorType::Rgba8)
            .expect("PNG encoding failed");
    }

    base64::engine::general_purpose::STANDARD.encode(bytes)
}
