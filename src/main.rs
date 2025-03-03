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

#[derive(Debug, Serialize, Deserialize)]
enum WebSocketMessage {
    CreateBrowser {
        url: String,
        width: u32,
        height: u32,
    },
    MouseMove {
        x: i32,
        y: i32,
    },
    MouseClick {
        x: i32,
        y: i32,
        down: bool,
    },
}

fn main() -> Result<()> {
    let cef = cef::new()?;
    if let Some(code) = cef.is_cef_subprocess() {
        std::process::exit(code);
    }

    _ = cef.initialize();

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.spawn(run_server());

    cef.run_message_loop();
    cef.shutdown();

    Ok(())
}

async fn run_server() {
    let server = TcpListener::bind("127.0.0.1:8080")
        .await
        .expect("failed to start a tcp linstener");

    println!("server listening on 127.0.0.1:8080");
    loop {
        let (stream, _) = server
            .accept()
            .await
            .expect("failed to accept a tcp stream");
        let websocket = tokio_tungstenite::accept_async(stream)
            .await
            .expect("failed to accept a websocket");

        tokio::spawn(handle_connection(websocket));
    }
}

async fn handle_connection(websocket: tokio_tungstenite::WebSocketStream<TcpStream>) {
    let (mut outgoing, mut incoming) = websocket.split();

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

    let (sender, mut reader) = mpsc::unbounded_channel::<cef::Buffer>();
    let browser = cef::create_browser(width, height, &url, sender);

    tokio::spawn(handle_incoming_messages(incoming, browser));

    while let Some(buffer) = reader.recv().await {
        println!("received buffer in: {:?}", buffer.timestamp.elapsed());
        println!("channel size: {}", reader.len());

        let msg = Message::Binary(buffer.data.into());

        let now = std::time::Instant::now();
        _ = outgoing.send(msg).await;
        println!("write msg: {:?}", now.elapsed());
    }
}

async fn handle_incoming_messages(
    mut incoming: SplitStream<WebSocketStream<TcpStream>>,
    browser: cef_ui::Browser,
) {
    while let Some(msg) = incoming.next().await {
        let msg = msg.expect("failed to read a message");
        let msg = serde_json::from_slice::<WebSocketMessage>(&msg.into_data())
            .expect("got unknown message from a client");

        println!("got message: {:?}", msg);

        match msg {
            WebSocketMessage::MouseMove { x, y } => {
                let event = cef_ui::MouseEvent {
                    x: x,
                    y: y,
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
                    x: x,
                    y: y,
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
