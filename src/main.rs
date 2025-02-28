use std::{
    iter,
    net::{TcpListener, TcpStream},
    sync::{Arc, Condvar, Mutex},
};

use anyhow::Result;
use cef_ui::{Browser, CefTask, CefTaskCallbacks};
use crossbeam_channel::Sender;
use futures::lock;
use serde::{Deserialize, Serialize};
use tungstenite::{accept, Message};

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

struct MyTaskCallback {
    shared_state: Arc<(Mutex<Option<cef_ui::Browser>>, Condvar)>,
    width: u32,
    height: u32,
    url: String,
    sender: Sender<Vec<u8>>,
}

impl CefTaskCallbacks for MyTaskCallback {
    fn execute(&mut self) {
        let (lock, cvar) = &*self.shared_state;
        let mut browser = lock.lock().unwrap();

        *browser = Some(cef::create_browser(
            self.width,
            self.height,
            &self.url,
            self.sender.clone(),
        ));

        println!("execute_task: {:?}", std::thread::current().id());

        cvar.notify_one();
    }
}

fn create_browser_task(
    width: u32,
    height: u32,
    url: &str,
    sender: Sender<Vec<u8>>,
) -> cef_ui::Browser {
    let shared_state: Arc<(Mutex<Option<cef_ui::Browser>>, Condvar)> =
        Arc::new((Mutex::new(None), Condvar::new()));

    let d = MyTaskCallback {
        shared_state: shared_state.clone(),
        width: width,
        height: height,
        url: url.to_string(),
        sender,
    };

    let task = CefTask::new(d);

    cef_ui::post_task(cef_ui::ThreadId::UI, task);

    let (lock, cvar) = &*shared_state;
    let mut browser_ready = lock.lock().unwrap();

    while browser_ready.is_none() {
        println!("Consumer: Waiting for producer...");
        browser_ready = cvar.wait(browser_ready).unwrap();
    }

    browser_ready.take().unwrap()
}

fn run_server() {
    let server = TcpListener::bind("127.0.0.1:8080").expect("failed to bind");
    server.set_nonblocking(true).unwrap();

    let stream = loop {
        let stream = server.accept();
        match stream {
            Ok(s) => {
                break s.0;
            }
            Err(_) => (),
        }
    };

    println!("stream: {:?}", stream.local_addr().unwrap().ip());

    let mut websocket = tungstenite::accept(stream).unwrap();

    let msg = loop {
        if let Ok(msg) = websocket.read() {
            break msg;
        }
    };
    let msg = serde_json::from_slice::<WebSocketMessage>(&msg.into_data()).unwrap();
    let WebSocketMessage::CreateBrowser { url, width, height } = msg else {
        panic!("Unknown message");
    };

    let (sender, reader) = crossbeam_channel::bounded::<Vec<u8>>(1);
    let browser = create_browser_task(width, height, &url, sender);

    loop {
        let msg = websocket.read();
        println!("try read");

        if let Ok(msg) = msg {
            _ = handle_message(msg, &browser);
        }
        let now = std::time::Instant::now();
        let buffer = reader.recv().unwrap();
        println!("recv buffer: {:?}", now.elapsed());

        let msg = tungstenite::Message::Binary(buffer.into());

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
