use std::time::Duration;

use tokio::time::{timeout, Instant};

use crate::world::village::handle_from_world::SafeVillageInternal;

use super::VillageMain;

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
        self.timeout = self.timeout - self.elapsed;
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
        while let Ok(_) = self.next().await {
            continue;
        }

        match self.exit_err {
            ExitFlag::VillageDead => true,
            _ => false,
        }
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
}
