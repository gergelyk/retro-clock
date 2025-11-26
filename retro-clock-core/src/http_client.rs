use std::time::Duration;

pub const REQUEST_TIMEOUT_SECS: u32 = 5;
pub trait HttpClient {
    fn new() -> anyhow::Result<Self>
    where
        Self: Sized;
    fn http_get(&mut self, url: &str) -> anyhow::Result<String>;
    fn http_post(&mut self, url: &str, body: &str) -> anyhow::Result<String>;

    fn sleep(time_secs: u32) {
        let duration = Duration::from_secs(time_secs as u64);
        std::thread::sleep(duration);
    }

    fn http_get_with_retry(
        &mut self,
        url: &str,
        retry_times: u32,
        retry_delay_secs: u32,
    ) -> anyhow::Result<String> {
        for i in 1..=retry_times {
            if let Ok(resp) = self.http_get(url) {
                return Ok(resp);
            } else {
                log::info!("Cannot get {}", url);
                Self::sleep(retry_delay_secs);
                log::info!("Retrying {}/{}...", i, retry_times);
            }
        }

        self.http_get(url)
    }
}
