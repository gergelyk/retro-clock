use crate::config_set::ConfigSet;
use crate::settings_admin::{AdminSettings, AdminSettingsSignals};
use crate::settings_wifi::{WifiSettings, WifiSettingsSignals};
use leptos::prelude::*;
use std::collections::HashSet;

#[allow(non_snake_case)]
#[component]
pub fn AccessPane(
    admin_signals: AdminSettingsSignals,
    wifi_signals: WifiSettingsSignals,
    set_config_updates: WriteSignal<HashSet<ConfigSet>>,
    set_show_popup: WriteSignal<Option<String>>,
    login_automatic: ReadSignal<bool>,
) -> impl IntoView {
    view! {
        <AdminSettings signals=admin_signals set_show_popup=set_show_popup />
        <hr class="m-4" />
        <WifiSettings
            signals=wifi_signals
            set_config_updates=set_config_updates
            set_show_popup=set_show_popup
        />

        <Show when=move || login_automatic.get()>
            <hr class="m-4" />
            <div class="border-l-4 border-blue-500 bg-blue-50 p-4 rounded">
                <p class="font-semibold text-blue-700">Hint</p>
                <p class="text-blue-700">
                    Certificate of this website has been issued by our private Certification Authority.
                    Root certificate of this CA can be downloaded from
                    <a href="/rootCA.pem" class="underline" download>
                        here
                    </a>.
                    Install this certificate in your browser or operating system to avoid security warnings.
                    <br />After downloading, CRC32 checksum of the file will show up on the display.
                    Compare it against checksum of the downloaded file to ensure its integrity.
                </p>
            </div>
        </Show>
    }
}
