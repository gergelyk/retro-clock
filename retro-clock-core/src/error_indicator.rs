use crate::channel_utils::{channel_blocking, Receiver, SyncSenderBlocking};

#[derive(Copy, Clone, Debug)]
pub enum ErrorKey {
    DownloadForecast0,
    DownloadForecast1,
    DownloadWeather,
    NoWifiConnection,
}

impl ErrorKey {
    pub fn bit(self) -> u32 {
        1 << (self as u32)
    }
}

pub enum ErrorIndicatorCommand {
    Nop,
    Set(ErrorKey),
    Unset(ErrorKey),
}

pub struct ErrorIndicator {
    cmd_tx: SyncSenderBlocking<ErrorIndicatorCommand>,
    cmd_rx: Receiver<ErrorIndicatorCommand>,
    errors: u32,
}

#[allow(clippy::new_without_default)]
impl ErrorIndicator {
    pub fn new() -> Self {
        let (cmd_tx, cmd_rx) = channel_blocking("ErrorIndicator", 10);
        Self {
            cmd_tx,
            cmd_rx,
            errors: 0,
        }
    }

    pub fn get_handle(&self) -> ErrorIndicatorHandle {
        ErrorIndicatorHandle::new(self.cmd_tx.clone())
    }

    pub fn wait_for_change(&mut self) -> anyhow::Result<u32> {
        let cmd = self.cmd_rx.recv()?;
        match cmd {
            ErrorIndicatorCommand::Nop => {}
            ErrorIndicatorCommand::Set(key) => {
                self.errors |= key.bit();
            }
            ErrorIndicatorCommand::Unset(key) => {
                self.errors &= !key.bit();
            }
        }
        Ok(self.errors)
    }
}

#[derive(Clone)]
pub struct ErrorIndicatorHandle {
    cmd_tx: SyncSenderBlocking<ErrorIndicatorCommand>,
}

impl ErrorIndicatorHandle {
    fn new(cmd_tx: SyncSenderBlocking<ErrorIndicatorCommand>) -> Self {
        Self { cmd_tx }
    }

    pub fn arm(&self, key: ErrorKey) -> anyhow::Result<ErrorIndicatorGuard> {
        Ok(ErrorIndicatorGuard::new(self.cmd_tx.clone(), key))
    }

    pub fn nop(&self) -> anyhow::Result<()> {
        self.cmd_tx.send(ErrorIndicatorCommand::Nop)?;
        Ok(())
    }
}

pub struct ErrorIndicatorGuard {
    cmd_tx: SyncSenderBlocking<ErrorIndicatorCommand>,
    key: ErrorKey,
    armed: bool,
}

impl ErrorIndicatorGuard {
    pub fn new(cmd_tx: SyncSenderBlocking<ErrorIndicatorCommand>, key: ErrorKey) -> Self {
        Self {
            cmd_tx,
            key,
            armed: true,
        }
    }

    pub fn disarm(&mut self) {
        self.armed = false;
    }
}
impl Drop for ErrorIndicatorGuard {
    fn drop(&mut self) {
        if self.armed {
            self.cmd_tx
                .send(ErrorIndicatorCommand::Set(self.key))
                .unwrap_or_else(|e| {
                    log::error!("Failed to set error indicator for {:?}: {}", self.key, e)
                })
        } else {
            self.cmd_tx
                .send(ErrorIndicatorCommand::Unset(self.key))
                .unwrap_or_else(|e| {
                    log::error!("Failed to unset error indicator for {:?}: {}", self.key, e)
                })
        }
    }
}
