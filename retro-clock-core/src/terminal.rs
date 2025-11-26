use anyhow::Ok;

const TERMINAL_WIDTH: usize = 16;

pub fn trim_and_extend(line: &str) -> String {
    let mut truncated = line.chars().take(TERMINAL_WIDTH).collect::<String>();
    while truncated.len() < TERMINAL_WIDTH {
        truncated.push(' ');
    }
    truncated
}

pub trait Terminal {
    fn print0(&mut self, line: impl AsRef<str>) -> anyhow::Result<()>;
    fn print1(&mut self, line: impl AsRef<str>) -> anyhow::Result<()>;

    fn print(&mut self, line0: impl AsRef<str>, line1: impl AsRef<str>) -> anyhow::Result<()> {
        self.print0(line0)?;
        self.print1(line1)?;
        Ok(())
    }

    fn log(&mut self, text: impl AsRef<str>) -> anyhow::Result<()> {
        let line = text.as_ref().lines().collect::<Vec<_>>().join(" ");
        log::info!("{}", line);
        let mut text_iter = text.as_ref().lines();
        let line0 = text_iter.next().unwrap_or_default();
        let line1 = text_iter.next().unwrap_or_default();
        self.print(line0, line1)?;
        Ok(())
    }

    fn close(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}
