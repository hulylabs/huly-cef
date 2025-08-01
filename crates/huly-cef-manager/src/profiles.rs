use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Profile {
    name: String,
}

pub struct ProfileManager {
    cache_dir: String,
    profiles: HashMap<String, Profile>,
}

impl ProfileManager {
    pub fn new(cache_dir: String) -> Self {
        let mut profiles = Self::restore(cache_dir.clone()).unwrap_or_else(|e| {
            info!("Failed to restore profiles: {}", e);
            HashMap::new()
        });

        profiles
            .entry("Default".to_string())
            .or_insert_with(|| Profile {
                name: "Default".to_string(),
            });

        Self {
            cache_dir,
            profiles,
        }
    }

    pub fn create(&mut self, id: &str) -> Result<(), String> {
        if self.profiles.contains_key(id) {
            return Err(format!("Profile {} already exists", id));
        }
        self.profiles.insert(
            id.to_string(),
            Profile {
                name: id.to_string(),
            },
        );
        match self.save() {
            Err(e) => info!("Failed to save profile {}: {}", id, e),
            _ => {}
        };
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<&Profile> {
        self.profiles.get(id)
    }

    pub fn list(&self) -> Vec<String> {
        self.profiles.keys().cloned().collect()
    }

    pub fn exists(&self, id: &str) -> bool {
        self.profiles.contains_key(id)
    }

    fn save(&self) -> Result<(), String> {
        let json = serde_json::to_string(&self.profiles).map_err(|e| e.to_string())?;
        std::fs::write(format!("{}/profiles.json", self.cache_dir), json).map_err(|e| e.to_string())
    }

    fn restore(cache_dir: String) -> Result<HashMap<String, Profile>, String> {
        let data = std::fs::read_to_string(format!("{}/profiles.json", cache_dir))
            .map_err(|e| e.to_string())?;
        serde_json::from_str::<HashMap<String, Profile>>(&data).map_err(|e| e.to_string())
    }
}
