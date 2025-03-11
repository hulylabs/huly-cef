use futures::{stream::SplitStream, SinkExt, StreamExt};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc,
};
use tokio_tungstenite::WebSocketStream;
use tungstenite::Message;

use crate::cef;

mod messages;

use messages::WebSocketMessage;

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
    let (mut outgoing, mut incoming) = websocket.split();

    // Read the first message from the WebSocket. Should be a CreateBrowser message.
    let msg = incoming
        .next()
        .await
        .expect("failed to read a message")
        .expect("failed to read a message");

    let msg = serde_json::from_slice::<WebSocketMessage>(&msg.into_data())
        .expect("got unknown message from a client");

    let WebSocketMessage::CreateBrowser { url, width, height } = msg else {
        panic!("Unknown message");
    };
    println!(
        "got create browser message: ({}, {}, {})",
        width, height, url
    );

    println!("creating browser");
    // Create a browser
    let (sender, mut reader) = mpsc::unbounded_channel::<cef::messages::CefMessage>();
    let browser = cef::create_browser(width, height, &url, sender);
    println!("browser created");

    tokio::spawn(handle_incoming_messages(incoming, browser));

    while let Some(message) = reader.recv().await {
        let msg = match message {
            cef::messages::CefMessage::Render(buffer) => Message::Binary(buffer.into()),
            cef::messages::CefMessage::IsLoading => Message::Text("IsLoading".into()),
            cef::messages::CefMessage::Loaded => Message::Text("Loaded".into()),
            cef::messages::CefMessage::LoadError => Message::Text("LoadError".into()),
        };
        _ = outgoing.send(msg).await;
    }

    println!("finished handling connection");
}

/// Handles incoming WebSocket messages and processes browser events.
async fn handle_incoming_messages(
    mut incoming: SplitStream<WebSocketStream<TcpStream>>,
    browser: cef::Browser,
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

        let msg = serde_json::from_slice::<WebSocketMessage>(&msg.into_data())
            .expect("got unknown message from a client");

        process_message(msg, &browser);
    }
}

fn process_message(msg: WebSocketMessage, browser: &cef::Browser) {
    let host = browser.inner.get_host().unwrap();

    match msg {
        WebSocketMessage::MouseMove { x, y } => {
            let event = cef_ui::MouseEvent {
                x,
                y,
                modifiers: cef_ui::EventFlags::empty(),
            };
            host.send_mouse_move_event(&event, false)
                .expect("failed to send mouse move event");
        }
        WebSocketMessage::MouseClick { x, y, button, down } => {
            let event = cef_ui::MouseEvent {
                x,
                y,
                modifiers: cef_ui::EventFlags::empty(),
            };

            let button = match button {
                messages::MouseType::Left => cef_ui::MouseButtonType::Left,
                messages::MouseType::Middle => cef_ui::MouseButtonType::Middle,
                messages::MouseType::Right => cef_ui::MouseButtonType::Right,
            };
            host.send_mouse_click_event(&event, button, !down, 1)
                .expect("failed to send mouse click event");
        }
        WebSocketMessage::SetActive => {
            let mut state = browser.state.lock().unwrap();
            println!("setting browser active");
            state.active = true;
        }
        WebSocketMessage::SetIdle => {
            let mut state = browser.state.lock().unwrap();
            println!("setting browser idle");
            state.active = false;
        }
        WebSocketMessage::CreateBrowser { .. } => {
            let mut state = browser.state.lock().unwrap();
            println!("got create browser message");
            state.active = true;
            let _ = host.invalidate(cef_ui::PaintElementType::View);
        }
        WebSocketMessage::Resize { width, height } => {
            println!("Resize: ({}, {})", width, height);
            let mut state = browser.state.lock().unwrap();
            state.width = width;
            state.height = height;
            let _ = host.was_resized();
            let _ = host.invalidate(cef_ui::PaintElementType::View);
        }
        _ => {
            println!("Unknown message");
        }
    }
}
