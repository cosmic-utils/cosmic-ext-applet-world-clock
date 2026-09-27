use cosmic::cosmic_config::{
    self, Config, CosmicConfigEntry, cosmic_config_derive::CosmicConfigEntry,
};
use serde::{Deserialize, Serialize};

const CONFIG_VERSION: u64 = 1;

pub(crate) const APP_ID: &str = "io.github.cosmic-utils.cosmic-ext-applet-world-clock";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct ClockConfig {
    /// IANA timezone name, e.g. "UTC" or "America/New_York".
    pub(crate) timezone: String,
    /// Optional short label shown next to the time. Falls back to the timezone name.
    pub(crate) label: Option<String>,
}

#[derive(Debug, Clone, CosmicConfigEntry)]
pub(crate) struct WorldClockConfig {
    pub(crate) clocks: Vec<ClockConfig>,
    pub(crate) show_seconds: bool,
    pub(crate) military_time: bool,
    pub(crate) show_weekday: bool,
    pub(crate) show_date: bool,
}

impl Default for WorldClockConfig {
    fn default() -> Self {
        Self {
            // Default: one UTC clock, labelled with its timezone name.
            // It can be removed; the applet also works with zero clocks.
            clocks: vec![ClockConfig {
                timezone: "UTC".to_string(),
                label: None,
            }],
            show_seconds: false,
            military_time: false,
            show_weekday: false,
            show_date: false,
        }
    }
}

impl WorldClockConfig {
    fn config_handler() -> Option<Config> {
        Config::new(APP_ID, CONFIG_VERSION).ok()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Flags {
    pub(crate) config: WorldClockConfig,
    pub(crate) config_handler: Option<cosmic_config::Config>,
}

impl Flags {
    pub(crate) fn new() -> Self {
        let config_handler = WorldClockConfig::config_handler();
        let config = match &config_handler {
            Some(handler) => WorldClockConfig::get_entry(handler)
                .map_err(|error| tracing::info!("error whilst loading config: {:#?}", error))
                .unwrap_or_default(),
            None => WorldClockConfig::default(),
        };

        Self {
            config,
            config_handler,
        }
    }
}
