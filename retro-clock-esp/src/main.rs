mod buffers;
mod button;
mod buzzer;
mod debug_utils;
#[cfg(not(feature = "no_file_server"))]
mod file_server;
mod hd44780;
mod http_client;
mod http_handlers;
mod http_opt_handlers;
mod http_server_setup;
mod http_utils;
mod illumination;
mod illumination_updater;
mod led_drivers;
mod params;
mod serial_monitor;
mod storage;
mod string_utils;
mod system_utils;
mod terminal;
mod threading;
mod user_command;
mod wifi_setup;

use anyhow::Context;
use std::sync::{atomic, mpsc, Arc};
use std::time::Duration;

use esp_idf_hal::{
    adc::oneshot::{config::AdcChannelConfig, AdcChannelDriver, AdcDriver},
    cpu::Core,
    delay, gpio,
    ledc::{config::TimerConfig, LedcDriver, LedcTimerDriver},
    prelude::*,
};

use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::peripherals::Peripherals,
    http::server as esp_http_server,
    mdns::EspMdns,
    netif::IpEvent,
    nvs::EspDefaultNvsPartition,
    sntp::{EspSntp, SyncStatus},
    tls::X509,
    wifi::EspWifi,
};

use retro_clock_core::{
    application::Application,
    arc_mtx::{arc_mtx_new, ArcMtx, MutexExt},
    busy_indicator::BusyIndicator,
    channel_utils::{channel_blocking, channel_dropping, flush_channel},
    creds_models::WifiCredentials,
    error_indicator::ErrorIndicator,
    forecast_page::ForecastPage,
    http_client::HttpClient,
    location_client::LocationClient,
    messages::BootupMessage,
    pager::Pager,
    settings_models::SystemSettings,
    system_settings::overlay_location_info,
    terminal::Terminal,
    threading::ThreadBuilder,
    weather_models::WeatherStation,
    weather_page::WeatherPage,
    worldtime_models::LocationData,
    worldtime_models::LocationInfo,
    worldtime_page::WorldTimePage,
};

use buffers::ChangeDetector;
use button::{ButtonDriver, ButtonEvent};
use buzzer::BuzzerDriver;
use debug_utils::{log_stack_stats, log_thread_info};
use http_server_setup::register_http_handlers;
use illumination::{IlluminationSettings, IlluminationUpdate};
use illumination_updater::IlluminationUpdater;
use led_drivers::{LedDriverGuards, LedDrivers};
use params::{CPU_MAX_FREQ_MHZ, CPU_MIN_FREQ_MHZ};
use serial_monitor::SerialMonitor;
use storage::{
    EspConfigStorage, SYSTEM_SETTINGS_NVS_KEY, WEATHER_STATIONS_NVS_KEY, WIFI_CREDS_NVS_KEY,
    WORLD_TIME_PLACES_NVS_KEY,
};
use system_utils::{configure_power_management, enter_power_safe_mode, restart_sntp, BusyWait};
use user_command::UserCommand;
use wifi_setup::{setup_wifi_access_point, setup_wifi_station, WifiMonitor};

// we enforce null-terminated strings (needed for compilation in release mode)
const SERVER_CERT: &str = concat!(include_str!("../certs/retro-clock.local.crt"), "\0");
const SERVER_KEY: &str = concat!(include_str!("../certs/retro-clock.local.key"), "\0");

fn main() -> anyhow::Result<()> {
    if let Err(e) = submain() {
        log::error!("Submain thread error: {:?}", e);
        #[cfg(feature = "panic_on_thread_error")]
        panic!("Submain thread error: {:?}", e);
    }
    Ok(())
}

fn submain() -> anyhow::Result<()> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    log::info!("Starting up");
    log_thread_info();
    configure_power_management(CPU_MAX_FREQ_MHZ, CPU_MIN_FREQ_MHZ, false)?;

    ///////////////////////////////////////////////////////////////////////////
    // region: Storage
    ///////////////////////////////////////////////////////////////////////////
    let nvs_partition = EspDefaultNvsPartition::take()?;
    let mut storage = EspConfigStorage::new(nvs_partition.clone(), "config")?;

    let places_json = storage.get_str(WORLD_TIME_PLACES_NVS_KEY)?;
    let places = places_json
        .and_then(|s| serde_json::from_str::<Vec<LocationData>>(&s).ok())
        .unwrap_or_default();
    let stations_json = storage.get_str(WEATHER_STATIONS_NVS_KEY)?;
    let stations = stations_json
        .and_then(|s| serde_json::from_str::<Vec<WeatherStation>>(&s).ok())
        .unwrap_or_default();
    let wifi_creds_json = storage.get_str(WIFI_CREDS_NVS_KEY)?;
    let wifi_creds = wifi_creds_json
        .and_then(|s| serde_json::from_str::<WifiCredentials>(&s).ok())
        .unwrap_or_default();
    let system_settings_json = storage.get_str(SYSTEM_SETTINGS_NVS_KEY)?;
    let system_settings = system_settings_json
        .and_then(|s| serde_json::from_str::<SystemSettings>(&s).ok())
        .unwrap_or_default();
    let illumination_settings_init = IlluminationSettings::from_system_settings(&system_settings);

    ///////////////////////////////////////////////////////////////////////////
    // region: Shared Data
    ///////////////////////////////////////////////////////////////////////////
    let storage_arc = arc_mtx_new(storage);
    let places_arc = arc_mtx_new(places);
    let stations_arc = arc_mtx_new(stations);
    let system_settings_init = system_settings.clone();
    let system_settings_arc = arc_mtx_new(system_settings);
    let simulate_on_battery = Arc::new(atomic::AtomicBool::new(false));
    // Mutexes for hardware drivers are defined in Peripherals Setup section

    // dropping commands is ok if they are submitted too fast
    let (user_command_tx, user_command_rx) = channel_dropping("UserCommand", 10);
    let (illumination_tx, illumination_rx) = channel_blocking("IlluminationUpdate", 10);
    let (ap_sta_ip_assigned_tx, ap_sta_ip_assigned_rx) = channel_blocking("ApStaIpAssigned", 1);
    let (settings_update_tx, settings_update_rx) = channel_blocking("SettingsUpdate", 10);
    // short channel and send_try is ok, we don't want too much sounds
    let (buzzer_tx, buzzer_rx) = channel_dropping("SoundEffect", 4);
    let (message_tx, message_rx) = channel_blocking("TextMessage", 1);
    // Channels for the button and LED indicator are defined in corresponding modules

    ///////////////////////////////////////////////////////////////////////////
    // region: Peripherals Setup
    ///////////////////////////////////////////////////////////////////////////

    let peripherals = Peripherals::take()?;
    let sysloop = EspSystemEventLoop::take()?;

    let usb_d_minus = peripherals.pins.gpio19;
    let usb_d_plus = peripherals.pins.gpio20;
    let config = esp_idf_hal::usb_serial::UsbSerialConfig::default();
    let mut usb_serial = esp_idf_hal::usb_serial::UsbSerialDriver::new(
        peripherals.usb_serial,
        usb_d_minus,
        usb_d_plus,
        &config,
    )?;

    // We don't want debug cable to be connected/disconnected during normal operation
    // so we just check its state at bootup and assume it doesn't change.
    let is_jtag_connected = usb_serial.is_connected();

    let button_thread_builder = threading::EspBuilder::new()
        .name("Button".into())
        .stack_size(4 * 1024)
        .priority(11)
        .pin_to_core(Core::Core1);

    let user_command_tx_clone = user_command_tx.clone();
    let button = ButtonDriver::new(
        peripherals.pins.gpio0.into(),
        button_thread_builder,
        move |event| {
            let user_command_tx_clone = user_command_tx_clone.clone();
            let handle_event = move || -> anyhow::Result<()> {
                match event {
                    ButtonEvent::Pressed => {
                        user_command_tx_clone.send(UserCommand::GoNextCard)?;
                    }
                    ButtonEvent::Released => {}
                    ButtonEvent::Held => {
                        user_command_tx_clone.send(UserCommand::GoNextPage)?;
                    }
                }
                Ok(())
            };
            if let Err(e) = handle_event() {
                log::error!("Failed to handle button event: {}", e);
            }
        },
    )?;

    let mut display = hd44780::Hd44780::new(
        peripherals.pins.gpio18.into(),
        peripherals.pins.gpio35.into(),
        peripherals.pins.gpio36.into(),
        peripherals.pins.gpio14.into(),
        peripherals.pins.gpio9.into(),
        peripherals.pins.gpio8.into(),
        peripherals.pins.gpio7.into(),
        peripherals.pins.gpio6.into(),
        peripherals.pins.gpio5.into(),
        peripherals.pins.gpio3.into(),
        peripherals.pins.gpio1.into(),
    )?;
    display.init()?;

    let mut term = terminal::EspTerminal::new(display);
    term.init()?;

    let led_timer_driver = LedcTimerDriver::new(
        peripherals.ledc.timer0,
        &TimerConfig::default().frequency(25.kHz().into()),
    )?;

    let mut led_bg_driver = LedcDriver::new(
        peripherals.ledc.channel0,
        &led_timer_driver,
        peripherals.pins.gpio44,
    )?;
    let led_bg_max_duty = led_bg_driver.get_max_duty();
    led_bg_driver.set_duty(0)?;
    let led_bg_driver_arc = arc_mtx_new(led_bg_driver);

    let mut led_green_driver = LedcDriver::new(
        peripherals.ledc.channel2,
        &led_timer_driver,
        peripherals.pins.gpio43,
    )?;
    let led_green_max_duty = led_green_driver.get_max_duty();
    led_green_driver.set_duty(0)?;
    let led_green_driver_arc = arc_mtx_new(led_green_driver);

    let mut led_red_driver = LedcDriver::new(
        peripherals.ledc.channel1,
        &led_timer_driver,
        peripherals.pins.gpio33,
    )?;
    let led_red_max_duty = led_red_driver.get_max_duty();
    led_red_driver.set_duty(0)?;
    let led_red_driver_arc = arc_mtx_new(led_red_driver);

    let mut busy_indicator = BusyIndicator::new();
    let busy_indicator_handle = busy_indicator.get_handle();
    let led_green_indicator_handle = busy_indicator.get_handle();

    let mut error_indicator = ErrorIndicator::new();
    let error_indicator_handle = error_indicator.get_handle();
    let led_red_indicator_handle = error_indicator.get_handle();

    let led_green_duty = Arc::new(atomic::AtomicU32::new(led_green_max_duty / 32));
    let led_red_duty = Arc::new(atomic::AtomicU32::new(led_red_max_duty / 32));

    let power_connected_pin = peripherals.pins.gpio21;
    let mut power_connected = gpio::PinDriver::input(power_connected_pin)?;
    power_connected.set_pull(gpio::Pull::Up)?;

    let adc1 = AdcDriver::new(peripherals.adc1)?;
    let ambient_light_adc =
        AdcChannelDriver::new(adc1, peripherals.pins.gpio2, &AdcChannelConfig::new())?;
    let ambient_light_adc_arc = arc_mtx_new(ambient_light_adc);

    // let mut battery_voltage_adc =9
    //     AdcChannelDriver::new(&adc1, peripherals.pins.gpio4, &AdcChannelConfig::new())?;

    ///////////////////////////////////////////////////////////////////////////
    // region: Network Setup
    ///////////////////////////////////////////////////////////////////////////

    let ap_sta_ip_assigned_tx_clone = ap_sta_ip_assigned_tx.clone();
    // Important: keep the subscription object alive, otherwise the subscription will be cancelled
    let _subscription = sysloop
        .subscribe::<IpEvent, _>(move |event| {
            if let IpEvent::ApStaIpAssigned(detail) = event {
                log::info!("DHCP assigned IP address to the client: {}", detail.ip());
                ap_sta_ip_assigned_tx_clone
                    .send(detail.ip())
                    .unwrap_or_else(|e| {
                        log::error!("Failed to send IP assigned event: {}", e);
                    });
            }
        })
        .context("Failed to subscribe to IP events")?;

    let mut wifi = EspWifi::new(peripherals.modem, sysloop, Some(nvs_partition.clone()))
        .context("Failed to create EspWiFi instance")?;

    let mut on_private_net = button.was_initially_held() || wifi_creds.ssid.is_empty();

    let wifi_station_connected = if on_private_net {
        false
    } else {
        log::info!("Setting up WiFi station");
        setup_wifi_station(&mut wifi, &mut term, &wifi_creds)
            .context("Failed to set up WiFi station")?
    };

    if !wifi_station_connected {
        if on_private_net {
            log::info!("Setting up WiFi access point");
        } else {
            log::info!("Falling back to WiFi access point");
            on_private_net = true;
        }
        setup_wifi_access_point(&mut wifi, &ap_sta_ip_assigned_rx, &mut term)
            .context("Failed to set up WiFi access point")?;
        drop(_subscription); // we don't need subscription anymore
    }

    log::info!("On private network: {}", on_private_net);
    delay::FreeRtos::delay_ms(1000);

    let hostname = "retro-clock";
    let mut mdns = EspMdns::take()?;
    log::info!("Setting mDNS domain to {}.local", hostname);
    mdns.set_hostname(hostname)?;
    mdns.set_instance_name("Retro Clock")?;
    mdns.add_service(None, "_https", "_tcp", 443, &[])?;

    let server_cert_cstr = unsafe { std::ffi::CStr::from_ptr(SERVER_CERT.as_ptr() as *const i8) };
    let server_key_cstr = unsafe { std::ffi::CStr::from_ptr(SERVER_KEY.as_ptr() as *const i8) };

    let http_server_config = esp_http_server::Configuration {
        //max_sessions: 2,
        stack_size: 8 * 1024,
        max_resp_headers: 4,
        max_open_sockets: 2,
        max_uri_handlers: 24,
        uri_match_wildcard: true,
        server_certificate: Some(X509::pem(server_cert_cstr)),
        private_key: Some(X509::pem(server_key_cstr)),
        ..Default::default()
    };

    let mut http_server = esp_http_server::EspHttpServer::new(&http_server_config)?;
    let http_handlers_arc = Arc::new(http_handlers::HttpHandlers::new(on_private_net));

    register_http_handlers(
        &mut http_server,
        http_handlers_arc,
        system_settings_arc.clone(),
        storage_arc.clone(),
        places_arc.clone(),
        stations_arc.clone(),
        settings_update_tx.clone(),
        message_tx.clone(),
    )?;

    // region: AP mode main loop
    if on_private_net {
        loop {
            if let Ok(msg) = message_rx.recv_timeout(Duration::from_secs(1)) {
                term.log(msg)?;
            }
        }
    }

    ///////////////////////////////////////////////////////////////////////////
    // region: Download Initial Data
    ///////////////////////////////////////////////////////////////////////////

    let ntp = EspSntp::new_default()?;
    term.log(BootupMessage::SynchronizingWithNtp)?;

    while ntp.get_sync_status() != SyncStatus::Completed {
        delay::FreeRtos::delay_ms(100);
    }
    log::info!("Time synchronization completed");

    term.log(BootupMessage::ObtainingLocationInfo)?;
    let mut esp_idf_http_client =
        http_client::EspHttpClient::new().context("Failed to create HTTP client")?;
    let mut location_client = LocationClient::new(&mut esp_idf_http_client);
    let fetched_location_info = location_client
        .get_location_info()
        .context("Failed to fetch location info")?;
    log::info!("fetched_location_info: {:?}", fetched_location_info);

    let location_info_arc: ArcMtx<LocationInfo> = arc_mtx_new(overlay_location_info(
        &fetched_location_info,
        &system_settings_init,
    ));

    term.log(BootupMessage::ObtainingUtcOffset)?;

    ///////////////////////////////////////////////////////////////////////////
    // region: Spawn Static Threads
    ///////////////////////////////////////////////////////////////////////////

    if is_jtag_connected {
        let mut serial_monitor = SerialMonitor::new(
            user_command_tx.clone(),
            storage_arc.clone(),
            simulate_on_battery.clone(),
        );
        let mut buf = [0u8; 64];
        threading::EspBuilder::new()
            .name("SerialMonitor".into())
            .stack_size(4 * 1024)
            .priority(11)
            .pin_to_core(Core::Core1)
            .spawn_another_loop(move || -> anyhow::Result<()> {
                let n = usb_serial.read(&mut buf, 1000)?;
                serial_monitor.handle_new_data(buf.iter().take(n))
            })?;
    }

    let led_green_duty_clone = led_green_duty.clone();
    let led_green_driver_arc_clone = led_green_driver_arc.clone();
    threading::EspBuilder::new()
        .name("BusyLED".into())
        .stack_size(4 * 1024)
        .priority(3)
        .pin_to_core(Core::Core0)
        .spawn_another_loop(move || -> anyhow::Result<()> {
            let is_set = busy_indicator.wait_for_change()?;
            let duty = if is_set {
                led_green_duty_clone.load(atomic::Ordering::Relaxed)
            } else {
                0
            };
            {
                let mut led_green_driver = led_green_driver_arc_clone.lock_anyhow()?;
                led_green_driver.set_duty(duty)?;
            }
            Ok(())
        })?;

    let led_red_duty_clone = led_red_duty.clone();
    let led_red_driver_arc_clone = led_red_driver_arc.clone();
    threading::EspBuilder::new()
        .name("ErrorLED".into())
        .stack_size(4 * 1024)
        .priority(3)
        .pin_to_core(Core::Core0)
        .spawn_another_loop(move || -> anyhow::Result<()> {
            let errors = error_indicator.wait_for_change()?;
            let duty = if errors > 0 {
                led_red_duty_clone.load(atomic::Ordering::Relaxed)
            } else {
                0
            };
            {
                let mut led_red_driver = led_red_driver_arc_clone.lock_anyhow()?;
                led_red_driver.set_duty(duty)?;
            }
            Ok(())
        })?;

    let mut illumination_updater = IlluminationUpdater::new(
        illumination_rx,
        illumination_settings_init,
        led_bg_driver_arc.clone(),
        led_bg_max_duty,
        led_green_indicator_handle.clone(),
        led_green_max_duty,
        led_green_duty,
        led_red_indicator_handle.clone(),
        led_red_max_duty,
        led_red_duty,
    );
    threading::EspBuilder::new()
        .name("IlluminationUpdater".into())
        .stack_size(4 * 1024)
        .priority(2)
        .pin_to_core(Core::Core0)
        .spawn_another_loop(move || -> anyhow::Result<()> { illumination_updater.do_update() })?;

    let mut buzzer = BuzzerDriver::new(peripherals.pins.gpio34.into())?;
    threading::EspBuilder::new()
        .name("Buzzer".into())
        .stack_size(4 * 1024)
        .priority(12)
        .pin_to_core(Core::Core1)
        .spawn_another_loop(move || -> anyhow::Result<()> {
            buzzer_rx.recv()?;
            buzzer.click_once()
        })?;

    let location_info_arc_clone = location_info_arc.clone();
    let illumination_tx_clone = illumination_tx.clone();
    threading::EspBuilder::new()
        .name("SettingsUpdater".into())
        .stack_size(4 * 1024)
        .priority(4)
        .pin_to_core(Core::Core0)
        .spawn_another_loop(move || {
            let system_settings = settings_update_rx.recv()?;
            let new_location_info = overlay_location_info(&fetched_location_info, &system_settings);
            let mut old_location_info = location_info_arc_clone.lock_anyhow()?;
            *old_location_info = new_location_info;

            let ilumination_settings = IlluminationSettings::from_system_settings(&system_settings);
            illumination_tx_clone.send(IlluminationUpdate::Settings(ilumination_settings))?;
            Ok(())
        })?;

    let illumination_tx_clone = illumination_tx.clone();
    let ambient_light_adc_arc_clone = ambient_light_adc_arc.clone();
    threading::EspBuilder::new()
        .name("AmbientMonitor".into())
        .stack_size(4 * 1024)
        .priority(2)
        .pin_to_core(Core::Core0)
        .spawn_another_unit(move || -> anyhow::Result<()> {
            let do_monitor = move || -> anyhow::Result<()> {
                let ambient_light = {
                    let mut ambient_light_adc = ambient_light_adc_arc_clone.lock_anyhow()?;
                    ambient_light_adc.read()? as u32 // 0...AMBIENT_LIGHT_MAX
                };
                illumination_tx_clone.send(IlluminationUpdate::AmbientLight(ambient_light))?;
                Ok(())
            };

            loop {
                do_monitor()?;
                delay::FreeRtos::delay_ms(50);
            }
        })?;

    // button thread starts in Peripherals Setup section

    ///////////////////////////////////////////////////////////////////////////
    // region: Application Initialization
    ///////////////////////////////////////////////////////////////////////////

    let page0 = WorldTimePage::new(location_info_arc.clone(), places_arc.clone());

    let thread_builder0 = threading::EspBuilder::new()
        .name("Forecast0".into())
        .stack_size(8 * 1024)
        .priority(7)
        .pin_to_core(Core::Core0);

    let thread_builder1 = threading::EspBuilder::new()
        .name("Forecast1".into())
        .stack_size(8 * 1024)
        .priority(7)
        .pin_to_core(Core::Core1);

    let page1 = ForecastPage::new(
        location_info_arc.clone(),
        busy_indicator_handle.clone(),
        error_indicator_handle.clone(),
        thread_builder0,
        thread_builder1,
    );

    let thread_builder = threading::EspBuilder::new()
        .name("Weather".into())
        .stack_size(8 * 1024)
        .priority(6)
        .pin_to_core(Core::Core1);

    let page2 = WeatherPage::new(
        stations_arc.clone(),
        busy_indicator_handle.clone(),
        error_indicator_handle.clone(),
        thread_builder,
    );

    let mut pager = Pager::<terminal::EspTerminal, http_client::EspHttpClient>::new();
    pager.add_page(Box::new(page0));
    pager.add_page(Box::new(page1));
    pager.add_page(Box::new(page2));

    let is_on_battery = move || -> anyhow::Result<bool> {
        let is_simulate_on_battery = simulate_on_battery.load(atomic::Ordering::Relaxed);
        let is_power_connected = power_connected.is_low();
        let result = is_simulate_on_battery || (!is_power_connected & !is_jtag_connected);
        Ok(result)
    };

    let mut wifi_monitor = WifiMonitor::new(wifi, error_indicator_handle.clone());
    let home_screen_timeout: u32 = option_env!("HOME_SCREEN_TIMEOUT_SECONDS")
        .unwrap_or("60")
        .parse()?;
    let mut app = Application::new(term, pager, home_screen_timeout);
    app.initialize()?;
    threading::set_current_thread_priority(8);
    log_thread_info();
    log_stack_stats();

    ///////////////////////////////////////////////////////////////////////////
    // region: Application Main Loop
    ///////////////////////////////////////////////////////////////////////////

    let busy_wait = BusyWait::new();
    let mut is_on_battery_buf = ChangeDetector::new();

    let led_drivers = LedDrivers::new(led_bg_driver_arc, led_green_driver_arc, led_red_driver_arc);
    let mut led_driver_guards = LedDriverGuards::default();
    loop {
        let (is_on_battery_now, is_on_battery_changed) = is_on_battery_buf.set(is_on_battery()?);

        if is_on_battery_changed {
            if is_on_battery_now {
                log::info!("Device switched to battery");
                app.home_page()?;
                // acquire drivers to put other tasks on hold
                led_driver_guards.acquire_and_reset_all(&led_drivers, &ambient_light_adc_arc)?;
                wifi_monitor.stop_wifi()?;
                configure_power_management(CPU_MIN_FREQ_MHZ, CPU_MIN_FREQ_MHZ, false)?;
            } else {
                log::info!("Device switched to external power");
                configure_power_management(CPU_MAX_FREQ_MHZ, CPU_MIN_FREQ_MHZ, false)?;
                wifi_monitor.start_wifi()?;
                illumination_tx.send(IlluminationUpdate::Reset)?;
                // release drivers to resume other tasks
                led_driver_guards.release_all();
                led_green_indicator_handle.nop()?;
                led_red_indicator_handle.nop()?;
                // enforce early NTP resync
                restart_sntp();
            }
        }

        if !is_on_battery_now {
            let is_wifi_up = wifi_monitor.check()?;
            log::info!("is_wifi_up: {:?}", is_wifi_up);
        }

        let time_to_round_second_ms = app.get_time_to_round_second_ms();
        let event = if is_on_battery_now {
            if is_jtag_connected {
                delay::FreeRtos::delay_ms(time_to_round_second_ms);
            } else {
                enter_power_safe_mode(time_to_round_second_ms);
            }
            // flush user command events in case we received something in the breaks between sleep mode
            flush_channel(&user_command_rx)?;
            Err(mpsc::RecvTimeoutError::Timeout)
        } else {
            user_command_rx.recv_timeout(Duration::from_millis(time_to_round_second_ms.into()))
        };
        if let Ok(event) = event {
            log::info!("UserCommand: {:?}", event);
            match event {
                UserCommand::GoNextCard => {
                    app.next_card()?;
                }
                UserCommand::GoNextPage => {
                    app.next_page()?;
                    buzzer_tx.send(())?;
                }
                UserCommand::GoHome => {
                    app.home_page()?;
                    buzzer_tx.send(())?;
                }
            }
        } else {
            // delay above may be inaccurate, so in case when we awaken too early,
            // let's do busy wait for the remaining time (as long as it is a short delay)
            let residual_time_to_round_second_ms = app.get_time_to_round_second_ms();
            if residual_time_to_round_second_ms < 50 {
                busy_wait.at_least_ms(residual_time_to_round_second_ms);
            }

            app.refresh_display()?; // do it ASAP, it should be aligned with the round second
            log::info!(
                "Delay {{ time_to_round_second_ms: {}, residual_time_to_round_second_ms: {} }}",
                time_to_round_second_ms,
                residual_time_to_round_second_ms
            );
        }
    }
}
