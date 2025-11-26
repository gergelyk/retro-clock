use crate::http_client::HttpClient;
use crate::worldtime_models::LocationInfo;
use anyhow;

const LOCATION_API_URL: &str = env!("LOCATION_API_URL");

pub struct LocationClient<'a, T>
where
    T: HttpClient,
{
    http_client: &'a mut T,
}

impl<'a, T> LocationClient<'a, T>
where
    T: HttpClient,
{
    pub fn new(http_client: &'a mut T) -> Self {
        LocationClient { http_client }
    }

    pub fn get_location_info(&mut self) -> anyhow::Result<LocationInfo> {
        let body = self.http_client.http_get(LOCATION_API_URL)?;
        let parsed: LocationInfo = serde_json::from_str(&body)?;
        Ok(parsed)
    }
}
