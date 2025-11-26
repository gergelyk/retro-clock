use leptos::logging::log;
use leptos::prelude::*;
use leptos::task::spawn_local;
use retro_clock_core::creds_models::{AdminCredentialsPost, WifiCredentialsPost};
use retro_clock_core::settings_models::SystemSettings;
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use retro_clock_core::{weather_models::WeatherStation, worldtime_models::LocationData};

use crate::api::{
    upload_admin_creds, upload_system_settings, upload_weather_stations, upload_wifi_creds,
    upload_world_time_places,
};
use crate::config_set::ConfigSet;
use crate::models::UserDataBackup;
use crate::pane_system::SystemSettingsForm;
use crate::settings_admin::AdminSettingsSignals;
use crate::settings_wifi::WifiSettingsSignals;
use crate::window_utils::WaitCursorGuard;

fn parse_coordinates(input: &str) -> Option<(f64, f64)> {
    let parts: Vec<&str> = input.split(',').map(|s| s.trim()).collect();
    if parts.len() != 2 {
        return None;
    }
    let latitude = parts[0].parse::<f64>().ok()?;
    let longitude = parts[1].parse::<f64>().ok()?;
    Some((latitude, longitude))
}

#[allow(clippy::too_many_arguments)]
pub fn handle_save_button(
    config_updates: ReadSignal<HashSet<ConfigSet>>,
    set_config_updates: WriteSignal<HashSet<ConfigSet>>,
    admin_signals: AdminSettingsSignals,
    admin_pass_update: Signal<bool>,
    wifi_signals: WifiSettingsSignals,
    wifi_pass_update: Memo<bool>,
    system_settings: ReadSignal<SystemSettingsForm>,
    world_time_places: ReadSignal<Option<Vec<LocationData>>>,
    weather_stations: ReadSignal<Option<Vec<WeatherStation>>>,
    user_data_backup: Rc<RefCell<Option<UserDataBackup>>>,
    set_show_popup: WriteSignal<Option<String>>,
    set_show_restart: WriteSignal<bool>,
    set_busy: WriteSignal<bool>,
) {
    let config_updates_value = config_updates.get_untracked();
    let admin_pass1 = admin_signals.admin_pass1.get_untracked();
    let admin_pass2 = admin_signals.admin_pass2.get_untracked();
    let admin_pass_update = admin_pass_update.get_untracked();
    let wifi_ssid = wifi_signals.wifi_ssid.get_untracked();
    let wifi_pass = wifi_signals.wifi_pass.get_untracked();
    let wifi_pass_empty = wifi_signals.wifi_pass_empty.get_untracked();
    let wifi_pass_update = wifi_pass_update.get_untracked();
    let system_settings_form = system_settings.get_untracked();
    let places = world_time_places.get_untracked();
    let stations = weather_stations.get_untracked();

    spawn_local({
        async move {
            let mut restart_required = false;
            set_busy.set(true);
            defer! {
                set_busy.set(false);
            };
            let _wait_cursor_guard = WaitCursorGuard::new();
            if admin_pass_update && admin_pass1 != admin_pass2 {
                log!("Admin passwords do not match");
                set_show_popup.set(Some("Admin passwords do not match.".to_owned()));
                admin_signals.set_admin_pass1.set(String::new());
                admin_signals.set_admin_pass2.set(String::new());
                return;
            }
            let location_settings = if config_updates_value.contains(&ConfigSet::SystemSettings) {
                let Some((latitude, longitude)) =
                    parse_coordinates(&system_settings_form.coordinates)
                else {
                    set_show_popup
                        .set(
                            Some(
                                "Coordinates must be in Decimal Degrees format. That is two comma separated numbers, e.g., 37.77, -122.42"
                                    .to_owned(),
                            ),
                        );
                    return;
                };
                let Ok(time_zone) = system_settings_form.time_zone.parse::<chrono_tz::Tz>() else {
                    set_show_popup
                        .set(
                            Some(
                                "Time Zone must be a valid IANA timezone identifier. Please select one from the list."
                                    .to_owned(),
                            ),
                        );
                    return;
                };
                Some((system_settings_form, latitude, longitude, time_zone))
            } else {
                None
            };
            if admin_pass_update {
                let admin_creds = AdminCredentialsPost {
                    password: admin_pass1.clone(),
                };
                if let Err(e) = upload_admin_creds(&admin_creds).await {
                    log!("Failed to upload admin creds: {}", e);
                    set_show_popup.set(Some("Failed to upload admin creds.".to_owned()));
                } else {
                    log!("Successfully uploaded admin creds");
                    admin_signals.set_admin_pass1.set(String::new());
                    admin_signals.set_admin_pass2.set(String::new());
                }
            } else {
                log!("No admin creds changes to save");
            }
            if wifi_pass_update || config_updates_value.contains(&ConfigSet::WifiSsid) {
                let wifi_creds = WifiCredentialsPost {
                    ssid: wifi_ssid.clone(),
                    password: if wifi_pass_empty {
                        Some(String::new())
                    } else if !wifi_pass.is_empty() {
                        Some(wifi_pass.clone())
                    } else {
                        None
                    },
                };
                if let Err(e) = upload_wifi_creds(&wifi_creds).await {
                    log!("Failed to upload wifi creds: {}", e);
                    set_show_popup.set(Some("Failed to upload wifi creds.".to_owned()));
                } else {
                    set_config_updates.update(|updates| {
                        updates.remove(&ConfigSet::WifiSsid);
                    });
                    if let Some(ref mut backup) = *user_data_backup.borrow_mut() {
                        backup.wifi_ssid = wifi_ssid;
                    }
                    wifi_signals.set_wifi_pass.set(String::new());
                    wifi_signals.set_wifi_pass_empty.set(false);
                    log!("Successfully uploaded wifi creds");
                    restart_required = true;
                }
            } else {
                log!("No wifi creds changes to save");
            }
            if let Some((system_settings_form, latitude, longitude, time_zone)) = location_settings
            {
                let system_settings = SystemSettings {
                    latitude,
                    longitude,
                    coordinates_auto: system_settings_form.coordinates_auto,
                    time_zone,
                    time_zone_auto: system_settings_form.time_zone_auto,
                    bg_brightness: system_settings_form.bg_brightness,
                    bg_brightness_auto: system_settings_form.bg_brightness_auto,
                    led_brightness: system_settings_form.led_brightness,
                    led_brightness_auto: system_settings_form.led_brightness_auto,
                };
                if let Err(e) = upload_system_settings(&system_settings).await {
                    log!("Failed to upload system settings: {}", e);
                    set_show_popup.set(Some("Failed to upload system settings.".to_owned()));
                } else {
                    set_config_updates.update(|updates| {
                        updates.remove(&ConfigSet::SystemSettings);
                    });
                    if let Some(ref mut backup) = *user_data_backup.borrow_mut() {
                        backup.system_settings = system_settings_form;
                    }
                    log!("Successfully uploaded system settings");
                }
            } else {
                log!("No system settings changes to save");
            }
            if config_updates_value.contains(&ConfigSet::WorldTime) {
                if let Some(places) = places {
                    if let Err(e) = upload_world_time_places(&places).await {
                        log!("Failed to upload world time places: {}", e);
                        set_show_popup.set(Some("Failed to upload world time places.".to_owned()));
                    } else {
                        set_config_updates.update(|updates| {
                            updates.remove(&ConfigSet::WorldTime);
                        });
                        if let Some(ref mut backup) = *user_data_backup.borrow_mut() {
                            backup.world_time_places = places;
                        }
                        log!("Successfully uploaded world time places");
                    }
                } else {
                    log!("No world time places to save");
                }
            } else {
                log!("No world time changes to save");
            }
            if config_updates_value.contains(&ConfigSet::Weather) {
                if let Some(stations) = stations {
                    if let Err(e) = upload_weather_stations(&stations).await {
                        log!("Failed to upload weather stations: {}", e);
                        set_show_popup.set(Some("Failed to upload weather stations.".to_owned()));
                    } else {
                        set_config_updates.update(|updates| {
                            updates.remove(&ConfigSet::Weather);
                        });
                        if let Some(ref mut backup) = *user_data_backup.borrow_mut() {
                            backup.weather_stations = stations;
                        }
                        log!("Successfully uploaded weather stations");
                    }
                } else {
                    log!("No weather stations to save");
                }
            } else {
                log!("No weather changes to save");
            }
            if restart_required {
                set_show_restart.set(true);
            }
        }
    });
}
