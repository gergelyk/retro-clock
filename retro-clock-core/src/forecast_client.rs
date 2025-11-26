use anyhow;
use chrono::DateTime;
use chrono_tz::Tz;

use crate::forecast_models::{ForecastDefault, ForecastMetnoSeamless, ForecastReport};
use crate::http_client::HttpClient;
use crate::worldtime_models::LocationInfo;

const FORECAST_API_URL: &str = env!("FORECAST_API_URL");

fn degrees_to_direction(angle: i32) -> &'static str {
    let directions = [
        "N", "NNE", "NE", "ENE", "E", "ESE", "SE", "SSE", "S", "SSW", "SW", "WSW", "W", "WNW",
        "NW", "NNW",
    ];
    let sector = (((angle % 360) as f64) / 22.5).round() as usize % 16;
    directions[sector]
}

pub struct ForecastClient<'a, T>
where
    T: HttpClient,
{
    http_client: &'a mut T,
}

impl<'a, T> ForecastClient<'a, T>
where
    T: HttpClient,
{
    pub fn new(http_client: &'a mut T) -> Self {
        ForecastClient { http_client }
    }

    fn check_units_metno_seamless(parsed: &ForecastMetnoSeamless) -> anyhow::Result<()> {
        if parsed.hourly_units.time != "iso8601" {
            anyhow::bail!(format!(
                "Unexpected hourly_units/time: {}",
                parsed.hourly_units.time
            ))
        }

        if parsed.hourly_units.temperature_2m != "°C" {
            anyhow::bail!(format!(
                "Unexpected hourly_units/temperature_2m: {}",
                parsed.hourly_units.temperature_2m
            ))
        }

        if parsed.hourly_units.wind_speed_10m != "km/h" {
            anyhow::bail!(format!(
                "Unexpected hourly_units/wind_speed_10m: {}",
                parsed.hourly_units.wind_speed_10m
            ))
        }

        if parsed.hourly_units.wind_direction_10m != "°" {
            anyhow::bail!(format!(
                "Unexpected hourly_units/wind_direction_10m: {}",
                parsed.hourly_units.wind_direction_10m
            ))
        }

        if parsed.hourly_units.wind_gusts_10m != "km/h" {
            anyhow::bail!(format!(
                "Unexpected hourly_units/wind_gusts_10m: {}",
                parsed.hourly_units.wind_gusts_10m
            ))
        }

        if parsed.hourly_units.cloud_cover != "%" {
            anyhow::bail!(format!(
                "Unexpected hourly_units/cloud_cover: {}",
                parsed.hourly_units.cloud_cover
            ))
        }

        if parsed.daily_units.time != "iso8601" {
            anyhow::bail!(format!(
                "Unexpected daily_units/time: {}",
                parsed.daily_units.time
            ))
        }

        if parsed.daily_units.sunrise != "iso8601" {
            anyhow::bail!(format!(
                "Unexpected daily_units/sunrise: {}",
                parsed.daily_units.sunrise
            ))
        }

        if parsed.daily_units.sunset != "iso8601" {
            anyhow::bail!(format!(
                "Unexpected daily_units/sunset: {}",
                parsed.daily_units.sunset
            ))
        }

        if parsed.daily_units.precipitation_sum != "mm" {
            anyhow::bail!(format!(
                "Unexpected daily_units/precipitation_sum: {}",
                parsed.daily_units.precipitation_sum
            ))
        }

        if parsed.daily_units.precipitation_probability_max != "%" {
            anyhow::bail!(format!(
                "Unexpected daily_units/precipitation_probability_max: {}",
                parsed.daily_units.precipitation_probability_max
            ))
        }

        if parsed.daily_units.precipitation_hours != "h" {
            anyhow::bail!(format!(
                "Unexpected daily_units/precipitation_hours: {}",
                parsed.daily_units.precipitation_hours
            ))
        }

        Ok(())
    }

    pub fn get_forecast_metno_seamless(
        &mut self,
        location_info: &LocationInfo,
        date: DateTime<Tz>,
        report: &mut ForecastReport,
    ) -> anyhow::Result<()> {
        let date_str = date.format("%Y-%m-%d");

        let url = format!(
            "{}/forecast?latitude={}&longitude={}&hourly=temperature_2m,wind_speed_10m,wind_direction_10m,wind_gusts_10m,cloud_cover&daily=sunrise,sunset,precipitation_sum,precipitation_hours,precipitation_probability_max&models=metno_seamless&timezone={}&start_date={}&end_date={}",
            FORECAST_API_URL,
            location_info.latitude,
            location_info.longitude,
            location_info.timezone,
            date_str,
            date_str
        );

        let body = self.http_client.http_get(&url)?;
        let parsed: ForecastMetnoSeamless = serde_json::from_str(&body)?;
        Self::check_units_metno_seamless(&parsed)?;

        let min_value_with_index = parsed
            .hourly
            .temperature_2m
            .iter()
            .cloned()
            .enumerate()
            .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .ok_or_else(|| anyhow::anyhow!("Unable to find min temperature_2m"))?;

        report.min_temperature = min_value_with_index.1;
        report.min_temperature_time = parsed.hourly.time[min_value_with_index.0];

        let max_value_with_index = parsed
            .hourly
            .temperature_2m
            .iter()
            .cloned()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .ok_or_else(|| anyhow::anyhow!("Unable to find max temperature_2m"))?;

        report.max_temperature = max_value_with_index.1;
        report.max_temperature_time = parsed.hourly.time[max_value_with_index.0];

        report.sunrise = parsed.daily.sunrise[0];
        report.sunset = parsed.daily.sunset[0];
        report.day_length = parsed.daily.sunset[0] - parsed.daily.sunrise[0];

        // Mean value
        let cloud_cover_mean = parsed.hourly.cloud_cover.iter().cloned().sum::<i32>()
            / parsed.hourly.cloud_cover.len() as i32;

        // Mean absolute deviation
        let cloud_cover_abs_deviations: Vec<i32> = parsed
            .hourly
            .cloud_cover
            .iter()
            .map(|&x| (x - cloud_cover_mean).abs())
            .collect();
        let cloud_cover_mad =
            cloud_cover_abs_deviations.iter().sum::<i32>() / parsed.hourly.cloud_cover.len() as i32;

        report.cloud_cover_min = std::cmp::max(0, cloud_cover_mean - cloud_cover_mad);
        report.cloud_cover_max = std::cmp::min(100, cloud_cover_mad + cloud_cover_mad);

        let max_value_with_index = parsed
            .hourly
            .wind_speed_10m
            .iter()
            .cloned()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .ok_or_else(|| anyhow::anyhow!("Unable to find max wind_speed_10m"))?;

        report.wind_max_speed = max_value_with_index.1;
        report.wind_direction_at_max_speed =
            degrees_to_direction(parsed.hourly.wind_direction_10m[max_value_with_index.0]);

        report.wind_max_gusts = parsed
            .hourly
            .wind_gusts_10m
            .iter()
            .cloned()
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .ok_or_else(|| anyhow::anyhow!("Unable to find max wind_gusts_10m"))?;

        report.precipitation_sum = parsed.daily.precipitation_sum[0];
        report.precipitation_max_probability = parsed.daily.precipitation_probability_max[0];
        report.precipitation_hours = parsed.daily.precipitation_hours[0];

        Ok(())
    }

    fn check_units_default(parsed: &ForecastDefault) -> anyhow::Result<()> {
        if parsed.daily_units.time != "iso8601" {
            anyhow::bail!(format!(
                "Unexpected daily_units/time: {}",
                parsed.daily_units.time
            ))
        }

        if !parsed.daily_units.uv_index_max.is_empty() {
            anyhow::bail!(format!(
                "Unexpected daily_units/uv_index_max: {}",
                parsed.daily_units.uv_index_max
            ))
        }

        Ok(())
    }

    pub fn get_forecast_default(
        &mut self,
        location_info: &LocationInfo,
        date: DateTime<Tz>,
        report: &mut ForecastReport,
    ) -> anyhow::Result<()> {
        let date_str = date.format("%Y-%m-%d");

        let url = format!(
            "{}/forecast?latitude={}&longitude={}&daily=uv_index_max&timezone={}&start_date={}&end_date={}",
            FORECAST_API_URL,
            location_info.latitude,
            location_info.longitude,
            location_info.timezone,
            date_str,
            date_str
        );

        let body = self.http_client.http_get(&url)?;
        let parsed: ForecastDefault = serde_json::from_str(&body)?;
        Self::check_units_default(&parsed)?;

        report.max_uv_index = parsed.daily.uv_index_max[0];
        Ok(())
    }

    pub fn get_forecast(
        &mut self,
        location_info: &LocationInfo,
        date: DateTime<Tz>,
    ) -> anyhow::Result<ForecastReport> {
        let mut report: ForecastReport = Default::default();
        self.get_forecast_metno_seamless(location_info, date, &mut report)?;
        self.get_forecast_default(location_info, date, &mut report)?;
        Ok(report)
    }
}
