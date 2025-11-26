use crate::config_set::ConfigSet;
use leptos::prelude::*;
use retro_clock_core::params::{SSID_MAX_LEN, WIFI_PASSWORD_MAX_LEN};
use std::collections::HashSet;

#[derive(Clone)]
pub struct WifiSettingsSignals {
    pub wifi_ssid: ReadSignal<String>,
    pub set_wifi_ssid: WriteSignal<String>,
    pub wifi_pass: ReadSignal<String>,
    pub set_wifi_pass: WriteSignal<String>,
    pub wifi_pass_empty: ReadSignal<bool>,
    pub set_wifi_pass_empty: WriteSignal<bool>,
}

impl WifiSettingsSignals {
    pub fn new() -> Self {
        let (wifi_ssid, set_wifi_ssid) = signal(String::new());
        let (wifi_pass, set_wifi_pass) = signal(String::new());
        let (wifi_pass_empty, set_wifi_pass_empty) = signal(false);
        Self {
            wifi_ssid,
            set_wifi_ssid,
            wifi_pass,
            set_wifi_pass,
            wifi_pass_empty,
            set_wifi_pass_empty,
        }
    }
}

#[allow(non_snake_case)]
#[component]
pub fn WifiSettings(
    signals: WifiSettingsSignals,
    set_config_updates: WriteSignal<HashSet<ConfigSet>>,
    set_show_popup: WriteSignal<Option<String>>,
) -> impl IntoView {
    view! {
        <h2 class="text-lg font-semibold mb-4">
            WiFi
            <span
                on:click=move |_| {
                    set_show_popup.set(Some("Password to your WiFi network.".to_owned()));
                }
                class="text-blue-500 text-sm font-bold cursor-pointer"
            >
                "ⓘ"
            </span>
        </h2>
        <table class="w-2/3">
            <tr>
                <td class="px-4">
                    <label for="wifi_ssid" class="block text-sm font-medium text-gray-700">
                        Network Name:
                    </label>
                    <input
                        class="my-2 border border-gray-300 rounded px-2 py-1"
                        id="wifi_ssid"
                        autocomplete="off"
                        type="text"
                        placeholder="SSID"
                        maxlength=SSID_MAX_LEN
                        prop:value=signals.wifi_ssid
                        on:input=move |ev| {
                            let value = event_target_value(&ev);
                            signals.set_wifi_ssid.set(value);
                            set_config_updates
                                .update(|updates| {
                                    updates.insert(ConfigSet::WifiSsid);
                                });
                        }
                    />
                </td>
                <td class="px-4">
                    <label for="wifi_password" class="block text-sm font-medium text-gray-700">
                        Password:
                    </label>
                    <div class="flex items-center space-x-2 my-2">
                        <input
                            class="my-2 border border-gray-300 rounded px-2 py-1"
                            id="wifi_password"
                            type="password"
                            placeholder="••••••••"
                            maxlength=WIFI_PASSWORD_MAX_LEN
                            prop:value=signals.wifi_pass
                            on:input=move |ev| {
                                let value = event_target_value(&ev);
                                signals.set_wifi_pass.set(value);
                                signals.set_wifi_pass_empty.set(false);
                            }
                        />
                        <button
                            class="whitespace-nowrap text-sm ml-2 px-2 py-1 bg-blue-500 text-white rounded hover:bg-blue-600 disabled:text-gray-300"
                            disabled=move || signals.wifi_pass_empty.get()
                            on:click=move |_| {
                                signals.set_wifi_pass.set(String::new());
                                signals.set_wifi_pass_empty.set(true);
                            }
                        >
                            "Set Empty"
                        </button>
                    </div>
                </td>
            </tr>
        </table>
    }
}
