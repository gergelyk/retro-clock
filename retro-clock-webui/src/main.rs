#[macro_use(defer)]
extern crate scopeguard;

mod api;
mod config_set;
mod dialog_info;
mod dialog_restart;
mod handler_cancel_button;
mod handler_save_button;
mod models;
mod pane_access;
mod pane_system;
mod pane_weather;
mod pane_world_time;
mod screen_home;
mod screen_login;
mod settings_admin;
mod settings_wifi;
mod window_utils;

use api::{check_authorization, API_URL};
use leptos::logging::log;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};
use screen_home::HomeScreen;
use screen_login::{LoginScreen, LoginStatus};

fn main() {
    leptos::mount::mount_to_body(Routing);
}

#[allow(non_snake_case)]
#[component]
pub fn Routing() -> impl IntoView {
    view! {
        <Router>
            <Routes fallback=|| view! { <div>"Page not found."</div> }>
                <Route path=path!("/") view=Home />
            </Routes>
        </Router>
    }
}

#[allow(non_snake_case)]
#[component]
fn Home() -> impl IntoView {
    log!("api_url: {:?}", API_URL);

    let (login_status, set_login_status) = signal(LoginStatus::Unknown);
    let (login_automatic, set_login_automatic) = signal(false);

    spawn_local({
        async move {
            match check_authorization().await {
                Ok(authorization_status) => {
                    log!(
                        "Authorized: {}, automatically: {}",
                        authorization_status.authorized,
                        authorization_status.automatically
                    );
                    set_login_status.set(if authorization_status.authorized {
                        LoginStatus::LoggedIn
                    } else {
                        LoginStatus::LoggedOut
                    });
                    set_login_automatic.set(authorization_status.automatically);
                }
                Err(e) => {
                    log!("Failed to check authorization: {:?}", e);
                }
            }
        }
    });

    view! {
        {move || {
            match login_status.get() {
                LoginStatus::Unknown => {
                    view! {
                        <div class="flex items-center justify-center min-h-screen bg-gray-100 text-2xl">
                            <h2>"Loading..."</h2>
                        </div>
                    }
                        .into_any()
                }
                LoginStatus::LoggedIn => {
                    view! {
                        <HomeScreen
                            set_login_status=set_login_status
                            login_automatic=login_automatic
                        />
                    }
                        .into_any()
                }
                LoginStatus::LoggedOut => {
                    view! { <LoginScreen set_login_status=set_login_status /> }.into_any()
                }
                LoginStatus::Terminated => {
                    view! {
                        <div class="flex flex-col items-center justify-center min-h-screen bg-gray-100">
                            <h2 class="text-2xl">"See you later!"</h2>
                            <p>You can refresh this page to log in again.</p>
                        </div>
                    }
                        .into_any()
                }
            }
        }}
    }
    .into_any()
}
