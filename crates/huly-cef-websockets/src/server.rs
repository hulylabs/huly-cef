use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use log::{self, error, info};
use tokio::{
    net::TcpListener,
    sync::mpsc::{self, UnboundedSender},
};

use huly_cef::{browser::Browser, messages::TabMessage};

mod browser;
mod tab;

enum ConnectionType {
    Browser,
    Tab(i32),
    None,
}

struct ServerState {
    cache_path: String,
    tabs: HashMap<i32, Browser>,

    tab_event_receivers: HashMap<i32, UnboundedSender<TabMessage>>,
}

/// Runs the websocket server that listens for incoming connections.
pub async fn serve(addr: String, cache_path: String) {
    let server = TcpListener::bind(addr)
        .await
        .expect("failed to start a TCP listener");

    let state = Arc::new(Mutex::new(ServerState {
        cache_path: cache_path,
        tabs: HashMap::new(),
        tab_event_receivers: HashMap::new(),
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

                if req.uri().path().contains("/tab/") {
                    if let Some(tab_id) = req.uri().path().strip_prefix("/tab/") {
                        if let Ok(id) = tab_id.parse::<i32>() {
                            connection_type = ConnectionType::Tab(id);
                        }
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
                let mut state = state.lock().unwrap();

                let (tx, rx) = mpsc::unbounded_channel::<TabMessage>();
                state.tab_event_receivers.insert(id, tx);

                tokio::spawn(tab::transfer_tab_messages(rx, websocket));
            }
            ConnectionType::None => {
                error!("unknown connection type, expected /browser or /tab/<id>");
            }
        }
    }
}
