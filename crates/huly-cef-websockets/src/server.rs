use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use log::{self, info};
use tokio::net::TcpListener;

use huly_cef::browser::Browser;

mod browser;
mod tab;

enum ConnectionType {
    Browser,
    Tab,
    None,
}

struct ServerState {
    cache_path: String,
    tabs: HashMap<i32, Browser>,
}

/// Runs the websocket server that listens for incoming connections.
pub async fn serve(addr: String, cache_path: String) {
    let server = TcpListener::bind(addr)
        .await
        .expect("failed to start a TCP listener");

    let state = Arc::new(Mutex::new(ServerState {
        cache_path: cache_path,
        tabs: HashMap::new(),
    }));

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

                if req.uri().path() == "/tab" {
                    connection_type = ConnectionType::Tab;
                }
                Ok(resp)
            },
        )
        .await
        .expect("failed to accept a websocket connection");

        match connection_type {
            ConnectionType::Browser => {
                info!("new browser connection established");
                tokio::spawn(browser::handle(state.clone(), websocket));
            }
            ConnectionType::Tab => {
                info!("new tab connection established");
                tokio::spawn(tab::handle(state.clone(), websocket));
            }
            ConnectionType::None => {
                panic!("unknown connection type, expected /browser or /tab");
            }
        }
    }
}
