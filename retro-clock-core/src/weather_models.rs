use crate::params::{
    WEATHER_STATIONS_MAX_NUM, WEATHER_STATION_NAME_MAX_LEN, WEATHER_STATION_URL_MAX_LEN,
};
use crate::serde_limiter::{StringLenLimiter, VecLenLimiter};
use chrono::NaiveDateTime;
use serde::{Deserialize, Deserializer, Serialize};

/// chrono can only deserialize datetime in format %Y-%m-%dT%H:%M:%SZ. Here we have a short version of it
fn deserialize_datetime_short<'de, D>(deserializer: D) -> Result<Option<NaiveDateTime>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    if let Some(s) = opt {
        NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M")
            .map(Some)
            .map_err(serde::de::Error::custom)
    } else {
        Ok(None)
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Measurement {
    pub gusts_speed: Option<i32>,
    pub humidity: Option<i32>,
    pub precipitation: Option<f64>,
    pub pressure: Option<i32>,
    pub temperature: Option<f64>,
    #[serde(deserialize_with = "deserialize_datetime_short")]
    pub update_time: Option<NaiveDateTime>,
    pub wind_direction: Option<String>,
    pub wind_speed: Option<i32>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct WeatherReport {
    pub measurements: Vec<Measurement>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WeatherStation {
    #[serde(deserialize_with = "StringLenLimiter::<WEATHER_STATION_NAME_MAX_LEN>::deserialize")]
    pub name: String,
    #[serde(deserialize_with = "StringLenLimiter::<WEATHER_STATION_URL_MAX_LEN>::deserialize")]
    pub url: String,
}

pub type WeatherStationsLimited = VecLenLimiter<WEATHER_STATIONS_MAX_NUM, WeatherStation>;
