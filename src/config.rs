use cosmic::cosmic_config::{self, CosmicConfigEntry, cosmic_config_derive::CosmicConfigEntry};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, CosmicConfigEntry, Eq, PartialEq)]
#[version = 1]
pub struct Config {
    pub api_key: String,
    pub default_city: String,
    pub default_lat: String,
    pub default_lon: String,
    pub use_fahrenheit: bool,
    pub saved_locations: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedPlace {
    pub name: String,
    pub lat: String,
    pub lon: String,
}

impl Config {
    pub fn get_saved_places(&self) -> Vec<SavedPlace> {
        if self.saved_locations.trim().is_empty() {
            return Vec::new();
        }
        serde_json::from_str(&self.saved_locations).unwrap_or_default()
    }

    pub fn set_saved_places(&mut self, places: &[SavedPlace]) {
        self.saved_locations = serde_json::to_string(places).unwrap_or_default();
    }
}

