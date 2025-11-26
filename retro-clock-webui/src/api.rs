use crate::window_utils::refresh_page;
use anyhow::Context;
use gloo_net::http::{Request, Response};
use http::StatusCode;
use leptos::logging::log;
use retro_clock_core::creds_models::{
    AdminCredentialsPost, WifiCredentialsGet, WifiCredentialsPost,
};
use retro_clock_core::login_models::{AuthorizationStatus, LoginRequest};
use retro_clock_core::settings_models::SystemSettings;
use retro_clock_core::weather_models::WeatherStation;
use retro_clock_core::worldtime_models::LocationData;
use web_sys::RequestCredentials;

pub const API_URL: &str = match option_env!("API_URL") {
    Some(url) => url,
    None => "",
};

fn url(path: &str) -> String {
    format!("{}{}", API_URL, path)
}

fn check_response(resp: &Response) -> anyhow::Result<()> {
    if !resp.ok() {
        if resp.status() == StatusCode::UNAUTHORIZED {
            refresh_page();
        }
        return Err(anyhow::anyhow!(
            "Invalid response status: {}",
            resp.status()
        ));
    }
    Ok(())
}

pub async fn send_login_request(password: &str) -> anyhow::Result<bool> {
    let body = LoginRequest {
        password: password.to_string(),
    };

    let resp = Request::post(&url("/login"))
        .credentials(RequestCredentials::Include)
        .json(&body)?
        .send()
        .await?;

    if resp.status() == 401 {
        return Ok(false);
    } else if !resp.ok() {
        return Err(anyhow::anyhow!(
            "Invalid response status: {}",
            resp.status()
        ));
    }

    Ok(true)
}

pub async fn send_logout_request() -> anyhow::Result<()> {
    let resp = Request::post(&url("/logout"))
        .credentials(RequestCredentials::Include)
        .send()
        .await?;
    check_response(&resp)
}

pub async fn upload_admin_creds(creds: &AdminCredentialsPost) -> anyhow::Result<()> {
    let resp = Request::post(&url("/admin-creds"))
        .credentials(RequestCredentials::Include)
        .json(creds)
        .context("Failed to serialize admin creds")?
        .send()
        .await?;
    check_response(&resp)?;
    log!("Admin creds uploaded");
    Ok(())
}

pub async fn download_wifi_creds() -> anyhow::Result<WifiCredentialsGet> {
    let resp = Request::get(&url("/wifi-creds"))
        .credentials(RequestCredentials::Include)
        .send()
        .await?;
    check_response(&resp)?;

    let body = resp.text().await?;
    let creds: WifiCredentialsGet =
        serde_json::from_str(&body).context("Failed to parse wifi credentials")?;
    log!("Wifi credentials downloaded");
    Ok(creds)
}

pub async fn upload_wifi_creds(creds: &WifiCredentialsPost) -> anyhow::Result<()> {
    let resp = Request::post(&url("/wifi-creds"))
        .credentials(RequestCredentials::Include)
        .json(creds)
        .context("Failed to serialize wifi creds")?
        .send()
        .await?;
    check_response(&resp)?;
    log!("Wifi credentials uploaded");
    Ok(())
}

pub async fn download_system_settings() -> anyhow::Result<SystemSettings> {
    let resp = Request::get(&url("/system-settings"))
        .credentials(RequestCredentials::Include)
        .send()
        .await?;
    check_response(&resp)?;

    let body = resp.text().await?;
    let settings: SystemSettings =
        serde_json::from_str(&body).context("Failed to parse system settings")?;
    log!("System settings downloaded");
    Ok(settings)
}

pub async fn upload_system_settings(settings: &SystemSettings) -> anyhow::Result<()> {
    let resp = Request::post(&url("/system-settings"))
        .credentials(RequestCredentials::Include)
        .json(settings)
        .context("Failed to serialize system settings")?
        .send()
        .await?;
    check_response(&resp)?;
    log!("System settings uploaded");
    Ok(())
}

pub async fn download_world_time_places() -> anyhow::Result<Vec<LocationData>> {
    let resp = Request::get(&url("/world-time-places"))
        .credentials(RequestCredentials::Include)
        .send()
        .await?;
    check_response(&resp)?;

    let body = resp.text().await?;
    let places: Vec<LocationData> =
        serde_json::from_str(&body).context("Failed to parse world time places")?;
    log!("World time places downloaded");
    Ok(places)
}

pub async fn upload_world_time_places(places: &Vec<LocationData>) -> anyhow::Result<()> {
    let resp = Request::post(&url("/world-time-places"))
        .credentials(RequestCredentials::Include)
        .json(places)
        .context("Failed to serialize world time places")?
        .send()
        .await?;
    check_response(&resp)?;
    log!("World time places uploaded");
    Ok(())
}

pub async fn download_weather_stations() -> anyhow::Result<Vec<WeatherStation>> {
    let resp = Request::get(&url("/weather-stations"))
        .credentials(RequestCredentials::Include)
        .send()
        .await?;
    check_response(&resp)?;
    let body = resp.text().await?;
    let stations: Vec<WeatherStation> =
        serde_json::from_str(&body).context("Failed to parse weather stations")?;
    log!("Weather stations downloaded");
    Ok(stations)
}

pub async fn upload_weather_stations(stations: &Vec<WeatherStation>) -> anyhow::Result<()> {
    let resp = Request::post(&url("/weather-stations"))
        .credentials(RequestCredentials::Include)
        .json(stations)
        .context("Failed to serialize weather stations")?
        .send()
        .await?;
    check_response(&resp)?;
    log!("Weather stations uploaded");
    Ok(())
}

pub async fn reboot_device() -> anyhow::Result<()> {
    log!("Requesting device reboot...");
    let resp = Request::post(&url("/reboot"))
        .credentials(RequestCredentials::Include)
        .send()
        .await?;
    check_response(&resp)?;
    log!("Device reboot requested");
    Ok(())
}

pub async fn check_authorization() -> anyhow::Result<AuthorizationStatus> {
    let resp = Request::get(&url("/check-authorization"))
        .credentials(RequestCredentials::Include)
        .send()
        .await?;

    if !resp.ok() {
        return Err(anyhow::anyhow!(
            "Invalid response status: {}",
            resp.status()
        ));
    }
    let auth_status: AuthorizationStatus = resp
        .json()
        .await
        .context("Failed to parse AuthorizationStatus")?;
    Ok(auth_status)
}
