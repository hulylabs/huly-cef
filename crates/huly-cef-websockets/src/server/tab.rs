use futures::SinkExt;
use huly_cef::{browser::Browser, TabMessage};
use log::{error, info};
use tokio::{
    net::TcpStream,
    sync::mpsc::{self, UnboundedSender},
};
use tokio_tungstenite::WebSocketStream;
use tungstenite::Message;

pub const DEFAULT_WIDTH: u32 = 1280;
pub const DEFAULT_HEIGHT: u32 = 720;

pub async fn event_loop(mut tab: Browser, mut websocket: WebSocketStream<TcpStream>) {
    let (tx, mut rx) = mpsc::unbounded_channel();
    let id = tab.subscribe(tx.clone());
    generate_events(&tab, tx);

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

            tab.unsubscribe(id);
            break;
        }
    }
}

pub fn generate_events(tab: &Browser, tx: UnboundedSender<TabMessage>) {
    let state = tab.state.lock();

    _ = tx.send(TabMessage::Url(state.url.clone()));
    _ = tx.send(TabMessage::Title(state.title.clone()));
    _ = tx.send(TabMessage::Cursor(state.cursor.clone()));
    _ = tx.send(TabMessage::LoadState {
        status: state.load_status.clone(),
        can_go_back: state.can_go_back,
        can_go_forward: state.can_go_forward,
        error_code: state.error_code,
        error_text: state.error_text.clone(),
    });

    if let Some(favicon) = &state.favicon {
        _ = tx.send(TabMessage::Favicon(favicon.clone()));
    }

    info!("Generated initial state events for tab {}", tab.get_id());
}
