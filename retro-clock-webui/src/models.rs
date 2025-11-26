use crate::pane_system::SystemSettingsForm;
use retro_clock_core::{weather_models::WeatherStation, worldtime_models::LocationData};

#[derive(Clone, PartialEq)]
pub enum Tab {
    Access,
    System,
    //Alarms,
    WorldTime,
    Weather,
}

pub struct UserDataBackup {
    pub wifi_ssid: String,
    pub system_settings: SystemSettingsForm,
    pub weather_stations: Vec<WeatherStation>,
    pub world_time_places: Vec<LocationData>,
}
