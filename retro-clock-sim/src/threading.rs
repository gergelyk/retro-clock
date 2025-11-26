use anyhow::Context;
use retro_clock_core::threading::ThreadBuilder;
use std::thread;
pub struct StdBuilder {
    name: Option<String>,
    stack_size: Option<usize>,
}

impl ThreadBuilder for StdBuilder {
    fn name(mut self, name: String) -> Self {
        self.name = Some(name);
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
        let mut builder = thread::Builder::new();
        if let Some(name) = &self.name {
            builder = builder.name(name.clone());
        }
        if let Some(size) = self.stack_size {
            builder = builder.stack_size(size);
        }
        builder.spawn(f).context("failed to spawn thread")
    }
}

impl StdBuilder {
    pub fn new() -> Self {
        Self {
            name: None,
            stack_size: None,
        }
    }
}
