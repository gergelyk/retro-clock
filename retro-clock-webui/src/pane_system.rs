use crate::config_set::ConfigSet;
use chrono_tz::TZ_VARIANTS;
use leptos::prelude::*;
use std::collections::HashSet;

#[derive(Default, Clone)]
pub struct SystemSettingsForm {
    pub coordinates: String,
    pub coordinates_auto: bool,
    pub time_zone: String,
    pub time_zone_auto: bool,
    pub bg_brightness: u8,
    pub bg_brightness_auto: bool,
    pub led_brightness: u8,
    pub led_brightness_auto: bool,
}

#[allow(non_snake_case)]
#[component]
pub fn SystemPane(
    system_settings: ReadSignal<SystemSettingsForm>,
    set_system_settings: WriteSignal<SystemSettingsForm>,
    set_config_updates: WriteSignal<HashSet<ConfigSet>>,
    set_show_popup: WriteSignal<Option<String>>,
) -> impl IntoView {
    let coordinates = Memo::new(move |_| system_settings.get().coordinates);
    let coordinates_auto = Memo::new(move |_| system_settings.get().coordinates_auto);
    let time_zone = Memo::new(move |_| system_settings.get().time_zone);
    let time_zone_auto = Memo::new(move |_| system_settings.get().time_zone_auto);
    let bg_brightness = Memo::new(move |_| system_settings.get().bg_brightness);
    let bg_brightness_auto = Memo::new(move |_| system_settings.get().bg_brightness_auto);
    let led_brightness = Memo::new(move |_| system_settings.get().led_brightness);
    let led_brightness_auto = Memo::new(move |_| system_settings.get().led_brightness_auto);

    view! {
        <h2 class="text-lg font-semibold mb-4">
            Location
            <span
                on:click=move |_| {
                    set_show_popup
                        .set(
                            Some(
                                "<b>Coordinates</b> are used for collecting forecast data.<br><br><b>Time Zone</b> is used for determining local time.<br><br>If either of them is set manually, this doesn't affect the other."
                                    .to_owned(),
                            ),
                        );
                }
                class="text-blue-500 text-sm font-bold cursor-pointer"
            >
                "ⓘ"
            </span>
        </h2>
        <table class="w-2/3">
            <tr>
                <td class="px-4">
                    <label for="local_coordinates" class="block text-sm font-medium text-gray-700">
                        Local Coordinates:
                    </label>
                    <input
                        class="block my-2 border border-gray-300 rounded px-2 py-1"
                        id="local_coordinates"
                        autocomplete="off"
                        type="text"
                        placeholder="Latitude, Longitude"
                        disabled=coordinates_auto
                        maxlength=64
                        prop:value=coordinates
                        on:input=move |ev| {
                            let value = event_target_value(&ev);
                            set_system_settings
                                .update(|settings| {
                                    settings.coordinates = value;
                                });
                            set_config_updates
                                .update(|updates| {
                                    updates.insert(ConfigSet::SystemSettings);
                                });
                        }
                    />
                    <div class="flex items-center space-x-2">
                        <input
                            type="checkbox"
                            id="coordinates_auto"
                            class="w-4 h-4 text-blue-600 bg-gray-100 border-gray-300 rounded focus:ring-blue-500"
                            prop:checked=coordinates_auto
                            on:change=move |ev| {
                                let checked = event_target_checked(&ev);
                                set_system_settings
                                    .update(|settings| {
                                        settings.coordinates_auto = checked;
                                    });
                                set_config_updates
                                    .update(|updates| {
                                        updates.insert(ConfigSet::SystemSettings);
                                    });
                            }
                        />
                        <label for="coordinates_auto" class="text-sm text-gray-700 py-2">
                            Detect Automatically
                        </label>
                    </div>
                </td>
                <td class="px-4">
                    <label for="time_zone" class="block text-sm font-medium text-gray-700">
                        Local Time Zone:
                    </label>
                    <input
                        class="my-2 border border-gray-300 rounded px-2 py-1"
                        id="time_zone"
                        type="text"
                        placeholder="Identifier"
                        list="valid_place_identifiers"
                        disabled=time_zone_auto
                        prop:value=time_zone
                        on:input=move |ev| {
                            let value = event_target_value(&ev);
                            set_system_settings
                                .update(|settings| {
                                    settings.time_zone = value;
                                });
                            set_config_updates
                                .update(|updates| {
                                    updates.insert(ConfigSet::SystemSettings);
                                });
                        }
                    />
                    <datalist id="valid_place_identifiers">
                        {TZ_VARIANTS
                            .iter()
                            .map(|tz| {
                                view! { <option value=tz.to_string() /> }
                            })
                            .collect::<Vec<_>>()
                            .into_any()}
                    </datalist>
                    <div class="flex items-center space-x-2">
                        <input
                            type="checkbox"
                            id="time_zone_auto"
                            class="w-4 h-4 text-blue-600 bg-gray-100 border-gray-300 rounded focus:ring-blue-500"
                            prop:checked=time_zone_auto
                            on:change=move |ev| {
                                let checked = event_target_checked(&ev);
                                set_system_settings
                                    .update(|settings| {
                                        settings.time_zone_auto = checked;
                                    });
                                set_config_updates
                                    .update(|updates| {
                                        updates.insert(ConfigSet::SystemSettings);
                                    });
                            }
                        />
                        <label for="time_zone_auto" class="text-sm text-gray-700 py-2">
                            Detect Automatically
                        </label>
                    </div>
                </td>
            </tr>
        </table>

        <hr class="m-4" />

        <h2 class="text-lg font-semibold mb-4">
            Ilumination
            <span
                on:click=move |_| {
                    set_show_popup
                        .set(
                            Some(
                                "Brightness of the physical components. Automatic adjustment uses ambient light sensor.<br>If setting is adjusted automatically, value at the slider will determine the maximum brightness."
                                    .to_owned(),
                            ),
                        );
                }
                class="text-blue-500 text-sm font-bold cursor-pointer"
            >
                "ⓘ"
            </span>
        </h2>
        <table class="w-2/3">
            <tr>
                <td class="px-4">
                    <label for="bg_brightness" class="block text-sm font-medium text-gray-700">
                        Backlight Brightness:
                    </label>
                    <input
                        type="range"
                        id="bg_brightness"
                        min="0"
                        max="100"
                        value="50"
                        class="h-2 bg-gray-200 rounded-lg accent-blue-600 cursor-pointer"
                        disabled=bg_brightness_auto
                        prop:value=bg_brightness
                        on:input=move |ev| {
                            let value = event_target_value(&ev);
                            if let Ok(parsed) = value.parse::<u8>() {
                                set_system_settings
                                    .update(|settings| {
                                        settings.bg_brightness = parsed;
                                    });
                                set_config_updates
                                    .update(|updates| {
                                        updates.insert(ConfigSet::SystemSettings);
                                    });
                            }
                        }
                    />
                    <span class="px-4 text-sm font-medium text-gray-700">
                        {move || bg_brightness.get()}%
                    </span>
                    <div class="flex items-center space-x-2">
                        <input
                            type="checkbox"
                            id="bg_brightness_auto"
                            class="w-4 h-4 text-blue-600 bg-gray-100 border-gray-300 rounded focus:ring-blue-500"
                            prop:checked=bg_brightness_auto
                            on:change=move |ev| {
                                let checked = event_target_checked(&ev);
                                set_system_settings
                                    .update(|settings| {
                                        settings.bg_brightness_auto = checked;
                                    });
                                set_config_updates
                                    .update(|updates| {
                                        updates.insert(ConfigSet::SystemSettings);
                                    });
                            }
                        />
                        <label for="bg_brightness_auto" class="text-sm text-gray-700 py-2">
                            Adjust Automatically
                        </label>
                    </div>
                </td>
                <td class="px-4">
                    <label for="led_brightness" class="block text-sm font-medium text-gray-700">
                        Status Indicators Brightness:
                    </label>
                    <input
                        type="range"
                        id="led_brightness"
                        min="0"
                        max="100"
                        value="50"
                        class="h-2 bg-gray-200 rounded-lg accent-blue-600 cursor-pointer"
                        disabled=led_brightness_auto
                        prop:value=led_brightness
                        on:input=move |ev| {
                            let value = event_target_value(&ev);
                            if let Ok(parsed) = value.parse::<u8>() {
                                set_system_settings
                                    .update(|settings| {
                                        settings.led_brightness = parsed;
                                    });
                                set_config_updates
                                    .update(|updates| {
                                        updates.insert(ConfigSet::SystemSettings);
                                    });
                            }
                        }
                    />
                    <span class="px-4 text-sm font-medium text-gray-700">
                        {move || led_brightness.get()}%
                    </span>
                    <div class="flex items-center space-x-2">
                        <input
                            type="checkbox"
                            id="led_brightness_auto"
                            class="w-4 h-4 text-blue-600 bg-gray-100 border-gray-300 rounded focus:ring-blue-500"
                            prop:checked=led_brightness_auto
                            on:change=move |ev| {
                                let checked = event_target_checked(&ev);
                                set_system_settings
                                    .update(|settings| {
                                        settings.led_brightness_auto = checked;
                                    });
                                set_config_updates
                                    .update(|updates| {
                                        updates.insert(ConfigSet::SystemSettings);
                                    });
                            }
                        />
                        <label for="led_brightness_auto" class="text-sm text-gray-700 py-2">
                            Adjust Automatically
                        </label>
                    </div>
                </td>
            </tr>
        </table>
    }
}
