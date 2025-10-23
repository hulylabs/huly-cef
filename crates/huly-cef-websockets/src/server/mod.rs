use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use log::{self, error, info};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{broadcast, oneshot},
};

use huly_cef::browser::Browser;

mod browser;
mod tab;

pub const WIDTH: u32 = 1280;
pub const HEIGHT: u32 = 720;

enum ConnectionType {
    Browser,
    Tab(i32),
    None,
}

struct ServerState {
    #[allow(dead_code)]
    cache_dir: String,
    tabs: HashMap<i32, Browser>,

    use_server_size: bool,
    size: (u32, u32),

    shutdown_tx: broadcast::Sender<()>,
}

struct SharedServerState(Arc<Mutex<ServerState>>);

impl Clone for SharedServerState {
    fn clone(&self) -> Self {
        SharedServerState(self.0.clone())
    }
}

impl SharedServerState {
    fn new(cache_dir: String, use_server_size: bool, shutdown_tx: broadcast::Sender<()>) -> Self {
        Self(Arc::new(Mutex::new(ServerState {
            cache_dir,
            tabs: HashMap::new(),
            use_server_size,
            size: (WIDTH, HEIGHT),
            shutdown_tx,
        })))
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, ServerState> {
        self.0.lock().unwrap()
    }

    fn set_tab(&self, id: i32, browser: Browser) {
        let mut state = self.0.lock().unwrap();
        state.tabs.insert(id, browser);
    }

    fn get_tab(&self, id: i32) -> Option<Browser> {
        let state = self.0.lock().unwrap();
        state.tabs.get(&id).cloned()
    }

    fn remove_tab(&self, id: i32) -> Option<Browser> {
        let mut state = self.0.lock().unwrap();
        state.tabs.remove(&id)
    }
}

pub async fn serve(
    addr: String,
    cache_dir: String,
    use_server_size: bool,
    shutdown_cef: oneshot::Sender<()>,
) {
    let server = TcpListener::bind(addr)
        .await
        .expect("failed to start a TCP listener");

    let (tx, mut rx) = broadcast::channel(16);
    let state = SharedServerState::new(cache_dir, use_server_size, tx);
    loop {
        tokio::select! {
            result = server.accept() => {
                let Ok((stream, _)) = result else {
                    error!("failed to accept connection: {:?}", result.err());
                    continue;
                };

                process_connection(stream, &state).await;
            },
            _ = rx.recv() => {
                info!("Shutting down server...");
                if let Err(_) = shutdown_cef.send(()) {
                    error!("failed to send shutdown signal to CEF");
                }
                break;
            }
        }
    }
}

async fn process_connection(stream: TcpStream, state: &SharedServerState) {
    let mut connection_type = ConnectionType::None;
    let websocket = tokio_tungstenite::accept_hdr_async(
        stream,
        |req: &tungstenite::http::Request<()>, resp| {
            if req.uri().path() == "/browser" {
                connection_type = ConnectionType::Browser;
            }

            if req.uri().path().contains("/tab/") {
                if let Some(id) = req
                    .uri()
                    .path()
                    .strip_prefix("/tab/")
                    .and_then(|s| s.parse::<i32>().ok())
                {
                    connection_type = ConnectionType::Tab(id);
                } else {
                    error!("Invalid path for tab connection: {}", req.uri().path());
                }
            }
            Ok(resp)
        },
    )
    .await;

    let ws = match websocket {
        Ok(ws) => ws,
        Err(e) => {
            error!("failed to upgrade connection: {:?}", e);
            return;
        }
    };

    match connection_type {
        ConnectionType::Browser => {
            info!("new browser connection established");
            tokio::spawn(browser::handle(state.clone(), ws));
        }
        ConnectionType::Tab(id) => {
            info!("new tab connection established");

            match state.get_tab(id) {
                Some(tab) => tokio::spawn(tab::event_loop(tab, ws)),
                None => {
                    error!("tab with id {} not found", id);
                    return;
                }
            };
        }
        ConnectionType::None => {
            error!("unknown connection type, expected /browser or /tab/<id>");
        }
    }
}
