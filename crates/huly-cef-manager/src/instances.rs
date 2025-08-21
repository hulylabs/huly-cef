use std::{
    collections::{HashMap, HashSet},
    fs::create_dir_all,
    net::TcpListener,
    path::Path,
    process::{Child, Command, Stdio},
};

use log::info;
use tokio_tungstenite::tungstenite::connect;

#[derive(Default)]
pub struct InstanceManager {
    cef_exe: String,
    cache_dir: String,
    use_server_size: bool,
    available_ports: HashSet<u16>,
    ports: HashMap<String, u16>,
    instances: HashMap<String, Child>,
}

impl InstanceManager {
    pub fn new(cef_exe: String, cache_dir: String, port_range: (u16, u16), use_server_size: bool) -> Self {
        Self {
            cef_exe,
            cache_dir,
            use_server_size,
            available_ports: (port_range.0..=port_range.1).collect(),
            ports: HashMap::new(),
            instances: HashMap::new(),
        }
    }

    pub fn create(&mut self, id: &str, host: &str) -> Result<u16, String> {
        if self.instances.contains_key(id) {
            return Ok(self.ports.get(id).cloned().expect("port can't be None"));
        }

        let port = self.find_available_port()?;
        let instance = self.start_cef_instance(id, port)?;

        let mut healthy = false;
        let retries = 10;
        for i in 0..retries {
            if healthcheck(host, port) {
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

        self.instances.insert(id.to_string(), instance);
        self.ports.insert(id.to_string(), port);
        self.available_ports.remove(&port);

        Ok(port)
    }

    pub fn destroy(&mut self, id: &str) -> Result<(), String> {
        let instance_to_remove = {
            if let Some(port) = self.ports.remove(id) {
                self.available_ports.insert(port);
                self.instances.remove(id)
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

    pub fn cleanup(&mut self) {
        for (id, mut instance) in self.instances.drain() {
            if let Err(e) = instance.kill() {
                info!("Failed to kill instance {}: {}", id, e);
            }
        }
    }

    fn start_cef_instance(&self, id: &str, port: u16) -> Result<Child, String> {
        let cache_dir = format!("{}/{}", self.cache_dir, id);
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

        let mut args = vec![
            String::from("--port"),
            port.to_string(),
            String::from("--cache-path"),
            cache_dir,
            String::from("--no-sandbox"),
        ];

        if self.use_server_size {
            args.push(String::from("--use-server-size"));
        }
        
        let instance = Command::new(&self.cef_exe)
            .args(args)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("Failed to start CEF instance: {}", e))?;

        Ok(instance)
    }

    fn find_available_port(&mut self) -> Result<u16, String> {
        let available_ports = &mut self.available_ports;

        for port in available_ports.iter() {
            if let Ok(_) = TcpListener::bind(format!("0.0.0.0:{}", port)) {
                return Ok(*port);
            }
        }

        Err("No available ports found".into())
    }
}

fn healthcheck(host: &str, port: u16) -> bool {
    let url = format!("ws://localhost:{}", port);
    info!("Healthchecking CEF instance at {}", url);
    let result = connect(url.clone()).is_ok();
    info!("Healthcheck result for {}: {}", url, result);

    let url = format!("ws://{}:{}", host, port);
    info!("Healthchecking CEF instance at {}", url);
    let result = connect(url.clone()).is_ok();
    info!("Healthcheck result for {}: {}", url, result);
    result
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
