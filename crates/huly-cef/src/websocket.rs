use futures::{stream::SplitStream, SinkExt, StreamExt};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc::{self},
};
use tokio_tungstenite::WebSocketStream;
use tungstenite::Message;

use crate::cef::messages::CefMessage;
use crate::cef::{browser::Browser, messages::BrowserMessage};

/// Runs the WebSocket server that listens for incoming connections.
pub async fn serve() {
    let server = TcpListener::bind("127.0.0.1:8080")
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
    let (mut outgoing, incoming) = websocket.split();

    // Create a browser
    let (cef_message_sender, mut cef_message_reader) = mpsc::unbounded_channel::<CefMessage>();
    let browser = Browser::new(100, 100, cef_message_sender);

    tokio::spawn(handle_incoming_messages(incoming, browser));

    while let Some(message) = cef_message_reader.recv().await {
        let msg = match message {
            CefMessage::Frame(buffer) => Message::Binary(buffer.into()),
            CefMessage::Closed => {
                break;
            }
            message => Message::Text(
                serde_json::to_string(&message)
                    .expect("failed to serialize a message")
                    .into(),
            ),
        };
        _ = outgoing.send(msg).await;
    }

    println!("+++++++++++++++++++++++++++++");
    println!("handle_connection is finished");
    println!("+++++++++++++++++++++++++++++");
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
                println!("failed to read a message: {:?}, closing connection", error);
                break;
            }
        };

        if msg.is_close() {
            println!("websocket connection closed");
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

    println!("====================================");
    println!("handle_incoming_messages is finished");
    println!("====================================");
}

fn process_message(msg: BrowserMessage, browser: &Browser) -> bool {
    match msg {
        BrowserMessage::MouseMove { x, y } => browser.mouse_move(x, y),
        BrowserMessage::MouseClick { x, y, button, down } => {
            browser.mouse_click(x, y, button, down);
        }
        BrowserMessage::MouseWheel { x, y, dx, dy } => {
            browser.mouse_wheel(x, y, dx, dy);
        }
        BrowserMessage::KeyPress {
            character,
            code,
            down,
        } => {
            println!("keypress: ({}, {}, {})", character, code, down);
            browser.key_press(character, code, down);
        }
        BrowserMessage::StartVideo => {
            println!("StartVideo");
            browser.start_video();
        }
        BrowserMessage::StopVideo => {
            println!("StopVideo");
            browser.stop_video();
        }
        BrowserMessage::GoTo { url } => {
            println!("GoTo: {}", url);
            browser.go_to(&url);
        }
        BrowserMessage::Resize { width, height } => {
            println!("Resize: ({}, {})", width, height);
            browser.resize(width, height);
        }
        BrowserMessage::Close => return true,
        BrowserMessage::GoBack => {
            println!("GoBack");
            browser.go_back();
        }
        BrowserMessage::GoForward => {
            println!("GoForward");
            browser.go_forward();
        }
        BrowserMessage::Reload => {
            println!("Reload");
            browser.reload();
        }
    }

    return false;
}
