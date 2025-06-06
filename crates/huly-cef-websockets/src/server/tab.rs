use futures::SinkExt;
use log::{error, trace};
use std::sync::{Arc, Mutex};
use tokio::{
    net::TcpStream,
    sync::mpsc::{self, UnboundedReceiver},
};
use tokio_tungstenite::WebSocketStream;
use tungstenite::Message;

use huly_cef::{browser::Browser, messages::TabMessage};

use crate::server::ServerState;

pub fn create(state: Arc<Mutex<ServerState>>, _url: String) -> Arc<Mutex<Browser>> {
    let (tab_msg_writer, tab_msg_reader) = mpsc::unbounded_channel::<TabMessage>();
    let tab = Arc::new(Mutex::new(Browser::new(100, 100, tab_msg_writer)));

    tokio::spawn(handle_tab_messages(state, tab.clone(), tab_msg_reader));

    tab
}

/// Handles incoming WebSocket messages and processes browser events.
async fn handle_tab_messages(
    state: Arc<Mutex<ServerState>>,
    tab: Arc<Mutex<Browser>>,
    mut msg_channel: mpsc::UnboundedReceiver<TabMessage>,
) {
    while let Some(message) = msg_channel.recv().await {
        let mut tab = tab.lock().unwrap();

        trace!("tab" = tab.get_id(); "received a message from tab: {:?}", message);

        match &message {
            TabMessage::CursorChanged(cursor) => tab.cursor = cursor.clone(),
            TabMessage::TitleChanged(title) => tab.title = title.clone(),
            TabMessage::UrlChanged(url) => tab.url = url.clone(),
            TabMessage::FaviconUrlChanged(favicon) => tab.favicon = Some(favicon.clone()),
            _ => {}
        };

        state
            .lock()
            .unwrap()
            .tab_event_receivers
            .get(&tab.get_id())
            .map(|rx| rx.send(message));
    }
}

pub async fn translate_tab_messages(
    mut rx: UnboundedReceiver<TabMessage>,
    mut websocket: WebSocketStream<TcpStream>,
) {
    while let Some(msg) = rx.recv().await {
        let msg = match msg {
            TabMessage::Frame(data) => {
                let mut buffer = Vec::new();
                buffer.extend(0_i8.to_ne_bytes());
                buffer.extend(data);
                Message::Binary(buffer.into())
            }
            TabMessage::Popup {
                x,
                y,
                width,
                height,
                data,
            } => {
                let mut buffer = Vec::new();
                buffer.extend(1_i8.to_ne_bytes());
                buffer.extend(x.to_ne_bytes());
                buffer.extend(y.to_ne_bytes());
                buffer.extend(width.to_ne_bytes());
                buffer.extend(height.to_ne_bytes());
                buffer.extend(data);

                Message::Binary(buffer.into())
            }
            TabMessage::Closed => {
                break;
            }
            message => serde_json::to_string(&message)
                .expect("failed to serialize a message")
                .into(),
        };

        if let Err(e) = websocket.send(msg).await {
            error!("failed to send message: {:?}", e);
            break;
        }
    }
}
