use derive_new::new;
use esp_idf_hal::adc::oneshot::{AdcChannelDriver, AdcDriver};
use esp_idf_hal::adc::ADC1;
use esp_idf_hal::gpio::Gpio2;
use esp_idf_hal::ledc::LedcDriver;
use retro_clock_core::arc_mtx::{ArcMtx, MutexExt};
use std::sync::MutexGuard;

#[derive(new)]
pub struct LedDrivers<'b> {
    bg: ArcMtx<LedcDriver<'b>>,
    green: ArcMtx<LedcDriver<'b>>,
    red: ArcMtx<LedcDriver<'b>>,
}

#[derive(Default)]
pub struct LedDriverGuards<'a, 'b> {
    bg: Option<MutexGuard<'a, LedcDriver<'b>>>,
    green: Option<MutexGuard<'a, LedcDriver<'b>>>,
    red: Option<MutexGuard<'a, LedcDriver<'b>>>,
    adc: Option<MutexGuard<'a, AdcChannelDriver<'b, Gpio2, AdcDriver<'b, ADC1>>>>,
}

impl<'a, 'b> LedDriverGuards<'a, 'b> {
    pub fn acquire_and_reset_all(
        &mut self,
        led_drivers: &'a LedDrivers<'b>,
        adc: &'a ArcMtx<AdcChannelDriver<'b, Gpio2, AdcDriver<'b, ADC1>>>,
    ) -> anyhow::Result<()> {
        {
            self.adc = Some(adc.lock_anyhow()?);
        }
        {
            let mut guard = led_drivers.bg.lock_anyhow()?;
            guard.set_duty(0)?; // force background light off
            self.bg = Some(guard);
        }
        {
            let mut guard = led_drivers.green.lock_anyhow()?;
            guard.set_duty(0)?; // force busy LED off
            self.green = Some(guard);
        }
        {
            let mut guard = led_drivers.red.lock_anyhow()?;
            guard.set_duty(0)?; // force error LED off
            self.red = Some(guard);
        }
        Ok(())
    }

    pub fn release_all(&mut self) {
        drop(self.bg.take());
        drop(self.green.take());
        drop(self.red.take());
        drop(self.adc.take());
    }
}
