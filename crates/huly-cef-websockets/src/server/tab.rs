use futures::SinkExt;
use huly_cef::{browser::Browser, Framebuffer, TabMessage};
use log::{error, info};
use tokio::{
    net::TcpStream,
    sync::mpsc::{self, UnboundedSender},
};
use tokio_tungstenite::WebSocketStream;
use tungstenite::Message;

fn serialize(frame: &Framebuffer, buffer: &mut Vec<u8>) {
    let width_bytes = 4;
    let height_bytes = 4;

    if buffer.len() != frame.data.len() + width_bytes + height_bytes {
        buffer.resize(frame.data.len() + width_bytes + height_bytes, 0);
    }

    buffer[0..width_bytes].copy_from_slice(&frame.width.to_le_bytes());
    buffer[width_bytes..width_bytes + height_bytes].copy_from_slice(&frame.height.to_le_bytes());
    buffer[width_bytes + height_bytes..].copy_from_slice(&frame.data);
}

pub async fn event_loop(mut tab: Browser, mut websocket: WebSocketStream<TcpStream>) {
    let (tx, mut rx) = mpsc::unbounded_channel();
    let id = tab.subscribe(tx.clone());
    generate_events(&tab, tx);

    let mut buffer = Vec::new();
    while let Some(message) = rx.recv().await {
        let message = match message {
            TabMessage::Frame(data) => {
                let frame = data.lock().unwrap();
                serialize(&frame, &mut buffer);
                Message::Binary(buffer.clone().into())
            }
            TabMessage::Closed => break,
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
    _ = tx.send(TabMessage::ExternalLink(
        tab.state.read(|state| state.external_link.clone()),
    ));

    if let Some(favicon) = tab.state.read(|state| state.favicon.clone()) {
        _ = tx.send(TabMessage::Favicon(favicon.clone()));
    }

    info!("Generated initial state events for tab {}", tab.get_id());
}
