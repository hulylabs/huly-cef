use std::{
    fs::read_to_string,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use futures::{SinkExt, StreamExt};
use huly_cef::{
    browser::Browser,
    messages::{CefMessage, ClientBrowserMessage, ServerBrowserMessage},
};
use log::{error, info};
use tokio::{net::TcpStream, sync::mpsc};
use tokio_tungstenite::WebSocketStream;

use super::ServerState;

pub async fn handle(state: Arc<Mutex<ServerState>>, mut websocket: WebSocketStream<TcpStream>) {
    while let Some(msg) = websocket.next().await {
        let msg = match msg {
            Ok(msg) => msg,
            Err(e) => {
                error!("failed to read a message: {:?}", e);
                break;
            }
        };

        if msg.is_close() {
            break;
        }

        let msg = serde_json::from_slice::<ClientBrowserMessage>(&msg.into_data())
            .expect("got unknown message from a client");

        let mut resp = None;
        match msg {
            ClientBrowserMessage::RestoreSession => {
                let state = state.lock().unwrap();
                let tabs = restore_session(&state.cache_path);

                resp = Some(ServerBrowserMessage::Session(tabs));
            }
            ClientBrowserMessage::Close => break,
            ClientBrowserMessage::OpenTab(url) => resp = Some(create_tab(&state, url)),
            ClientBrowserMessage::CloseTab(id) => close_tab(&state, id),
        };

        if let Some(resp) = resp {
            let resp = serde_json::to_string(&resp).expect("failed to serialize response message");
            websocket
                .send(tungstenite::Message::Text(resp.into()))
                .await
                .expect("failed to send session message");
        }
    }

    close(state).await;
}

async fn close(state: Arc<Mutex<ServerState>>) {
    let state = state.lock().unwrap();
    let tabs: Vec<String> = state.tabs.values().map(|tab| tab.get_url()).collect();
    save_session(&state.cache_path, &tabs);

    for (_, tab) in state.tabs.iter() {
        tab.close()
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

fn restore_session(cache_path: &str) -> Vec<String> {
    let session_file_path = PathBuf::from(cache_path).join("session.json");
    info!(
        "restoring session from cache path: {}",
        session_file_path.display()
    );
    read_to_string(session_file_path)
        .and_then(|content| serde_json::from_str::<Vec<String>>(&content).map_err(|e| e.into()))
        .unwrap_or_else(|error| {
            error!("failed to read session file: {:?}", error);
            Vec::new()
        })
}

fn create_tab(state: &Arc<Mutex<ServerState>>, url: String) -> ServerBrowserMessage {
    let mut state = state.lock().unwrap();

    let (tab_msg_writer, mut tab_msg_reader) = mpsc::unbounded_channel::<CefMessage>();
    let tab = Browser::new(100, 100, tab_msg_writer.clone());
    let id = tab.get_id();

    state.tabs.insert(id, tab.clone());

    ServerBrowserMessage::Tab(id)
}

fn close_tab(state: &Arc<Mutex<ServerState>>, id: i32) {
    let state = state.lock().unwrap();
    state.tabs.get(&id).map(|tab| tab.close());
}
