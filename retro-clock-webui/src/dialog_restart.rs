use crate::api::reboot_device;
use crate::screen_login::LoginStatus;
use crate::window_utils::WaitCursorGuard;
use gloo_timers::future::sleep;
use leptos::logging::log;
use leptos::prelude::*;
use leptos::task::spawn_local;
use std::time::Duration;

#[allow(non_snake_case)]
#[component]
pub fn RestartDialog(
    set_show_restart: WriteSignal<bool>,
    set_busy: WriteSignal<bool>,
    set_login_status: WriteSignal<LoginStatus>,
    popup_button_ref: NodeRef<leptos::html::Button>,
) -> impl IntoView {
    view! {
        <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center">
            <div class="bg-white rounded-lg p-6 max-w-sm shadow-lg relative">
                <p class="mb-4 text-left">
                    Changing WiFi credentials take effect after the next restart. Would you like to restart your device now?
                </p>
                <p class="mb-4 text-right">
                    <button
                        node_ref=popup_button_ref
                        class="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700"
                        on:click=move |_| set_show_restart.set(false)
                    >
                        "No"
                    </button>
                    <button
                        node_ref=popup_button_ref
                        class="ml-2 px-4 py-2 bg-yellow-600 text-white rounded hover:bg-yellow-700"
                        on:click=move |_| {
                            set_show_restart.set(false);
                            spawn_local(async move {
                                set_busy.set(true);
                                defer! {
                                    set_busy.set(false);
                                }
                                let _wait_cursor_guard = WaitCursorGuard::new();
                                reboot_device()
                                    .await
                                    .unwrap_or_else(|e| {
                                        log!("Failed to reboot device: {}", e);
                                    });
                                log!("Waiting for device to start reboot...");
                                sleep(Duration::from_secs(1)).await;
                                log!("Assuming that reboot is in progress");
                                set_login_status.set(LoginStatus::Terminated);
                            });
                        }
                    >
                        "Yes, restart now"
                    </button>
                </p>
            </div>
        </div>
    }
}
