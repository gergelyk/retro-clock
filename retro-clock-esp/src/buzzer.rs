use esp_idf_hal::{
    delay::Delay,
    gpio::{self, AnyIOPin, PinDriver},
};

pub struct BuzzerDriver<'a> {
    pin_driver: PinDriver<'a, AnyIOPin, gpio::Output>,
}

impl BuzzerDriver<'_> {
    pub fn new(pin: AnyIOPin) -> anyhow::Result<Self> {
        let pin_driver = PinDriver::output(pin)?;
        Ok(BuzzerDriver { pin_driver })
    }

    pub fn click_once(&mut self) -> anyhow::Result<()> {
        let delay = Delay::new_default();
        self.pin_driver.set_high()?;
        delay.delay_us(200);
        self.pin_driver.set_low()?;
        delay.delay_ms(50);
        Ok(())
    }
}
