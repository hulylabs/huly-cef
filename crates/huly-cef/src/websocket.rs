use futures::{stream::SplitStream, SinkExt, StreamExt};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc,
};
use tokio_tungstenite::WebSocketStream;
use tungstenite::Message;

use crate::cef;

use crate::cef::messages::BrowserMessage;
use crate::cef::messages::CefMessage;
use crate::cef::messages::MouseType;

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

    let msg = serde_json::from_slice::<BrowserMessage>(&msg.into_data())
        .expect("got unknown message from a client");

    let BrowserMessage::CreateBrowser { url, width, height } = msg else {
        panic!("Unknown message");
    };
    println!(
        "got create browser message: ({}, {}, {})",
        width, height, url
    );

    // Create a browser
    let (cef_message_sender, mut cef_message_reader) = mpsc::unbounded_channel::<CefMessage>();
    let browser = cef::create_browser(width, height, &url, cef_message_sender);

    tokio::spawn(handle_incoming_messages(incoming, browser));

    while let Some(message) = cef_message_reader.recv().await {
        let msg = match message {
            CefMessage::Render(buffer) => Message::Binary(buffer.into()),
            CefMessage::Close => {
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

        let msg = serde_json::from_slice::<BrowserMessage>(&msg.into_data())
            .expect("got unknown message from a client");

        let closed = process_message(msg, &browser);
        if closed {
            break;
        }
    }

    _ = browser.inner.get_host().unwrap().close_browser(false);

    println!("====================================");
    println!("handle_incoming_messages is finished");
    println!("====================================");
}

fn process_message(msg: BrowserMessage, browser: &cef::Browser) -> bool {
    let host = browser.inner.get_host().unwrap();

    match msg {
        BrowserMessage::MouseMove { x, y } => {
            let event = cef_ui::MouseEvent {
                x,
                y,
                modifiers: cef_ui::EventFlags::empty(),
            };
            host.send_mouse_move_event(&event, false)
                .expect("failed to send mouse move event");
        }
        BrowserMessage::MouseClick { x, y, button, down } => {
            let event = cef_ui::MouseEvent {
                x,
                y,
                modifiers: cef_ui::EventFlags::empty(),
            };

            let button = match button {
                MouseType::Left => cef_ui::MouseButtonType::Left,
                MouseType::Middle => cef_ui::MouseButtonType::Middle,
                MouseType::Right => cef_ui::MouseButtonType::Right,
            };
            host.send_mouse_click_event(&event, button, !down, 1)
                .expect("failed to send mouse click event");
        }
        BrowserMessage::MouseWheel { x, y, dx, dy } => {
            let event = cef_ui::MouseEvent {
                x,
                y,
                modifiers: cef_ui::EventFlags::empty(),
            };
            host.send_mouse_wheel_event(&event, dx, -dy)
                .expect("failed to send mouse wheel event");
        }
        BrowserMessage::KeyPress {
            character,
            code,
            down,
        } => {
            println!("keypress: ({}, {}, {})", character, code, down);
            let event_type = if down {
                cef_ui::KeyEventType::KeyDown
            } else {
                cef_ui::KeyEventType::KeyUp
            };
            let mut event = cef_ui::KeyEvent {
                event_type: event_type,
                modifiers: cef_ui::EventFlags::empty(),
                windows_key_code: code.into(),
                native_key_code: code as i32,
                is_system_key: false,
                character: character,
                unmodified_character: character,
                focus_on_editable_field: false,
            };

            _ = host.send_key_event(event.clone());

            if event_type == cef_ui::KeyEventType::KeyDown && character != 0 {
                event.event_type = cef_ui::KeyEventType::Char;
                _ = host.send_key_event(event);
            }
        }
        BrowserMessage::SetActive => {
            let mut state = browser.state.lock().unwrap();
            println!("setting browser active");
            state.active = true;
        }
        BrowserMessage::SetIdle => {
            let mut state = browser.state.lock().unwrap();
            println!("setting browser idle");
            state.active = false;
        }
        BrowserMessage::CreateBrowser { .. } => {
            let mut state = browser.state.lock().unwrap();
            println!("got create browser message");
            state.active = true;
            let _ = host.invalidate(cef_ui::PaintElementType::View);
        }
        BrowserMessage::Resize { width, height } => {
            println!("Resize: ({}, {})", width, height);
            let mut state = browser.state.lock().unwrap();
            state.width = width;
            state.height = height;
            let _ = host.was_resized();
            let _ = host.invalidate(cef_ui::PaintElementType::View);
        }
        BrowserMessage::Close => return true,
        BrowserMessage::GoBack => {
            println!("GoBack");
            let _ = browser.inner.go_back();
        }
        BrowserMessage::GoForward => {
            println!("GoForward");
            let _ = browser.inner.go_forward();
        }
        BrowserMessage::Reload => {
            println!("Reload");
            let _ = browser.inner.reload();
        }
        _ => {
            panic!("Unknown message: {:?}", msg);
        }
    }

    return false;
}
