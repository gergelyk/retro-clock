use crate::serde_limiter::StringLenLimiter;
use serde::{Deserialize, Serialize};

use crate::params::{ADMIN_PASSWORD_MAX_LEN, SSID_MAX_LEN, WIFI_PASSWORD_MAX_LEN};

#[derive(Serialize, Deserialize, Debug)]
pub struct AdminCredentials {
    pub password_hash: String,
}

#[derive(Serialize, Deserialize)]
pub struct AdminCredentialsPost {
    #[serde(deserialize_with = "StringLenLimiter::<ADMIN_PASSWORD_MAX_LEN>::deserialize")]
    pub password: String,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct WifiCredentials {
    pub ssid: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct WifiCredentialsGet {
    pub ssid: String,
}

#[derive(Serialize, Deserialize, Default)]
pub struct WifiCredentialsPost {
    #[serde(deserialize_with = "StringLenLimiter::<SSID_MAX_LEN>::deserialize")]
    pub ssid: String,
    #[serde(deserialize_with = "StringLenLimiter::<WIFI_PASSWORD_MAX_LEN>::deserialize_opt")]
    pub password: Option<String>,
}
