use crate::counter::CountDown;
use crate::http_client::HttpClient;
use crate::pager::Pager;
use crate::terminal::Terminal;
use chrono::Utc;

pub struct Application<T, H> {
    term: T,
    pager: Pager<T, H>,
    align: bool,
    home_count_down: CountDown,
}

impl<T: Terminal, H: HttpClient> Application<T, H> {
    pub fn new(term: T, pager: Pager<T, H>, home_screen_timeout: u32) -> Self {
        let home_count_down = CountDown::new(home_screen_timeout);
        Application {
            term,
            pager,
            align: false,
            home_count_down,
        }
    }

    pub fn initialize(&mut self) -> anyhow::Result<()> {
        self.align = !self.pager.render_initial(&mut self.term)?;
        log::info!("App initialized");
        Ok(())
    }

    pub fn get_time_to_round_second_ms(&self) -> u32 {
        if self.align {
            let utc_time = Utc::now();
            let time_past_round_second_ms = utc_time.timestamp_millis() % 1000;
            (1000 - time_past_round_second_ms) as u32
        } else {
            1000
        }
    }

    pub fn next_card(&mut self) -> anyhow::Result<()> {
        self.pager.go_to_next_card()?;
        self.align = !self.pager.render_initial(&mut self.term)?;
        self.home_count_down.reset();
        Ok(())
    }

    pub fn next_page(&mut self) -> anyhow::Result<()> {
        self.pager.go_to_next_page()?;
        self.align = !self.pager.render_initial(&mut self.term)?;
        self.home_count_down.reset();
        Ok(())
    }

    pub fn home_page(&mut self) -> anyhow::Result<()> {
        self.pager.go_to_first_page()?;
        self.align = !self.pager.render_initial(&mut self.term)?;
        Ok(())
    }

    pub fn refresh_display(&mut self) -> anyhow::Result<()> {
        let is_last_one = self.home_count_down.decrement();

        if is_last_one {
            log::info!("Going to the home screen");
            self.pager.go_to_first_page()?;
            self.align = !self.pager.render_initial(&mut self.term)?;
        } else {
            self.pager.render_following(&mut self.term)?;
            self.align = true;
        }
        Ok(())
    }

    pub fn close(&mut self) -> anyhow::Result<()> {
        log::info!("App closed");
        self.term.close()?;
        Ok(())
    }
}
