use std::sync::Arc;

use esp_idf_svc::http::server as esp_http_server;

use embedded_svc::http::Method as HttpMethod;

use retro_clock_core::{
    arc_mtx::ArcMtx, channel_utils::SyncSenderBlocking, settings_models::SystemSettings,
    weather_models::WeatherStation, worldtime_models::LocationData,
};

use crate::http_opt_handlers::{
    options_admin_creds, options_check_authorization, options_login, options_logout,
    options_reboot, options_system_settings, options_weather_stations, options_wifi_creds,
    options_world_time_places,
};

#[cfg(not(feature = "no_file_server"))]
use crate::file_server;

use crate::{http_handlers::HttpHandlers, storage::EspConfigStorage};

#[allow(clippy::too_many_arguments)]
pub fn register_http_handlers(
    http_server: &mut esp_http_server::EspHttpServer,
    http_handlers_arc: Arc<HttpHandlers>,
    system_settings_arc: ArcMtx<SystemSettings>,
    storage_arc: ArcMtx<EspConfigStorage>,
    places_arc: ArcMtx<Vec<LocationData>>,
    stations_arc: ArcMtx<Vec<WeatherStation>>,
    settings_update_tx: SyncSenderBlocking<SystemSettings>,
    message_tx: SyncSenderBlocking<String>,
) -> anyhow::Result<()> {
    http_server.fn_handler(
        "/check-authorization",
        HttpMethod::Options,
        options_check_authorization,
    )?;
    let http_handlers_clone = http_handlers_arc.clone();
    http_server.fn_handler("/check-authorization", HttpMethod::Get, move |req| {
        http_handlers_clone.get_check_authorization(req)
    })?;
    http_server.fn_handler("/login", HttpMethod::Options, options_login)?;
    let http_handlers_clone = http_handlers_arc.clone();
    let storage_arc_clone = storage_arc.clone();
    http_server.fn_handler(
        "/login",
        HttpMethod::Post,
        move |req| -> Result<_, anyhow::Error> {
            http_handlers_clone.post_login(req, &storage_arc_clone)
        },
    )?;
    http_server.fn_handler("/logout", HttpMethod::Options, options_logout)?;
    let http_handlers_clone = http_handlers_arc.clone();
    http_server.fn_handler("/logout", HttpMethod::Post, move |req| {
        http_handlers_clone.post_logout(req)
    })?;
    http_server.fn_handler("/admin-creds", HttpMethod::Options, options_admin_creds)?;
    let http_handlers_clone = http_handlers_arc.clone();
    let storage_arc_clone = storage_arc.clone();
    http_server.fn_handler(
        "/admin-creds",
        HttpMethod::Post,
        move |req| -> Result<_, anyhow::Error> {
            http_handlers_clone.post_admin_creds(req, &storage_arc_clone)
        },
    )?;
    http_server.fn_handler("/wifi-creds", HttpMethod::Options, options_wifi_creds)?;
    let http_handlers_clone = http_handlers_arc.clone();
    let storage_arc_clone = storage_arc.clone();
    http_server.fn_handler(
        "/wifi-creds",
        HttpMethod::Post,
        move |req| -> Result<_, anyhow::Error> {
            http_handlers_clone.post_wifi_creds(req, &storage_arc_clone)
        },
    )?;
    let http_handlers_clone = http_handlers_arc.clone();
    let storage_arc_clone = storage_arc.clone();
    http_server.fn_handler(
        "/wifi-creds",
        HttpMethod::Get,
        move |req| -> Result<_, anyhow::Error> {
            http_handlers_clone.get_wifi_creds(req, &storage_arc_clone)
        },
    )?;
    http_server.fn_handler(
        "/system-settings",
        HttpMethod::Options,
        options_system_settings,
    )?;
    let http_handlers_clone = http_handlers_arc.clone();
    let system_settings_arc_clone = system_settings_arc.clone();
    let storage_arc_clone = storage_arc.clone();
    http_server.fn_handler(
        "/system-settings",
        HttpMethod::Post,
        move |req| -> Result<_, anyhow::Error> {
            http_handlers_clone.post_system_settings(
                req,
                &system_settings_arc_clone,
                &storage_arc_clone,
                &settings_update_tx,
            )
        },
    )?;
    let http_handlers_clone = http_handlers_arc.clone();
    let system_settings_arc_clone = system_settings_arc.clone();
    http_server.fn_handler(
        "/system-settings",
        HttpMethod::Get,
        move |req| -> Result<_, anyhow::Error> {
            http_handlers_clone.get_system_settings(req, &system_settings_arc_clone)
        },
    )?;
    http_server.fn_handler("/reboot", HttpMethod::Options, options_reboot)?;
    let http_handlers_clone = http_handlers_arc.clone();
    http_server.fn_handler(
        "/reboot",
        HttpMethod::Post,
        move |req| -> Result<_, anyhow::Error> { http_handlers_clone.post_reboot(req) },
    )?;
    http_server.fn_handler(
        "/weather-stations",
        HttpMethod::Options,
        options_weather_stations,
    )?;
    let http_handlers_clone = http_handlers_arc.clone();
    let stations_arc_clone = stations_arc.clone();
    let storage_arc_clone = storage_arc.clone();
    http_server.fn_handler(
        "/weather-stations",
        HttpMethod::Post,
        move |req| -> Result<_, anyhow::Error> {
            http_handlers_clone.post_weather_stations(req, &stations_arc_clone, &storage_arc_clone)
        },
    )?;
    let http_handlers_clone = http_handlers_arc.clone();
    let stations_arc_clone = stations_arc.clone();
    http_server.fn_handler(
        "/weather-stations",
        HttpMethod::Get,
        move |req| -> Result<_, anyhow::Error> {
            http_handlers_clone.get_weather_stations(req, &stations_arc_clone)
        },
    )?;
    http_server.fn_handler(
        "/world-time-places",
        HttpMethod::Options,
        options_world_time_places,
    )?;
    let http_handlers_clone = http_handlers_arc.clone();
    let places_arc_clone = places_arc.clone();
    let storage_arc_clone = storage_arc.clone();
    http_server.fn_handler(
        "/world-time-places",
        HttpMethod::Post,
        move |req| -> Result<_, anyhow::Error> {
            http_handlers_clone.post_world_time_places(req, &places_arc_clone, &storage_arc_clone)
        },
    )?;
    let http_handlers_clone = http_handlers_arc.clone();
    let places_arc_clone = places_arc.clone();
    http_server.fn_handler(
        "/world-time-places",
        HttpMethod::Get,
        move |req| -> Result<_, anyhow::Error> {
            http_handlers_clone.get_world_time_places(req, &places_arc_clone)
        },
    )?;
    #[cfg(not(feature = "no_file_server"))]
    http_server.fn_handler(
        "*",
        HttpMethod::Get,
        move |req| -> Result<_, anyhow::Error> { file_server::get_file_handler(req, &message_tx) },
    )?;
    Ok(())
}
