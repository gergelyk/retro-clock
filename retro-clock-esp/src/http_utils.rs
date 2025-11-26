use crate::params::HTTP_SERVER_READ_BUF_SIZE;
use array_concat::concat_arrays;
use cookie::Cookie;
use esp_idf_hal::io::Write;
use esp_idf_svc::http::server as esp_http_server;
use retro_clock_core::login::verify_jwt;
use std::collections::HashMap;

#[cfg(feature = "no_cors")]
pub const UI_ORIGIN: &str = env!("UI_ORIGIN");

#[cfg(feature = "no_cors")]
pub const HEADERS_UI_TEST: [(&str, &str); 2] = [
    ("Access-Control-Allow-Origin", UI_ORIGIN),
    ("Access-Control-Allow-Credentials", "true"),
];
#[cfg(not(feature = "no_cors"))]
pub const HEADERS_UI_TEST: [(&str, &str); 0] = [];
pub const HEADERS_DEFAULT_RESP: [(&str, &str); HEADERS_UI_TEST.len()] = HEADERS_UI_TEST;
const HEADERS_JSON_RESP_BASE: [(&str, &str); 1] = [("Content-Type", "application/json")];
pub const HEADERS_JSON_RESP: [(&str, &str); HEADERS_UI_TEST.len() + HEADERS_JSON_RESP_BASE.len()] =
    concat_arrays!(HEADERS_UI_TEST, HEADERS_JSON_RESP_BASE);

pub fn get_cookies(
    req: &esp_http_server::Request<&mut esp_http_server::EspHttpConnection<'_>>,
) -> Option<HashMap<String, String>> {
    req.header("Cookie").map(parse_cookies)
}

fn parse_cookies(cookie_header: &str) -> HashMap<String, String> {
    cookie_header
        .split(';')
        .filter_map(|pair| {
            let mut parts = pair.trim().splitn(2, '=');
            if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                Some((key.trim().to_string(), value.trim().to_string()))
            } else {
                None
            }
        })
        .collect()
}

pub fn read_body(
    req: &mut esp_http_server::Request<&mut esp_http_server::EspHttpConnection<'_>>,
) -> Result<String, anyhow::Error> {
    let mut body_bytes = Vec::new();
    let mut buffer = [0u8; 1024];
    let mut len_total = 0;
    loop {
        let len = req.read(&mut buffer)?;
        if len == 0 {
            break; // EOF
        }
        len_total += len;
        if len_total > HTTP_SERVER_READ_BUF_SIZE {
            anyhow::bail!("HTTP request body too large");
        }
        body_bytes.extend_from_slice(&buffer[..len]);
    }
    let body = std::str::from_utf8(&body_bytes[..len_total])?;
    Ok(body.to_string())
}

pub fn build_access_token_cookie(
    token: Option<String>,
    expires: time::OffsetDateTime,
) -> cookie::CookieBuilder<'static> {
    #[cfg(feature = "no_cors")]
    let same_site = cookie::SameSite::None;
    #[cfg(not(feature = "no_cors"))]
    let same_site = cookie::SameSite::Strict;

    let name = "access_token";
    let builder = match token {
        Some(t) => Cookie::build((name, t)),
        None => Cookie::build(name),
    };

    builder
        .path("/")
        .secure(true) // If cookie wasn't secure Firefox would drop the cookie whenever its window is closed.
        .http_only(true)
        .same_site(same_site)
        .expires(expires)
}

pub fn is_authorized(
    req: &esp_http_server::Request<&mut esp_http_server::EspHttpConnection<'_>>,
) -> bool {
    let cookies = get_cookies(req);

    if let Some(jar) = cookies {
        if let Some(access_token) = jar.get("access_token") {
            log::debug!("Found access token cookie: {}", access_token);
            match verify_jwt(access_token) {
                Ok(claims) => {
                    log::info!("Access token is valid");
                    log::debug!("Claims: {:?}", claims);
                    return true;
                }
                Err(e) => {
                    log::info!("Access token verification failed: {}", e);
                }
            }
        } else {
            log::info!("No access token cookie found");
        }
    } else {
        log::info!("No cookies found");
    };
    false
}

pub fn report_unauthorized(
    req: esp_http_server::Request<&mut esp_http_server::EspHttpConnection<'_>>,
) -> Result<(), anyhow::Error> {
    let status = http::StatusCode::UNAUTHORIZED;
    let mut resp = req.into_response(status.into(), None, &HEADERS_DEFAULT_RESP)?;
    resp.write_all(status.to_string().as_bytes())?;
    Ok(())
}

pub fn report_no_content(
    req: esp_http_server::Request<&mut esp_http_server::EspHttpConnection<'_>>,
) -> Result<(), anyhow::Error> {
    let status = http::StatusCode::NO_CONTENT;
    let _resp = req.into_response(status.into(), None, &HEADERS_DEFAULT_RESP)?;
    Ok(())
}

pub fn report_json_content(
    req: esp_http_server::Request<&mut esp_http_server::EspHttpConnection<'_>>,
    body: &String,
) -> Result<(), anyhow::Error> {
    let status = http::StatusCode::OK;
    let mut resp = req.into_response(status.into(), None, &HEADERS_JSON_RESP)?;
    resp.write_all(body.as_bytes())?;
    Ok(())
}
