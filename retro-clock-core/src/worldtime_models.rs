use crate::params::{LOCATIONS_MAX_NUM, LOCATION_NAME_MAX_LEN};
use crate::serde_limiter::{StringLenLimiter, VecLenLimiter};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Clone)]
pub struct LocationInfo {
    pub timezone: Tz,
    #[serde(alias = "lat")]
    pub latitude: f64,
    #[serde(alias = "lon")]
    pub longitude: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LocationData {
    #[serde(deserialize_with = "StringLenLimiter::<LOCATION_NAME_MAX_LEN>::deserialize_opt")]
    pub name: Option<String>,
    pub timezone: Tz,
}

pub type LocationsLimited = VecLenLimiter<LOCATIONS_MAX_NUM, LocationData>;
