use esp_idf_hal::io::Write;
use esp_idf_svc::http::server as esp_http_server;
use esp_idf_sys::esp_restart;

use crate::{
    http_utils::{
        build_access_token_cookie, is_authorized, read_body, report_json_content,
        report_no_content, report_unauthorized, HEADERS_DEFAULT_RESP, HEADERS_JSON_RESP,
    },
    storage::{
        EspConfigStorage, ADMIN_CREDS_NVS_KEY, SYSTEM_SETTINGS_NVS_KEY, WEATHER_STATIONS_NVS_KEY,
        WIFI_CREDS_NVS_KEY, WORLD_TIME_PLACES_NVS_KEY,
    },
};
use retro_clock_core::{
    arc_mtx::{ArcMtx, MutexExt},
    channel_utils::SyncSenderBlocking,
    creds_models::{
        AdminCredentials, AdminCredentialsPost, WifiCredentials, WifiCredentialsGet,
        WifiCredentialsPost,
    },
    login::{generate_jwt, hash_password, verify_password},
    login_models::{AuthorizationStatus, LoginRequest},
    settings_models::SystemSettings,
    weather_models::{WeatherStation, WeatherStationsLimited},
    worldtime_models::{LocationData, LocationsLimited},
};

pub struct HttpHandlers {
    auto_authorized: bool,
}

impl HttpHandlers {
    pub fn new(auto_authorized: bool) -> Self {
        Self { auto_authorized }
    }

    fn is_authorized(
        &self,
        req: &esp_http_server::Request<&mut esp_http_server::EspHttpConnection<'_>>,
    ) -> bool {
        self.auto_authorized || is_authorized(req)
    }

    pub fn get_check_authorization(
        &self,
        req: esp_http_server::Request<&mut esp_http_server::EspHttpConnection<'_>>,
    ) -> Result<(), anyhow::Error> {
        let authorized = self.is_authorized(&req);
        let status = http::StatusCode::OK;
        let mut resp = req.into_response(status.into(), None, &HEADERS_JSON_RESP)?;
        let authorization_status = AuthorizationStatus {
            authorized,
            automatically: self.auto_authorized,
        };
        let body = serde_json::to_string(&authorization_status)?;
        resp.write_all(body.as_bytes())?;
        Ok(())
    }

    pub fn post_login(
        &self,
        mut req: esp_http_server::Request<&mut esp_http_server::EspHttpConnection<'_>>,
        storage_arc: &ArcMtx<EspConfigStorage>,
    ) -> Result<(), anyhow::Error> {
        let body = read_body(&mut req)?;
        let login_request: LoginRequest = serde_json::from_str(&body)?;

        let admin_creds_str_opt = storage_arc.lock_anyhow()?.get_str(ADMIN_CREDS_NVS_KEY)?;

        let Some(admin_creds_str) = admin_creds_str_opt else {
            log::info!("Admin creds not set");
            let status = http::StatusCode::UNAUTHORIZED;
            let mut resp = req.into_response(status.into(), None, &HEADERS_DEFAULT_RESP)?;
            resp.write_all(status.to_string().as_bytes())?;
            return Ok(());
        };

        let admin_creds = serde_json::from_str::<AdminCredentials>(&admin_creds_str)?;
        match verify_password(&login_request.password, &admin_creds.password_hash) {
            Ok(is_valid) => {
                if !is_valid {
                    log::info!("Invalid password attempt");
                    let status = http::StatusCode::UNAUTHORIZED;
                    let mut resp = req.into_response(status.into(), None, &HEADERS_DEFAULT_RESP)?;
                    resp.write_all(status.to_string().as_bytes())?;
                    return Ok(());
                }
            }
            Err(e) => {
                log::error!("Error verifying password: {}", e);
                anyhow::bail!(http::StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
        let token = match generate_jwt() {
            Ok(token) => token,
            Err(e) => {
                log::error!("Error generating JWT: {}", e);
                anyhow::bail!(http::StatusCode::INTERNAL_SERVER_ERROR);
            }
        };
        let expires = time::OffsetDateTime::now_utc() + time::Duration::hours(1);
        let cookie = build_access_token_cookie(Some(token), expires);
        let cookie_string = cookie.to_string();
        let mut headers = vec![("Set-Cookie", cookie_string.as_str())];
        headers.extend_from_slice(&HEADERS_DEFAULT_RESP);
        let status = http::StatusCode::NO_CONTENT;
        let _resp = req.into_response(status.into(), None, &headers)?;
        Ok(())
    }

    pub fn post_logout(
        &self,
        req: esp_http_server::Request<&mut esp_http_server::EspHttpConnection<'_>>,
    ) -> Result<(), anyhow::Error> {
        let status = http::StatusCode::NO_CONTENT;
        if self.is_authorized(&req) {
            let expires = time::OffsetDateTime::UNIX_EPOCH;
            let cookie = build_access_token_cookie(None, expires);
            let cookie_string = cookie.to_string();
            let mut headers = vec![("Set-Cookie", cookie_string.as_str())];
            headers.extend_from_slice(&HEADERS_DEFAULT_RESP);
            let _resp = req.into_response(status.into(), None, &headers)?;
            return Ok(());
        }
        let _resp = req.into_response(status.into(), None, &HEADERS_DEFAULT_RESP)?;
        Ok(())
    }

    pub fn post_reboot(
        &self,
        req: esp_http_server::Request<&mut esp_http_server::EspHttpConnection<'_>>,
    ) -> Result<(), anyhow::Error> {
        if !self.is_authorized(&req) {
            report_unauthorized(req)?;
            return Ok(());
        }
        std::thread::spawn(move || {
            // Give the HTTP response some time to be sent before restarting
            esp_idf_hal::delay::FreeRtos::delay_ms(500);
            unsafe {
                esp_restart();
            }
        });
        let status = http::StatusCode::NO_CONTENT;
        let _resp = req.into_response(status.into(), None, &HEADERS_DEFAULT_RESP)?;
        Ok(())
    }

    pub fn post_weather_stations(
        &self,
        mut req: esp_http_server::Request<&mut esp_http_server::EspHttpConnection<'_>>,
        stations_arc: &ArcMtx<Vec<WeatherStation>>,
        storage_arc: &ArcMtx<EspConfigStorage>,
    ) -> Result<(), anyhow::Error> {
        if !self.is_authorized(&req) {
            report_unauthorized(req)?;
            return Ok(());
        }
        let mut stations = stations_arc.lock_anyhow()?;
        let body = read_body(&mut req)?;
        let mut stations_new: WeatherStationsLimited = serde_json::from_str(&body)?;
        storage_arc
            .lock_anyhow()?
            .set_str(WEATHER_STATIONS_NVS_KEY, &body)?;
        stations.clear();
        stations.append(&mut stations_new.0);
        report_no_content(req)?;
        Ok(())
    }

    pub fn get_weather_stations(
        &self,
        req: esp_http_server::Request<&mut esp_http_server::EspHttpConnection<'_>>,
        stations_arc: &ArcMtx<Vec<WeatherStation>>,
    ) -> Result<(), anyhow::Error> {
        if !self.is_authorized(&req) {
            report_unauthorized(req)?;
            return Ok(());
        }
        let stations = stations_arc.lock_anyhow()?;
        let body = serde_json::to_string(&stations.clone())?;
        report_json_content(req, &body)?;
        Ok(())
    }

    pub fn post_world_time_places(
        &self,
        mut req: esp_http_server::Request<&mut esp_http_server::EspHttpConnection<'_>>,
        places_arc: &ArcMtx<Vec<LocationData>>,
        storage_arc: &ArcMtx<EspConfigStorage>,
    ) -> Result<(), anyhow::Error> {
        if !self.is_authorized(&req) {
            report_unauthorized(req)?;
            return Ok(());
        }
        let mut places = places_arc.lock_anyhow()?;
        let body = read_body(&mut req)?;
        let mut places_new: LocationsLimited = serde_json::from_str(&body)?;
        storage_arc
            .lock_anyhow()?
            .set_str(WORLD_TIME_PLACES_NVS_KEY, &body)?;
        places.clear();
        places.append(&mut places_new.0);
        report_no_content(req)?;
        Ok(())
    }

    pub fn get_world_time_places(
        &self,
        req: esp_http_server::Request<&mut esp_http_server::EspHttpConnection<'_>>,
        places_arc: &ArcMtx<Vec<LocationData>>,
    ) -> Result<(), anyhow::Error> {
        if !self.is_authorized(&req) {
            report_unauthorized(req)?;
            return Ok(());
        }
        let places = places_arc.lock_anyhow()?;
        let body = serde_json::to_string(&places.clone())?;
        report_json_content(req, &body)?;
        Ok(())
    }

    pub fn post_admin_creds(
        &self,
        mut req: esp_http_server::Request<&mut esp_http_server::EspHttpConnection<'_>>,
        storage_arc: &ArcMtx<EspConfigStorage>,
    ) -> Result<(), anyhow::Error> {
        if !self.is_authorized(&req) {
            report_unauthorized(req)?;
            return Ok(());
        }
        let body = read_body(&mut req)?;
        let admin_creds_req: AdminCredentialsPost = serde_json::from_str(&body)?;

        if admin_creds_req.password.is_empty() {
            log::info!("Admin password cannot be empty");
            let status = http::StatusCode::BAD_REQUEST;
            let mut resp = req.into_response(status.into(), None, &HEADERS_DEFAULT_RESP)?;
            resp.write_all(status.to_string().as_bytes())?;
            return Ok(());
        }

        let admin_creds_new = AdminCredentials {
            password_hash: hash_password(&admin_creds_req.password)?,
        };
        let payload = serde_json::to_string(&admin_creds_new)?;
        storage_arc
            .lock_anyhow()?
            .set_str(ADMIN_CREDS_NVS_KEY, &payload)?;
        report_no_content(req)?;
        Ok(())
    }

    pub fn post_wifi_creds(
        &self,
        mut req: esp_http_server::Request<&mut esp_http_server::EspHttpConnection<'_>>,
        storage_arc: &ArcMtx<EspConfigStorage>,
    ) -> Result<(), anyhow::Error> {
        if !self.is_authorized(&req) {
            report_unauthorized(req)?;
            return Ok(());
        }
        let body = read_body(&mut req)?;
        let wifi_creds: WifiCredentialsPost = serde_json::from_str(&body)?;

        let mut storage = storage_arc.lock_anyhow()?;
        let current_creds = storage.get_str(WIFI_CREDS_NVS_KEY)?;
        let current_pass = if let Some(current_creds_str) = current_creds {
            let current_wifi_creds: WifiCredentials = serde_json::from_str(&current_creds_str)?;
            current_wifi_creds.password
        } else {
            String::new()
        };

        let new_wifi_creds = WifiCredentials {
            ssid: wifi_creds.ssid,
            password: wifi_creds.password.unwrap_or(current_pass),
        };

        let payload = serde_json::to_string(&new_wifi_creds)?;
        storage.set_str(WIFI_CREDS_NVS_KEY, &payload)?;
        report_no_content(req)?;
        Ok(())
    }

    pub fn get_wifi_creds(
        &self,
        req: esp_http_server::Request<&mut esp_http_server::EspHttpConnection<'_>>,
        storage_arc: &ArcMtx<EspConfigStorage>,
    ) -> Result<(), anyhow::Error> {
        if !self.is_authorized(&req) {
            report_unauthorized(req)?;
            return Ok(());
        }
        let wifi_creds_str_opt = storage_arc.lock_anyhow()?.get_str(WIFI_CREDS_NVS_KEY)?;

        let wifi_creds_get = match wifi_creds_str_opt {
            None => WifiCredentialsGet {
                ssid: String::new(),
            },
            Some(wifi_creds_str) => {
                let wifi_creds: WifiCredentials = serde_json::from_str(&wifi_creds_str)?;
                WifiCredentialsGet {
                    ssid: wifi_creds.ssid,
                }
            }
        };

        let status = http::StatusCode::OK;
        let mut resp = req.into_response(status.into(), None, &HEADERS_JSON_RESP)?;
        let body = serde_json::to_string(&wifi_creds_get)?;
        resp.write_all(body.as_bytes())?;
        Ok(())
    }

    pub fn post_system_settings(
        &self,
        mut req: esp_http_server::Request<&mut esp_http_server::EspHttpConnection<'_>>,
        system_settings_arc: &ArcMtx<SystemSettings>,
        storage_arc: &ArcMtx<EspConfigStorage>,
        system_settings_subscriber: &SyncSenderBlocking<SystemSettings>,
    ) -> Result<(), anyhow::Error> {
        if !self.is_authorized(&req) {
            report_unauthorized(req)?;
            return Ok(());
        }
        let body = read_body(&mut req)?;
        let system_settings_new: SystemSettings = serde_json::from_str(&body)?;
        storage_arc
            .lock_anyhow()?
            .set_str(SYSTEM_SETTINGS_NVS_KEY, &body)?;
        let mut system_settings = system_settings_arc.lock_anyhow()?;
        *system_settings = system_settings_new;
        system_settings_subscriber
            .send(system_settings.clone())
            .unwrap_or_else(|e| log::error!("Failed to notify system settings subscriber: {}", e));
        report_no_content(req)?;
        Ok(())
    }

    pub fn get_system_settings(
        &self,
        req: esp_http_server::Request<&mut esp_http_server::EspHttpConnection<'_>>,
        system_settings_arc: &ArcMtx<SystemSettings>,
    ) -> Result<(), anyhow::Error> {
        if !self.is_authorized(&req) {
            report_unauthorized(req)?;
            return Ok(());
        }
        let system_settings = system_settings_arc.lock_anyhow()?;
        let status = http::StatusCode::OK;
        let mut resp = req.into_response(status.into(), None, &HEADERS_JSON_RESP)?;
        let body = serde_json::to_string(&system_settings.clone())?;
        resp.write_all(body.as_bytes())?;
        Ok(())
    }
}
