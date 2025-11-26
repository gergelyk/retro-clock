use anyhow::{bail, Context, Ok};
use chrono::Utc;
use std::sync::Arc;

use crate::arc_mtx::{ArcMtx, MutexExt};
use crate::formatters;
use crate::http_client::HttpClient;
use crate::pager::Page;
use crate::semaphore::Semaphore;
use crate::terminal::Terminal;
use crate::worldtime_models::{LocationData, LocationInfo};

pub struct WorldTimePage {
    card_index: usize,
    location_info_arc: ArcMtx<LocationInfo>,
    places: ArcMtx<Vec<LocationData>>,
    places_snapshot: Vec<LocationData>,
}

impl WorldTimePage {
    pub fn new(location_info_arc: ArcMtx<LocationInfo>, places: ArcMtx<Vec<LocationData>>) -> Self {
        Self {
            card_index: 0,
            location_info_arc,
            places,
            places_snapshot: vec![],
        }
    }

    pub fn get_places_arc(&self) -> ArcMtx<Vec<LocationData>> {
        self.places.clone()
    }
}

impl<T: Terminal, H: HttpClient> Page<T, H> for WorldTimePage {
    fn has_titles(&self) -> bool {
        false
    }

    fn reset(&mut self) {
        <Self as Page<T, H>>::go_to_first_card(self)
    }

    fn render_title(&self, _term: &mut T) -> anyhow::Result<()> {
        bail!("No titles")
    }

    fn start_fetching(&mut self, _fetch_sem: Arc<Semaphore>) -> anyhow::Result<()> {
        Ok(())
    }

    fn render_content(&mut self, term: &mut T) -> anyhow::Result<()> {
        let timezone = { self.location_info_arc.lock_anyhow()?.timezone };
        let local_place = LocationData {
            name: None,
            timezone,
        };
        let remote_location_data = std::iter::once(&local_place)
            .chain(self.places_snapshot.iter())
            .nth(self.card_index)
            .context("Invalid card index")?;
        let utc_time = Utc::now();
        let remote_time = utc_time.with_timezone(&remote_location_data.timezone);

        let line0: String;
        let line0_str = if let Some(ref location_name) = remote_location_data.name {
            location_name
        } else {
            line0 = format!("{}", remote_time.format(formatters::DATE_LINE_FMT));
            line0.as_str()
        };
        let line1 = format!("{}", remote_time.format(formatters::TIME_LINE_FMT));

        term.print(line0_str, &line1)?;

        log::info!(
            "WorldTime {{ milisecond: {} }} # {}",
            utc_time.timestamp_millis() % 1000,
            line1
        );
        Ok(())
    }

    fn go_to_first_card(&mut self) {
        self.card_index = 0;
    }

    fn go_to_next_card(&mut self) -> anyhow::Result<()> {
        if self.card_index == 0 {
            self.places_snapshot = self.places.lock_anyhow()?.clone();
        }
        let places_len = self.places_snapshot.len() + 1;
        self.card_index += 1;
        if self.card_index >= places_len {
            <Self as Page<T, H>>::go_to_first_card(self)
        }
        Ok(())
    }
}
