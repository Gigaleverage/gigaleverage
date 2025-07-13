use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

slint::include_modules!();

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppConfig {
    api_key: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
        }
    }
}

#[derive(Clone)]
struct AppState {
    config: AppConfig,
    config_path: PathBuf,
}

#[allow(dead_code)]
impl AppState {
    fn new() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
            .join("gigaleverage");

        // Create config directory if it doesn't exist
        fs::create_dir_all(&config_dir)?;

        let config_path = config_dir.join("config.json");

        // Load existing config or create default
        let config = if config_path.exists() {
            let config_str = fs::read_to_string(&config_path)?;
            serde_json::from_str(&config_str).unwrap_or_default()
        } else {
            AppConfig::default()
        };

        Ok(Self {
            config,
            config_path,
        })
    }

    fn save_config(&self) -> Result<()> {
        let config_str = serde_json::to_string_pretty(&self.config)?;
        fs::write(&self.config_path, config_str)?;
        Ok(())
    }

    fn update_api_key(&mut self, api_key: String) -> Result<()> {
        self.config.api_key = api_key;
        self.save_config()?;
        Ok(())
    }

    fn get_api_key(&self) -> &str {
        &self.config.api_key
    }
}

fn main() -> Result<()> {
    // Initialize application state
    let _app_state = AppState::new()?;

    // Create the main window
    let ui = App::new()?;

    // Note: API key handling will be done through the UI state management
    // The Slint UI will handle the callbacks internally
    // Initial API key loading will be implemented in future updates

    // Run the application
    ui.run()?;

    Ok(())
}

// Clone implementation for AppState to work with closures
// impl Clone for AppState {
//     fn clone(&self) -> Self {
//         Self {
//             config: self.config.clone(),
//             config_path: self.config_path.clone(),
//         }
//     }
// }
