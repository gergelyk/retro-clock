use crate::channel_utils::{channel_blocking, Receiver, SyncSenderBlocking};

pub enum BusyIndicatorCommand {
    Nop,
    Set,
    Unset,
}

pub struct BusyIndicator {
    cmd_tx: SyncSenderBlocking<BusyIndicatorCommand>,
    cmd_rx: Receiver<BusyIndicatorCommand>,
    count: u32,
}

#[allow(clippy::new_without_default)]
impl BusyIndicator {
    pub fn new() -> Self {
        let (cmd_tx, cmd_rx) = channel_blocking("BusyIndicator", 10);
        Self {
            cmd_tx,
            cmd_rx,
            count: 0,
        }
    }

    pub fn get_handle(&self) -> BusyIndicatorHandle {
        BusyIndicatorHandle::new(self.cmd_tx.clone())
    }

    pub fn wait_for_change(&mut self) -> anyhow::Result<bool> {
        let cmd = self.cmd_rx.recv()?;
        match cmd {
            BusyIndicatorCommand::Nop => {}
            BusyIndicatorCommand::Set => {
                self.count += 1;
            }
            BusyIndicatorCommand::Unset => {
                if self.count > 0 {
                    self.count -= 1;
                }
            }
        }
        Ok(self.count > 0)
    }
}

#[derive(Clone)]
pub struct BusyIndicatorHandle {
    cmd_tx: SyncSenderBlocking<BusyIndicatorCommand>,
}

impl BusyIndicatorHandle {
    fn new(cmd_tx: SyncSenderBlocking<BusyIndicatorCommand>) -> Self {
        Self { cmd_tx }
    }

    pub fn set(&self) -> anyhow::Result<BusyIndicatorGuard> {
        self.cmd_tx.send(BusyIndicatorCommand::Set)?;
        Ok(BusyIndicatorGuard::new(self.cmd_tx.clone()))
    }

    pub fn nop(&self) -> anyhow::Result<()> {
        self.cmd_tx.send(BusyIndicatorCommand::Nop)?;
        Ok(())
    }
}

pub struct BusyIndicatorGuard {
    cmd_tx: SyncSenderBlocking<BusyIndicatorCommand>,
}

impl BusyIndicatorGuard {
    pub fn new(cmd_tx: SyncSenderBlocking<BusyIndicatorCommand>) -> Self {
        Self { cmd_tx }
    }
}
impl Drop for BusyIndicatorGuard {
    fn drop(&mut self) {
        self.cmd_tx
            .send(BusyIndicatorCommand::Unset)
            .unwrap_or_else(|e| log::error!("Failed to unset busy indicator: {}", e));
    }
}
