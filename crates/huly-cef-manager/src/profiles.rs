use std::{fs, io, path::PathBuf};

use log::info;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Profile;

pub struct ProfileManager {
    cache_dir: String,
}

impl ProfileManager {
    pub fn new(cache_dir: String) -> Self {
        Self { cache_dir }
    }

    pub fn create(&mut self, id: &str) -> Result<(), String> {
        let profile_dir = PathBuf::from(&self.cache_dir).join(id);
        fs::create_dir_all(profile_dir)
            .map_err(|e| format!("Failed to create profile {}: {}", id, e))?;
        info!("Profile {} created successfully", id);
        Ok(())
    }

    pub fn list(&self) -> Result<Vec<String>, String> {
        Self::enumerate_dirs(self.cache_dir.clone())
            .map_err(|e| format!("Failed to list profiles: {}", e))
    }

    pub fn exists(&self, id: &str) -> bool {
        let profile_dir = PathBuf::from(&self.cache_dir).join(id);
        profile_dir.exists()
    }

    fn enumerate_dirs(cache_dir: String) -> io::Result<Vec<String>> {
        let mut dirs = Vec::new();
        for entry in fs::read_dir(cache_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                dirs.push(entry.file_name().to_string_lossy().to_string());
            }
        }
        Ok(dirs)
    }
}
