use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use log::{self, error, info};
use tokio::net::TcpListener;

use huly_cef::browser::Browser;

mod browser;
mod tab;

enum ConnectionType {
    Browser,
    Tab(i32),
    None,
}

struct ServerState {
    cache_dir: String,
    tabs: HashMap<i32, Browser>,
}

struct SharedServerState(Arc<Mutex<ServerState>>);

impl Clone for SharedServerState {
    fn clone(&self) -> Self {
        SharedServerState(self.0.clone())
    }
}

impl SharedServerState {
    fn new(cache_dir: String) -> Self {
        Self(Arc::new(Mutex::new(ServerState {
            cache_dir,
            tabs: HashMap::new(),
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

pub async fn serve(addr: String, cache_dir: String) {
    let server = TcpListener::bind(addr)
        .await
        .expect("failed to start a TCP listener");

    let state = SharedServerState::new(cache_dir);
    loop {
        let (stream, _) = server
            .accept()
            .await
            .expect("failed to accept a TCP stream");

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

        let websocket = match websocket {
            Ok(ws) => ws,
            Err(e) => {
                error!("failed to upgrade connection: {:?}", e);
                continue;
            }
        };

        match connection_type {
            ConnectionType::Browser => {
                info!("new browser connection established");
                tokio::spawn(browser::handle(state.clone(), websocket));
            }
            ConnectionType::Tab(id) => {
                info!("new tab connection established");

                let tab = {
                    let state = state.lock();
                    state.tabs.get(&id).cloned()
                };
                let Some(tab) = tab else {
                    error!("tab with id {} not found", id);
                    continue;
                };
                tokio::spawn(tab::event_loop(tab, websocket));
            }
            ConnectionType::None => {
                error!("unknown connection type, expected /browser or /tab/<id>");
            }
        }
    }
}
