use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get},
};
use clap::Parser;
use log::{SetLoggerError, info};
use log4rs::{
    Config,
    append::console::ConsoleAppender,
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
};
use serde::Serialize;
use serde_json::json;
use tokio::sync::Notify;

use crate::{instances::InstanceManager, profiles::ProfileManager};

mod instances;
mod profiles;

#[derive(Parser, Debug, Clone)]
struct Arguments {
    #[clap(long, env = "CACHE_DIR", help = "Root directory for CEF cache storage")]
    cache_dir: String,
    #[clap(long, env = "CEF_EXE", help = "Path to the CEF executable")]
    cef_exe: String,
    #[clap(long, env = "PORT_RANGE", value_parser = parse_port_range, help = "Port range for CEF instances in format START-END")]
    port_range: (u16, u16),
    #[clap(long, env = "HOST", help = "Huly CEF servers and Manager host")]
    host: String,
    #[clap(long, env = "MANAGER_PORT", help = "Huly CEF Manager port")]
    manager_port: u16,
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

fn setup_logging() -> Result<log4rs::Handle, SetLoggerError> {
    let stdout_pattern = "\x1b[90m{d(%H:%M:%S)} \x1b[0m{h({l})} \x1b[90m{f}:{L} \x1b[0m{m}{n}";
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(stdout_pattern)))
        .build();

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(
            Root::builder()
                .appender("stdout")
                .build(log::LevelFilter::Info),
        )
        .unwrap();

    log4rs::init_config(config)
}

#[derive(Serialize)]
struct Response {
    status: bool,
    data: Option<serde_json::Value>,
    error: Option<String>,
}

impl Response {
    fn new(data: serde_json::Value) -> Self {
        Self {
            status: true,
            data: Some(data),
            error: None,
        }
    }

    fn new_with_error(error: String) -> Self {
        Self {
            status: false,
            data: None,
            error: Some(error),
        }
    }
}

struct ServerState {
    args: Arguments,
    instances: InstanceManager,
    profiles: ProfileManager,
}

#[tokio::main]
async fn main() {
    setup_logging().expect("Failed to set up logging");

    let args = Arguments::parse();
    info!("Starting huly-cef-manager with args: {:?}", args);

    if !PathBuf::from(&args.cache_dir).exists() {
        std::fs::create_dir_all(&args.cache_dir).expect("failed to create cache directory");
    }

    let state = Arc::new(Mutex::new(ServerState {
        args: args.clone(),
        instances: InstanceManager::new(args.cef_exe, args.cache_dir.clone(), args.port_range),
        profiles: ProfileManager::new(args.cache_dir),
    }));

    let app = Router::new()
        .route("/profiles", get(list_profiles))
        .route("/profiles/{id}/cef", get(create_cef_instance))
        .route("/profiles/{id}/cef", delete(destroy_cef_instance))
        .with_state(state.clone());

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", args.manager_port))
        .await
        .unwrap();

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

async fn list_profiles(
    State(state): State<Arc<Mutex<ServerState>>>,
) -> (StatusCode, Json<Response>) {
    info!("Received request to list all profiles");

    let Ok(profiles) = state.lock().unwrap().profiles.list() else {
        info!("Failed to list profiles");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(Response::new_with_error(
                "Failed to list profiles".to_string(),
            )),
        );
    };

    info!("Returning profiles: {:?}", profiles);
    (
        StatusCode::OK,
        Json(Response::new(json!({ "profiles": profiles }))),
    )
}

async fn create_cef_instance(
    State(state): State<Arc<Mutex<ServerState>>>,
    Path(id): Path<String>,
) -> (StatusCode, Json<Response>) {
    info!(
        "Received request to create CEF instance for profile ID: {}",
        id
    );

    {
        let mut state = state.lock().unwrap();
        if !state.profiles.exists(&id) {
            info!(
                "Profile with id {} does not exist, creating new profile",
                id
            );
            if let Err(e) = state.profiles.create(&id) {
                info!("Failed to create profile with id {}: {}", id, e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(Response::new_with_error(e)),
                );
            };
            info!("Profile with id {} created", id);
        }
    }

    let port = state.lock().unwrap().instances.create(&id);
    match port {
        Ok(port) => {
            info!(
                "CEF instance created for profile ID: {} on port {}",
                id, port
            );
            let address = format!("ws://{}:{}/browser", state.lock().unwrap().args.host, port);
            (
                StatusCode::OK,
                Json(Response::new(json!({ "address": address }))),
            )
        }
        Err(e) => {
            info!("Failed to create CEF instance for profile ID {}: {}", id, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(Response::new_with_error(e)),
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
            Json(Response::new_with_error(format!(
                "Profile with id {} does not exist",
                id
            ))),
        );
    }

    match state.lock().unwrap().instances.destroy(&id) {
        Ok(_) => {
            info!("CEF instance for profile ID {} destroyed successfully", id);
            (
                StatusCode::OK,
                Json(Response::new(json!({
                    "message": format!("CEF instance {} destroyed", id)
                }))),
            )
        }
        Err(e) => {
            info!(
                "Failed to destroy CEF instance for profile ID {}: {}",
                id, e
            );
            (StatusCode::NO_CONTENT, Json(Response::new_with_error(e)))
        }
    }
}
