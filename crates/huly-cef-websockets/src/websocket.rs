use futures::{stream::SplitStream, SinkExt, StreamExt};
use log;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc::{self},
};
use tokio_tungstenite::WebSocketStream;
use tungstenite::Message;

use huly_cef::{
    browser::Browser,
    messages::{BrowserMessage, CefMessage},
};

/// Runs the WebSocket server that listens for incoming connections.
pub async fn serve(addr: String) {
    let server = TcpListener::bind(addr)
        .await
        .expect("failed to start a TCP listener");

    loop {
        let (stream, _) = server
            .accept()
            .await
            .expect("failed to accept a TCP stream");

        let websocket = tokio_tungstenite::accept_async(stream)
            .await
            .expect("failed to accept a WebSocket connection");

        tokio::spawn(handle_connection(websocket));
    }
}

/// Handles a single WebSocket connection.
async fn handle_connection(websocket: tokio_tungstenite::WebSocketStream<TcpStream>) {
    log::info!("New WebSocket connection");
    let (mut outgoing, incoming) = websocket.split();

    // Create a browser
    let (cef_message_sender, mut cef_message_reader) = mpsc::unbounded_channel::<CefMessage>();
    let browser = Browser::new(100, 100, cef_message_sender);
    let browser_id = browser.get_id();

    tokio::spawn(handle_incoming_messages(incoming, browser));

    while let Some(message) = cef_message_reader.recv().await {
        log::trace!("Received a message from CEF: {:?}", message);

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
                log::info!("Sending Close Frame");
                outgoing
                    .send(Message::Close(None))
                    .await
                    .expect("failed to send a close message");
                break;
            }
            message => serde_json::to_string(&message)
                .expect("failed to serialize a message")
                .into(),
        };
        _ = outgoing.send(msg).await;
    }

    log::info!(
        "handle_connection function is finished for browser {}",
        browser_id
    );
}

/// Handles incoming WebSocket messages and processes browser events.
async fn handle_incoming_messages(
    mut incoming: SplitStream<WebSocketStream<TcpStream>>,
    browser: Browser,
) {
    while let Some(msg) = incoming.next().await {
        let msg = match msg {
            Ok(msg) => msg,
            Err(error) => {
                log::error!("Failed to read a message: {:?}", error);
                break;
            }
        };

        if msg.is_close() {
            log::info!("WebSocket connection closed");
            break;
        }

        let msg = serde_json::from_slice::<BrowserMessage>(&msg.into_data())
            .expect("got unknown message from a client");

        let closed = process_message(msg, &browser);
        if closed {
            break;
        }
    }

    _ = browser.close();

    log::info!(
        "handle_incoming_messages function is finished for browser: {}",
        browser.get_id()
    );
}

fn process_message(msg: BrowserMessage, browser: &Browser) -> bool {
    match msg {
        BrowserMessage::MouseMove { x, y } => browser.mouse_move(x, y),
        BrowserMessage::MouseClick { x, y, button, down } => {
            log::trace!("mouse_click: ({}, {}, {:?}, {})", x, y, button, down);
            browser.mouse_click(x, y, button, down);
        }
        BrowserMessage::MouseWheel { x, y, dx, dy } => {
            log::trace!("mouse_wheel: ({}, {}, {}, {})", x, y, dx, dy);
            browser.mouse_wheel(x, y, dx, dy);
        }
        BrowserMessage::KeyPress {
            character,
            code,
            down,
            ctrl,
            shift,
        } => {
            log::trace!("keypress: ({}, {}, {})", character, code, down);
            browser.key_press(character, code, down, ctrl, shift);
        }
        BrowserMessage::StartVideo => {
            log::trace!("StartVideo");
            browser.start_video();
        }
        BrowserMessage::StopVideo => {
            log::trace!("StopVideo");
            browser.stop_video();
        }
        BrowserMessage::GoTo { url } => {
            log::info!("GoTo: {}", url);
            browser.go_to(&url);
        }
        BrowserMessage::Resize { width, height } => {
            log::info!("Resize: ({}, {})", width, height);
            browser.resize(width, height);
        }
        BrowserMessage::Close => return true,
        BrowserMessage::GoBack => {
            log::trace!("GoBack");
            browser.go_back();
        }
        BrowserMessage::GoForward => {
            log::trace!("GoForward");
            browser.go_forward();
        }
        BrowserMessage::Reload => {
            log::trace!("Reload");
            browser.reload();
        }
    }

    return false;
}
