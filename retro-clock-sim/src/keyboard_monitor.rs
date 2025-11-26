use getch_rs::{Getch, Key};
use std::sync::mpsc;
use std::thread;

pub struct KeyboardMonitor {}

impl KeyboardMonitor {
    pub fn new() -> Self {
        Self {}
    }

    pub fn start(&self) -> mpsc::Receiver<getch_rs::Key> {
        let (key_tx, key_rx) = mpsc::channel();
        thread::spawn(move || {
            let kbd = Getch::new();
            loop {
                match kbd.getch() {
                    Ok(ch) => {
                        if ch == Key::Ctrl('c') || ch == Key::Char('q') {
                            log::info!("Exiting keyboard monitor on user's request");
                            break;
                        }
                        if let Err(e) = key_tx.send(ch) {
                            log::error!("Keyboard listener exited early: {}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        log::error!("Error getting key: {}", e);
                        break;
                    }
                }
            }
            log::info!("Keyboard monitor exited");
        });
        key_rx
    }
}
