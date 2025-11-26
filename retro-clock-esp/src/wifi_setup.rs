use anyhow::Context;
use esp_idf_sys::{esp_efuse_mac_get_default, esp_wifi_set_ps, wifi_ps_type_t_WIFI_PS_MAX_MODEM};

use embedded_svc::{ipv4::Ipv4Addr, wifi};
use esp_idf_svc::wifi::EspWifi;

use retro_clock_core::{
    channel_utils::Receiver as ChannelReceiver,
    creds_models::WifiCredentials,
    error_indicator::{ErrorIndicatorHandle, ErrorKey},
    messages::BootupMessage,
    terminal::Terminal,
};

use crate::{
    params::{
        WIFI_CONNECT_INTERVAL_MS, WIFI_CONNECT_ITERATIONS, WIFI_GET_IP_INTERVAL_MS,
        WIFI_GET_IP_ITERATIONS, WIFI_RESTORE_ITERATIONS,
    },
    string_utils, terminal,
};

pub const WIFI_AP_PASS: Option<&str> = option_env!("WIFI_AP_PASS");

fn get_ssid() -> String {
    let mut mac: [u8; 6] = [0; 6];
    unsafe {
        esp_efuse_mac_get_default(mac.as_mut_ptr());
    }
    log::debug!(
        "Device MAC address: {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
        mac[0],
        mac[1],
        mac[2],
        mac[3],
        mac[4],
        mac[5]
    );
    format!("Retro-{:02X}{:02X}", mac[4], mac[5])
}

pub fn setup_wifi_access_point(
    wifi: &mut EspWifi,
    ap_sta_ip_assigned_rx: &ChannelReceiver<Ipv4Addr>,
    term: &mut terminal::EspTerminal,
) -> anyhow::Result<()> {
    let ap_ssid = get_ssid();
    let ap_password = match WIFI_AP_PASS {
        Some(p) => {
            log::info!("Using compile-time AP password");
            p.to_string()
        }
        None => string_utils::get_random_string(10),
    };

    let ap_config = wifi::Configuration::AccessPoint(wifi::AccessPointConfiguration {
        ssid: string_utils::to_heapless::<32>(&ap_ssid)?,
        password: string_utils::to_heapless::<64>(&ap_password)?,
        channel: 1,
        secondary_channel: None,
        auth_method: wifi::AuthMethod::WPA2Personal,
        max_connections: 1,
        ssid_hidden: false,
        ..Default::default()
    });

    wifi.set_configuration(&ap_config)?;
    wifi.start()?;
    log::info!("WiFi started in Access Point mode");

    if let Ok(ip_info) = wifi.ap_netif().get_ip_info() {
        log::debug!("Access Point IP info: {:#?}", ip_info);
    }

    term.print(
        format!("WiFi: {}", ap_ssid),
        format!("Pass: {}", ap_password),
    )?;

    log::info!("Waiting for client to connect and obtain IP address via DHCP...");
    log::info!("WiFi: {}", ap_ssid);
    log::info!("Pass: {}", ap_password);

    let client_ip = ap_sta_ip_assigned_rx
        .recv()
        .context("Failed to receive IP address notification")?;
    log::info!(
        "DHCP assigned IP address to the client: {}",
        client_ip.to_string()
    );
    term.print("Go to: https://r", "etro-clock.local")?;

    Ok(())
}

pub fn setup_wifi_station(
    wifi: &mut EspWifi,
    term: &mut terminal::EspTerminal,
    wifi_creds: &WifiCredentials,
) -> anyhow::Result<bool> {
    let client_ssid = string_utils::to_heapless::<32>(&wifi_creds.ssid)?;
    let client_password = string_utils::to_heapless::<64>(&wifi_creds.password)?;

    wifi.set_configuration(&wifi::Configuration::Client(wifi::ClientConfiguration {
        ssid: client_ssid,
        password: client_password,
        scan_method: wifi::ScanMethod::FastScan,
        //scan_method: wifi::ScanMethod::CompleteScan(wifi::ScanSortMethod::Signal),
        ..Default::default()
    }))?;

    start_connection(wifi)?;

    term.log(BootupMessage::ConnectingToWiFi)?;

    let mut connected = false;
    for _ in 0..WIFI_CONNECT_ITERATIONS {
        if wifi.is_connected()? {
            connected = true;
            break;
        }
        esp_idf_hal::delay::FreeRtos::delay_ms(WIFI_CONNECT_INTERVAL_MS);
    }

    let mut ip_obtained = false;
    if connected {
        log::info!("Connected!");
        term.log(BootupMessage::ObtainingIp)?;
        for _ in 0..WIFI_GET_IP_ITERATIONS {
            let ip = get_station_ip(wifi)?;
            if !ip.is_unspecified() {
                log::info!("Obtained! IP address: {}", ip.to_string());
                ip_obtained = true;
                break;
            }
            esp_idf_hal::delay::FreeRtos::delay_ms(WIFI_GET_IP_INTERVAL_MS);
        }
        if !ip_obtained {
            log::warn!("Failed to obtain IP address");
        }
    } else {
        log::warn!("Failed to connect to WiFi");
    }
    Ok(connected && ip_obtained)
}

fn start_connection(wifi: &mut EspWifi) -> anyhow::Result<()> {
    // After ESP32 is rebooted Android doesn't want to assign IP address, because it still remembers
    // old connection. Another reboot helps. But here we do it differenty - we randomize MAC, so that
    // we always look like a new client.
    let rand_byte = (unsafe { esp_idf_sys::esp_random() } & 0xFF) as u8;
    let interface = esp_idf_svc::wifi::WifiDeviceId::Sta;
    let mut mac = wifi.get_mac(interface)?;
    mac[0] |= 0x02; // set locally administered bit (only formal requirement)
    mac[mac.len() - 1] = rand_byte;
    log::info!(
        "Setting randomized MAC address for STA interface: {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
        mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
    );
    wifi.set_mac(interface, mac)?;
    wifi.start()?;

    unsafe {
        // Modem-sleep optimized for lowest power (higher latency).
        esp_idf_sys::esp!(esp_wifi_set_ps(wifi_ps_type_t_WIFI_PS_MAX_MODEM))
            .context("Failed to set modem-sleep")?;
        log::info!("Modem-sleep enabled");
    }

    wifi.connect()?;
    Ok(())
}

fn get_station_ip(wifi: &EspWifi) -> anyhow::Result<Ipv4Addr> {
    let ip_info = wifi.sta_netif().get_ip_info()?;
    Ok(ip_info.ip)
}

enum WifiState {
    Disconnected { counter: u32 },
    WaitingForIp { counter: u32 },
    Unknown,
}

pub struct WifiMonitor<'a> {
    wifi: EspWifi<'a>,
    state: WifiState,
    error_indicator_handle: ErrorIndicatorHandle,
}

impl<'a> WifiMonitor<'a> {
    pub fn new(wifi: EspWifi<'a>, error_indicator_handle: ErrorIndicatorHandle) -> Self {
        Self {
            wifi,
            state: WifiState::Unknown,
            error_indicator_handle,
        }
    }

    fn reset(&mut self) -> anyhow::Result<()> {
        self.wifi.stop()?;
        start_connection(&mut self.wifi)?;
        Ok(())
    }

    pub fn stop_wifi(&mut self) -> anyhow::Result<()> {
        self.wifi.stop()?;
        let guard = self
            .error_indicator_handle
            .arm(ErrorKey::NoWifiConnection)?;
        drop(guard);
        Ok(())
    }

    pub fn start_wifi(&mut self) -> anyhow::Result<()> {
        start_connection(&mut self.wifi)?;
        Ok(())
    }

    fn has_ip(&self) -> anyhow::Result<bool> {
        let ip = get_station_ip(&self.wifi)?;
        Ok(!ip.is_unspecified())
    }

    pub fn check(&mut self) -> anyhow::Result<bool> {
        match &self.state {
            WifiState::Unknown => {
                if !self.wifi.is_connected()? {
                    log::warn!("WiFi disconnected, resetting connection...");
                    self.reset()?;
                    self.state = WifiState::Disconnected {
                        counter: WIFI_RESTORE_ITERATIONS,
                    };
                    let guard = self
                        .error_indicator_handle
                        .arm(ErrorKey::NoWifiConnection)?;
                    drop(guard);
                    self.check()?;
                }
            }
            WifiState::Disconnected { counter } => {
                if self.wifi.is_connected()? {
                    log::info!("WiFi reconnected, waiting for IP address...");
                    self.state = WifiState::WaitingForIp { counter: *counter };
                    self.check()?;
                } else {
                    let counter = counter - 1;
                    if counter == 0 {
                        log::warn!("Failed to reconnect");
                        self.state = WifiState::Unknown;
                    } else {
                        log::info!("WiFi waiting to reconnect ({})...", counter);
                        self.state = WifiState::Disconnected { counter };
                    }
                }
            }
            WifiState::WaitingForIp { counter } => {
                if self.has_ip()? {
                    log::info!("WiFi IP address obtained");
                    self.state = WifiState::Unknown;
                    let mut guard = self
                        .error_indicator_handle
                        .arm(ErrorKey::NoWifiConnection)?;
                    guard.disarm();
                } else {
                    let counter = counter - 1;
                    if counter == 0 {
                        log::warn!("Failed to obtain IP address, disconnecting...");
                        self.wifi.disconnect()?;
                        self.state = WifiState::Unknown;
                    } else {
                        log::info!("Waiting to obtain IP address ({})...", counter);
                        self.state = WifiState::WaitingForIp { counter };
                    }
                }
            }
        }

        Ok(self.wifi.is_up()?)
    }
}
