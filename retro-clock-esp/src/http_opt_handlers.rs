use crate::http_utils::HEADERS_UI_TEST;
use array_concat::concat_arrays;
use esp_idf_svc::http::server::{EspHttpConnection, Request};

const HEADERS_DEFAULT_RESP_BASE: [(&str, &str); 1] =
    [("Access-Control-Allow-Headers", "Content-Type")];
const HEADERS_DEFAULT_RESP: [(&str, &str);
    HEADERS_UI_TEST.len() + HEADERS_DEFAULT_RESP_BASE.len()] =
    concat_arrays!(HEADERS_UI_TEST, HEADERS_DEFAULT_RESP_BASE);

fn get_options(
    req: Request<&mut EspHttpConnection<'_>>,
    methods: &str,
) -> Result<(), anyhow::Error> {
    let mut headers = vec![("Access-Control-Allow-Methods", methods)];
    headers.extend_from_slice(&HEADERS_DEFAULT_RESP);
    let status = http::StatusCode::NO_CONTENT;
    let _resp = req.into_response(status.into(), None, &headers)?;
    Ok(())
}

pub fn options_check_authorization(
    req: Request<&mut EspHttpConnection<'_>>,
) -> Result<(), anyhow::Error> {
    get_options(req, "OPTIONS, GET")
}

pub fn options_login(req: Request<&mut EspHttpConnection<'_>>) -> Result<(), anyhow::Error> {
    get_options(req, "OPTIONS, POST")
}

pub fn options_logout(req: Request<&mut EspHttpConnection<'_>>) -> Result<(), anyhow::Error> {
    get_options(req, "OPTIONS, POST")
}

pub fn options_admin_creds(req: Request<&mut EspHttpConnection<'_>>) -> Result<(), anyhow::Error> {
    get_options(req, "OPTIONS, POST")
}

pub fn options_wifi_creds(req: Request<&mut EspHttpConnection<'_>>) -> Result<(), anyhow::Error> {
    get_options(req, "OPTIONS, GET, POST")
}

pub fn options_weather_stations(
    req: Request<&mut EspHttpConnection<'_>>,
) -> Result<(), anyhow::Error> {
    get_options(req, "OPTIONS, GET, POST")
}

pub fn options_world_time_places(
    req: Request<&mut EspHttpConnection<'_>>,
) -> Result<(), anyhow::Error> {
    get_options(req, "OPTIONS, GET, POST")
}

pub fn options_system_settings(
    req: Request<&mut EspHttpConnection<'_>>,
) -> Result<(), anyhow::Error> {
    get_options(req, "OPTIONS, GET, POST")
}

pub fn options_reboot(req: Request<&mut EspHttpConnection<'_>>) -> Result<(), anyhow::Error> {
    get_options(req, "OPTIONS, POST")
}
