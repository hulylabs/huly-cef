use std::{
    collections::{HashMap, HashSet},
    fs::{File, create_dir_all},
    net::TcpListener,
    path::Path,
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex},
};

use tokio_tungstenite::tungstenite::connect;
use tracing::info;

#[cfg(unix)]
use nix::{
    sys::signal::{self, Signal},
    unistd::Pid,
};

#[derive(Default)]
struct State {
    cef_exe: String,
    cache_dir: String,
    available_ports: HashSet<u16>,
    ports: HashMap<String, u16>,
    instances: HashMap<String, Child>,
}

#[derive(Clone)]
pub struct InstanceManager(Arc<Mutex<State>>);

impl InstanceManager {
    pub fn new(cef_exe: String, cache_dir: String, port_range: (u16, u16)) -> Self {
        Self(Arc::new(Mutex::new(State {
            cef_exe,
            cache_dir,
            available_ports: (port_range.0..=port_range.1).collect(),
            ports: HashMap::new(),
            instances: HashMap::new(),
        })))
    }

    pub fn get_port(&self, id: &str) -> Option<u16> {
        let state = self.0.lock().unwrap();
        state.ports.get(id).cloned()
    }

    pub fn create_instance(&self, id: &str) -> Result<u16, String> {
        let port = self.find_available_port()?;
        let instance = self.start_cef_instance(id, port)?;

        let mut healthy = false;
        let retries = 10;
        for i in 0..retries {
            if healthcheck(port) {
                info!("CEF instance {} is healthy", id);
                healthy = true;
                break;
            } else {
                info!("Waiting for CEF instance {} to become healthy", id);
                std::thread::sleep(tokio::time::Duration::from_secs(i));
            }
        }

        if !healthy {
            return Err(format!(
                "CEF instance {} did not become healthy after {} retries",
                id, retries
            ));
        }

        let mut state = self.0.lock().unwrap();
        state.instances.insert(id.to_string(), instance);
        state.ports.insert(id.to_string(), port);
        state.available_ports.remove(&port);

        Ok(port)
    }

    pub fn destroy_instance(&self, id: &str) -> Result<(), String> {
        let instance_to_remove = {
            let mut state = self.0.lock().unwrap();
            if let Some(port) = state.ports.remove(id) {
                state.available_ports.insert(port);
                state.instances.remove(id)
            } else {
                None
            }
        };

        match instance_to_remove {
            None => Err(format!("No CEF instance found with ID: {}", id)),
            Some(mut instance) => {
                if let Err(e) = instance.kill() {
                    info!("Failed to kill instance {}: {}", id, e);
                }
                Ok(())
            }
        }
    }

    pub fn cleanup(&self) {
        let mut state = self.0.lock().unwrap();
        for (id, mut instance) in state.instances.drain() {
            if let Err(e) = instance.kill() {
                info!("Failed to kill instance {}: {}", id, e);
            }
        }
    }

    fn start_cef_instance(&self, id: &str, port: u16) -> Result<Child, String> {
        let state = self.0.lock().unwrap();
        let cache_dir = format!("{}/{}", state.cache_dir, id);
        create_dir_all(&cache_dir).map_err(|e| {
            format!(
                "Failed to create cache directory for instance with ID {}: {}",
                id, e
            )
        })?;

        remove_cache_locks(&cache_dir);

        info!(
            "Starting new CEF instance at {} with cache path {}",
            port, cache_dir
        );

        let log_file = File::create(format!("{}/huly-cef-websockets.log", cache_dir))
            .map_err(|e| format!("Failed to create log file: {}", e))?;
        let error_file = File::create(format!("{}/huly-cef-websockets.error", cache_dir))
            .map_err(|e| format!("Failed to create log file: {}", e))?;

        let instance = Command::new(&state.cef_exe)
            .args(["--port", port.to_string().as_str()])
            .args(["--cache-path", &cache_dir])
            .args(["--no-sandbox"])
            .stdout(Stdio::from(log_file))
            .stderr(Stdio::from(error_file))
            .spawn()
            .map_err(|e| format!("Failed to start CEF instance: {}", e))?;

        Ok(instance)
    }

    fn find_available_port(&self) -> Result<u16, String> {
        let mut state = self.0.lock().unwrap();
        let available_ports = &mut state.available_ports;

        for port in available_ports.iter() {
            if let Ok(_) = TcpListener::bind(format!("0.0.0.0:{}", port)) {
                return Ok(*port);
            }
        }

        Err("No available ports found".into())
    }
}

fn healthcheck(port: u16) -> bool {
    let url = format!("ws://localhost:{}", port);
    connect(url).is_ok()
}

fn remove_cache_locks(cache_dir: &str) {
    let path = Path::new(cache_dir);

    std::fs::remove_file(path.join("SingletonLock"))
        .map_err(|e| info!("Failed to remove SingletonLock: {}", e))
        .ok();
    std::fs::remove_file(path.join("SingletonCookie"))
        .map_err(|e| info!("Failed to remove SingletonCookie: {}", e))
        .ok();
    std::fs::remove_file(path.join("SingletonSocket"))
        .map_err(|e| info!("Failed to remove SingletonSocket: {}", e))
        .ok();
}
