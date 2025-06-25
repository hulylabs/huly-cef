use futures::SinkExt;
use log::{error, info};
use std::sync::{Arc, Mutex};
use tokio::{
    net::TcpStream,
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
};
use tokio_tungstenite::WebSocketStream;
use tungstenite::Message;

use huly_cef::{
    browser::Browser,
    messages::{TabEventType, TabMessage},
};

use crate::server::ServerState;

pub const DEFAULT_WIDTH: u32 = 1280;
pub const DEFAULT_HEIGHT: u32 = 720;

pub fn create(state: Arc<Mutex<ServerState>>, width: u32, height: u32, url: &str) -> Browser {
    let (tab_msg_writer, tab_msg_reader) = mpsc::unbounded_channel::<TabMessage>();
    let tab = Browser::new(width, height, url, tab_msg_writer);

    tokio::spawn(process_tab_events(state, tab.clone(), tab_msg_reader));

    tab
}

/// Handles incoming WebSocket messages and processes browser events.
async fn process_tab_events(
    state: Arc<Mutex<ServerState>>,
    tab: Browser,
    mut msg_channel: mpsc::UnboundedReceiver<TabMessage>,
) {
    while let Some(message) = msg_channel.recv().await {
        match &message {
            TabMessage::CursorChanged(cursor) => tab.state.lock().unwrap().cursor = cursor.clone(),
            TabMessage::TitleChanged(title) => tab.state.lock().unwrap().title = title.clone(),
            TabMessage::UrlChanged(url) => tab.state.lock().unwrap().url = url.clone(),
            TabMessage::LoadStateChanged { status, .. } => {
                tab.state.lock().unwrap().load_status = status.clone()
            }
            TabMessage::FaviconUrlChanged(favicon) => {
                tab.state.lock().unwrap().favicon = Some(favicon.clone())
            }
            _ => {}
        };

        let event_type = TabEventType::from(&message);
        tab.state
            .lock()
            .unwrap()
            .tab_events_subscribers
            .get(&event_type)
            .and_then(|ch| ch.send(message.clone()).ok());

        state
            .lock()
            .unwrap()
            .event_consumers
            .get(&tab.get_id())
            .map(|rx| rx.send(message));
    }
}

pub async fn transfer_tab_messages(
    mut rx: UnboundedReceiver<TabMessage>,
    mut websocket: WebSocketStream<TcpStream>,
) {
    while let Some(message) = rx.recv().await {
        let message = match message {
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

        if let Err(e) = websocket.send(message).await {
            error!("failed to send message: {:?}", e);
            break;
        }
    }
}

pub fn generate_events(tab: &Browser, tx: UnboundedSender<TabMessage>) {
    let state = tab.state.lock().unwrap();

    _ = tx.send(TabMessage::UrlChanged(state.url.clone()));
    _ = tx.send(TabMessage::TitleChanged(state.title.clone()));
    _ = tx.send(TabMessage::CursorChanged(state.cursor.clone()));
    _ = tx.send(TabMessage::LoadStateChanged {
        status: state.load_status.clone(),
        can_go_back: state.can_go_back,
        can_go_forward: state.can_go_forward,
        error_code: state.error_code,
        error_text: state.error_text.clone(),
    });

    if let Some(favicon) = &state.favicon {
        _ = tx.send(TabMessage::FaviconUrlChanged(favicon.clone()));
    }

    info!("Generated initial state events for tab {}", tab.get_id());
}
