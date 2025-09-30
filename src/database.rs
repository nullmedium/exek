use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppUsage {
    pub launch_count: u32,
    pub last_launched: Option<DateTime<Utc>>,
}

impl Default for AppUsage {
    fn default() -> Self {
        Self {
            launch_count: 0,
            last_launched: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Database {
    pub usage: HashMap<String, AppUsage>,
}

impl Database {
    pub fn new() -> Self {
        Self {
            usage: HashMap::new(),
        }
    }

    pub fn load() -> Result<Self> {
        let path = Self::db_path()?;
        if path.exists() {
            let data = fs::read_to_string(&path)?;
            Ok(serde_json::from_str(&data)?)
        } else {
            Ok(Self::new())
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::db_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_string_pretty(&self)?;
        fs::write(&path, data)?;
        Ok(())
    }

    pub fn record_launch(&mut self, app_name: &str) -> Result<()> {
        let usage = self.usage.entry(app_name.to_string()).or_default();
        usage.launch_count += 1;
        usage.last_launched = Some(Utc::now());
        self.save()
    }

    pub fn record_path_launch(&mut self, path: &std::path::Path) -> Result<()> {
        // Store with a special prefix to distinguish path-based launches
        let key = format!("path:{}", path.display());
        let usage = self.usage.entry(key).or_default();
        usage.launch_count += 1;
        usage.last_launched = Some(Utc::now());
        self.save()
    }

    pub fn get_usage(&self, app_name: &str) -> AppUsage {
        self.usage
            .get(app_name)
            .cloned()
            .unwrap_or_default()
    }

    pub fn calculate_frecency(&self, app_name: &str) -> f64 {
        let usage = self.get_usage(app_name);
        let base_score = usage.launch_count as f64;

        if let Some(last_launched) = usage.last_launched {
            let now = Utc::now();
            let days_ago = (now - last_launched).num_days() as f64;

            let recency_multiplier = if days_ago < 1.0 {
                2.0
            } else if days_ago < 7.0 {
                1.5
            } else if days_ago < 30.0 {
                1.0
            } else if days_ago < 90.0 {
                0.5
            } else {
                0.25
            };

            base_score * recency_multiplier
        } else {
            0.0
        }
    }

    pub fn get_frequent_paths(&self) -> Vec<(String, AppUsage)> {
        self.usage
            .iter()
            .filter(|(key, _)| key.starts_with("path:"))
            .map(|(key, usage)| (key.strip_prefix("path:").unwrap_or(key).to_string(), usage.clone()))
            .collect()
    }

    fn db_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;
        Ok(config_dir.join("exek").join("database.json"))
    }
}
