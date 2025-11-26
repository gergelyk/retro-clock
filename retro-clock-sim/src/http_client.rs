use std::time::Duration;

use retro_clock_core::http_client::{HttpClient, REQUEST_TIMEOUT_SECS};
pub struct ReqwestHttpClient {}

impl HttpClient for ReqwestHttpClient {
    fn new() -> anyhow::Result<Self> {
        Ok(Self {})
    }

    fn http_get(&mut self, url: &str) -> anyhow::Result<String> {
        let response = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS as u64))
            .build()?
            .get(url)
            .send()?;
        if response.status() != reqwest::StatusCode::OK {
            anyhow::bail!("Unsuccessful response from: {}", &url)
        }
        let body = response.text()?;
        Ok(body)
    }

    fn http_post(&mut self, url: &str, body: &str) -> anyhow::Result<String> {
        let response = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS as u64))
            .build()?
            .post(url)
            .body(body.to_owned())
            .send()?;
        if response.status() != reqwest::StatusCode::OK {
            anyhow::bail!("Unsuccessful response from: {}", &url)
        }
        let body = response.text()?;
        Ok(body)
    }
}
