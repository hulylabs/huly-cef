use anyhow::Result;
use futures::{stream::SplitStream, SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc,
};
use tokio_tungstenite::WebSocketStream;
use tungstenite::Message;

mod cef;

/// Represents different types of messages that can be sent over the WebSocket.
#[derive(Debug, Serialize, Deserialize)]
enum WebSocketMessage {
    /// Message to create a new browser instance.
    CreateBrowser {
        url: String,
        width: u32,
        height: u32,
    },
    /// Message to indicate a mouse movement event.
    MouseMove { x: i32, y: i32 },
    /// Message to indicate a mouse click event.
    MouseClick { x: i32, y: i32, down: bool },
}

fn main() -> Result<()> {
    let cef = cef::new()?;

    // If running as a CEF subprocess, exit with the appropriate code.
    if let Some(code) = cef.is_cef_subprocess() {
        std::process::exit(code);
    }

    // Initialize CEF.
    _ = cef.initialize();

    // Create a Tokio runtime and spawn the WebSocket server.
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.spawn(run_server());

    // Run the CEF message loop and shut down when done.
    cef.run_message_loop();
    cef.shutdown();

    Ok(())
}

/// Runs the WebSocket server that listens for incoming connections.
async fn run_server() {
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

    // Create a browser
    let (sender, mut reader) = mpsc::unbounded_channel::<cef::Buffer>();
    let browser = cef::create_browser(width, height, &url, sender);

    // Spawn a task to handle incoming WebSocket messages for this connection.
    tokio::spawn(handle_incoming_messages(incoming, browser));

    // Process and send rendered frames to the WebSocket client.
    while let Some(buffer) = reader.recv().await {
        let msg = Message::Binary(buffer.data.into());
        _ = outgoing.send(msg).await;
    }
}

/// Handles incoming WebSocket messages and processes browser events.
async fn handle_incoming_messages(
    mut incoming: SplitStream<WebSocketStream<TcpStream>>,
    browser: cef_ui::Browser,
) {
    while let Some(msg) = incoming.next().await {
        let msg = msg.expect("failed to read a message");
        let msg = serde_json::from_slice::<WebSocketMessage>(&msg.into_data())
            .expect("got unknown message from a client");

        match msg {
            WebSocketMessage::MouseMove { x, y } => {
                let event = cef_ui::MouseEvent {
                    x,
                    y,
                    modifiers: cef_ui::EventFlags::empty(),
                };
                browser
                    .get_host()
                    .unwrap()
                    .send_mouse_move_event(&event, false)
                    .expect("failed to send mouse move event");
            }
            WebSocketMessage::MouseClick { x, y, down } => {
                let event = cef_ui::MouseEvent {
                    x,
                    y,
                    modifiers: cef_ui::EventFlags::empty(),
                };
                browser
                    .get_host()
                    .unwrap()
                    .send_mouse_click_event(&event, cef_ui::MouseButtonType::Left, !down, 1)
                    .expect("failed to send mouse click event");
            }
            _ => {
                println!("Unknown message");
            }
        }
    }
}
