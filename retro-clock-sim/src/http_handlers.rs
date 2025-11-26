use crate::utils::simulate_delay_http_handler;
use retro_clock_core::{
    arc_mtx::{ArcMtx, MutexExt},
    creds_models::{
        AdminCredentials, AdminCredentialsPost, WifiCredentials, WifiCredentialsGet,
        WifiCredentialsPost,
    },
    login::hash_password,
    login::{generate_jwt, verify_jwt, verify_password},
    login_models::{AuthorizationStatus, LoginRequest},
    settings_models::SystemSettings,
    system_settings::overlay_location_info,
    weather_models::{WeatherStation, WeatherStationsLimited},
    worldtime_models::{LocationData, LocationInfo, LocationsLimited},
};

use axum::{
    extract::{Json, State},
    http::StatusCode,
};
use axum::{http::HeaderMap, response::IntoResponse};
use axum_extra::extract::CookieJar;

use cookie::Cookie;

#[axum::debug_handler]
pub async fn post_login(
    State(admin_creds_arc): State<ArcMtx<AdminCredentials>>,
    Json(login_request): Json<LoginRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    simulate_delay_http_handler();

    let admin_creds = match admin_creds_arc.lock_anyhow() {
        Ok(guard) => guard,
        Err(e) => {
            log::error!("{}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    match verify_password(&login_request.password, &admin_creds.password_hash) {
        Ok(is_valid) => {
            if !is_valid {
                return Err(StatusCode::UNAUTHORIZED);
            }
        }
        Err(e) => {
            log::error!("Error verifying password: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    let token = match generate_jwt() {
        Ok(token) => token,
        Err(e) => {
            log::error!("Error generating JWT: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    let mut cookie = Cookie::new("access_token", token);

    // If cookie wasn't secure Firefox would drop the cookie whenever its window is closed.
    cookie.set_secure(true);
    cookie.set_http_only(true);
    cookie.set_same_site(cookie::SameSite::Strict);
    cookie.set_path("/");
    cookie.set_expires(Some(
        time::OffsetDateTime::now_utc() + time::Duration::hours(1),
    ));

    let mut headers = HeaderMap::new();
    let mut set_cookies = || -> anyhow::Result<()> {
        headers.insert(axum::http::header::SET_COOKIE, cookie.to_string().parse()?);
        headers.insert(
            axum::http::header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
            "true".parse()?,
        );
        Ok(())
    };
    if let Err(e) = set_cookies() {
        log::error!("Error setting cookies: {}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    Ok((StatusCode::OK, headers, ""))
}

#[axum::debug_handler]
pub async fn post_logout(jar: CookieJar) -> CookieJar {
    simulate_delay_http_handler();
    if let Some(access_token) = jar.get("access_token") {
        log::debug!("Found access token cookie: {}", access_token.value());
        match verify_jwt(access_token.value()) {
            Ok(claims) => {
                log::debug!("Access token is valid. Claims: {:?}", claims);
                let updated_jar = jar.remove(Cookie::build("access_token"));
                return updated_jar;
            }
            Err(e) => {
                log::debug!("Access token verification failed: {}", e);
            }
        }
    } else {
        log::debug!("No access token cookie found");
    }
    jar
}

#[axum::debug_handler]
pub async fn get_check_authorization(
    State(auto_auth_arc): State<ArcMtx<bool>>,
    jar: CookieJar,
) -> Json<AuthorizationStatus> {
    simulate_delay_http_handler();
    match auto_auth_arc.lock_anyhow() {
        Ok(guard) => {
            if *guard {
                log::debug!("Auto authorization is enabled");
                return Json(AuthorizationStatus {
                    authorized: true,
                    automatically: true,
                });
            }
        }
        Err(e) => {
            log::error!("Cannot check auto auto-auth state: {}", e);
            return Json(AuthorizationStatus {
                authorized: false,
                automatically: false,
            });
        }
    };

    if let Some(access_token) = jar.get("access_token") {
        log::debug!("Found access token cookie: {}", access_token.value());
        match verify_jwt(access_token.value()) {
            Ok(claims) => {
                log::debug!("Access token is valid. Claims: {:?}", claims);
                return Json(AuthorizationStatus {
                    authorized: true,
                    automatically: false,
                });
            }
            Err(e) => {
                log::debug!("Access token verification failed: {}", e);
            }
        }
    } else {
        log::debug!("No access token cookie found");
    }
    Json(AuthorizationStatus {
        authorized: false,
        automatically: false,
    })
}

#[axum::debug_handler]
pub async fn post_admin_creds(
    State(admin_creds_arc): State<ArcMtx<AdminCredentials>>,
    Json(payload): Json<AdminCredentialsPost>,
) -> StatusCode {
    simulate_delay_http_handler();
    let mut admin_creds = match admin_creds_arc.lock_anyhow() {
        Ok(guard) => guard,
        Err(e) => {
            log::error!("{}", e);
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };
    let password_hash = match hash_password(&payload.password) {
        Ok(value) => value,
        Err(e) => {
            log::error!("Error hashing password: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };
    *admin_creds = AdminCredentials { password_hash };
    StatusCode::NO_CONTENT
}

#[axum::debug_handler]
pub async fn post_wifi_creds(
    State(wifi_creds_arc): State<ArcMtx<WifiCredentials>>,
    Json(payload): Json<WifiCredentialsPost>,
) -> StatusCode {
    simulate_delay_http_handler();
    let mut wifi_creds = match wifi_creds_arc.lock_anyhow() {
        Ok(guard) => guard,
        Err(e) => {
            log::error!("{}", e);
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };
    let current_pass = wifi_creds.password.clone();
    *wifi_creds = WifiCredentials {
        ssid: payload.ssid,
        password: payload.password.unwrap_or(current_pass),
    };
    StatusCode::NO_CONTENT
}

#[axum::debug_handler]
pub async fn get_wifi_creds(
    State(wifi_creds_arc): State<ArcMtx<WifiCredentials>>,
) -> Result<Json<WifiCredentialsGet>, StatusCode> {
    simulate_delay_http_handler();
    let wifi_creds = match wifi_creds_arc.lock_anyhow() {
        Ok(guard) => guard,
        Err(e) => {
            log::error!("{}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    Ok(Json(WifiCredentialsGet {
        ssid: wifi_creds.ssid.clone(),
    }))
}

#[axum::debug_handler]
pub async fn post_system_settings(
    State((system_settings_arc, location_info_arc)): State<(
        ArcMtx<SystemSettings>,
        ArcMtx<LocationInfo>,
    )>,
    Json(payload): Json<SystemSettings>,
) -> StatusCode {
    simulate_delay_http_handler();

    let mut system_settings = match system_settings_arc.lock_anyhow() {
        Ok(guard) => guard,
        Err(e) => {
            log::error!("{}", e);
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };
    *system_settings = payload;

    let mut location_info = match location_info_arc.lock_anyhow() {
        Ok(guard) => guard,
        Err(e) => {
            log::error!("{}", e);
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };
    *location_info = overlay_location_info(&location_info, &system_settings);

    StatusCode::NO_CONTENT
}

#[axum::debug_handler]
pub async fn get_system_settings(
    State((system_settings_arc, _)): State<(ArcMtx<SystemSettings>, ArcMtx<LocationInfo>)>,
) -> Result<Json<SystemSettings>, StatusCode> {
    simulate_delay_http_handler();
    let system_settings = match system_settings_arc.lock_anyhow() {
        Ok(guard) => guard,
        Err(e) => {
            log::error!("{}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    Ok(Json(system_settings.clone()))
}

#[axum::debug_handler]
pub async fn post_world_time_places(
    State(places_arc): State<ArcMtx<Vec<LocationData>>>,
    Json(mut payload): Json<LocationsLimited>,
) -> StatusCode {
    simulate_delay_http_handler();
    let mut places = match places_arc.lock_anyhow() {
        Ok(guard) => guard,
        Err(e) => {
            log::error!("{}", e);
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };

    places.clear();
    places.append(&mut payload.0);
    StatusCode::NO_CONTENT
}

#[axum::debug_handler]
pub async fn get_world_time_places(
    State(places_arc): State<ArcMtx<Vec<LocationData>>>,
) -> Result<Json<Vec<LocationData>>, StatusCode> {
    simulate_delay_http_handler();
    let places = match places_arc.lock_anyhow() {
        Ok(guard) => guard,
        Err(e) => {
            log::error!("{}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    Ok(Json(places.clone()))
}

#[axum::debug_handler]
pub async fn post_weather_stations(
    State(stations_arc): State<ArcMtx<Vec<WeatherStation>>>,
    Json(mut payload): Json<WeatherStationsLimited>,
) -> StatusCode {
    simulate_delay_http_handler();
    let mut stations = match stations_arc.lock_anyhow() {
        Ok(guard) => guard,
        Err(e) => {
            log::error!("{}", e);
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };

    stations.clear();
    stations.append(&mut payload.0);
    StatusCode::NO_CONTENT
}

#[axum::debug_handler]
pub async fn get_weather_stations(
    State(stations_arc): State<ArcMtx<Vec<WeatherStation>>>,
) -> Result<Json<Vec<WeatherStation>>, StatusCode> {
    simulate_delay_http_handler();
    let stations = match stations_arc.lock_anyhow() {
        Ok(guard) => guard,
        Err(e) => {
            log::error!("{}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    Ok(Json(stations.clone()))
}

#[axum::debug_handler]
pub async fn post_reboot() -> StatusCode {
    simulate_delay_http_handler();
    log::info!("Reboot requested via HTTP API");
    StatusCode::NO_CONTENT
}
