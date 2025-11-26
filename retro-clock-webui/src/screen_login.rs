use crate::api::send_login_request;
use crate::window_utils::WaitCursorGuard;
use leptos::html;
use leptos::logging::log;
use leptos::prelude::*;
use leptos::task::spawn_local;
use std::rc::Rc;

#[derive(Clone)]
pub enum LoginStatus {
    Unknown,
    LoggedIn,
    LoggedOut,
    Terminated,
}

#[allow(non_snake_case)]
#[component]
pub fn LoginScreen(set_login_status: WriteSignal<LoginStatus>) -> impl IntoView {
    let (password, set_password) = signal(String::new());
    let (error, set_error) = signal(None::<String>);
    let (busy, set_busy) = signal(false);
    let password_ref = NodeRef::<html::Input>::new();

    fn set_focus_on_password(password_ref: &NodeRef<html::Input>) {
        if let Some(input) = password_ref.get_untracked() {
            input.focus().unwrap_or_else(|e| {
                log!("Failed to focus password input: {:?}", e);
            });
        }
    }

    Effect::new(move |_| {
        set_focus_on_password(&password_ref);
    });

    let handle_login: Rc<dyn Fn()> = Rc::new({
        move || {
            let pwd = password.get();
            if pwd.is_empty() {
                set_error.set(Some("Password cannot be empty.".into()));
            } else {
                let pwd = pwd.clone();
                spawn_local(async move {
                    set_busy.set(true);
                    defer! { set_busy.set(false); }
                    let _wait_cursor_guard = WaitCursorGuard::new();
                    match send_login_request(&pwd).await {
                        Ok(password_valid) => {
                            if password_valid {
                                log!("Password valid");
                                set_error.set(None);
                                set_login_status.set(LoginStatus::LoggedIn);
                            } else {
                                log!("Password invalid");
                                set_error.set(Some("Invalid password. Try again.<br><br><small>You don't know the password? Restart your<br>device with the button held during bootup.</small>".into()));
                                set_password.set(String::new());
                                spawn_local(async move {
                                    // defer focus to next microtask so DOM updates complete first
                                    leptos::task::tick().await;
                                    set_focus_on_password(&password_ref);
                                });
                            }
                        }
                        Err(e) => {
                            log!("Could not login: {:?}", e);
                            set_error.set(Some("Internal error.".into()));
                            set_password.set(String::new());
                        }
                    }
                });
            }
        }
    });

    let handle_login_for_input = handle_login.clone();
    let handle_login_for_button = handle_login.clone();

    view! {
        <div class="flex items-center justify-center min-h-screen bg-gray-100">
            <div class="w-full max-w-sm p-6 bg-white rounded-lg shadow-md">
                <h2 class="text-2xl font-bold mb-6 text-center">"Retro Clock"</h2>

                {move || {
                    if let Some(err) = error.get() {
                        if password.get().is_empty() {
                            view! {
                                <div
                                    inner_html=err
                                    class="mb-4 text-red-500 text-sm text-center"
                                ></div>
                            }
                                .into_any()
                        } else {
                            ().into_any()
                        }
                    } else {
                        ().into_any()
                    }
                }}

                <input
                    type="password"
                    placeholder="Password"
                    node_ref=password_ref
                    prop:value=password
                    on:input=move |ev| {
                        set_password.set(event_target_value(&ev));
                        set_error.set(None);
                    }
                    on:keydown=move |ev| {
                        if ev.key() == "Enter" {
                            handle_login_for_input();
                        }
                    }
                    class="w-full p-2 mb-4 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
                    disabled=busy
                />

                <button
                    on:click=move |_| handle_login_for_button()
                    class="w-full bg-blue-500 text-white font-bold py-2 rounded hover:bg-blue-600 transition-colors disabled:text-gray-300"
                    disabled=busy
                >
                    "Login"
                </button>
            </div>
        </div>
    }
}
