use leptos::logging::log;
use leptos::prelude::*;
use leptos::task::spawn_local;
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use crate::config_set::ConfigSet;
use crate::dialog_info::InfoDialog;
use crate::dialog_restart::RestartDialog;
use crate::pane_access::AccessPane;
use crate::pane_system::{SystemPane, SystemSettingsForm};
use crate::pane_weather::WeatherPane;
use crate::pane_world_time::WorldTimePane;
use crate::screen_login::LoginStatus;
use crate::window_utils::WaitCursorGuard;

use crate::api::{
    download_system_settings, download_weather_stations, download_wifi_creds,
    download_world_time_places, send_logout_request,
};
use crate::handler_cancel_button::handle_cancel_button;
use crate::handler_save_button::handle_save_button;
use crate::models::{Tab, UserDataBackup};
use crate::settings_admin::AdminSettingsSignals;
use crate::settings_wifi::WifiSettingsSignals;

#[allow(non_snake_case)]
#[component]
pub fn HomeScreen(
    set_login_status: WriteSignal<LoginStatus>,
    login_automatic: ReadSignal<bool>,
) -> impl IntoView {
    let admin_signals = AdminSettingsSignals::new();
    let wifi_signals = WifiSettingsSignals::new();

    let (system_settings, set_system_settings) = signal(SystemSettingsForm::default());
    let (config_updates, set_config_updates) = signal(HashSet::<ConfigSet>::new());
    let admin_pass_update = Signal::derive(move || {
        !admin_signals.admin_pass1.get().is_empty() || !admin_signals.admin_pass2.get().is_empty()
    });
    let wifi_pass_update = Memo::new(move |_| {
        !wifi_signals.wifi_pass.get().is_empty() || wifi_signals.wifi_pass_empty.get()
    });
    let anything_to_save = Memo::new(move |_| {
        admin_pass_update.get() || wifi_pass_update.get() || !config_updates.get().is_empty()
    });
    let anything_to_cancel = Signal::derive(move || {
        admin_pass_update.get() || wifi_pass_update.get() || !config_updates.get().is_empty()
    });
    let (busy, set_busy) = signal(false);
    let busy_or_logout_disabled = move || busy.get() || login_automatic.get();
    let (active_tab, set_active_tab) = signal(Tab::Access);
    let (show_popup, set_show_popup) = signal(Option::None::<String>);
    let (show_restart, set_show_restart) = signal(false);

    let (weather_stations, set_weather_stations) = signal(None);
    let (world_time_places, set_world_time_places) = signal(None);

    let user_data_backup: Rc<RefCell<Option<UserDataBackup>>> = Rc::new(RefCell::new(None));

    let popup_button_ref = NodeRef::<leptos::html::Button>::new();
    Effect::new(move |_| {
        if show_popup.get().is_some() {
            if let Some(btn) = popup_button_ref.get() {
                let _ = btn.focus();
            }
        }
    });

    let user_data_backup_clone = user_data_backup.clone();
    spawn_local({
        async move {
            set_busy.set(true);
            defer! { set_busy.set(false); }
            let _wait_cursor_guard = WaitCursorGuard::new();

            let wifi_creds = match download_wifi_creds().await {
                Ok(creds) => creds,
                Err(e) => {
                    log!("Failed to download wifi creds: {}", e);
                    set_show_popup.set(Some("Failed to download WiFi credentials.".to_owned()));
                    return;
                }
            };

            let system_settings = match download_system_settings().await {
                Ok(settings) => SystemSettingsForm {
                    coordinates: format!("{}, {}", settings.latitude, settings.longitude),
                    coordinates_auto: settings.coordinates_auto,
                    time_zone: settings.time_zone.name().to_string(),
                    time_zone_auto: settings.time_zone_auto,
                    bg_brightness: settings.bg_brightness,
                    bg_brightness_auto: settings.bg_brightness_auto,
                    led_brightness: settings.led_brightness,
                    led_brightness_auto: settings.led_brightness_auto,
                },
                Err(e) => {
                    log!("Failed to download system settings: {}", e);
                    set_show_popup.set(Some("Failed to download system settings.".to_owned()));
                    return;
                }
            };

            let world_time_places = match download_world_time_places().await {
                Ok(places) => places,
                Err(e) => {
                    log!("Failed to download world time places: {}", e);
                    set_show_popup.set(Some("Failed to download world time places.".to_owned()));
                    return;
                }
            };

            let weather_stations = match download_weather_stations().await {
                Ok(stations) => stations,
                Err(e) => {
                    log!("Failed to download weather stations: {}", e);
                    set_show_popup.set(Some("Failed to download weather stations.".to_owned()));
                    return;
                }
            };

            user_data_backup_clone.replace(Some(UserDataBackup {
                wifi_ssid: wifi_creds.ssid.clone(),
                system_settings: system_settings.clone(),
                weather_stations: weather_stations.clone(),
                world_time_places: world_time_places.clone(),
            }));
            wifi_signals.set_wifi_ssid.set(wifi_creds.ssid);
            set_system_settings.set(system_settings);
            set_world_time_places.set(Some(world_time_places));
            set_weather_stations.set(Some(weather_stations));
        }
    });

    let admin_signals_clone = admin_signals.clone();
    let wifi_signals_clone = wifi_signals.clone();
    let user_data_backup_clone = user_data_backup.clone();
    let cancel_button_handler = move |_| {
        handle_cancel_button(
            config_updates,
            set_config_updates,
            admin_signals_clone.clone(),
            wifi_signals_clone.clone(),
            set_system_settings,
            set_world_time_places,
            set_weather_stations,
            user_data_backup_clone.clone(),
        );
    };

    let admin_signals_clone = admin_signals.clone();
    let wifi_signals_clone = wifi_signals.clone();
    let save_button_handler = move |_| {
        handle_save_button(
            config_updates,
            set_config_updates,
            admin_signals_clone.clone(),
            admin_pass_update,
            wifi_signals_clone.clone(),
            wifi_pass_update,
            system_settings,
            world_time_places,
            weather_stations,
            user_data_backup.clone(),
            set_show_popup,
            set_show_restart,
            set_busy,
        );
    };

    let logout_button_handler = move |_| {
        spawn_local(async move {
            set_busy.set(true);
            defer! {
                set_busy.set(false);
            }
            let _wait_cursor_guard = WaitCursorGuard::new();
            match send_logout_request().await {
                Ok(_) => {
                    log!("Logout successful");
                    set_login_status.set(LoginStatus::LoggedOut);
                }
                Err(e) => {
                    log!("Logout failed: {:?}", e);
                }
            }
        });
    };

    let tab_class = move |tab: Tab| {
        move || {
            if active_tab.get() == tab {
                "select-none border-b-2 border-blue-500 text-blue-600 whitespace-nowrap py-2 px-4 text-sm font-medium cursor-pointer"
            } else {
                "select-none text-gray-600 hover:text-blue-600 whitespace-nowrap py-2 px-4 text-sm font-medium cursor-pointer"
            }
        }
    };

    view! {
        <header class="bg-gray-100 shadow-md">
            <div class="max-w-7xl mx-auto px-4 py-3 flex items-center justify-between">
                <div class="text-xl font-bold text-blue-600">Retro Clock</div>

                <nav class="space-x-4">
                    <button
                        class="select-none enabled:hover:text-blue-600 enabled:text-gray-700 disabled:text-gray-300"
                        disabled=move || !anything_to_cancel.get() || busy.get()
                        on:click=cancel_button_handler
                    >
                        "✖️ Cancel"
                    </button>
                    <button
                        class="select-none enabled:hover:text-blue-600 enabled:text-gray-700 disabled:text-gray-300"
                        disabled=move || !anything_to_save.get() || busy.get()
                        on:click=save_button_handler
                    >
                        "💾 Save"
                    </button>
                    <button
                        class="select-none enabled:hover:text-blue-600 enabled:text-gray-700 disabled:text-gray-300"
                        disabled=busy_or_logout_disabled
                        on:click=logout_button_handler
                    >
                        "⤴️ Logout"
                    </button>
                </nav>
            </div>
        </header>

        <div class="w-full max-w-4xl mx-auto mt-6">
            <div class="border-b border-gray-300">
                <nav class="-mb-px flex space-x-4" aria-label="Tabs">
                    <a
                        class=tab_class(Tab::Access)
                        on:click=move |_| set_active_tab.set(Tab::Access)
                    >
                        "Access"
                    </a>
                    <a
                        class=tab_class(Tab::System)
                        on:click=move |_| set_active_tab.set(Tab::System)
                    >
                        "System"
                    </a>
                    // <a
                    // class=tab_class(Tab::Alarms)
                    // on:click=move |_| set_active_tab.set(Tab::Alarms)
                    // >
                    // "Alarms"
                    // </a>
                    <a
                        class=tab_class(Tab::WorldTime)
                        on:click=move |_| set_active_tab.set(Tab::WorldTime)
                    >
                        "World Time"
                    </a>
                    <a
                        class=tab_class(Tab::Weather)
                        on:click=move |_| set_active_tab.set(Tab::Weather)
                    >
                        "Weather"
                    </a>
                </nav>
            </div>

            <div class="mt-4">

                {move || match active_tab.get() {
                    Tab::Access => {
                        view! {
                            <div class="p-4 border rounded bg-white shadow-sm">
                                <AccessPane
                                    admin_signals=admin_signals.clone()
                                    wifi_signals=wifi_signals.clone()
                                    set_config_updates=set_config_updates
                                    set_show_popup=set_show_popup
                                    login_automatic=login_automatic
                                />
                            </div>
                        }
                            .into_any()
                    }
                    Tab::System => {
                        view! {
                            <div class="p-4 border rounded bg-white shadow-sm">
                                <SystemPane
                                    system_settings=system_settings
                                    set_system_settings=set_system_settings
                                    set_config_updates=set_config_updates
                                    set_show_popup=set_show_popup
                                />
                            </div>
                        }
                            .into_any()
                    }
                    Tab::WorldTime => {
                        // Tab::Alarms => {
                        // view! {
                        // <div class="p-4 border rounded bg-white shadow-sm">
                        // <p class="text-gray-600">"Alarms tab content"</p>
                        // </div>
                        // }
                        // .into_any()
                        // }
                        view! {
                            <div class="p-4 border rounded bg-white shadow-sm">
                                <WorldTimePane
                                    world_time_places=world_time_places
                                    set_world_time_places=set_world_time_places
                                    set_config_updates=set_config_updates
                                    set_show_popup=set_show_popup
                                />
                            </div>
                        }
                            .into_any()
                    }
                    Tab::Weather => {
                        view! {
                            <div class="p-4 border rounded bg-white shadow-sm">
                                <WeatherPane
                                    weather_stations=weather_stations
                                    set_weather_stations=set_weather_stations
                                    set_config_updates=set_config_updates
                                    set_show_popup=set_show_popup
                                />
                            </div>
                        }
                            .into_any()
                    }
                }}

            </div>
        </div>

        {move || {
            if let Some(content) = show_popup.get() {
                view! {
                    <InfoDialog
                        set_show_popup=set_show_popup
                        content=content
                        popup_button_ref=popup_button_ref
                    />
                }
                    .into_any()
            } else {
                view! { "" }.into_any()
            }
        }}

        <Show when=move || show_restart.get()>
            <RestartDialog
                set_show_restart=set_show_restart
                set_busy=set_busy
                set_login_status=set_login_status
                popup_button_ref=popup_button_ref
            />
        </Show>
    }
}
