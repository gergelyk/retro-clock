use std::thread;

pub trait ThreadBuilder {
    fn name(self, name: String) -> Self;
    fn get_name(&self) -> String;
    fn stack_size(self, size: usize) -> Self;

    // std::thread::Builder can spawn only one thread; here we can spawn many times
    fn spawn_another<F, T>(&self, f: F) -> anyhow::Result<thread::JoinHandle<T>>
    where
        F: FnOnce() -> T,
        F: Send + 'static,
        T: Send + 'static;
}
