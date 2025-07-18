use std::{
    collections::{HashMap, HashSet},
    fs::File,
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
use tokio_tungstenite::connect_async;
use tracing::info;

#[derive(Parser, Debug)]
struct Arguments {
    #[clap(long, help = "Root directory for CEF cache storage")]
    cache_dir: String,
    #[clap(long, help = "Path to the CEF executable")]
    cef_exe: String,
    #[arg(long, value_parser = parse_port_range, help = "Port range for CEF instances in format START-END")]
    port_range: (u16, u16),
}

fn parse_port_range(s: &str) -> Result<(u16, u16), String> {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 2 {
        return Err("Port range must be in format START-END".into());
    }
    let start = parts[0].parse::<u16>().map_err(|_| "Invalid start port")?;
    let end = parts[1].parse::<u16>().map_err(|_| "Invalid end port")?;
    if start > end {
        return Err("Start port must be <= end port".into());
    }
    Ok((start, end))
}

#[derive(Default)]
struct AppState {
    cef_exe: String,
    cache_dir: String,
    available_ports: HashSet<u16>,
    ports: HashMap<String, u16>,
    instances: HashMap<String, Child>,
}

#[derive(Default, Clone)]
struct App(Arc<Mutex<AppState>>);

impl App {
    fn new(cef_exe: String, cache_dir: String, port_range: (u16, u16)) -> Self {
        Self(Arc::new(Mutex::new(AppState {
            cef_exe,
            cache_dir,
            available_ports: (port_range.0..=port_range.1).collect(),
            ports: HashMap::new(),
            instances: HashMap::new(),
        })))
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

    fn new_cef_instance(&self, id: &str) -> Result<(Child, u16), String> {
        let mut state = self.0.lock().unwrap();
        let cache_dir = format!("{}/{}", state.cache_dir, id);
        std::fs::create_dir_all(&cache_dir)
            .map_err(|e| format!("Failed to create cache directory: {}", e))?;

        let port = find_available_port(&mut state.available_ports)?;

        info!(
            "Starting new CEF instance at {} with cache path {}",
            port, state.cache_dir
        );

        let log_file = File::create(format!("{}/huly-cef-websockets.log", cache_dir))
            .map_err(|e| format!("Failed to create log file: {}", e))?;
        let error_file = File::create(format!("{}/huly-cef-websockets.error", cache_dir))
            .map_err(|e| format!("Failed to create log file: {}", e))?;

        let instance = Command::new("xvfb-run")
            .arg("-a")
            .arg(&state.cef_exe)
            .args(["--port", port.to_string().as_str()])
            .args(["--cache-path", &cache_dir])
            .args(["--no-sandbox"])
            .stdout(Stdio::from(log_file))
            .stderr(Stdio::from(error_file))
            .spawn()
            .expect("failed to start huly-cef");

        info!("CEF instance started with PID: {}", instance.id());

        return Ok((instance, port));
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let args = Arguments::parse();
    info!("Starting huly-cef-manager with args: {:?}", args);

    let state = App::new(args.cef_exe, args.cache_dir, args.port_range);
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

async fn instance_handler(State(app): State<App>, Path(id): Path<String>) -> (StatusCode, String) {
    info!("Received request for instance with ID: {}", id);

    let port = app.get_instance_port(&id);
    if let Some(port) = port {
        info!("Returning existing CEF instance for ID: {}", id);
        return (StatusCode::OK, format!("ws://localhost:{}/browser", port));
    }

    let (instance, port) = match app.new_cef_instance(&id) {
        Ok((instance, port)) => (instance, port),
        Err(e) => {
            let error = format!("Failed to create new CEF instance: {}", e);
            info!(error);
            return (StatusCode::INTERNAL_SERVER_ERROR, error);
        }
    };

    let mut healthy = false;
    let retries = 10;
    for i in 0..retries {
        if healthcheck(port).await {
            info!("CEF instance {} is healthy", id);
            healthy = true;
            break;
        } else {
            info!("Waiting for CEF instance {} to become healthy", id);
            tokio::time::sleep(tokio::time::Duration::from_secs(i)).await;
        }
    }

    if !healthy {
        let error = format!(
            "CEF instance {} did not become healthy after {} retries",
            id, retries
        );
        info!(error);
        return (StatusCode::INTERNAL_SERVER_ERROR, error);
    }

    app.set_instance(id.clone(), port, instance);
    (
        StatusCode::CREATED,
        format!("ws://localhost:{}/browser", port),
    )
}

fn find_available_port(available_ports: &mut HashSet<u16>) -> Result<u16, String> {
    let mut available_port: u16 = 0;
    for port in available_ports.iter() {
        if let Ok(_) = TcpListener::bind(format!("0.0.0.0:{}", port)) {
            available_port = *port;
        }
    }

    if available_port > 0 {
        available_ports.remove(&available_port);
        Ok(available_port)
    } else {
        Err("No available ports found".into())
    }
}

async fn healthcheck(port: u16) -> bool {
    let url = format!("ws://localhost:{}", port);
    connect_async(url).await.is_ok()
}
