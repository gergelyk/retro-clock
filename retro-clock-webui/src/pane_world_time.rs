use crate::config_set::ConfigSet;
use chrono_tz::{Tz, TZ_VARIANTS};
use leptos::prelude::*;
use retro_clock_core::{
    params::{LOCATIONS_MAX_NUM, LOCATION_NAME_MAX_LEN, TIMEZONE_NAME_MAX_LEN},
    worldtime_models::LocationData,
};
use std::collections::HashSet;
use std::str::FromStr;

#[allow(non_snake_case)]
#[component]
pub fn WorldTimePane(
    world_time_places: ReadSignal<Option<Vec<LocationData>>>,
    set_world_time_places: WriteSignal<Option<Vec<LocationData>>>,
    set_config_updates: WriteSignal<HashSet<ConfigSet>>,
    set_show_popup: WriteSignal<Option<String>>,
) -> impl IntoView {
    let (name_input, set_name_input) = signal(String::new());
    let (identifier_input, set_identifier_input) = signal(String::new());

    view! {
        <table class="table-auto w-full border border-gray-200">
            <thead class="bg-gray-100">
                <tr>
                    <th class="w-1/3 px-4 py-2 text-left">
                        <div class="flex items-center space-x-1">
                            <span>Place Name</span>
                            <span
                                on:click=move |_| {
                                    set_show_popup
                                        .set(Some("Short name for the place.".to_owned()));
                                }
                                class="text-blue-500 text-sm font-bold cursor-pointer"
                            >
                                "ⓘ"
                            </span>
                        </div>
                    </th>
                    <th class="w-2/3 px-4 py-2 text-left">
                        <div class="flex items-center space-x-1">
                            <span>Time Zone</span>
                            <span
                                on:click=move |_| {
                                    set_show_popup
                                        .set(
                                            Some(
                                                r#"Time zone identifier, as specified in <a class="text-blue-600" target="_blank" href="https://data.iana.org/time-zones/tzdb-2021a/zone1970.tab">IANA</a>. Example: Europe/Warsaw"#
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
                    if let Some(places) = world_time_places.get() {
                        places
                            .iter()
                            .enumerate()
                            .map(|(index, place)| {

                                view! {
                                    <tr class="border-t">
                                        <td class="w-1/3 px-4 py-2">{place.name.clone()}</td>
                                        <td class="w-2/3 px-4 py-2">
                                            {place.timezone.to_string().clone()}
                                        </td>
                                        <td class="w-16 px-2 py-2 text-center w-1 whitespace-nowrap">
                                            <button
                                                on:click=move |_| {
                                                    set_world_time_places
                                                        .update(|places_opt| {
                                                            if let Some(places) = places_opt {
                                                                if index > 0 {
                                                                    places.swap(index, index - 1);
                                                                    set_config_updates
                                                                        .update(|updates| {
                                                                            updates.insert(ConfigSet::WorldTime);
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
                                                    set_world_time_places
                                                        .update(|places_opt| {
                                                            if let Some(places) = places_opt {
                                                                if index < places.len() - 1 {
                                                                    places.swap(index, index + 1);
                                                                    set_config_updates
                                                                        .update(|updates| {
                                                                            updates.insert(ConfigSet::WorldTime);
                                                                        });
                                                                }
                                                            }
                                                        });
                                                }
                                                disabled=index == places.len() - 1
                                                class="text-sm px-2 py-1 bg-blue-500 text-white rounded hover:bg-blue-600 mr-1 disabled:opacity-50 disabled:cursor-not-allowed"
                                            >
                                                "↓"
                                            </button>
                                            <button
                                                on:click=move |_| {
                                                    set_world_time_places
                                                        .update(|places_opt| {
                                                            if let Some(places) = places_opt {
                                                                places.remove(index);
                                                                set_config_updates
                                                                    .update(|updates| {
                                                                        updates.insert(ConfigSet::WorldTime);
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
                            maxlength=LOCATION_NAME_MAX_LEN
                            prop:value=name_input
                            class="w-full border border-gray-300 rounded px-2 py-1 focus:outline-none focus:ring-2 focus:ring-blue-400"
                        />
                    </td>
                    <td class="w-2/3 px-4 py-2">
                        <input
                            on:input=move |ev| {
                                let value = event_target_value(&ev);
                                set_identifier_input.set(value);
                            }
                            type="text"
                            list="valid_place_identifiers"
                            placeholder="Identifier"
                            maxlength=TIMEZONE_NAME_MAX_LEN
                            prop:value=identifier_input
                            class="w-full border border-gray-300 rounded px-2 py-1 focus:outline-none focus:ring-2 focus:ring-blue-400"
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
                    </td>
                    <td class="w-16 px-2 py-2 text-center w-1 whitespace-nowrap">
                        <button
                            on:click=move |_| {
                                let world_time_places_len = world_time_places
                                    .get()
                                    .map(|places| places.len())
                                    .unwrap_or(0);
                                if world_time_places_len >= LOCATIONS_MAX_NUM {
                                    let msg = format!(
                                        "Cannot add more than {} places.",
                                        LOCATIONS_MAX_NUM,
                                    );
                                    set_show_popup.set(Some(msg));
                                    return;
                                }
                                let place_name = name_input.get().trim().to_string();
                                if place_name.is_empty() {
                                    set_show_popup.set(Some("Name cannot be empty.".to_owned()));
                                    return;
                                }
                                let place_identifier = identifier_input.get().trim().to_string();
                                if place_identifier.is_empty() {
                                    set_show_popup
                                        .set(Some("Identifier cannot be empty.".to_owned()));
                                    return;
                                }
                                let tz = match Tz::from_str(&place_identifier) {
                                    Ok(tz) => tz,
                                    Err(_) => {
                                        set_show_popup
                                            .set(Some("Invalid time zone identifier.".to_owned()));
                                        return;
                                    }
                                };
                                set_world_time_places
                                    .update(|stations_opt| {
                                        if let Some(stations) = stations_opt {
                                            stations
                                                .push(LocationData {
                                                    name: Some(place_name),
                                                    timezone: tz,
                                                });
                                            set_config_updates
                                                .update(|updates| {
                                                    updates.insert(ConfigSet::WorldTime);
                                                });
                                        }
                                        set_name_input.set(String::new());
                                        set_identifier_input.set(String::new());
                                    });
                            }
                            class="text-sm px-10 py-1 bg-green-500 text-white rounded hover:bg-green-600 disabled:opacity-50 disabled:cursor-not-allowed"
                            disabled=move || {
                                name_input.get().trim().is_empty()
                                    || identifier_input.get().trim().is_empty()
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
