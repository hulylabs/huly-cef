use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post},
};
use clap::Parser;
use serde::Serialize;
use serde_json::json;
use tokio::sync::Notify;
use tracing::info;

use crate::{
    instances::InstanceManager,
    profiles::{Profile, ProfileManager},
};

mod instances;
mod profiles;

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

#[derive(Serialize)]
struct Response {
    status: bool,
    data: Option<serde_json::Value>,
    error: Option<String>,
}

struct ServerState {
    instances: InstanceManager,
    profiles: ProfileManager,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let args = Arguments::parse();
    info!("Starting huly-cef-manager with args: {:?}", args);

    if !PathBuf::from(&args.cache_dir).exists() {
        std::fs::create_dir_all(&args.cache_dir).expect("failed to create cache directory");
    }

    let state = Arc::new(Mutex::new(ServerState {
        instances: InstanceManager::new(args.cef_exe, args.cache_dir.clone(), args.port_range),
        profiles: ProfileManager::new(args.cache_dir),
    }));
    let app = Router::new()
        .route("/profiles/{id}", post(create_profile))
        .route("/profiles/{id}", get(get_profile))
        .route("/profiles", get(list_profiles))
        .route("/profiles/{id}/cef", get(create_cef_instance))
        .route("/profiles/{id}/cef", delete(destroy_cef_instance))
        .with_state(state.clone());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let signal = Arc::new(Notify::new());
            let signal_clone = signal.clone();
            tokio::spawn(async move {
                tokio::signal::ctrl_c().await.unwrap();
                state.lock().unwrap().instances.cleanup();
                signal_clone.notify_waiters();
            });

            signal.notified().await;
        })
        .await
        .unwrap();
}

async fn create_profile(
    State(state): State<Arc<Mutex<ServerState>>>,
    Path(id): Path<String>,
) -> (StatusCode, Json<Response>) {
    info!("Received request to create profile with ID: {}", id);
    match state.lock().unwrap().profiles.create(&id) {
        Ok(_) => {
            info!("Profile {} created successfully", id);
            (
                StatusCode::CREATED,
                Json(Response {
                    status: true,
                    data: Some(json!({ "id": id.clone() })),
                    error: None,
                }),
            )
        }
        Err(e) => {
            info!("Failed to create profile {}: {}", id, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(Response {
                    status: false,
                    data: None,
                    error: Some(e),
                }),
            )
        }
    }
}

async fn get_profile(
    State(state): State<Arc<Mutex<ServerState>>>,
    Path(id): Path<String>,
) -> (StatusCode, Json<Response>) {
    info!("Received request for profile with ID: {}", id);

    match state.lock().unwrap().profiles.get(&id).cloned() {
        Some(profile) => {
            info!("Returning profile: {:?}", profile);
            (
                StatusCode::OK,
                Json(Response {
                    status: true,
                    data: Some(json!({ "profile": profile })),
                    error: None,
                }),
            )
        }
        None => {
            info!("Profile {} not found", id);
            (
                StatusCode::NOT_FOUND,
                Json(Response {
                    status: false,
                    data: None,
                    error: Some(format!("Profile with id {} not found", id)),
                }),
            )
        }
    }
}

async fn list_profiles(
    State(state): State<Arc<Mutex<ServerState>>>,
) -> (StatusCode, Json<Response>) {
    info!("Received request to list all profiles");

    let profiles = state.lock().unwrap().profiles.list();
    if profiles.is_empty() {
        info!("No profiles found");
        return (
            StatusCode::NO_CONTENT,
            Json(Response {
                status: true,
                data: Some(json!({ "profiles": [] })),
                error: None,
            }),
        );
    }

    info!("Returning profiles: {:?}", profiles);
    let response = Response {
        status: true,
        data: Some(json!({ "profiles": profiles })),
        error: None,
    };

    (StatusCode::OK, Json(response))
}

async fn create_cef_instance(
    State(state): State<Arc<Mutex<ServerState>>>,
    Path(id): Path<String>,
) -> (StatusCode, Json<Response>) {
    info!(
        "Received request to create CEF instance for profile ID: {}",
        id
    );

    if !state.lock().unwrap().profiles.exists(&id) {
        info!("Profile with id {} does not exist", id);
        return (
            StatusCode::NOT_FOUND,
            Json(Response {
                status: false,
                data: None,
                error: Some(format!("Profile with id {} does not exist", id)),
            }),
        );
    }

    let port = state.lock().unwrap().instances.create(&id);
    match port {
        Ok(port) => {
            info!(
                "CEF instance created for profile ID: {} on port {}",
                id, port
            );
            (
                StatusCode::OK,
                Json(Response {
                    status: true,
                    data: Some(json!({ "address": format!("ws://localhost:{}/browser", port) })),
                    error: None,
                }),
            )
        }
        Err(e) => {
            info!("Failed to create CEF instance for profile ID {}: {}", id, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(Response {
                    status: false,
                    data: None,
                    error: Some(e),
                }),
            )
        }
    }
}

async fn destroy_cef_instance(
    State(state): State<Arc<Mutex<ServerState>>>,
    Path(id): Path<String>,
) -> (StatusCode, Json<Response>) {
    info!(
        "Received request to destroy CEF instance for profile ID: {}",
        id
    );

    if !state.lock().unwrap().profiles.exists(&id) {
        info!("Profile with id {} does not exist", id);
        return (
            StatusCode::NOT_FOUND,
            Json(Response {
                status: false,
                data: None,
                error: Some(format!("Profile with id {} does not exist", id)),
            }),
        );
    }

    match state.lock().unwrap().instances.destroy(&id) {
        Ok(_) => {
            info!("CEF instance for profile ID {} destroyed successfully", id);
            (
                StatusCode::OK,
                Json(Response {
                    status: true,
                    data: Some(json!({
                        "message": format!("CEF instance {} destroyed", id)
                    })),
                    error: None,
                }),
            )
        }
        Err(e) => {
            info!(
                "Failed to destroy CEF instance for profile ID {}: {}",
                id, e
            );
            (
                StatusCode::NO_CONTENT,
                Json(Response {
                    status: false,
                    data: None,
                    error: Some(e),
                }),
            )
        }
    }
}
