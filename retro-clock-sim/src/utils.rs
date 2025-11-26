use std::time::Duration;

pub fn simulate_delay_hw_init() {
    #[cfg(feature = "simulate_delay")]
    std::thread::sleep(Duration::from_millis(500));
}

pub fn simulate_delay_http_handler() {
    #[cfg(feature = "simulate_delay")]
    std::thread::sleep(Duration::from_millis(500));
}
