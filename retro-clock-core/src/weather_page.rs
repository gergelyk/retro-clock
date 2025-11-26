use anyhow::Context;
use chrono::Utc;
use std::sync::Arc;

use crate::arc_mtx::{ArcMtx, MutexExt};
use crate::busy_indicator::BusyIndicatorHandle;
use crate::error_indicator::{ErrorIndicatorHandle, ErrorKey};
use crate::formatters;
use crate::http_client::HttpClient;
use crate::pager::Page;
use crate::params::WEATHER_DATA_CHUNK_SIZE;
use crate::semaphore::Semaphore;
use crate::task::BackgroundTask;
use crate::terminal::Terminal;
use crate::threading::ThreadBuilder;
use crate::weather_client::WeatherClient;
use crate::weather_models::WeatherReport;
use crate::weather_models::WeatherStation;

fn weather_fetcher<H: HttpClient>(urls: Vec<String>) -> Option<WeatherReport> {
    match H::new() {
        Ok(ref mut http_client) => {
            let mut weather_client = WeatherClient::new(http_client);

            log::info!("Fetching weather...");
            let mut measurements_chunks = vec![];

            for (i, urls_chunk) in urls.chunks(WEATHER_DATA_CHUNK_SIZE).enumerate() {
                log::info!("Chunk #{}...", i + 1);

                let Ok(body) = serde_json::to_string(&urls_chunk) else {
                    log::error!("Failed to serialize weather request body");
                    return None;
                };

                match weather_client.get_weather(&body) {
                    Ok(report) => {
                        log::info!("Fetching weather done");
                        measurements_chunks.push(report.measurements);
                    }
                    Err(e) => {
                        log::error!("Failed to fetch weather: {}", e);
                        return None;
                    }
                }
            }
            Some(WeatherReport {
                measurements: measurements_chunks.into_iter().flatten().collect(),
            })
        }
        Err(e) => {
            log::error!("Failed to create HTTP client: {}", e);
            None
        }
    }
}

pub struct WeatherPage<TB: ThreadBuilder> {
    card_index: usize,
    stations: ArcMtx<Vec<WeatherStation>>,
    stations_snapshot: Vec<WeatherStation>,
    task: BackgroundTask<WeatherReport, TB>,
    busy_indicator_handle: BusyIndicatorHandle,
    error_indicator_handle: ErrorIndicatorHandle,
}

impl<TB: ThreadBuilder> WeatherPage<TB> {
    pub fn new(
        stations: ArcMtx<Vec<WeatherStation>>,
        busy_indicator_handle: BusyIndicatorHandle,
        error_indicator_handle: ErrorIndicatorHandle,
        thread_builder: TB,
    ) -> Self {
        Self {
            card_index: 0,
            stations,
            stations_snapshot: vec![],
            task: BackgroundTask::new(thread_builder),
            busy_indicator_handle,
            error_indicator_handle,
        }
    }

    pub fn get_stations_arc(&self) -> ArcMtx<Vec<WeatherStation>> {
        self.stations.clone()
    }
}

impl<T, H, TB> Page<T, H> for WeatherPage<TB>
where
    T: Terminal,
    H: HttpClient,
    TB: ThreadBuilder,
{
    fn reset(&mut self) {
        <Self as Page<T, H>>::go_to_first_card(self)
    }

    fn render_title(&self, term: &mut T) -> anyhow::Result<()> {
        let station = self.stations_snapshot.get(self.card_index);
        let line1 = match station {
            None => "No stations",
            Some(s) => &s.name,
        };
        term.print("Weather:", line1)?;
        Ok(())
    }

    fn start_fetching(&mut self, fetch_sem: Arc<Semaphore>) -> anyhow::Result<()> {
        self.stations_snapshot = self.stations.lock_anyhow()?.clone();
        let urls: Vec<String> = self
            .stations_snapshot
            .iter()
            .map(|s| s.url.clone())
            .collect();
        let busy_indicator_handle = self.busy_indicator_handle.clone();
        let error_indicator_handle = self.error_indicator_handle.clone();
        self.task.start(move || {
            let fetch_weather = move || -> anyhow::Result<Option<WeatherReport>> {
                let mut error_indicator_guard = error_indicator_handle
                    .arm(ErrorKey::DownloadWeather)
                    .context("Failed to arm error indicator")?;
                let _indicator_guard = busy_indicator_handle
                    .set()
                    .context("Failed to set busy indicator")?;
                let _fetch_sem_guard = fetch_sem
                    .acquire(30)
                    .context("Failed to acquire fetch semaphore")?;

                let weather = weather_fetcher::<H>(urls);
                if weather.is_some() {
                    error_indicator_guard.disarm();
                }
                Ok(weather)
            };
            fetch_weather().unwrap_or_else(|e| {
                log::error!("Weather fetch task failed: {}", e);
                None
            })
        })?;
        Ok(())
    }

    fn render_content(&mut self, term: &mut T) -> anyhow::Result<()> {
        if let Some(report) = self.task.join()? {
            let measurement = report.measurements.get(self.card_index);

            match measurement {
                None => {
                    term.print("Add stations", "in the config")?;
                }
                Some(m) => {
                    let utc_time = Utc::now();
                    let line0 = formatters::format_weather_line0(m, &utc_time);
                    let line1 = formatters::format_weather_line1(m);
                    term.print(line0, line1)?;
                }
            }
        } else {
            term.print("No data", "")?;
        }

        Ok(())
    }

    fn go_to_first_card(&mut self) {
        self.card_index = 0;
    }

    fn go_to_next_card(&mut self) -> anyhow::Result<()> {
        let stations_len = self.stations_snapshot.len();
        self.card_index += 1;
        if self.card_index >= stations_len {
            <Self as Page<T, H>>::go_to_first_card(self)
        }
        Ok(())
    }
}
