use std::{
    fs::read_to_string,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use futures::{SinkExt, StreamExt};
use huly_cef::messages::{BrowserMessage, BrowserMessageType, ServerMessage, ServerMessageType};
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
            (BrowserMessageType::OpenTab(url), _) => resp = Some(open_tab(&state, &url)),
            (BrowserMessageType::CloseTab, _) => close_tab(&state, msg.tab_id),
            (BrowserMessageType::GetTabs, _) => resp = Some(get_tabs(&state)),
            (BrowserMessageType::Resize { width, height }, _) => resize(&state, width, height),
            (BrowserMessageType::TakeScreenshot, Some(tab)) => {
                resp = tab
                    .state
                    .lock()
                    .unwrap()
                    .last_frame
                    .clone()
                    .map(ServerMessageType::Screenshot);
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
            (BrowserMessageType::StopVideo, Some(tab)) => tab.stop_video(),
            (BrowserMessageType::StartVideo, Some(tab)) => tab.start_video(),
            (BrowserMessageType::Reload, Some(tab)) => tab.reload(),
            (BrowserMessageType::GoBack, Some(tab)) => tab.go_back(),
            (BrowserMessageType::GoForward, Some(tab)) => tab.go_forward(),
            (BrowserMessageType::SetFocus(focus), Some(tab)) => tab.set_focus(focus),
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

    close(state);
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

fn open_tab(state: &Arc<Mutex<ServerState>>, url: &str) -> ServerMessageType {
    let tab = tab::create(state.clone(), url);
    let id = tab.get_id();
    let mut state = state.lock().unwrap();
    state.tabs.insert(id, tab);
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
    let urls = {
        let state_guard = state.lock().unwrap();

        info!(
            "load_state: {:?}",
            state_guard.tabs.values().collect::<Vec<_>>()[0]
                .state
                .lock()
                .unwrap()
                .load_state
        );
        state_guard
            .tabs
            .values()
            .map(|tab| tab.state.lock().unwrap().url.clone())
            .collect::<Vec<_>>()
    };

    ServerMessageType::Tabs(urls)
}

fn resize(state: &Arc<Mutex<ServerState>>, width: u32, height: u32) {
    state
        .lock()
        .unwrap()
        .tabs
        .iter()
        .for_each(|t| t.1.resize(width, height));
}
