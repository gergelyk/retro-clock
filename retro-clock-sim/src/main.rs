mod http_client;
mod http_handlers;
mod keyboard_monitor;
mod terminal;
mod threading;
mod utils;

use anyhow::Context;
#[cfg(feature = "no_cors")]
use axum::http;
use getch_rs::Key;
use http_client::ReqwestHttpClient;
use keyboard_monitor::KeyboardMonitor;
use retro_clock_core::{
    application::Application,
    arc_mtx::{arc_mtx_new, MutexExt},
    busy_indicator::BusyIndicator,
    creds_models::{AdminCredentials, WifiCredentials},
    error_indicator::ErrorIndicator,
    forecast_page::ForecastPage,
    http_client::HttpClient,
    location_client::LocationClient,
    login::hash_password,
    messages::BootupMessage,
    pager::Pager,
    settings_models::SystemSettings,
    terminal::Terminal,
    threading::ThreadBuilder,
    weather_models::WeatherStation,
    weather_page::WeatherPage,
    worldtime_models::LocationData,
    worldtime_page::WorldTimePage,
};
use std::thread;

use std::sync::mpsc;
use std::time::Duration;
use terminal::NativeTerminal;
use utils::simulate_delay_hw_init;

use axum::{routing, Router};
use tokio::runtime::Builder;

#[cfg(feature = "no_cors")]
use tower_http::cors::CorsLayer;

use http_handlers::{
    get_check_authorization, get_system_settings, get_weather_stations, get_wifi_creds,
    get_world_time_places, post_admin_creds, post_login, post_logout, post_reboot,
    post_system_settings, post_weather_stations, post_wifi_creds, post_world_time_places,
};

#[cfg(feature = "no_cors")]
pub const UI_ORIGIN: &str = env!("UI_ORIGIN");

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let mut term = NativeTerminal::new()?;
    term.init()?;

    let mut http_client = ReqwestHttpClient::new()?;
    let mut location_client = LocationClient::new(&mut http_client);

    term.log(BootupMessage::ConnectingToWiFi)?;
    simulate_delay_hw_init();
    term.log(BootupMessage::ObtainingIp)?;
    simulate_delay_hw_init();
    term.log(BootupMessage::SynchronizingWithNtp)?;
    simulate_delay_hw_init();

    term.log(BootupMessage::ObtainingLocationInfo)?;
    let location_info = location_client.get_location_info()?;
    log::info!("Local TZ: {}", location_info.timezone);
    log::info!(
        "Coordinates: {},{}",
        location_info.latitude,
        location_info.longitude
    );
    let location_info_arc = arc_mtx_new(location_info);

    term.log(BootupMessage::ObtainingUtcOffset)?;

    let mut busy_indicator = BusyIndicator::new();
    let busy_indicator_handle = busy_indicator.get_handle();
    let mut error_indicator = ErrorIndicator::new();
    let error_indicator_handle = error_indicator.get_handle();

    let auto_auth = false;
    let auto_auth_arc = arc_mtx_new(auto_auth);
    let admin_creds = AdminCredentials {
        password_hash: hash_password(option_env!("ADMIN_PASS").unwrap_or("123"))?,
    };
    let admin_creds_arc = arc_mtx_new(admin_creds);
    let wifi_creds = WifiCredentials {
        ssid: String::new(),
        password: String::new(),
    };
    let wifi_creds_arc = arc_mtx_new(wifi_creds);
    let system_settings = SystemSettings::default();
    let system_settings_arc = arc_mtx_new(system_settings);
    let places: Vec<LocationData> = Vec::new();
    let places_arc = arc_mtx_new(places);
    let stations: Vec<WeatherStation> = Vec::new();
    let stations_arc = arc_mtx_new(stations);

    let page0 = WorldTimePage::new(location_info_arc.clone(), places_arc);
    let places_arc = page0.get_places_arc();

    let thread_builder0 = threading::StdBuilder::new().name("Forecast0".into());

    let thread_builder1 = threading::StdBuilder::new().name("Forecast1".into());

    let page1 = ForecastPage::new(
        location_info_arc.clone(),
        busy_indicator_handle.clone(),
        error_indicator_handle.clone(),
        thread_builder0,
        thread_builder1,
    );

    let thread_builder = threading::StdBuilder::new().name("Weather".into());

    let page2 = WeatherPage::new(
        stations_arc,
        busy_indicator_handle.clone(),
        error_indicator_handle.clone(),
        thread_builder,
    );
    let stations_arc = page2.get_stations_arc();

    let kbd_monitor = KeyboardMonitor::new();
    let key_rx = kbd_monitor.start();

    let mut pager = Pager::<NativeTerminal, ReqwestHttpClient>::new();
    pager.add_page(Box::new(page0));
    pager.add_page(Box::new(page1));
    pager.add_page(Box::new(page2));

    let home_screen_timeout: u32 = option_env!("HOME_SCREEN_TIMEOUT_SECONDS")
        .unwrap_or("10")
        .parse()?;
    let mut app = Application::new(term, pager, home_screen_timeout);
    app.initialize()?;

    use std::sync::Arc;

    let server_port: u16 = env!("RCLOCK_SIM_SERVER_PORT").parse()?;
    log::info!("Starting HTTP server at port {}", server_port);
    let admin_creds_arc_clone0 = Arc::clone(&admin_creds_arc);
    let admin_creds_arc_clone1 = Arc::clone(&admin_creds_arc);
    let wifi_creds_arc_clone = Arc::clone(&wifi_creds_arc);
    let auto_auth_arc_clone = Arc::clone(&auto_auth_arc);
    let system_settings_arc_clone = Arc::clone(&system_settings_arc);
    let http_server_handle = std::thread::spawn(move || -> anyhow::Result<()> {
        let runtime = Builder::new_current_thread()
            .enable_all()
            .build()
            .context("Failed to create Tokio runtime")?;
        let app = Router::new()
            .route("/login", routing::post(post_login))
            .with_state(admin_creds_arc_clone0)
            .route("/logout", routing::post(post_logout))
            .route(
                "/check-authorization",
                routing::get(get_check_authorization),
            )
            .with_state(auto_auth_arc_clone)
            .route("/admin-creds", routing::post(post_admin_creds))
            .with_state(admin_creds_arc_clone1)
            .route(
                "/wifi-creds",
                routing::get(get_wifi_creds).post(post_wifi_creds),
            )
            .with_state(wifi_creds_arc_clone)
            .route(
                "/system-settings",
                routing::get(get_system_settings).post(post_system_settings),
            )
            .with_state((system_settings_arc_clone, location_info_arc))
            .route("/reboot", routing::post(post_reboot))
            .route(
                "/world-time-places",
                routing::get(get_world_time_places).post(post_world_time_places),
            )
            .with_state(places_arc)
            .route(
                "/weather-stations",
                routing::get(get_weather_stations).post(post_weather_stations),
            )
            .with_state(stations_arc);

        #[cfg(feature = "no_cors")]
        let app = app.layer(
            CorsLayer::new()
                .allow_origin(http::HeaderValue::from_str(UI_ORIGIN)?)
                .allow_credentials(true)
                .allow_headers([http::header::CONTENT_TYPE, http::header::ACCEPT]),
        );

        let async_result: anyhow::Result<()> = runtime.block_on(async {
            let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", server_port))
                .await
                .context("Failed to bind TCP listener")?;

            axum::serve(listener, app)
                .await
                .context("Failed to serve HTTP")?;
            Ok(())
        });

        async_result
    });

    thread::spawn(move || loop {
        match busy_indicator.wait_for_change() {
            Ok(is_busy) => {
                if is_busy {
                    log::info!("Busy indicator ON");
                } else {
                    log::info!("Busy indicator OFF");
                }
            }
            Err(e) => {
                log::error!("Busy indicator error: {}", e);
                break;
            }
        }
    });

    thread::spawn(move || loop {
        match error_indicator.wait_for_change() {
            Ok(mask) => {
                if mask != 0 {
                    log::info!("Error indicator ON: mask={}", mask);
                } else {
                    log::info!("Error indicator OFF");
                }
            }
            Err(e) => {
                log::error!("Error indicator error: {}", e);
                break;
            }
        }
    });

    loop {
        let time_to_round_second_ms = app.get_time_to_round_second_ms();
        match key_rx.recv_timeout(Duration::from_millis(time_to_round_second_ms as u64)) {
            Ok(key) => match key {
                Key::Char('?') => {
                    log::info!("n - next card");
                    log::info!("N - next page");
                    log::info!("h - home page");
                    log::info!("a - set auto authorization");
                    log::info!("A - unset auto authorization");
                    log::info!("c - show credentials");
                    log::info!("s - show system settings");
                }
                Key::Char('n') => {
                    app.next_card()?;
                }
                Key::Char('N') => {
                    app.next_page()?;
                }
                Key::Char('h') => {
                    app.home_page()?;
                }
                Key::Char('a') => {
                    let mut auto_auth = auto_auth_arc.lock_anyhow()?;
                    *auto_auth = true;
                    log::info!("Auto authorization set");
                }
                Key::Char('A') => {
                    let mut auto_auth = auto_auth_arc.lock_anyhow()?;
                    *auto_auth = false;
                    log::info!("Auto authorization unset");
                }
                Key::Char('c') => {
                    let admin_creds = admin_creds_arc.lock_anyhow()?;
                    let wifi_creds = wifi_creds_arc.lock_anyhow()?;
                    log::info!("Admin creds:\n{:#?}", admin_creds);
                    log::info!("Wifi creds:\n{:#?}", wifi_creds);
                }
                Key::Char('s') => {
                    let system_settings = system_settings_arc.lock_anyhow()?;
                    log::info!("System settings:\n{:#?}", system_settings);
                }
                _ => log::debug!("Unsupported key pressed: {:?}", key),
            },
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if http_server_handle.is_finished() {
                    match http_server_handle.join() {
                        Ok(result) => {
                            if let Err(e) = result {
                                log::error!("HTTP server thread exited with error: {}", e);
                            } else {
                                log::info!("HTTP server thread unexpectedly exited with no errors");
                            }
                        }
                        Err(e) => {
                            log::error!("Failed to join HTTP server thread: {:?}", e);
                        }
                    }
                    break;
                }
                app.refresh_display()?;
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                log::info!("Keyboard monitor disconnected");
                break;
            }
        }
    }
    log::info!("Main loop exited");

    app.close()?;
    Ok(())
}
