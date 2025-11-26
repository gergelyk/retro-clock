use anyhow::{bail, Context};
use chrono::DateTime;
use chrono::Utc;
use chrono_tz::Tz;
use std::sync::Arc;

use crate::arc_mtx::{ArcMtx, MutexExt};
use crate::busy_indicator::BusyIndicatorHandle;
use crate::error_indicator::{ErrorIndicatorHandle, ErrorKey};
use crate::forecast_client::ForecastClient;
use crate::forecast_models::ForecastReport;
use crate::formatters;
use crate::http_client::HttpClient;
use crate::pager::Page;
use crate::semaphore::Semaphore;
use crate::task::BackgroundTask;
use crate::terminal::Terminal;
use crate::threading::ThreadBuilder;
use crate::worldtime_models::LocationInfo;

fn forecast_fetcher<H: HttpClient>(
    location_info: LocationInfo,
    date: DateTime<Tz>,
) -> Option<ForecastReport> {
    match H::new() {
        Ok(ref mut http_client) => {
            let mut forecast_client = ForecastClient::new(http_client);

            log::info!("Fetching forecast...");
            match forecast_client.get_forecast(&location_info, date) {
                Ok(report) => {
                    log::info!("Fetching forecast done");
                    Some(report)
                }
                Err(e) => {
                    log::error!("Failed to fetch forecast: {}", e);
                    None
                }
            }
        }
        Err(e) => {
            log::error!("Failed to create HTTP client: {}", e);
            None
        }
    }
}

pub struct ForecastPage<TB: ThreadBuilder> {
    location_info_arc: ArcMtx<LocationInfo>,
    card_index: usize,
    task0: BackgroundTask<ForecastReport, TB>,
    task1: BackgroundTask<ForecastReport, TB>,
    busy_indicator_handle: BusyIndicatorHandle,
    error_indicator_handle: ErrorIndicatorHandle,
}

impl<TB: ThreadBuilder> ForecastPage<TB> {
    const NUMBER_OF_CARDS: usize = 5;

    pub fn new(
        location_info_arc: ArcMtx<LocationInfo>,
        busy_indicator_handle: BusyIndicatorHandle,
        error_indicator_handle: ErrorIndicatorHandle,
        thread_builder0: TB,
        thread_builder1: TB,
    ) -> Self {
        Self {
            location_info_arc,
            card_index: 0,
            task0: BackgroundTask::new(thread_builder0),
            task1: BackgroundTask::new(thread_builder1),
            busy_indicator_handle,
            error_indicator_handle,
        }
    }
}

impl<T, H, TB> Page<T, H> for ForecastPage<TB>
where
    T: Terminal,
    H: HttpClient,
    TB: ThreadBuilder,
{
    fn reset(&mut self) {
        <Self as Page<T, H>>::go_to_first_card(self)
    }

    fn render_title(&self, term: &mut T) -> anyhow::Result<()> {
        let title = match self.card_index {
            0 => "Temperature Min",
            1 => "Temperature Max",
            2 => "Wind speed/gusts",
            3 => "Precipitation",
            4 => "Clouds mean/mad",
            5 => "Sunrise/Sunset",
            _ => bail!("Invalid card index"),
        };

        term.print("Forecast:", title)?;
        Ok(())
    }

    fn start_fetching(&mut self, fetch_sem: Arc<Semaphore>) -> anyhow::Result<()> {
        let location_info = { self.location_info_arc.lock_anyhow()?.clone() };
        let utc_time = Utc::now();
        let local_time = utc_time.with_timezone(&location_info.timezone);
        let today = local_time;
        let tomorrow = local_time + chrono::Duration::days(1);

        let location_info_clone = location_info.clone();
        let fetch_sem_clone = fetch_sem.clone();
        let busy_indicator_hnd = self.busy_indicator_handle.clone();
        let error_indicator_hnd = self.error_indicator_handle.clone();
        self.task0.start(move || {
            let fetch_forecast = move || -> anyhow::Result<Option<ForecastReport>> {
                let mut error_indicator_guard = error_indicator_hnd
                    .arm(ErrorKey::DownloadForecast0)
                    .context("Failed to arm error indicator")?;
                let _indicator_guard = busy_indicator_hnd
                    .set()
                    .context("Failed to set busy indicator")?;
                let _fetch_sem_guard = fetch_sem_clone
                    .acquire(30)
                    .context("Failed to acquire fetch semaphore")?;

                let forecast = forecast_fetcher::<H>(location_info_clone, today);
                if forecast.is_some() {
                    error_indicator_guard.disarm();
                }
                Ok(forecast)
            };
            fetch_forecast().unwrap_or_else(|e| {
                log::error!("Forecast fetch task0 failed: {}", e);
                None
            })
        })?;

        let location_info_clone = location_info.clone();
        let fetch_sem_clone = fetch_sem.clone();
        let busy_indicator_hnd = self.busy_indicator_handle.clone();
        let error_indicator_hnd = self.error_indicator_handle.clone();
        self.task1.start(move || {
            let fetch_forecast = move || -> anyhow::Result<Option<ForecastReport>> {
                let mut error_indicator_guard = error_indicator_hnd
                    .arm(ErrorKey::DownloadForecast1)
                    .context("Failed to arm error indicator")?;
                let _indicator_guard = busy_indicator_hnd
                    .set()
                    .context("Failed to set busy indicator")?;
                let _fetch_sem_guard = fetch_sem_clone
                    .acquire(30)
                    .context("Failed to acquire fetch semaphore")?;

                let forecast = forecast_fetcher::<H>(location_info_clone, tomorrow);
                if forecast.is_some() {
                    error_indicator_guard.disarm();
                }
                Ok(forecast)
            };
            fetch_forecast().unwrap_or_else(|e| {
                log::error!("Forecast fetch task1 failed: {}", e);
                None
            })
        })?;

        Ok(())
    }

    fn render_content(&mut self, term: &mut T) -> anyhow::Result<()> {
        let product0 = self.task0.join()?;
        let product1 = self.task1.join()?;

        if let (Some(report0), Some(report1)) = (product0, product1) {
            let formatter = match self.card_index {
                0 => formatters::format_min_temperature,
                1 => formatters::format_max_temperature,
                2 => formatters::format_wind,
                3 => formatters::format_precipitation,
                4 => formatters::format_clouds,
                5 => formatters::format_sunrise_sunset,
                _ => bail!("Invalid card index"),
            };
            term.print(formatter(report0), formatter(report1))?;
        } else {
            term.print("No data", "")?;
        }

        Ok(())
    }

    fn go_to_first_card(&mut self) {
        self.card_index = 0;
    }

    fn go_to_next_card(&mut self) -> anyhow::Result<()> {
        self.card_index += 1;
        if self.card_index >= Self::NUMBER_OF_CARDS {
            <Self as Page<T, H>>::go_to_first_card(self)
        }
        Ok(())
    }
}
