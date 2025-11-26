use std::sync::{Condvar, Mutex};

pub struct Semaphore {
    remaining: Mutex<usize>,
    cvar: Condvar,
}

impl Semaphore {
    pub fn new(count: usize) -> Self {
        Self {
            remaining: Mutex::new(count),
            cvar: Condvar::new(),
        }
    }

    pub fn acquire(&self, count: usize) -> anyhow::Result<SemaphoreGuard<'_>> {
        log::debug!("SEM: acquiring: count={}", count);
        let mut remaining = self
            .remaining
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock semaphore: {}", e))?;

        log::debug!("SEM: acquiring: remaining={}", *remaining);
        while *remaining < count {
            remaining = self
                .cvar
                .wait(remaining)
                .map_err(|e| anyhow::anyhow!("Failed to access cvar: {}", e))?;
        }
        *remaining -= count;
        log::debug!("SEM: acquired: remaining={}", *remaining);
        Ok(SemaphoreGuard { sem: self, count })
    }

    pub fn release(&self, count: usize) -> anyhow::Result<()> {
        log::debug!("SEM: releasing: count={}", count);
        let mut remaining = self
            .remaining
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock semaphore: {}", e))?;
        *remaining += count;
        log::debug!("SEM: released: remaining={}", *remaining);
        self.cvar.notify_one();
        Ok(())
    }
}

pub struct SemaphoreGuard<'a> {
    sem: &'a Semaphore,
    count: usize,
}

impl<'a> Drop for SemaphoreGuard<'a> {
    fn drop(&mut self) {
        if let Err(e) = self.sem.release(self.count) {
            // sorry, can't do much in Drop
            panic!("Failed to release semaphore: {}", e);
        }
    }
}
