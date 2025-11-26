use chrono_tz::Tz;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SystemSettings {
    pub latitude: f64,
    pub longitude: f64,
    pub coordinates_auto: bool,
    pub time_zone: Tz,
    pub time_zone_auto: bool,
    pub bg_brightness: u8,
    pub bg_brightness_auto: bool,
    pub led_brightness: u8,
    pub led_brightness_auto: bool,
}

impl Default for SystemSettings {
    fn default() -> Self {
        Self {
            latitude: 0.0,
            longitude: 0.0,
            coordinates_auto: true,
            time_zone: Tz::UTC,
            time_zone_auto: true,
            bg_brightness: 50,
            bg_brightness_auto: true,
            led_brightness: 50,
            led_brightness_auto: true,
        }
    }
}
