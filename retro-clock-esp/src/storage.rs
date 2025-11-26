use esp_idf_svc::nvs::{EspDefaultNvs, EspDefaultNvsPartition, EspNvs};

use retro_clock_core::{
    creds_models::{AdminCredentials, WifiCredentials},
    login::hash_password,
    settings_models::SystemSettings,
};

use crate::params::NVS_BUF_SIZE;
use crate::string_utils::get_random_string;

pub const ADMIN_CREDS_NVS_KEY: &str = "admin";
pub const WIFI_CREDS_NVS_KEY: &str = "wifi";
pub const WORLD_TIME_PLACES_NVS_KEY: &str = "places";
pub const WEATHER_STATIONS_NVS_KEY: &str = "stations";
pub const SYSTEM_SETTINGS_NVS_KEY: &str = "system";

const WIFI_ST_SSID: &str = match option_env!("WIFI_ST_SSID") {
    Some(v) => v,
    None => "",
};

const WIFI_ST_PASS: &str = match option_env!("WIFI_ST_PASS") {
    Some(v) => v,
    None => "",
};

pub struct EspConfigStorage {
    nvs: EspDefaultNvs,
    read_buf: Vec<u8>,
}

impl EspConfigStorage {
    pub fn new(partition: EspDefaultNvsPartition, namespace: &str) -> anyhow::Result<Self> {
        let nvs = EspNvs::new(partition, namespace, true)?;
        let read_buf = vec![0u8; NVS_BUF_SIZE];
        Ok(Self { nvs, read_buf })
    }

    pub fn set_str(&mut self, name: &str, val: &str) -> anyhow::Result<()> {
        self.nvs.set_str(name, val)?;
        Ok(())
    }

    pub fn get_str(&mut self, name: &str) -> anyhow::Result<Option<String>> {
        let value = self.nvs.get_str(name, &mut self.read_buf)?;
        let value_owned = value.map(|v| v.to_owned());
        Ok(value_owned)
    }

    pub fn reset(&mut self) -> anyhow::Result<()> {
        log::info!("Resetting storage...");

        let admin_pass = match option_env!("ADMIN_PASS") {
            Some(v) => v.to_owned(),
            None => get_random_string(10),
        };
        let admin_creds = AdminCredentials {
            password_hash: hash_password(&admin_pass)?,
        };
        let payload = serde_json::to_string(&admin_creds)?;
        self.set_str(ADMIN_CREDS_NVS_KEY, &payload)?;

        let wifi_creds = WifiCredentials {
            ssid: WIFI_ST_SSID.to_string(),
            password: WIFI_ST_PASS.to_string(),
        };
        let payload = serde_json::to_string(&wifi_creds)?;
        self.set_str(WIFI_CREDS_NVS_KEY, &payload)?;

        let places = "[]";
        self.set_str(WORLD_TIME_PLACES_NVS_KEY, places)?;

        let stations = "[]";
        self.set_str(WEATHER_STATIONS_NVS_KEY, stations)?;

        let system_settings = SystemSettings::default();
        let payload = serde_json::to_string(&system_settings)?;
        self.set_str(SYSTEM_SETTINGS_NVS_KEY, &payload)?;

        log::info!("Storage reset complete");
        Ok(())
    }
}
