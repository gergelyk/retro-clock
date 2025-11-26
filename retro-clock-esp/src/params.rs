// Must be at least as large as the biggest expected HTTP response body
// Consider:
// - location info: 284 B
// - forecast: max 1.70 KB for metno-seamless
// - weather: ~159 B boilerplate + ~185 B * WEATHER_STATIONS_MAX_NUM
pub const HTTP_CLIENT_READ_BUF_SIZE: usize = 2048;

// Must be at least as large as the biggest expected HTTP request body
// Consider:
// - SET /weather-stations:
//   ~5 B boilerplate + (~14 B boilerplate + WEATHER_STATION_NAME_MAX_LEN + WEATHER_STATION_URL_MAX_LEN) * WEATHER_STATIONS_MAX_NUM
//   = 1585 B
// - SET /world-time-places:
//   ~5 B boilerplate + (~44 B boilerplate + LOCATION_NAME_MAX_LEN + TIMEZONE_NAME_MAX_LEN) * LOCATIONS_MAX_NUM
//   = 945 B
pub const HTTP_SERVER_READ_BUF_SIZE: usize = 2048;

// Must be at least as large as the biggest expected payload read from NVS
// Consider HTTP_SERVER_READ_BUF_SIZE + some boilerplate
pub const NVS_BUF_SIZE: usize = HTTP_SERVER_READ_BUF_SIZE + 1024;

// For explanations, read the comments in button.rs
pub const BUTTON_SAMPLING_INTERVAL_TICKS: u32 = 10; // 100ms
pub const BUTTON_DEBOUNCING_MS: u32 = 50;
pub const BUTTON_HOLD_MS: u32 = 400;

// WiFi connection establishment
pub const WIFI_CONNECT_ITERATIONS: u32 = 50;
pub const WIFI_CONNECT_INTERVAL_MS: u32 = 100;
pub const WIFI_GET_IP_ITERATIONS: u32 = 50;
pub const WIFI_GET_IP_INTERVAL_MS: u32 = 100;
pub const WIFI_RESTORE_ITERATIONS: u32 = 10;

// CPU frequency limits
pub const CPU_MAX_FREQ_MHZ: i32 = 240;
pub const CPU_MIN_FREQ_MHZ: i32 = 80;
