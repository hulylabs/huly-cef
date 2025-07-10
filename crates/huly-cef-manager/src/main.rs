use std::{
    collections::HashMap,
    net::TcpListener,
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex},
};

use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    routing::get,
};
use clap::Parser;
use tokio::sync::Notify;
use tracing::info;

#[derive(Parser, Debug)]
struct Arguments {
    #[clap(long, help = "Root directory for CEF cache storage")]
    cache_dir: String,
    #[clap(long, help = "Path to the CEF executable")]
    cef_exe: String,
}

#[derive(Default)]
struct AppState {
    cef_exe: String,
    cache_dir: String,
    ports: HashMap<String, u16>,
    instances: HashMap<String, Child>,
}

#[derive(Default, Clone)]
struct SharedAppState(Arc<Mutex<AppState>>);

impl SharedAppState {
    fn new(cef_exe: String, cache_dir: String) -> Self {
        let state = AppState {
            cef_exe,
            cache_dir,
            ports: HashMap::new(),
            instances: HashMap::new(),
        };
        SharedAppState(Arc::new(Mutex::new(state)))
    }

    fn get_instance_port(&self, id: &str) -> Option<u16> {
        let state = self.0.lock().unwrap();
        state.ports.get(id).cloned()
    }

    fn set_instance(&self, id: String, port: u16, instance: Child) {
        let mut state = self.0.lock().unwrap();
        state.ports.insert(id.clone(), port);
        state.instances.insert(id, instance);
    }

    fn cleanup(&self) {
        let mut state = self.0.lock().unwrap();
        for (id, mut instance) in state.instances.drain() {
            if let Err(e) = instance.kill() {
                info!("Failed to kill instance {}: {}", id, e);
            }
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let args = Arguments::parse();
    let state = SharedAppState::new(args.cef_exe, args.cache_dir);
    let app = Router::new()
        .route("/instances/{id}", get(instance_handler))
        .with_state(state.clone());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let signal = Arc::new(Notify::new());
            let signal_clone = signal.clone();
            tokio::spawn(async move {
                tokio::signal::ctrl_c().await.unwrap();
                state.cleanup();
                signal_clone.notify_waiters();
            });

            signal.notified().await;
        })
        .await
        .unwrap();
}

async fn instance_handler(
    State(state): State<SharedAppState>,
    Path(id): Path<String>,
) -> (StatusCode, String) {
    let port = state.get_instance_port(&id);
    if let Some(port) = port {
        return (StatusCode::OK, format!("ws://localhost:{}/browser", port));
    }

    let (instance, port) = {
        let state = state.0.lock().unwrap();
        new_cef_instance(state.cef_exe.clone(), format!("{}/{}", state.cache_dir, id))
    };

    state.set_instance(id.clone(), port, instance);
    (
        StatusCode::CREATED,
        format!("ws://localhost:{}/browser", port),
    )
}

fn new_cef_instance(exe_path: String, cache_path: String) -> (Child, u16) {
    let port = find_available_port();

    info!(
        "Starting new CEF instance at {} with cache path {}",
        port, cache_path
    );

    let instance = Command::new(exe_path)
        .args(["--port", port.to_string().as_str()])
        .args(["--cache-path", cache_path.as_str()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to start huly-cef");

    info!("CEF instance started with PID: {}", instance.id());

    return (instance, port);
}

fn find_available_port() -> u16 {
    let listener = TcpListener::bind("localhost:0").expect("failed to find available port");
    return listener.local_addr().unwrap().port();
}
