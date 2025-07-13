use anyhow::Result;
use serde::{Deserialize, Serialize};
use slint::SharedString;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

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

fn get_providers() -> Vec<Provider> {
    vec![Provider {
        id: "coingecko".into(),
        name: "CoinGecko".into(),
        logo: slint::Image::load_from_path(std::path::Path::new("assets/coingecko-logo.png"))
            .unwrap_or_default(),
    }]
}

fn get_data_streams_for_provider(provider_id: &str) -> Vec<DataStream> {
    match provider_id {
        "coingecko" => vec![
            DataStream {
                id: "btc-price".into(),
                name: "BTC Price".into(),
            },
            DataStream {
                id: "eth-price".into(),
                name: "ETH Price".into(),
            },
        ],
        _ => vec![],
    }
}

fn main() -> Result<()> {
    // Initialize application state
    let _app_state = AppState::new()?;

    // Create the main window
    let ui = App::new()?;

    // Set up initial providers
    let providers = get_providers();
    let providers_model = std::rc::Rc::new(slint::VecModel::from(providers));
    ui.set_providers(providers_model.clone().into());

    // Set up initial empty leverages
    let leverages: Vec<Leverage> = Vec::new();
    let leverages_model = std::rc::Rc::new(slint::VecModel::from(leverages));
    ui.set_leverages(leverages_model.clone().into());

    // Handle provider selection
    let ui_weak = ui.as_weak();
    ui.on_provider_selected(move |provider_id| {
        let ui = ui_weak.upgrade().unwrap();
        let streams = get_data_streams_for_provider(provider_id.as_str());
        let streams_model = std::rc::Rc::new(slint::VecModel::from(streams));
        ui.set_data_streams(streams_model.clone().into());
    });

    // Handle leverage creation
    let leverages_model_clone = leverages_model.clone();
    ui.on_create_leverage(move |provider_id, data_stream_id, data_stream_name| {
        let new_leverage = Leverage {
            id: Uuid::new_v4().to_string().into(),
            provider: Provider {
                id: provider_id.clone(),
                name: "CoinGecko".into(),
                logo: slint::Image::load_from_path(std::path::Path::new(
                    "assets/coingecko-logo.png",
                ))
                .unwrap_or_default(),
            },
            data_stream: DataStream {
                id: data_stream_id,
                name: data_stream_name,
            },
        };

        leverages_model_clone.push(new_leverage);
    });

    // Run the application
    ui.run()?;

    Ok(())
}
