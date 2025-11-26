use anyhow::Context;
use esp_idf_sys::{
    esp, esp_light_sleep_start, esp_pm_config_esp32_t, esp_pm_configure, esp_restart,
    esp_sleep_disable_wakeup_source, esp_sleep_enable_timer_wakeup,
    esp_sleep_source_t_ESP_SLEEP_WAKEUP_ALL, xthal_get_ccount,
};

extern "C" {
    fn esp_clk_cpu_freq() -> u32;
    fn esp_clk_apb_freq() -> u32;
}

fn get_cpu_freq_mhz() -> u32 {
    unsafe { esp_clk_cpu_freq() / 1_000_000 }
}
fn get_apb_freq_mhz() -> u32 {
    unsafe { esp_clk_apb_freq() / 1_000_000 }
}

pub fn configure_power_management(
    max_freq_mhz: i32,
    min_freq_mhz: i32,
    light_sleep_enable: bool,
) -> anyhow::Result<()> {
    unsafe {
        let config = esp_pm_config_esp32_t {
            max_freq_mhz,
            min_freq_mhz,
            light_sleep_enable,
        };
        esp!(esp_pm_configure(
            &config as *const _ as *const core::ffi::c_void
        ))
        .context("Failed to configure power management")?;
    }
    log::info!("Current CPU frequency: {} MHz", get_cpu_freq_mhz());
    log::info!("Current APB frequency: {} MHz", get_apb_freq_mhz());
    Ok(())
}

pub fn restart_device() -> ! {
    unsafe {
        esp_restart();
    }
}

pub struct BusyWait {
    cycles_per_ns: f32,
}

impl BusyWait {
    pub fn new() -> Self {
        Self {
            cycles_per_ns: get_cpu_freq_mhz() as f32 / 1_000.0,
        }
    }

    pub fn at_least_ns(&self, ns: u32) {
        unsafe {
            let cycles = (ns as f32 * self.cycles_per_ns).ceil() as u32;
            let start = xthal_get_ccount();
            while xthal_get_ccount().wrapping_sub(start) < cycles {}
        }
    }

    pub fn at_least_ms(&self, ms: u32) {
        self.at_least_ns(ms * 1_000_000);
    }
}

pub fn enter_power_safe_mode(duration_ms: u32) {
    unsafe {
        esp_sleep_disable_wakeup_source(esp_sleep_source_t_ESP_SLEEP_WAKEUP_ALL);
        esp_sleep_enable_timer_wakeup((duration_ms * 1000).into());
        esp_light_sleep_start();
    }
}

pub fn restart_sntp() {
    unsafe {
        esp_idf_sys::sntp_restart();
    }
}
