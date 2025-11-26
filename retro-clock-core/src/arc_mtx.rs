use std::sync::{Arc, Mutex, MutexGuard};

pub type ArcMtx<T> = Arc<Mutex<T>>;

pub fn arc_mtx_new<T>(value: T) -> ArcMtx<T> {
    Arc::new(Mutex::new(value))
}

pub trait MutexExt<T> {
    fn lock_anyhow(&self) -> anyhow::Result<MutexGuard<'_, T>>;
}
impl<T> MutexExt<T> for Mutex<T> {
    fn lock_anyhow(&self) -> anyhow::Result<MutexGuard<'_, T>> {
        self.lock()
            .map_err(|e| anyhow::anyhow!("Lock failed: {}", e))
    }
}
