use crate::settings_models::SystemSettings;
use crate::worldtime_models::LocationInfo;

pub fn overlay_location_info(
    fetched_location_info: &LocationInfo,
    system_settings: &SystemSettings,
) -> LocationInfo {
    let mut location_info = LocationInfo {
        timezone: system_settings.time_zone,
        latitude: system_settings.latitude,
        longitude: system_settings.longitude,
    };

    if system_settings.coordinates_auto || system_settings.time_zone_auto {
        if system_settings.coordinates_auto {
            location_info.latitude = fetched_location_info.latitude;
            location_info.longitude = fetched_location_info.longitude;
        }
        if system_settings.time_zone_auto {
            location_info.timezone = fetched_location_info.timezone;
        }
    };

    location_info
}
