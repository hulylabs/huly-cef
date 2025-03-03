use std::{
    net::{TcpListener, TcpStream},
    sync::{Arc, Condvar, Mutex},
};

use anyhow::Result;
use cef_ui::{CefTask, CefTaskCallbacks};
use crossbeam_channel::Sender;
use image::codecs::webp;
use serde::{Deserialize, Serialize};
use tungstenite::Message;

mod cef;

#[derive(Serialize, Deserialize)]
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

    println!("cef init: {:?}", std::thread::current().id());

    std::thread::spawn(run_server);

    cef.run_message_loop();
    cef.shutdown();

    Ok(())
}

fn run_server() {
    let server = TcpListener::bind("127.0.0.1:8080").expect("failed to start a tcp linstener");
    for stream in server.incoming() {
        let stream = stream.expect("failed to accept a tcp stream");
        let websocket = tungstenite::accept(stream).expect("failed to accept a websocket");
        std::thread::spawn(move || handle_connection(websocket));
    }
}

fn handle_connection(mut websocket: tungstenite::WebSocket<TcpStream>) {
    let msg = websocket.read().expect("failed to read a message");
    let msg = serde_json::from_slice::<WebSocketMessage>(&msg.into_data())
        .expect("got unknown message from a client");
    let WebSocketMessage::CreateBrowser { url, width, height } = msg else {
        panic!("Unknown message");
    };

    let (sender, reader) = crossbeam_channel::unbounded::<cef::Buffer>();
    let browser = cef::create_browser(width, height, &url, sender);

    loop {
        let buffer = reader.recv().unwrap();
        println!("received buffer in: {:?}", buffer.timestamp.elapsed());
        println!("channel size: {}", reader.len());

        let msg = tungstenite::Message::Binary(buffer.data.into());

        let now = std::time::Instant::now();
        _ = websocket.write(msg);
        println!("write msg: {:?}", now.elapsed());
    }
}

fn handle_message(msg: Message, browser: &cef_ui::Browser) -> Result<()> {
    let msg = serde_json::from_slice::<WebSocketMessage>(&msg.into_data())?;

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

            println!("MouseMove: {x}, {y}");
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

    Ok(())
}
