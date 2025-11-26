// There are crates providing similar functionality, but I couldn't find any that would:
// - be implemented in Rust
// - support 8 bit data bus
// - support defining custom characters

use crate::system_utils::BusyWait;
use esp_idf_hal::{
    delay::{Delay, FreeRtos},
    gpio::{self, AnyOutputPin, PinDriver},
};
use esp_idf_sys as _;

pub struct Hd44780 {
    pin_rs: PinDriver<'static, AnyOutputPin, gpio::Output>,
    pin_rw: PinDriver<'static, AnyOutputPin, gpio::Output>,
    pin_e: PinDriver<'static, AnyOutputPin, gpio::Output>,
    pins_db: [PinDriver<'static, AnyOutputPin, gpio::Output>; 8],
}

impl Hd44780 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        pin_rs: AnyOutputPin,
        pin_rw: AnyOutputPin,
        pin_e: AnyOutputPin,
        pin_db0: AnyOutputPin,
        pin_db1: AnyOutputPin,
        pin_db2: AnyOutputPin,
        pin_db3: AnyOutputPin,
        pin_db4: AnyOutputPin,
        pin_db5: AnyOutputPin,
        pin_db6: AnyOutputPin,
        pin_db7: AnyOutputPin,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            pin_rs: PinDriver::output(pin_rs)?,
            pin_rw: PinDriver::output(pin_rw)?,
            pin_e: PinDriver::output(pin_e)?,
            pins_db: [
                PinDriver::output(pin_db0)?,
                PinDriver::output(pin_db1)?,
                PinDriver::output(pin_db2)?,
                PinDriver::output(pin_db3)?,
                PinDriver::output(pin_db4)?,
                PinDriver::output(pin_db5)?,
                PinDriver::output(pin_db6)?,
                PinDriver::output(pin_db7)?,
            ],
        })
    }

    fn set_byte(&mut self, data: u8) -> anyhow::Result<()> {
        let mut buf = data;
        for pin_db in self.pins_db.iter_mut() {
            if buf & 1 == 1 {
                pin_db.set_high()?;
            } else {
                pin_db.set_low()?;
            }
            buf >>= 1;
        }
        Ok(())
    }

    fn write_byte(&mut self, data: u8) -> anyhow::Result<()> {
        let wait = BusyWait::new();
        let delay: Delay = Default::default();
        self.pin_rw.set_low()?;
        wait.at_least_ns(40);
        self.pin_e.set_high()?;
        self.set_byte(data)?;
        wait.at_least_ns(80);
        self.pin_e.set_low()?;
        delay.delay_us(40); // Theoretically 500-80 = 420 ns should be enough
        Ok(())
    }

    fn write_command(&mut self, cmd: u8) -> anyhow::Result<()> {
        self.pin_rs.set_low()?;
        self.write_byte(cmd)
    }

    pub fn write_data(&mut self, data: u8) -> anyhow::Result<()> {
        self.pin_rs.set_high()?;
        self.write_byte(data)
    }

    pub fn cmd_clear_display(&mut self) -> anyhow::Result<()> {
        self.write_command(0x01)?;
        FreeRtos::delay_ms(3); // for clearing
        Ok(())
    }

    #[allow(dead_code)]
    #[deprecated(
        note = "This function reveals strange behavior. Consider using goto(0, 0) instead."
    )]
    pub fn cmd_return_home(&mut self) -> anyhow::Result<()> {
        self.write_command(0x02)
    }

    #[allow(dead_code)]
    pub fn cmd_entry_mode(&mut self, increment: bool, shift: bool) -> anyhow::Result<()> {
        let mut cmd: u8 = 0x04;
        if increment {
            cmd += 0x02;
        }
        if shift {
            cmd += 0x01;
        }
        self.write_command(cmd)
    }

    #[allow(dead_code)]
    pub fn cmd_display_onoff(
        &mut self,
        display_on: bool,
        cursor_on: bool,
        blinking_on: bool,
    ) -> anyhow::Result<()> {
        let mut cmd: u8 = 0x08;
        if display_on {
            cmd += 0x04;
        }
        if cursor_on {
            cmd += 0x02;
        }
        if blinking_on {
            cmd += 0x01;
        }
        self.write_command(cmd)
    }

    #[allow(dead_code)]
    pub fn cmd_shift(&mut self, shift_screen: bool, right: bool) -> anyhow::Result<()> {
        let mut cmd: u8 = 0x10;
        if shift_screen {
            cmd += 0x08;
        }
        if right {
            cmd += 0x04;
        }
        self.write_command(cmd)
    }

    #[allow(dead_code)]
    pub fn cmd_function_set(
        &mut self,
        db_width_8bit: bool,
        two_lines: bool,
        alternative_font: bool,
    ) -> anyhow::Result<()> {
        let mut cmd: u8 = 0x20;
        if db_width_8bit {
            cmd += 0x10;
        } else {
            anyhow::bail!("4-bit bus is not supported by the driver.");
        }
        if two_lines {
            // this affects the contrast, so adjustments of the analog line may be needed
            cmd += 0x08;
        }
        if alternative_font {
            cmd += 0x04;
        }
        self.write_command(cmd)
    }

    #[allow(dead_code)]
    pub fn cmd_set_cgram_address(&mut self, address: u8) -> anyhow::Result<()> {
        let mut cmd: u8 = 0x40;
        cmd += address & 0x3F;
        self.write_command(cmd)
    }

    #[allow(dead_code)]
    pub fn cmd_set_ddram_address(&mut self, address: u8) -> anyhow::Result<()> {
        let mut cmd: u8 = 0x80;
        cmd += address & 0x7F;
        self.write_command(cmd)
    }

    #[allow(dead_code)]
    pub fn goto(&mut self, x: u8, y: u8) -> anyhow::Result<()> {
        let addr: u8 = x + 0x40 * y;
        self.cmd_set_ddram_address(addr)
    }

    #[allow(dead_code)]
    pub fn write_text(&mut self, text: impl AsRef<str>) -> anyhow::Result<()> {
        for char32bit in text.as_ref().chars() {
            let char8bit = u8::try_from(char32bit as u32).unwrap_or(b'?');
            self.write_data(char8bit)?;
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub fn write_line0(&mut self, text: impl AsRef<str>) -> anyhow::Result<()> {
        self.goto(0, 0)?;
        self.write_text(text)?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn write_line1(&mut self, text: impl AsRef<str>) -> anyhow::Result<()> {
        self.goto(0, 1)?;
        self.write_text(text)?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn show_screen(&mut self, text: impl AsRef<str>) -> anyhow::Result<()> {
        let mut text_iter = text.as_ref().chars();
        let line1: String = text_iter.by_ref().take(16).collect();
        let line2: String = text_iter.take(16).collect();
        self.goto(0, 0)?;
        self.write_text(line1)?;
        self.goto(0, 1)?;
        self.write_text(line2)?;
        Ok(())
    }

    pub fn init(&mut self) -> anyhow::Result<()> {
        FreeRtos::delay_ms(10); // power supply rise time
        self.cmd_function_set(true, true, false)?;
        self.cmd_display_onoff(true, false, false)?;
        self.cmd_clear_display()?;
        Ok(())
    }

    // pub fn demo1(&mut self) -> anyhow::Result<()> {
    //     self.init()?;
    //     self.goto(2, 0)?;
    //     self.write_text("Hello world!")?;
    //     Ok(())
    // }

    // pub fn demo2(&mut self) -> anyhow::Result<()> {
    //     self.init()?;
    //     self.show_screen("Hello world! Have a good day!")?;
    //     Ok(())
    // }

    // pub fn demo3(&mut self) -> anyhow::Result<()> {
    //     self.init()?;

    //     // print all the characters
    //     for j in 0..16 {
    //         self.goto(0, 0)?;
    //         for i in 0..16 {
    //             self.write_data(32 * j + i)?;
    //         }
    //         self.goto(0, 1)?;
    //         for i in 16..32 {
    //             self.write_data(32 * j + i)?;
    //         }
    //         FreeRtos::delay_ms(5000);
    //     }

    //     Ok(())
    // }
}
