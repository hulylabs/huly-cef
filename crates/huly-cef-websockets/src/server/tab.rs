use futures::{
    stream::{SplitStream, StreamExt},
    SinkExt,
};
use log::{error, info, trace};
use serde_json;
use std::sync::{Arc, Mutex};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;

use huly_cef::{
    browser::Browser,
    messages::{BrowserMessage, CefMessage},
};

use super::ServerState;

/// Handles a tab connection, reads and processes tab and CEF messages
pub async fn handle(state: Arc<Mutex<ServerState>>, websocket: WebSocketStream<TcpStream>) {
    let (mut outgoing, incoming) = websocket.split();

    // Create a browser
    let (cef_message_sender, mut cef_message_reader) = mpsc::unbounded_channel::<CefMessage>();
    let tab = Browser::new(100, 100, cef_message_sender.clone());
    let tab_id = tab.get_id();

    {
        let mut state = state.lock().unwrap();
        state.tabs.insert(tab_id, tab.clone());
        info!(id = tab_id; "tab created");
    }

    tokio::spawn(handle_tab_messages(incoming, tab));

    while let Some(message) = cef_message_reader.recv().await {
        trace!(id = tab_id; "received a message from CEF: {:?}", message);

        let msg = match message {
            CefMessage::Frame(data) => {
                let mut buffer = Vec::new();
                buffer.extend(0_i8.to_ne_bytes());
                buffer.extend(data);
                Message::Binary(buffer.into())
            }
            CefMessage::Popup {
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
            CefMessage::Closed => {
                log::info!(id = tab_id; "sending close message");
                if let Err(e) = outgoing.send(Message::Close(None)).await {
                    error!("failed to send close message: {:?}", e);
                };
                break;
            }
            message => serde_json::to_string(&message)
                .expect("failed to serialize a message")
                .into(),
        };
        _ = outgoing.send(msg).await;
    }

    info!("id" = tab_id; "cef reader is closed");
}

/// Handles incoming WebSocket messages and processes browser events.
async fn handle_tab_messages(mut incoming: SplitStream<WebSocketStream<TcpStream>>, tab: Browser) {
    while let Some(msg) = incoming.next().await {
        let msg = match msg {
            Ok(msg) => msg,
            Err(e) => {
                error!("failed to read a message: {:?}", e);
                break;
            }
        };

        if msg.is_close() {
            info!(id = tab.get_id(); "client sent close message");
            break;
        }

        let msg = serde_json::from_slice::<BrowserMessage>(&msg.into_data())
            .expect("got unknown message from a client");

        let closed = process_message(msg, &tab);
        if closed {
            break;
        }
    }

    info!(id = tab.get_id(); "client message reader is closed");
}

fn process_message(msg: BrowserMessage, tab: &Browser) -> bool {
    trace!(id = tab.get_id(); "proccessing browser message: {:?}", msg);
    match msg {
        BrowserMessage::StartVideo => tab.start_video(),
        BrowserMessage::StopVideo => tab.stop_video(),
        BrowserMessage::GoTo { url } => tab.go_to(&url),
        BrowserMessage::Resize { width, height } => tab.resize(width, height),
        BrowserMessage::Close => return true,
        BrowserMessage::GoBack => tab.go_back(),
        BrowserMessage::GoForward => tab.go_forward(),
        BrowserMessage::Reload => tab.reload(),
        BrowserMessage::SetFocus(focus) => tab.set_focus(focus),
        BrowserMessage::MouseMove { x, y } => tab.mouse_move(x, y),
        BrowserMessage::MouseWheel { x, y, dx, dy } => tab.mouse_wheel(x, y, dx, dy),
        BrowserMessage::MouseClick { x, y, button, down } => tab.mouse_click(x, y, button, down),
        BrowserMessage::KeyPress {
            character,
            windowscode,
            code,
            down,
            ctrl,
            shift,
        } => tab.key_press(character, windowscode, code, down, ctrl, shift),
    }

    return false;
}
