use anyhow::Context;
use embedded_svc::http::{client::Client as EmbHttpClient, Method};
use embedded_svc::utils::io;
use esp_idf_hal::io::Write;
use esp_idf_svc::http::client::EspHttpConnection;
use esp_idf_sys as _;
use std::time::Duration;

use crate::params::HTTP_CLIENT_READ_BUF_SIZE;
use retro_clock_core::http_client::{HttpClient, REQUEST_TIMEOUT_SECS};

pub struct EspHttpClient {
    http_client: EmbHttpClient<EspHttpConnection>,
}

fn send_http_request(
    http_client: &mut EmbHttpClient<EspHttpConnection>,
    url: &str,
    method: Method,
    body: Option<&str>,
) -> anyhow::Result<String> {
    let headers = [];
    let mut request = http_client.request(method, url, &headers)?;
    if let Some(body) = body {
        request.write_all(body.as_bytes())?;
    }
    let mut response = request.submit()?;
    let status = response.status();
    if !(200..300).contains(&status) {
        anyhow::bail!("Unsuccessful response ({}) from: {}", &status, &url)
    }

    let mut buf = vec![0u8; HTTP_CLIENT_READ_BUF_SIZE];
    let bytes_read = io::try_read_full(&mut response, &mut buf)
        .map_err(|e| e.0)
        .context("Failed to read HTTP response body")?;
    let body = std::str::from_utf8(&buf[0..bytes_read])?;
    Ok(body.to_owned())
}

impl HttpClient for EspHttpClient {
    fn new() -> anyhow::Result<Self> {
        let connection = EspHttpConnection::new(&esp_idf_svc::http::client::Configuration {
            timeout: Some(Duration::from_secs(REQUEST_TIMEOUT_SECS.into())),
            crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach),
            ..Default::default()
        })?;
        let http_client = EmbHttpClient::wrap(connection);

        Ok(EspHttpClient { http_client })
    }

    fn http_get(&mut self, url: &str) -> anyhow::Result<String> {
        send_http_request(&mut self.http_client, url, Method::Get, None)
    }

    fn http_post(&mut self, url: &str, body: &str) -> anyhow::Result<String> {
        send_http_request(&mut self.http_client, url, Method::Post, Some(body))
    }

    fn sleep(time_secs: u32) {
        esp_idf_hal::delay::FreeRtos::delay_ms(time_secs * 1000);
    }
}
