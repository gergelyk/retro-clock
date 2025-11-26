use crate::config_set::ConfigSet;
use leptos::prelude::*;
use retro_clock_core::{
    params::{WEATHER_STATIONS_MAX_NUM, WEATHER_STATION_NAME_MAX_LEN, WEATHER_STATION_URL_MAX_LEN},
    weather_models::WeatherStation,
};
use std::collections::HashSet;

#[allow(non_snake_case)]
#[component]
pub fn WeatherPane(
    weather_stations: ReadSignal<Option<Vec<WeatherStation>>>,
    set_weather_stations: WriteSignal<Option<Vec<WeatherStation>>>,
    set_config_updates: WriteSignal<HashSet<ConfigSet>>,
    set_show_popup: WriteSignal<Option<String>>,
) -> impl IntoView {
    let (name_input, set_name_input) = signal(String::new());
    let (url_input, set_url_input) = signal(String::new());

    view! {
        <table class="table-auto w-full border border-gray-200">
            <thead class="bg-gray-100">
                <tr>
                    <th class="w-1/3 px-4 py-2 text-left">
                        <div class="flex items-center space-x-1">
                            <span>Station Name</span>
                            <span
                                on:click=move |_| {
                                    set_show_popup
                                        .set(
                                            Some("Short name for the weather station.".to_owned()),
                                        );
                                }
                                class="text-blue-500 text-sm font-bold cursor-pointer"
                            >
                                "ⓘ"
                            </span>
                        </div>
                    </th>
                    <th class="w-2/3 px-4 py-2 text-left">
                        <div class="flex items-center space-x-1">
                            <span>Data Source</span>
                            <span
                                on:click=move |_| {
                                    set_show_popup
                                        .set(
                                            Some(
                                                r#"URL to the weather station. Acceptable sources are listed in configuration page at <a class="text-blue-600" target="_blank" href="https://weather.krason.dev/">https://weather.krason.dev/</a> "#
                                                    .to_owned(),
                                            ),
                                        );
                                }
                                class="text-blue-500 text-sm font-bold cursor-pointer"
                            >
                                "ⓘ"
                            </span>
                        </div>
                    </th>
                    <th class="w-16 px-2 py-2 text-center w-1 whitespace-nowrap">Actions</th>
                </tr>
            </thead>
            <tbody>

                {move || {
                    if let Some(stations) = weather_stations.get() {
                        stations
                            .iter()
                            .enumerate()
                            .map(|(index, station)| {

                                view! {
                                    <tr class="border-t">
                                        <td class="w-1/3 px-4 py-2">{station.name.clone()}</td>
                                        <td class="w-2/3 px-4 py-2">
                                            <input
                                                class="w-full border border-gray-300 rounded px-2 py-1"
                                                readonly
                                                type="text"
                                                value=station.url.clone()
                                            />
                                        </td>
                                        <td class="w-16 px-2 py-2 text-center w-1 whitespace-nowrap">
                                            <button
                                                on:click=move |_| {
                                                    set_weather_stations
                                                        .update(|stations_opt| {
                                                            if let Some(stations) = stations_opt {
                                                                if index > 0 {
                                                                    stations.swap(index, index - 1);
                                                                    set_config_updates
                                                                        .update(|updates| {
                                                                            updates.insert(ConfigSet::Weather);
                                                                        });
                                                                }
                                                            }
                                                        });
                                                }
                                                disabled=index == 0
                                                class="text-sm px-2 py-1 bg-blue-500 text-white rounded hover:bg-blue-600 mr-1 disabled:opacity-50 disabled:cursor-not-allowed"
                                            >
                                                "↑"
                                            </button>
                                            <button
                                                on:click=move |_| {
                                                    set_weather_stations
                                                        .update(|stations_opt| {
                                                            if let Some(stations) = stations_opt {
                                                                if index < stations.len() - 1 {
                                                                    stations.swap(index, index + 1);
                                                                    set_config_updates
                                                                        .update(|updates| {
                                                                            updates.insert(ConfigSet::Weather);
                                                                        });
                                                                }
                                                            }
                                                        });
                                                }
                                                disabled=index == stations.len() - 1
                                                class="text-sm px-2 py-1 bg-blue-500 text-white rounded hover:bg-blue-600 mr-1 disabled:opacity-50 disabled:cursor-not-allowed"
                                            >
                                                "↓"
                                            </button>
                                            <button
                                                on:click=move |_| {
                                                    set_weather_stations
                                                        .update(|stations_opt| {
                                                            if let Some(stations) = stations_opt {
                                                                stations.remove(index);
                                                                set_config_updates
                                                                    .update(|updates| {
                                                                        updates.insert(ConfigSet::Weather);
                                                                    });
                                                            }
                                                        });
                                                }
                                                class="text-sm px-2 py-1 bg-red-500 text-white rounded hover:bg-red-600"
                                            >
                                                "✕"
                                            </button>
                                        </td>
                                    </tr>
                                }
                                    .into_view()
                            })
                            .collect::<Vec<_>>()
                            .into_any()
                    } else {
                        ().into_any()
                    }
                }} <tr class="border-t bg-gray-50">
                    <td class="w-1/3 px-4 py-2">
                        <input
                            on:input=move |ev| {
                                let value = event_target_value(&ev);
                                set_name_input.set(value);
                            }
                            type="text"
                            placeholder="Name"
                            maxlength=WEATHER_STATION_NAME_MAX_LEN
                            prop:value=name_input
                            class="w-full border border-gray-300 rounded px-2 py-1 focus:outline-none focus:ring-2 focus:ring-blue-400"
                        />
                    </td>
                    <td class="w-2/3 px-4 py-2">
                        <input
                            on:input=move |ev| {
                                let value = event_target_value(&ev);
                                set_url_input.set(value);
                            }
                            type="text"
                            placeholder="URL"
                            maxlength=WEATHER_STATION_URL_MAX_LEN
                            prop:value=url_input
                            class="w-full border border-gray-300 rounded px-2 py-1 focus:outline-none focus:ring-2 focus:ring-blue-400"
                        />
                    </td>
                    <td class="w-16 px-2 py-2 text-center w-1 whitespace-nowrap">
                        <button
                            on:click=move |_| {
                                let weather_stations_len = weather_stations
                                    .get()
                                    .map(|stations| stations.len())
                                    .unwrap_or(0);
                                if weather_stations_len >= WEATHER_STATIONS_MAX_NUM {
                                    let msg = format!(
                                        "Cannot add more than {} weather stations.",
                                        WEATHER_STATIONS_MAX_NUM,
                                    );
                                    set_show_popup.set(Some(msg));
                                    return;
                                }
                                let name = name_input.get().trim().to_string();
                                if name.is_empty() {
                                    set_show_popup.set(Some("Name cannot be empty.".to_owned()));
                                    return;
                                }
                                let url = url_input.get().trim().to_string();
                                if url.is_empty() {
                                    set_show_popup.set(Some("URL cannot be empty.".to_owned()));
                                    return;
                                }
                                set_weather_stations
                                    .update(|stations_opt| {
                                        if let Some(stations) = stations_opt {
                                            stations.push(WeatherStation { name, url });
                                            set_config_updates
                                                .update(|updates| {
                                                    updates.insert(ConfigSet::Weather);
                                                });
                                        }
                                        set_name_input.set(String::new());
                                        set_url_input.set(String::new());
                                    });
                            }
                            class="text-sm px-10 py-1 bg-green-500 text-white rounded hover:bg-green-600 disabled:opacity-50 disabled:cursor-not-allowed"
                            disabled=move || {
                                name_input.get().trim().is_empty()
                                    || url_input.get().trim().is_empty()
                            }
                        >
                            "+"
                        </button>
                    </td>
                </tr>

            </tbody>
        </table>
    }
}
