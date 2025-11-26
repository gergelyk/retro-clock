use leptos::logging::log;
use leptos::prelude::*;
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use retro_clock_core::{weather_models::WeatherStation, worldtime_models::LocationData};

use crate::config_set::ConfigSet;
use crate::models::UserDataBackup;
use crate::pane_system::SystemSettingsForm;
use crate::settings_admin::AdminSettingsSignals;
use crate::settings_wifi::WifiSettingsSignals;

#[allow(clippy::too_many_arguments)]
pub fn handle_cancel_button(
    config_updates: ReadSignal<HashSet<ConfigSet>>,
    set_config_updates: WriteSignal<HashSet<ConfigSet>>,
    admin_signals: AdminSettingsSignals,
    wifi_signals: WifiSettingsSignals,
    set_system_settings: WriteSignal<SystemSettingsForm>,
    set_world_time_places: WriteSignal<Option<Vec<LocationData>>>,
    set_weather_stations: WriteSignal<Option<Vec<WeatherStation>>>,
    user_data_backup: Rc<RefCell<Option<UserDataBackup>>>,
) {
    let config_updates_value = config_updates.get_untracked();
    admin_signals.set_admin_pass1.set(String::new());
    admin_signals.set_admin_pass2.set(String::new());
    wifi_signals.set_wifi_pass.set(String::new());
    wifi_signals.set_wifi_pass_empty.set(false);

    let Some(user_data_backup) = &*user_data_backup.borrow() else {
        log!("No user data backup available for cancel");
        return;
    };

    if config_updates_value.contains(&ConfigSet::WifiSsid) {
        wifi_signals
            .set_wifi_ssid
            .set(user_data_backup.wifi_ssid.clone());
    }
    if config_updates_value.contains(&ConfigSet::SystemSettings) {
        set_system_settings.set(user_data_backup.system_settings.clone());
    }
    if config_updates_value.contains(&ConfigSet::WorldTime) {
        set_world_time_places.set(Some(user_data_backup.world_time_places.clone()));
    }
    if config_updates_value.contains(&ConfigSet::Weather) {
        set_weather_stations.set(Some(user_data_backup.weather_stations.clone()));
    }
    set_config_updates.set(HashSet::new());
}
