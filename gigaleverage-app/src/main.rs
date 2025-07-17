use anyhow::Result;
use serde::{Deserialize, Serialize};
use slint::Model;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use uuid::Uuid;

// On macOS Slint 1.4+ defaults to the Regular activation policy, which makes
// Mission Control briefly switch to another Space when launching our app
// from the terminal. Set the activation policy back to `Accessory` prior to
// creating any Slint windows to keep the window in the current desktop Space.
#[cfg(target_os = "macos")]
fn install_macos_activation_policy() {
    // Build a customised Winit backend that requests the `Accessory` policy.
    // This roughly matches Slint ≤ 1.3 behaviour and avoids the Spaces
    // round-trip/flicker when starting the application from a shell.
    use i_slint_backend_winit::Backend;
    use slint::platform::set_platform;
    use winit::platform::macos::{ActivationPolicy, EventLoopBuilderExtMacOS};

    // SAFETY: `set_platform` must be called exactly once and before any other
    // Slint UI types are instantiated. We call it at the very start of `main`
    // and use `expect` to crash early if something goes wrong.
    // Construct a custom winit `EventLoopBuilder` so we can tweak macOS-specific
    // options (namely the activation policy) before Slint turns it into an
    // event loop.
    let mut elb = winit::event_loop::EventLoop::<i_slint_backend_winit::SlintEvent>::with_user_event();
    elb.with_activation_policy(ActivationPolicy::Accessory);

    let backend = Backend::builder()
        .with_event_loop_builder(elb)
        .build()
        .expect("Failed to build Slint winit backend with Accessory policy");

    set_platform(Box::new(backend)).expect("Failed to install custom Slint backend");
}

#[cfg(not(target_os = "macos"))]
fn install_macos_activation_policy() {}

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

const COINGECKO_LOGO_PATH: &str = "assets/images/coingecko-logo.png";

fn get_providers() -> Vec<Provider> {
    vec![Provider {
        id: "coingecko".into(),
        name: "CoinGecko".into(),
        logo: slint::Image::load_from_path(std::path::Path::new(COINGECKO_LOGO_PATH))
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
    // Ensure we initialise the backend with the desired activation policy *before*
    // creating any Slint windows or components.
    install_macos_activation_policy();

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

    // Transition from loading screen to API key setup after a short delay
    let ui_weak = ui.as_weak();
    slint::Timer::single_shot(Duration::from_secs(2), move || {
        let ui = ui_weak.upgrade().unwrap();
        ui.set_current_screen(Screen::ApiKeySetup.into());
    });

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
                logo: slint::Image::load_from_path(std::path::Path::new(COINGECKO_LOGO_PATH))
                    .unwrap_or_default(),
            },
            data_stream: DataStream {
                id: data_stream_id,
                name: data_stream_name,
            },
        };

        leverages_model_clone.push(new_leverage);
    });

    // Handle leverage editing (simplified)
    let ui_weak2 = ui.as_weak();
    let leverages_model_clone2 = leverages_model.clone();
    ui.on_edit_leverage(move |leverage_id| {
        let _ui = ui_weak2.upgrade().unwrap();

        // Find the leverage to edit and trigger provider selection
        for i in 0..leverages_model_clone2.row_count() {
            if let Some(leverage) = leverages_model_clone2.row_data(i) {
                if leverage.id == leverage_id {
                    // Load the data streams for the provider
                    let streams = get_data_streams_for_provider(leverage.provider.id.as_str());
                    let streams_model = std::rc::Rc::new(slint::VecModel::from(streams));
                    _ui.set_data_streams(streams_model.clone().into());
                    break;
                }
            }
        }
    });

    // Handle leverage updating
    let leverages_model_clone3 = leverages_model.clone();
    ui.on_update_leverage(
        move |leverage_id, provider_id, data_stream_id, data_stream_name| {
            // Find and update the leverage
            for i in 0..leverages_model_clone3.row_count() {
                if let Some(mut leverage) = leverages_model_clone3.row_data(i) {
                    if leverage.id == leverage_id {
                        // Update the leverage
                        leverage.provider = Provider {
                            id: provider_id.clone(),
                            name: "CoinGecko".into(),
                            logo: slint::Image::load_from_path(std::path::Path::new(
                                COINGECKO_LOGO_PATH,
                            ))
                            .unwrap_or_default(),
                        };
                        leverage.data_stream = DataStream {
                            id: data_stream_id,
                            name: data_stream_name,
                        };

                        // Update the model
                        leverages_model_clone3.set_row_data(i, leverage);
                        break;
                    }
                }
            }
        },
    );

    // Handle leverage deletion
    let leverages_model_clone4 = leverages_model.clone();
    ui.on_delete_leverage(move |leverage_id| {
        // Find and remove the leverage
        for i in 0..leverages_model_clone4.row_count() {
            if let Some(leverage) = leverages_model_clone4.row_data(i) {
                if leverage.id == leverage_id {
                    leverages_model_clone4.remove(i);
                    break;
                }
            }
        }
    });

    // Run the application
    ui.run()?;

    Ok(())
}
