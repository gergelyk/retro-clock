use derive_new::new;
use std::iter::Take;
use std::slice::Iter;
use std::sync::{
    atomic::{self, AtomicBool},
    Arc,
};

use retro_clock_core::{
    arc_mtx::{ArcMtx, MutexExt},
    channel_utils::SyncSenderDropping,
};

use crate::{storage::EspConfigStorage, system_utils::restart_device, user_command::UserCommand};

#[derive(new)]
pub struct SerialMonitor {
    user_command_tx: SyncSenderDropping<UserCommand>,
    storage_arc: ArcMtx<EspConfigStorage>,
    simulate_on_battery: Arc<AtomicBool>,
}

impl SerialMonitor {
    pub fn handle_new_data(&mut self, characters: Take<Iter<'_, u8>>) -> anyhow::Result<()> {
        for ch in characters {
            match *ch as char {
                '?' => {
                    log::info!("n - next card");
                    log::info!("N - next page");
                    log::info!("h - home page");
                    log::info!("r - restart device");
                    log::info!("R - clear storage and restart device");
                }
                'n' => self.user_command_tx.send(UserCommand::GoNextCard)?,
                'N' => self.user_command_tx.send(UserCommand::GoNextPage)?,
                'h' => self.user_command_tx.send(UserCommand::GoHome)?,
                'b' => self
                    .simulate_on_battery
                    .store(true, atomic::Ordering::Relaxed),
                'B' => self
                    .simulate_on_battery
                    .store(false, atomic::Ordering::Relaxed),
                'r' => restart_device(),
                'R' => {
                    self.storage_arc.lock_anyhow()?.reset()?;
                    restart_device();
                }
                _ => {}
            }
        }
        Ok(())
    }
}
