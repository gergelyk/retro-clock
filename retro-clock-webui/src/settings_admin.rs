use leptos::prelude::*;
use retro_clock_core::params::ADMIN_PASSWORD_MAX_LEN;

#[derive(Clone)]
pub struct AdminSettingsSignals {
    pub admin_pass1: ReadSignal<String>,
    pub set_admin_pass1: WriteSignal<String>,
    pub admin_pass2: ReadSignal<String>,
    pub set_admin_pass2: WriteSignal<String>,
}

impl AdminSettingsSignals {
    pub fn new() -> Self {
        let (admin_pass1, set_admin_pass1) = signal(String::new());
        let (admin_pass2, set_admin_pass2) = signal(String::new());
        Self {
            admin_pass1,
            set_admin_pass1,
            admin_pass2,
            set_admin_pass2,
        }
    }
}

#[allow(non_snake_case)]
#[component]
pub fn AdminSettings(
    signals: AdminSettingsSignals,
    set_show_popup: WriteSignal<Option<String>>,
) -> impl IntoView {
    view! {
        <h2 class="text-lg font-semibold mb-4">
            Admin
            <span
                on:click=move |_| {
                    set_show_popup
                        .set(
                            Some(
                                "Password used for logging into this configuration page.".to_owned(),
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
                    <label for="admin_password" class="block text-sm font-medium text-gray-700">
                        Password:
                    </label>
                    <input
                        class="my-2 border border-gray-300 rounded px-2 py-1"
                        id="admin_password"
                        type="password"
                        placeholder="••••••••"
                        maxlength=ADMIN_PASSWORD_MAX_LEN
                        prop:value=signals.admin_pass1
                        on:input=move |ev| {
                            let value = event_target_value(&ev);
                            signals.set_admin_pass1.set(value);
                        }
                    />
                </td>
                <td class="px-4">
                    <label
                        for="admin_password_repeat"
                        class="block text-sm font-medium text-gray-700"
                    >
                        Repeat Password:
                    </label>
                    <input
                        class="my-2 border border-gray-300 rounded px-2 py-1"
                        id="admin_password_repeat"
                        type="password"
                        placeholder="••••••••"
                        maxlength=ADMIN_PASSWORD_MAX_LEN
                        prop:value=signals.admin_pass2
                        on:input=move |ev| {
                            let value = event_target_value(&ev);
                            signals.set_admin_pass2.set(value);
                        }
                    />
                </td>
            </tr>
        </table>
    }
}
