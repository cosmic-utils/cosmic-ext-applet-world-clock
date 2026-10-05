use cosmic::cosmic_config::{
    self, Config, CosmicConfigEntry, cosmic_config_derive::CosmicConfigEntry,
};
use serde::{Deserialize, Serialize};

const CONFIG_VERSION: u64 = 1;

pub(crate) const APP_ID: &str = "io.github.cosmic-utils.cosmic-ext-applet-world-clock";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub(crate) struct ClockConfig {
    /// IANA timezone name, e.g. "UTC" or "America/New_York".
    pub(crate) timezone: String,
    /// Optional short label shown next to the time. Falls back to the timezone name.
    pub(crate) label: Option<String>,
    #[serde(default)]
    pub(crate) show_seconds: bool,
    #[serde(default)]
    pub(crate) military_time: bool,
    #[serde(default)]
    pub(crate) show_weekday: bool,
    #[serde(default)]
    pub(crate) show_date: bool,
    /// Optional hex colour (`#rrggbb`/`#rgb`) for the clock's text. `None` = theme default.
    #[serde(default)]
    pub(crate) color: Option<String>,
}

#[derive(Debug, Clone, CosmicConfigEntry)]
pub(crate) struct WorldClockConfig {
    pub(crate) clocks: Vec<ClockConfig>,
    /// Show a month calendar (like the system time applet's) in the popup.
    pub(crate) show_calendar: bool,
}

impl Default for WorldClockConfig {
    fn default() -> Self {
        Self {
            // Default: one UTC clock, labelled with its timezone name.
            // It can be removed; the applet also works with zero clocks.
            show_calendar: false,
            clocks: vec![ClockConfig {
                timezone: "UTC".to_string(),
                ..ClockConfig::default()
            }],
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
