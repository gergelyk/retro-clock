use chrono::Local;
use spin_sleep::sleep;
use std::io::Write;
use std::time::Duration;

fn main() {
    print!("\x1B[2J\x1B[1;1H"); // clear terminal
    loop {
        let local_time = Local::now();
        print!("\r{}", local_time.time());
        if let Err(e) = std::io::stdout().flush() {
            eprintln!("Error flushing stdout: {}", e);
            return;
        }
        let micros = local_time.timestamp_subsec_micros();
        sleep(Duration::from_micros(1_000_000_u64 - micros as u64));
    }
}
