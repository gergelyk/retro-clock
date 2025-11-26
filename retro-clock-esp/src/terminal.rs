use crate::hd44780::Hd44780;

use retro_clock_core::terminal::trim_and_extend;
use retro_clock_core::terminal::Terminal;

pub struct EspTerminal {
    display: Hd44780,
}

fn insert_special_characters(line: &str) -> String {
    line.replace("°", "\x00")
        .replace("…", "\x01")
        .replace("‥", "\x02")
        .replace("Δ", "\x03")
        .replace("⌀", "\x04")
}

impl EspTerminal {
    pub fn new(display: Hd44780) -> Self {
        Self { display }
    }

    pub fn init(&mut self) -> anyhow::Result<()> {
        // Set custom characters

        // ASCII: 0 = degree
        self.display.cmd_set_cgram_address(0x00)?;
        self.display.write_data(0b_00010)?;
        self.display.write_data(0b_00101)?;
        self.display.write_data(0b_00010)?;
        self.display.write_data(0b_00000)?;
        self.display.write_data(0b_00000)?;
        self.display.write_data(0b_00000)?;
        self.display.write_data(0b_00000)?;
        self.display.write_data(0b_00000)?;
        self.display.cmd_set_ddram_address(0)?;

        // ASCII: 1 = ellipsis
        self.display.cmd_set_cgram_address(0x08)?;
        self.display.write_data(0b_00000)?;
        self.display.write_data(0b_00000)?;
        self.display.write_data(0b_00000)?;
        self.display.write_data(0b_00000)?;
        self.display.write_data(0b_00000)?;
        self.display.write_data(0b_00000)?;
        self.display.write_data(0b_10101)?;
        self.display.write_data(0b_00000)?;
        self.display.cmd_set_ddram_address(0)?;

        // ASCII: 2 = two dots
        self.display.cmd_set_cgram_address(0x10)?;
        self.display.write_data(0b_00000)?;
        self.display.write_data(0b_00000)?;
        self.display.write_data(0b_00000)?;
        self.display.write_data(0b_00000)?;
        self.display.write_data(0b_00000)?;
        self.display.write_data(0b_00000)?;
        self.display.write_data(0b_01010)?;
        self.display.write_data(0b_00000)?;
        self.display.cmd_set_ddram_address(0)?;

        // ASCII: 3 = delta
        self.display.cmd_set_cgram_address(0x18)?;
        self.display.write_data(0b_00000)?;
        self.display.write_data(0b_00000)?;
        self.display.write_data(0b_00000)?;
        self.display.write_data(0b_00100)?;
        self.display.write_data(0b_01010)?;
        self.display.write_data(0b_10001)?;
        self.display.write_data(0b_11111)?;
        self.display.write_data(0b_00000)?;
        self.display.cmd_set_ddram_address(0)?;

        // ASCII: 4 = mean
        self.display.cmd_set_cgram_address(0x20)?;
        self.display.write_data(0b_00000)?;
        self.display.write_data(0b_00000)?;
        self.display.write_data(0b_00011)?;
        self.display.write_data(0b_01110)?;
        self.display.write_data(0b_10101)?;
        self.display.write_data(0b_01110)?;
        self.display.write_data(0b_11000)?;
        self.display.write_data(0b_00000)?;
        self.display.cmd_set_ddram_address(0)?;

        Ok(())
    }
}

impl Terminal for EspTerminal {
    fn print0(&mut self, line: impl AsRef<str>) -> anyhow::Result<()> {
        let line = insert_special_characters(line.as_ref());
        let line = trim_and_extend(&line);
        self.display.write_line0(line)
    }

    fn print1(&mut self, line: impl AsRef<str>) -> anyhow::Result<()> {
        let line = insert_special_characters(line.as_ref());
        let line = trim_and_extend(&line);
        self.display.write_line1(line)
    }
}
