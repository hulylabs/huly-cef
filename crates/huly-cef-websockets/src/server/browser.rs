use std::{
    fs::read_to_string,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use futures::{SinkExt, StreamExt};
use huly_cef::messages::{BrowserMessage, BrowserMessageType, ServerBrowserMessage};
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
            (BrowserMessageType::OpenTab(url), _) => resp = Some(open_tab(&state, url)),
            (BrowserMessageType::CloseTab(id), _) => close_tab(&state, id),
            (BrowserMessageType::Resize { width, height }, _) => resize(width, height),
            (BrowserMessageType::GoTo { url }, Some(tab)) => tab.lock().unwrap().go_to(&url),
            (BrowserMessageType::MouseMove { x, y }, Some(tab)) => {
                tab.lock().unwrap().mouse_move(x, y)
            }
            (BrowserMessageType::MouseClick { x, y, button, down }, Some(tab)) => {
                tab.lock().unwrap().mouse_click(x, y, button, down)
            }
            (BrowserMessageType::MouseWheel { x, y, dx, dy }, Some(tab)) => {
                tab.lock().unwrap().mouse_wheel(x, y, dx, dy)
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
            ) => tab
                .lock()
                .unwrap()
                .key_press(character, windowscode, code, down, ctrl, shift),
            (BrowserMessageType::StopVideo, Some(tab)) => tab.lock().unwrap().start_video(),
            (BrowserMessageType::StartVideo, Some(tab)) => tab.lock().unwrap().stop_video(),
            (BrowserMessageType::Reload, Some(tab)) => tab.lock().unwrap().reload(),
            (BrowserMessageType::GoBack, Some(tab)) => tab.lock().unwrap().go_back(),
            (BrowserMessageType::GoForward, Some(tab)) => tab.lock().unwrap().go_forward(),
            (BrowserMessageType::SetFocus(focus), Some(tab)) => {
                tab.lock().unwrap().set_focus(focus)
            }
            (_, None) => {
                error!("tab with id {} not found", msg.tab_id);
                continue;
            }
        }

        if let Some(resp) = resp {
            let resp = serde_json::to_string(&resp).expect("failed to serialize response message");
            websocket
                .send(tungstenite::Message::Text(resp.into()))
                .await
                .expect("failed to send session message");
        }
    }

    close(state);
}

fn close(state: Arc<Mutex<ServerState>>) {
    let state = state.lock().unwrap();
    let tabs: Vec<String> = state
        .tabs
        .values()
        .map(|tab| tab.lock().unwrap().get_url())
        .collect();
    save_session(&state.cache_path, &tabs);

    for (_, tab) in state.tabs.iter() {
        tab.lock().unwrap().close();
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

fn restore_session(state: &Arc<Mutex<ServerState>>) -> ServerBrowserMessage {
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

    ServerBrowserMessage::Session(vec)
}

fn open_tab(state: &Arc<Mutex<ServerState>>, url: String) -> ServerBrowserMessage {
    let tab = tab::create(state.clone(), url);
    let id = tab.lock().unwrap().get_id();
    let mut state = state.lock().unwrap();
    state.tabs.insert(id, tab);
    ServerBrowserMessage::Tab(id)
}

fn close_tab(state: &Arc<Mutex<ServerState>>, id: i32) {
    let state = state.lock().unwrap();
    state.tabs.get(&id).map(|tab| tab.lock().unwrap().close());

    // TODO: remove tab from state
}

fn resize(width: u32, height: u32) {
    // This function is not used in the current implementation.
    // It can be implemented if needed.
    info!("Resizing browser to {}x{}", width, height);

    // TODO: Resize
}
