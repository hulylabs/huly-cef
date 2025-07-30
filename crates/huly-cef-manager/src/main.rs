use std::sync::Arc;

use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get},
};
use clap::Parser;
use tokio::sync::Notify;
use tracing::info;

use crate::manager::InstanceManager;

mod manager;

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

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let args = Arguments::parse();
    info!("Starting huly-cef-manager with args: {:?}", args);

    let state = InstanceManager::new(args.cef_exe, args.cache_dir, args.port_range);
    let app = Router::new()
        .route("/instances", get(list_instances))
        .route("/instance/{id}", get(create_instance))
        .route("/instance/{id}", delete(destroy_instance))
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

async fn list_instances(State(manager): State<InstanceManager>) -> (StatusCode, String) {
    info!("Received request to list all CEF instances");

    let ids = manager.get_instance_ids();
    if ids.is_empty() {
        info!("No CEF instances found");
        return (StatusCode::NO_CONTENT, "No instances found".to_string());
    }

    match serde_json::to_string(&ids) {
        Ok(json) => (StatusCode::OK, json),
        Err(e) => {
            info!("Failed to serialize instance IDs: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to serialize instance IDs".into())
        }
    }
}

async fn create_instance(
    State(manager): State<InstanceManager>,
    Path(id): Path<String>,
) -> (StatusCode, String) {
    info!("Received request for instance with ID: {}", id);

    let port = manager.get_port(&id);
    if let Some(port) = port {
        info!("Returning existing CEF instance with ID: {}", id);
        return (StatusCode::OK, format!("ws://localhost:{}/browser", port));
    }

    let id_clone = id.clone();
    let port = tokio::task::spawn_blocking(move || manager.create_instance(&id_clone))
        .await
        .unwrap_or_else(|e| Err(format!("Failed to create instance: {}", e)));

    match port {
        Ok(port) => {
            info!("Created new CEF instance with ID: {} on port {}", id, port);
            (StatusCode::OK, format!("ws://localhost:{}/browser", port))
        }
        Err(e) => {
            info!("Failed to create CEF instance with ID: {}: {}", id, e);
            (StatusCode::INTERNAL_SERVER_ERROR, e)
        }
    }
}

async fn destroy_instance(
    State(manager): State<InstanceManager>,
    Path(id): Path<String>,
) -> (StatusCode, String) {
    info!("Received request to destroy instance with ID: {}", id);

    let id_clone = id.clone();
    let result = tokio::task::spawn_blocking(move || manager.destroy_instance(&id_clone))
        .await
        .unwrap_or_else(|e| Err(format!("Failed to destroy instance: {}", e)));

    match result {
        Ok(_) => {
            info!("Successfully destroyed instance with ID: {}", id);
            (StatusCode::OK, format!("Instance {} destroyed", id))
        }
        Err(e) => {
            info!("Failed to destroy instance with ID {}: {}", id, e);
            (StatusCode::NO_CONTENT, e)
        }
    }
}
