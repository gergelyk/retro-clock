use anyhow::Context;
use std::thread;

use crate::debug_utils::log_thread_info;
use esp_idf_hal::task::thread as esp_thread;
use retro_clock_core::threading::ThreadBuilder;

#[derive(Debug, Default)]
pub struct EspBuilder {
    name: Option<String>,
    stack_size: Option<usize>,
    priority: Option<u8>,
    pin_to_core: Option<esp_idf_hal::cpu::Core>,
}

impl ThreadBuilder for EspBuilder {
    fn name(mut self, name: String) -> Self {
        self.name = Some(name.clone());
        self
    }

    fn get_name(&self) -> String {
        self.name.clone().unwrap_or_else(|| "<unnamed>".to_string())
    }

    fn stack_size(mut self, size: usize) -> Self {
        self.stack_size = Some(size);
        self
    }

    fn spawn_another<F, T>(&self, f: F) -> anyhow::Result<thread::JoinHandle<T>>
    where
        F: FnOnce() -> T,
        F: Send + 'static,
        T: Send + 'static,
    {
        let (std_builder, previous_config_opt) = self.before_spawn()?;

        let f_wrapped = move || {
            log_thread_info();
            f()
        };

        let handle = std_builder
            .spawn(f_wrapped)
            .context("Failed to spawn thread")?;

        self.after_spawn(previous_config_opt)?;

        Ok(handle)
    }
}

impl EspBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn priority(mut self, priority: u8) -> Self {
        self.priority = Some(priority);
        self
    }

    pub fn pin_to_core(mut self, pin_to_core: esp_idf_hal::cpu::Core) -> Self {
        self.pin_to_core = Some(pin_to_core);
        self
    }

    fn before_spawn(
        &self,
    ) -> anyhow::Result<(
        thread::Builder,
        Option<esp_thread::ThreadSpawnConfiguration>,
    )> {
        let default_priority = || esp_thread::ThreadSpawnConfiguration::default().priority;
        let previous_config_opt = esp_thread::ThreadSpawnConfiguration::get();

        esp_thread::ThreadSpawnConfiguration {
            priority: self.priority.unwrap_or_else(default_priority),
            pin_to_core: self.pin_to_core,
            inherit: false, // whether to inherit configuration from parent thread
            ..Default::default()
        }
        .set()?;

        let mut std_builder = thread::Builder::new();
        if let Some(name) = self.name.as_ref() {
            std_builder = std_builder.name(name.clone());
        }
        if let Some(size) = self.stack_size {
            std_builder = std_builder.stack_size(size);
        }
        Ok((std_builder, previous_config_opt))
    }

    fn after_spawn(
        &self,
        previous_config_opt: Option<esp_thread::ThreadSpawnConfiguration>,
    ) -> anyhow::Result<()> {
        previous_config_opt
            .unwrap_or_default()
            .set()
            .context("Failed to restore previous thread configuration")
    }

    /// Same as `spawn_another` but runs f in a loop. In case of an error,
    /// it logs the error and breaks.
    pub fn spawn_another_loop<F, T>(&self, mut f: F) -> anyhow::Result<thread::JoinHandle<()>>
    where
        F: FnMut() -> anyhow::Result<T>,
        F: Send + 'static,
        T: Send + 'static,
    {
        let (std_builder, previous_config_opt) = self.before_spawn()?;
        let name = self.get_name();
        let name_clone = name.clone();

        let f_wrapped = move || {
            log_thread_info();
            loop {
                if let Err(e) = f() {
                    log::error!("{} thread error: {}", name_clone, e);
                    #[cfg(feature = "panic_on_thread_error")]
                    panic!("{} thread error: {}", name_clone, e);
                    break;
                }
            }
        };

        let handle = std_builder
            .spawn(f_wrapped)
            .context(format!("Failed to spawn thread {}", name))?;

        self.after_spawn(previous_config_opt)?;

        Ok(handle)
    }

    /// Same as `spawn_another` but reports any errors. f needs to return
    /// anyhow::Result<T>. with any T, but T is discarded anyways.
    pub fn spawn_another_unit<F, T>(&self, f: F) -> anyhow::Result<thread::JoinHandle<()>>
    where
        F: FnOnce() -> anyhow::Result<T>,
        F: Send + 'static,
        T: Send + 'static,
    {
        let (std_builder, previous_config_opt) = self.before_spawn()?;
        let name = self.get_name();
        let name_clone = name.clone();

        let f_wrapped = move || {
            log_thread_info();
            if let Err(e) = f() {
                log::error!("{} thread error: {}", name_clone, e);
                #[cfg(feature = "panic_on_thread_error")]
                panic!("{} thread error: {}", name_clone, e);
            }
        };

        let handle = std_builder
            .spawn(f_wrapped)
            .context("Failed to spawn thread")?;

        self.after_spawn(previous_config_opt)?;

        Ok(handle)
    }
}

pub fn set_current_thread_priority(priority: u32) {
    unsafe {
        esp_idf_sys::vTaskPrioritySet(std::ptr::null_mut(), priority);
    }
}
