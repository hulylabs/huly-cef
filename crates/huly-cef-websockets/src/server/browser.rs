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

        info!("received message: {:?}", msg);

        let tab = state.lock().unwrap().tabs.get(&msg.tab_id).cloned();
        let tab = match tab {
            Some(tab) => tab,
            None => {
                error!("tab with id {} not found", msg.tab_id);
                continue;
            }
        };

        let mut resp = None;
        {
            let tab = tab.lock().unwrap();
            match msg.body {
                BrowserMessageType::Close => break,
                BrowserMessageType::RestoreSession => resp = Some(restore_session(&state)),
                BrowserMessageType::OpenTab(url) => {
                    let tab = tab::create(state.clone(), url);
                    let id = tab.lock().unwrap().get_id();
                    let mut state = state.lock().unwrap();
                    state.tabs.insert(id, tab);
                    resp = Some(ServerBrowserMessage::Tab(id));
                }
                BrowserMessageType::CloseTab(id) => close_tab(&state, id),
                BrowserMessageType::Resize { width, height } => resize(width, height),
                BrowserMessageType::GoTo { url } => tab.go_to(&url),
                BrowserMessageType::MouseMove { x, y } => tab.mouse_move(x, y),
                BrowserMessageType::MouseClick { x, y, button, down } => {
                    tab.mouse_click(x, y, button, down)
                }
                BrowserMessageType::MouseWheel { x, y, dx, dy } => tab.mouse_wheel(x, y, dx, dy),
                BrowserMessageType::KeyPress {
                    character,
                    code,
                    windowscode,
                    down,
                    ctrl,
                    shift,
                } => tab.key_press(character, windowscode, code, down, ctrl, shift),
                BrowserMessageType::StopVideo => tab.start_video(),
                BrowserMessageType::StartVideo => tab.stop_video(),
                BrowserMessageType::Reload => tab.reload(),
                BrowserMessageType::GoBack => tab.go_back(),
                BrowserMessageType::GoForward => tab.go_forward(),
                BrowserMessageType::SetFocus(focus) => tab.set_focus(focus),
            };
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
