use crate::threading::EspBuilder;
use esp_idf_hal::gpio::AnyIOPin;
use esp_idf_hal::{gpio, task::queue};
use esp_idf_sys as _;
use std::sync::atomic::{AtomicU32, Ordering};

/*
Button triggers an interrupt to provide immediate responce.
Interrupt is ignored if happens more often than within DEBOUNCING_MS period.
This is necessary to provide ignore bouncing, but at the same time some
transactions can be overlooked, for example when PRESS and RELEASE events
occure within DEBOUNCING_MS. For this reason, button is also sampled at
BUTTON_SAMPLING_INTERVAL_TICKS rate. This makes sure that overlooked events
are generated and we are in sync with the actual state of the button.

Button held for BUTTON_HOLD_MS generates HOLD event. Subsequent RELEASE event
is not generated.
*/
use crate::params::{BUTTON_DEBOUNCING_MS, BUTTON_HOLD_MS, BUTTON_SAMPLING_INTERVAL_TICKS};

static LAST_BUTTON_EVENT_TIME_MS: AtomicU32 = AtomicU32::new(0);
static mut BUTTON_QUEUE: Option<queue::Queue<()>> = None;

#[derive(Debug, Clone, Copy)]
pub enum ButtonEvent {
    Pressed,
    Released,
    Held,
}

fn now_millis() -> u32 {
    unsafe { (esp_idf_sys::esp_timer_get_time() / 1000) as u32 }
}

pub struct ButtonDriver {
    was_initially_held: bool,
}

impl ButtonDriver {
    /// callback is called in a time-critical thread. It should not perform
    /// blocking operations and should return as soon as possible.
    pub fn new<F>(pin: AnyIOPin, thread_builder: EspBuilder, callback: F) -> anyhow::Result<Self>
    where
        F: Fn(ButtonEvent) + std::marker::Send + 'static,
    {
        let mut button = gpio::PinDriver::input(pin)?;
        button.set_pull(gpio::Pull::Up)?;
        let was_initially_held = button.is_low();
        button.set_interrupt_type(gpio::InterruptType::AnyEdge)?;

        let button_queue = queue::Queue::<()>::new(10);
        unsafe {
            BUTTON_QUEUE = Some(button_queue);

            button.subscribe(move || {
                let timestamp = now_millis();
                let last_timestamp = LAST_BUTTON_EVENT_TIME_MS.swap(timestamp, Ordering::Relaxed);
                let elapsed = timestamp.wrapping_sub(last_timestamp);

                if elapsed > BUTTON_DEBOUNCING_MS {
                    if let Some(ref q) = BUTTON_QUEUE {
                        // if the queue is full we ignore an error and discard the event
                        q.send_back((), 0).ok();
                    }
                }
            })?;
        }

        button.enable_interrupt()?;
        thread_builder.spawn_another_unit(move || -> anyhow::Result<()> {
            let mut last_button_is_low = button.is_low();
            let mut button_press_timestamp = 0;
            let mut button_held_emited = false;

            loop {
                unsafe {
                    if let Some(ref q) = BUTTON_QUEUE {
                        q.recv_front(BUTTON_SAMPLING_INTERVAL_TICKS);
                    }
                }

                let button_is_low = button.is_low();

                if button_is_low && !last_button_is_low {
                    log::debug!("Button pressed");
                    callback(ButtonEvent::Pressed);

                    button_press_timestamp = now_millis();
                } else if !button_is_low && last_button_is_low {
                    if button_held_emited {
                        button_held_emited = false;
                    } else {
                        log::debug!("Button released");
                        callback(ButtonEvent::Released);
                    }
                }

                if button_is_low
                    && now_millis().wrapping_sub(button_press_timestamp) > BUTTON_HOLD_MS
                {
                    log::debug!("Button held");
                    callback(ButtonEvent::Held);
                    button_held_emited = true;
                    button_press_timestamp = now_millis();
                }

                button.enable_interrupt()?;
                last_button_is_low = button_is_low;
            }
        })?;

        Ok(Self { was_initially_held })
    }

    pub fn was_initially_held(&self) -> bool {
        self.was_initially_held
    }
}
