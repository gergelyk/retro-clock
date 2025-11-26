use std::sync::mpsc;

pub use mpsc::Receiver;

pub struct SyncSenderBlocking<T> {
    name: String,
    tx: mpsc::SyncSender<T>,
}

impl<T> SyncSenderBlocking<T> {
    pub fn new(name: &str, tx: mpsc::SyncSender<T>) -> Self {
        Self {
            name: name.to_string(),
            tx,
        }
    }

    pub fn send(&self, obj: T) -> anyhow::Result<()> {
        #[cfg(not(feature = "panic_on_tx_overflow"))]
        self.send_or_log_and_block(obj)?;
        #[cfg(feature = "panic_on_tx_overflow")]
        self.send_or_panic(obj)?;
        Ok(())
    }

    fn send_or_log_and_block(&self, obj: T) -> anyhow::Result<()> {
        match self.tx.try_send(obj) {
            Ok(()) => Ok(()),
            Err(mpsc::TrySendError::Full(obj)) => {
                let msg = format!("Channel reached its capacity: {}", self.name);
                log::error!("{}", msg);
                match self.tx.send(obj) {
                    Ok(()) => Ok(()),
                    Err(e) => Err(anyhow::anyhow!(
                        "Failed to send message to channel {}: {}",
                        self.name,
                        e
                    )),
                }
            }
            Err(mpsc::TrySendError::Disconnected(_)) => Err(anyhow::anyhow!(
                "Message sent to disconnected channel {}",
                self.name
            )),
        }
    }

    #[allow(dead_code)]
    fn send_or_panic(&self, obj: T) -> anyhow::Result<()> {
        match self.tx.try_send(obj) {
            Ok(()) => Ok(()),
            Err(mpsc::TrySendError::Full(_obj)) => {
                panic!("Channel reached its capacity: {}", self.name)
            }
            Err(mpsc::TrySendError::Disconnected(_)) => Err(anyhow::anyhow!(
                "Message sent to disconnected channel {}",
                self.name
            )),
        }
    }
}

impl<T> Clone for SyncSenderBlocking<T> {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            tx: self.tx.clone(),
        }
    }
}

pub fn channel_blocking<T>(name: &str, bound: usize) -> (SyncSenderBlocking<T>, mpsc::Receiver<T>) {
    let (tx, rx) = mpsc::sync_channel::<T>(bound);
    (SyncSenderBlocking::new(name, tx), rx)
}

pub struct SyncSenderDropping<T> {
    name: String,
    tx: mpsc::SyncSender<T>,
}

impl<T> SyncSenderDropping<T> {
    pub fn new(name: &str, tx: mpsc::SyncSender<T>) -> Self {
        Self {
            name: name.to_string(),
            tx,
        }
    }

    pub fn send(&self, obj: T) -> anyhow::Result<()> {
        self.send_or_drop(obj)
    }

    fn send_or_drop(&self, obj: T) -> anyhow::Result<()> {
        match self.tx.try_send(obj) {
            Ok(()) => Ok(()),
            Err(mpsc::TrySendError::Full(_obj)) => {
                Ok(()) // drop the message
            }
            Err(mpsc::TrySendError::Disconnected(_)) => Err(anyhow::anyhow!(
                "Message sent to disconnected channel {}",
                self.name
            )),
        }
    }
}

impl<T> Clone for SyncSenderDropping<T> {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            tx: self.tx.clone(),
        }
    }
}

pub fn channel_dropping<T>(name: &str, bound: usize) -> (SyncSenderDropping<T>, mpsc::Receiver<T>) {
    let (tx, rx) = mpsc::sync_channel::<T>(bound);
    (SyncSenderDropping::new(name, tx), rx)
}

pub fn flush_channel<T>(user_command_rx: &mpsc::Receiver<T>) -> anyhow::Result<()> {
    loop {
        match user_command_rx.try_recv() {
            Ok(_) => {}
            Err(mpsc::TryRecvError::Empty) => break,
            Err(mpsc::TryRecvError::Disconnected) => {
                anyhow::bail!("Failed to flush, channel disconnected");
            }
        }
    }
    Ok(())
}
