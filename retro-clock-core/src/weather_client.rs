use anyhow::{self, Context};

use crate::http_client::HttpClient;
use crate::weather_models::WeatherReport;

const WEATHER_API_URL: &str = env!("WEATHER_API_URL");

pub struct WeatherClient<'a, T>
where
    T: HttpClient,
{
    http_client: &'a mut T,
}

impl<'a, T> WeatherClient<'a, T>
where
    T: HttpClient,
{
    pub fn new(http_client: &'a mut T) -> Self {
        WeatherClient { http_client }
    }

    pub fn get_weather(&mut self, body: &str) -> anyhow::Result<WeatherReport> {
        let resp_body = self
            .http_client
            .http_post(WEATHER_API_URL, body)
            .context("Cannot download weather data")?;
        let report: WeatherReport =
            serde_json::from_str(&resp_body).context("Cannot parse weather data")?;
        Ok(report)
    }
}
