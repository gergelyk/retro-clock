use derive_new::new;
use retro_clock_core::arc_mtx::{ArcMtx, MutexExt};
use std::sync::{atomic, Arc};

use esp_idf_hal::ledc::LedcDriver;

use crate::buffers::SmoothingBuffer8;
use crate::debug_utils::get_heap_stats;
use crate::illumination::{
    BrightnessSettings, IlluminationSettings, IlluminationUpdate, AMBIENT_LIGHT_MAX,
};
use retro_clock_core::{
    busy_indicator::BusyIndicatorHandle, channel_utils::Receiver,
    error_indicator::ErrorIndicatorHandle,
};

fn percent_to_ambient_light(percent: u8) -> u32 {
    if AMBIENT_LIGHT_MAX == 950 {
        let percent = percent as u32;
        (percent << 3) + percent + (percent >> 1)
    } else {
        (percent as u32 * AMBIENT_LIGHT_MAX) / 100
    }
}

fn select_ambient_light(settings: &BrightnessSettings, ambient_light: u32) -> u32 {
    match settings {
        BrightnessSettings::Auto => ambient_light,
        BrightnessSettings::Manual { brightness } => percent_to_ambient_light(*brightness),
    }
}

#[allow(clippy::too_many_arguments)]
#[derive(new)]
pub struct IlluminationUpdater<'a> {
    illumination_rx: Receiver<IlluminationUpdate>,
    settings: IlluminationSettings,
    led_bg_driver_arc: ArcMtx<LedcDriver<'a>>,
    led_bg_max_duty: u32,
    led_green_indicator_handle: BusyIndicatorHandle,
    led_green_max_duty: u32,
    led_green_duty: Arc<atomic::AtomicU32>,
    led_red_indicator_handle: ErrorIndicatorHandle,
    led_red_max_duty: u32,
    led_red_duty: Arc<atomic::AtomicU32>,
    #[new(value = "0")]
    ambient_light_avr: u32,
    #[new(default)]
    smoothing_buffer: SmoothingBuffer8,
}

impl IlluminationUpdater<'_> {
    pub fn do_update(&mut self) -> anyhow::Result<()> {
        let mut cancel_calculations = false;
        match self.illumination_rx.recv()? {
            IlluminationUpdate::AmbientLight(new_ambient_light) => {
                self.smoothing_buffer.push(new_ambient_light);
                let new_ambient_light_avr = self.smoothing_buffer.average();

                if self.ambient_light_avr == new_ambient_light_avr {
                    cancel_calculations = true;
                }
                self.ambient_light_avr = new_ambient_light_avr;
                log::info!(
                    "AmbientLight {{ ambient_light: {}, ambient_light_avr: {} }}",
                    new_ambient_light,
                    self.ambient_light_avr
                );
                log::info!("{:?}", get_heap_stats());
            }
            IlluminationUpdate::Settings(new_settings) => {
                if self.settings == new_settings {
                    cancel_calculations = true;
                }
                self.settings = new_settings;
            }
            IlluminationUpdate::Reset => {
                self.ambient_light_avr = 0;
                self.smoothing_buffer.reset(self.ambient_light_avr);
            }
        }

        if !cancel_calculations {
            let ambient_light_for_bg =
                select_ambient_light(&self.settings.background, self.ambient_light_avr);
            let led_bg_duty = self.led_bg_max_duty * ambient_light_for_bg / AMBIENT_LIGHT_MAX;
            {
                let mut led_bg_driver = self.led_bg_driver_arc.lock_anyhow()?;
                led_bg_driver.set_duty(led_bg_duty)?;
            }

            let ambient_light_for_leds =
                select_ambient_light(&self.settings.indicators, self.ambient_light_avr);

            let new_led_green_max_duty = self.led_green_max_duty
                >> (8 * (AMBIENT_LIGHT_MAX - ambient_light_for_leds) / AMBIENT_LIGHT_MAX);
            self.led_green_duty
                .store(new_led_green_max_duty, atomic::Ordering::Relaxed);
            self.led_green_indicator_handle.nop()?;

            let new_led_red_duty = self.led_red_max_duty
                >> (9 * (AMBIENT_LIGHT_MAX - ambient_light_for_leds) / AMBIENT_LIGHT_MAX);
            self.led_red_duty
                .store(new_led_red_duty, atomic::Ordering::Relaxed);
            self.led_red_indicator_handle.nop()?;
        }

        Ok(())
    }
}
