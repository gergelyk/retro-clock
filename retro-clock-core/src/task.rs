use crate::threading::ThreadBuilder;
use anyhow::bail;
use std::thread;

pub struct BackgroundTask<T: Send + 'static, TB: ThreadBuilder> {
    worker: Option<thread::JoinHandle<Option<T>>>,
    product: Option<T>,
    finished: bool,
    thread_builder: TB,
}

impl<T: Send + 'static, TB: ThreadBuilder> BackgroundTask<T, TB> {
    pub fn new(thread_builder: TB) -> Self {
        Self {
            worker: None,
            product: None,
            finished: false,
            thread_builder,
        }
    }

    pub fn start<F>(&mut self, task: F) -> anyhow::Result<()>
    where
        F: FnOnce() -> Option<T>,
        F: Send + 'static,
    {
        self.finished = false;

        if let Some(worker) = self.worker.as_ref() {
            if !worker.is_finished() {
                log::info!(
                    "Product <{}> is under preparation",
                    self.thread_builder.get_name()
                );
                return Ok(());
            }
        }

        let worker = self.thread_builder.spawn_another(task)?;
        self.worker = Some(worker);
        Ok(())
    }

    pub fn join(&mut self) -> anyhow::Result<&Option<T>> {
        if self.finished {
            log::info!(
                "Product <{}> is already prepared",
                self.thread_builder.get_name()
            );
        } else if let Some(worker) = self.worker.take() {
            self.product = match worker.join() {
                Ok(product) => product,
                Err(e) => {
                    bail!(
                        "Worker for <{}> panicked: {:?}",
                        self.thread_builder.get_name(),
                        e
                    );
                }
            };
            self.finished = true;
        } else {
            bail!(
                "Worker for <{}> was not started",
                self.thread_builder.get_name()
            );
        }
        Ok(&self.product)
    }
}
