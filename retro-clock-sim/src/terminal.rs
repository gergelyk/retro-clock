use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Terminal as TuiTerminal,
};
//use crossterm::terminal::{enable_raw_mode, disable_raw_mode};
use crossterm::execute;
use std::io::{stdout, Stdout};

use retro_clock_core::terminal::{trim_and_extend, Terminal};
pub struct NativeTerminal {
    terminal: TuiTerminal<CrosstermBackend<Stdout>>,
    line0: String,
    line1: String,
}

impl NativeTerminal {
    pub fn new() -> anyhow::Result<Self> {
        //enable_raw_mode()?;
        let mut stream = stdout();
        execute!(stream, crossterm::terminal::EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stream);
        let terminal = TuiTerminal::new(backend)?;
        Ok(NativeTerminal {
            terminal,
            line0: "".to_owned(),
            line1: "".to_owned(),
        })
    }

    pub fn init(&mut self) -> anyhow::Result<()> {
        self.draw()?;
        Ok(())
    }

    fn draw(&mut self) -> anyhow::Result<()> {
        self.terminal.draw(|frame| {
            let area = Rect::new(5, 2, 18, 4);

            let paragraph = Paragraph::new(format!("{}\n{}", self.line0, self.line1))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::Black).bg(Color::Green)),
                )
                .alignment(Alignment::Left);

            frame.render_widget(paragraph, area);
        })?;

        Ok(())
    }
}

impl Terminal for NativeTerminal {
    fn print0(&mut self, line: impl AsRef<str>) -> anyhow::Result<()> {
        let line = trim_and_extend(line.as_ref());
        self.line0 = line;
        self.draw()
    }

    fn print1(&mut self, line: impl AsRef<str>) -> anyhow::Result<()> {
        let line = trim_and_extend(line.as_ref());
        self.line1 = line;
        self.draw()
    }

    fn close(&mut self) -> anyhow::Result<()> {
        //disable_raw_mode()?;
        execute!(
            self.terminal.backend_mut(),
            crossterm::terminal::LeaveAlternateScreen
        )?;
        self.terminal.clear()?;
        self.terminal.show_cursor()?;
        Ok(())
    }
}
