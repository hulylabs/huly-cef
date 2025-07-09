use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use log::{self, error, info};
use tokio::net::TcpListener;

use huly_cef::browser::Browser;

mod browser;
mod messages;
mod tab;

enum ConnectionType {
    Browser,
    Tab(i32),
    None,
}

struct ServerState {
    cache_path: String,
    tabs: HashMap<i32, Browser>,
    size: (u32, u32),
}

impl ServerState {
    fn new(cache_path: String) -> Self {
        ServerState {
            cache_path,
            tabs: HashMap::new(),
            size: (tab::DEFAULT_WIDTH, tab::DEFAULT_HEIGHT),
        }
    }
}

pub async fn serve(addr: String, cache_path: String) {
    let server = TcpListener::bind(addr)
        .await
        .expect("failed to start a TCP listener");

    let state = Arc::new(Mutex::new(ServerState::new(cache_path)));
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
                    let state = state.lock().unwrap();
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
