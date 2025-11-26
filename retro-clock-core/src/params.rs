pub const WEATHER_DATA_CHUNK_SIZE: usize = 10;

const LCD_WIDTH: usize = 16;
pub const WEATHER_STATION_NAME_MAX_LEN: usize = LCD_WIDTH;
pub const WEATHER_STATION_URL_MAX_LEN: usize = 128;
pub const WEATHER_STATIONS_MAX_NUM: usize = 10;
pub const LOCATION_NAME_MAX_LEN: usize = LCD_WIDTH;
pub const TIMEZONE_NAME_MAX_LEN: usize = 128;
pub const LOCATIONS_MAX_NUM: usize = 10;

pub const ADMIN_PASSWORD_MAX_LEN: usize = 64;
pub const WIFI_PASSWORD_MAX_LEN: usize = 64; // WPA / WPA2 / WPA3 it is defined as 63
pub const SSID_MAX_LEN: usize = 32; // as defined by IEEE 802.11
