use futures::SinkExt;
use huly_cef::{browser::Browser, TabMessage};
use log::{error, info};
use tokio::{
    net::TcpStream,
    sync::mpsc::{self, UnboundedSender},
};
use tokio_tungstenite::WebSocketStream;
use tungstenite::Message;

pub async fn event_loop(mut tab: Browser, mut websocket: WebSocketStream<TcpStream>) {
    let (tx, mut rx) = mpsc::unbounded_channel();
    let id = tab.subscribe(tx.clone());
    generate_events(&tab, tx);

    let mut buffer = vec![0u8; 4 + 4 + tab.state.read(|s| s.width as f64 * s.dpr * s.height as f64 * s.dpr * 4.0) as usize];

    while let Some(message) = rx.recv().await {
        let message = match message {
            TabMessage::Frame(data) => {
                let frame = data.lock().unwrap();
                if frame.data.len() + 8 != buffer.len() {
                    buffer = vec![0u8; (4 + 4 + frame.data.len()) as usize];
                }

                buffer[0..4].copy_from_slice(&frame.width.to_le_bytes());
                buffer[4..8].copy_from_slice(&frame.height.to_le_bytes());
                buffer[8..].copy_from_slice(&frame.data);
                Message::Binary(buffer.clone().into())
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
    _ = tx.send(TabMessage::Url(tab.state.read(|state| state.url.clone())));
    _ = tx.send(TabMessage::Title(
        tab.state.read(|state| state.title.clone()),
    ));
    _ = tx.send(TabMessage::Cursor(
        tab.state.read(|state| state.cursor.clone()),
    ));
    _ = tx.send(TabMessage::LoadState(
        tab.state.read(|state| state.load_state.clone()),
    ));

    if let Some(favicon) = tab.state.read(|state| state.favicon.clone()) {
        _ = tx.send(TabMessage::Favicon(favicon.clone()));
    }

    info!("Generated initial state events for tab {}", tab.get_id());
}
