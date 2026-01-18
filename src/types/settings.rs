use serde::{Deserialize, Serialize};

fn default_duration_val() -> String {
    "00:05:00".to_string()
}

fn default_end_time_val() -> String {
    "00:00:00".to_string()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSettings {
    pub pp_host: String,
    pub pp_port: String,
    pub ot_host: String,
    pub ot_port: String,
    #[serde(default = "default_duration_val")]
    pub default_duration: String,
    #[serde(default = "default_end_time_val")]
    pub default_end_time: String,
    #[serde(default)]
    pub favorite_durations: Vec<String>,
    #[serde(default)]
    pub favorite_end_times: Vec<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            pp_host: "localhost".to_string(),
            pp_port: "1025".to_string(),
            ot_host: "localhost".to_string(),
            ot_port: "4001".to_string(),
            default_duration: default_duration_val(),
            default_end_time: default_end_time_val(),
            favorite_durations: vec![],
            favorite_end_times: vec![],
        }
    }
}

impl AppSettings {
    /// Get the path to the settings file in the user's config directory
    fn settings_path() -> std::path::PathBuf {
        let config_dir = dirs::config_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
        let app_dir = config_dir.join("pp2ot-bridge");
        app_dir.join("settings.json")
    }

    pub fn load() -> Self {
        let path = Self::settings_path();
        if let Ok(contents) = std::fs::read_to_string(&path) {
            if let Ok(settings) = serde_json::from_str(&contents) {
                return settings;
            }
        }
        Self::default()
    }

    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::settings_path();
        // Ensure the config directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let contents = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, contents)
    }
}
