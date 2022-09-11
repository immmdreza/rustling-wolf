use std::time::Duration;

use tokio::time::{timeout, Instant};

use crate::world::village::handle_from_world::SafeVillageInternal;

use super::VillageMain;

#[derive(Debug, Clone, Copy)]
pub(super) enum ExitFlag {
    NotExited,
    TimedOut,
    VillageDead,
}

pub(super) struct InternalStreamer<'r> {
    village_main: &'r mut VillageMain,
    timeout: Duration,
    elapsed: Duration,
    exit_err: ExitFlag,
}

impl<'r> InternalStreamer<'r> {
    pub(super) fn new(village_main: &'r mut VillageMain, timeout: Duration) -> Self {
        InternalStreamer {
            village_main,
            timeout,
            elapsed: Duration::ZERO,
            exit_err: ExitFlag::NotExited,
        }
    }

    pub(super) async fn next(&mut self) -> Result<SafeVillageInternal, ExitFlag> {
        self.timeout -= self.elapsed;
        self.elapsed = Duration::ZERO;

        let start = Instant::now();
        match timeout(self.timeout, self.village_main.listen_to_safe_internals()).await {
            Ok(data) => {
                self.elapsed = start.elapsed();
                match data {
                    Some(data) => Ok(data),
                    None => {
                        self.exit_err = ExitFlag::VillageDead;
                        Err(ExitFlag::VillageDead)
                    }
                }
            }
            Err(_) => {
                self.exit_err = ExitFlag::TimedOut;
                Err(ExitFlag::TimedOut)
            }
        }
    }

    pub(super) async fn timeout_or_die(&mut self) -> bool {
        while (self.next().await).is_ok() {
            continue;
        }

        matches!(self.exit_err, ExitFlag::VillageDead)
    }

    pub(super) async fn wait_for_wolves_choice(&mut self) -> Result<String, ExitFlag> {
        while let Ok(thing) = self.next().await {
            match thing {
                SafeVillageInternal::WolvesVictimSelected(target) => return Ok(target),
                _ => continue,
            }
        }

        Err(self.exit_err)
    }

    pub(super) async fn wait_for_doctor_choice(&mut self) -> Result<String, ExitFlag> {
        while let Ok(thing) = self.next().await {
            match thing {
                SafeVillageInternal::DoctorTargetSelected(target) => return Ok(target),
                _ => continue,
            }
        }

        Err(self.exit_err)
    }

    pub(super) async fn wait_for_seer_choice(&mut self) -> Result<String, ExitFlag> {
        while let Ok(thing) = self.next().await {
            match thing {
                SafeVillageInternal::SeerTargetSelected(target) => return Ok(target),
                _ => continue,
            }
        }

        Err(self.exit_err)
    }

    pub(super) fn increase_timeout(&mut self, dur: Duration) {
        self.timeout += dur;
    }

    pub(super) fn vg(&mut self) -> &mut VillageMain {
        self.village_main
    }

    pub(super) fn get_exit_err(&self) -> &ExitFlag {
        &self.exit_err
    }

    pub(super) fn village_dead(&self) -> bool {
        matches!(self.exit_err, ExitFlag::VillageDead)
    }

    pub(super) fn reset(&mut self, timeout: Duration) {
        self.elapsed = Duration::ZERO;
        self.timeout = timeout;
        self.exit_err = ExitFlag::NotExited;
    }
}
