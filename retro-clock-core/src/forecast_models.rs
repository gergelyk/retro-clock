use chrono::{NaiveDateTime, TimeDelta};
use serde::{Deserialize, Deserializer};

/// chrono can only deserialize datetime in format %Y-%m-%dT%H:%M:%SZ. Here we have a short version of it
fn deserialize_datetime_short_vec<'de, D>(deserializer: D) -> Result<Vec<NaiveDateTime>, D::Error>
where
    D: Deserializer<'de>,
{
    let strings: Vec<String> = Deserialize::deserialize(deserializer)?;
    strings
        .into_iter()
        .map(|s| {
            NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M").map_err(serde::de::Error::custom)
        })
        .collect()
}

#[derive(Deserialize, Debug)]
pub struct ForecastMetnoSeamlessHourlyUnits {
    pub time: String,
    pub temperature_2m: String,
    pub wind_speed_10m: String,
    pub wind_direction_10m: String,
    pub wind_gusts_10m: String,
    pub cloud_cover: String,
}

#[derive(Deserialize, Debug)]
pub struct ForecastMetnoSeamlessHourly {
    #[serde(deserialize_with = "deserialize_datetime_short_vec")]
    pub time: Vec<NaiveDateTime>,
    pub temperature_2m: Vec<f64>,
    pub wind_speed_10m: Vec<f64>,
    pub wind_direction_10m: Vec<i32>,
    pub wind_gusts_10m: Vec<f64>,
    pub cloud_cover: Vec<i32>,
}

#[derive(Deserialize, Debug)]
pub struct ForecastMetnoSeamlessDailyUnits {
    pub time: String,
    pub sunrise: String,
    pub sunset: String,
    pub precipitation_sum: String,
    pub precipitation_hours: String,
    pub precipitation_probability_max: String,
}

#[derive(Deserialize, Debug)]
pub struct ForecastMetnoSeamlessDaily {
    #[serde(deserialize_with = "deserialize_datetime_short_vec")]
    pub sunrise: Vec<NaiveDateTime>,
    #[serde(deserialize_with = "deserialize_datetime_short_vec")]
    pub sunset: Vec<NaiveDateTime>,
    pub precipitation_sum: Vec<f64>,
    pub precipitation_hours: Vec<f64>,
    pub precipitation_probability_max: Vec<i32>,
}

#[derive(Deserialize, Debug)]
pub struct ForecastMetnoSeamless {
    pub hourly_units: ForecastMetnoSeamlessHourlyUnits,
    pub hourly: ForecastMetnoSeamlessHourly,
    pub daily_units: ForecastMetnoSeamlessDailyUnits,
    pub daily: ForecastMetnoSeamlessDaily,
}

#[derive(Deserialize, Debug)]
pub struct ForecastDefaultDailyUnits {
    pub time: String,
    pub uv_index_max: String,
}

#[derive(Deserialize, Debug)]
pub struct ForecastDefaultDaily {
    pub uv_index_max: Vec<f64>,
}

#[derive(Deserialize, Debug)]
pub struct ForecastDefault {
    pub daily_units: ForecastDefaultDailyUnits,
    pub daily: ForecastDefaultDaily,
}

#[derive(Default, Debug)]
pub struct ForecastReport {
    pub min_temperature: f64,
    pub max_temperature: f64,
    pub min_temperature_time: NaiveDateTime,
    pub max_temperature_time: NaiveDateTime,
    pub sunrise: NaiveDateTime,
    pub sunset: NaiveDateTime,
    pub day_length: TimeDelta,
    pub cloud_cover_min: i32,
    pub cloud_cover_max: i32,
    pub wind_max_speed: f64,
    pub wind_direction_at_max_speed: &'static str,
    pub wind_max_gusts: f64,
    pub precipitation_sum: f64,
    pub precipitation_max_probability: i32,
    pub precipitation_hours: f64,
    pub max_uv_index: f64,
}
