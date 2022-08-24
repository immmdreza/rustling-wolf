mod internal_streamer;

use std::time::Duration;

use mongodb::Client;
use tokio::{
    sync::mpsc::{self, Receiver},
    task::JoinHandle,
};

use crate::{
    mongo_fns::world::{
        person::{assign_roles, cleanup_persons, count_village_persons},
        village::{cleanup_village_period, set_or_update_village_period},
    },
    world::{
        village::{
            handle_from_world::received_from_world,
            periods::{Daytime, Period, RawPeriod},
            village_main::internal_streamer::ExitFlag,
        },
        world_inlet::FromVillage,
        WorldInlet,
    },
};

use self::internal_streamer::InternalStreamer;

use super::{
    handle_from_world::{SafeVillageInternal, VillageInternal},
    inlet_data::VillageInlet,
    transports::Transporter,
    village_info::VillageInfo,
};

pub(super) struct VillageMain {
    info: VillageInfo,
    internal_rx: Receiver<VillageInternal>,
    transporter_handle: JoinHandle<()>,
    current_period_raw: RawPeriod,
}

impl VillageMain {
    pub(super) fn new(info: VillageInfo, receiver: Receiver<VillageInlet>) -> Self {
        let (w_tx, internal_rx) = mpsc::channel::<VillageInternal>(1024);
        let max_persons = match info.resolve_period(&RawPeriod::Populating) {
            Period::Populating {
                min_persons: _,
                max_persons,
                max_dur: _,
            } => max_persons,
            _ => 0,
        };
        let transporter_handle = Transporter::spawn(
            "Village/World to InnerVillage!".to_string(),
            receiver,
            received_from_world,
            (info.clone(), w_tx, max_persons),
        );
        println!("Transporter spawned!");

        let current_period_raw = RawPeriod::None;

        VillageMain {
            info,
            internal_rx,
            transporter_handle,
            current_period_raw,
        }
    }

    fn get_current_period(&self) -> Period {
        self.info.resolve_period(&self.current_period_raw)
    }

    fn get_client(&self) -> &Client {
        &self.info.client
    }

    fn get_village_id(&self) -> &str {
        &self.info.village_id
    }

    fn get_village_name(&self) -> &str {
        &self.info.village_name
    }

    fn get_out_sender(&self) -> &mpsc::Sender<WorldInlet> {
        &self.info.sender
    }

    /// Tries to cross the period to the next for this [`VillageMain`].
    ///
    /// ## Errors
    ///
    /// This function will return an error if the village is in last period already.
    async fn cross_period(&mut self) -> Result<(), ()> {
        match self.current_period_raw.cross() {
            Ok(new) => {
                self.current_period_raw = new;
                set_or_update_village_period(
                    self.get_client(),
                    self.get_village_id(),
                    &self.current_period_raw,
                )
                .await;
                Ok(())
            }
            Err(_) => Err(()),
        }
    }

    async fn notify(&self, outlet: FromVillage) -> Result<(), mpsc::error::SendError<WorldInlet>> {
        self.get_out_sender()
            .send(WorldInlet::from_village(self.get_village_id(), outlet))
            .await
    }

    #[allow(dead_code)]
    async fn safe_notify(&self, outlet: FromVillage) {
        self.notify(outlet).await.unwrap()
    }

    async fn notify_period_cross(&self) -> Result<(), mpsc::error::SendError<WorldInlet>> {
        self.notify(FromVillage::NewPeriod(self.get_current_period()))
            .await
    }

    async fn cleanup_steps(&self) {
        let cli = self.get_client();
        let vid = self.get_village_id();
        cleanup_persons(cli, vid).await.unwrap();
        cleanup_village_period(cli, vid).await.unwrap();

        self.transporter_handle.abort();
    }

    async fn read_persons_count(&self) -> u64 {
        count_village_persons(self.get_client(), self.get_village_id()).await
    }

    /// If this returns None, the village_main should return.
    async fn listen_to_safe_internals(&mut self) -> Option<SafeVillageInternal> {
        match self.internal_rx.recv().await {
            Some(data) => match data {
                // Let finish it right now ...
                VillageInternal::Die => {
                    // ❌ No more sender? no more village ...
                    self.cleanup_steps().await;
                    None
                }
                _ => Some(data.into()),
            },
            None => {
                // ❌ No more sender? no more village ...
                self.cleanup_steps().await;
                None
            }
        }
    }

    async fn assign_roles(&self) -> Vec<crate::world::person::roles::Role> {
        assign_roles(&self.get_client(), &self.get_village_id())
            .await
            .unwrap()
    }

    fn get_streamer(&mut self, timeout: Duration) -> InternalStreamer {
        InternalStreamer::new(self, timeout)
    }

    async fn notify_wolves_and_get_choice(
        &mut self,
        timeout: Duration,
    ) -> Result<String, ExitFlag> {
        self.notify(FromVillage::WolvesTurn).await.unwrap();
        self.get_streamer(timeout).wait_for_wolves_choice().await
    }

    async fn notify_doctor_and_get_choice(
        &mut self,
        timeout: Duration,
    ) -> Result<String, ExitFlag> {
        self.notify(FromVillage::DoctorTurn).await.unwrap();
        self.get_streamer(timeout).wait_for_doctor_choice().await
    }

    async fn notify_seer_and_get_choice(&mut self, timeout: Duration) -> Result<String, ExitFlag> {
        self.notify(FromVillage::SeerTurn).await.unwrap();
        self.get_streamer(timeout).wait_for_seer_choice().await
    }

    pub(super) async fn run(&mut self) {
        use FromVillage::*;

        println!(
            "Village {} ( {} ) thread started.",
            self.get_village_name(),
            self.get_village_id()
        );

        loop {
            self.cross_period().await.unwrap();
            self.notify_period_cross().await.unwrap();

            let current_period = self.get_current_period();
            match current_period {
                Period::None => break,
                Period::Populating {
                    min_persons,
                    max_persons: _,
                    max_dur,
                } => {
                    // Callback should only call if players are filled!
                    let mut streamer = self.get_streamer(max_dur);

                    while let Ok(data) = streamer.next().await {
                        match data {
                            SafeVillageInternal::PersonsFilled => {
                                let vg = streamer.vg();
                                vg.notify(PeriodReady(vg.current_period_raw.cross().unwrap()))
                                    .await
                                    .unwrap();
                                break;
                            }
                            SafeVillageInternal::ExtendPopulationTime(time) => {
                                streamer.increase_timeout(time);
                                continue;
                            }
                            _ => continue,
                        };
                    }

                    match streamer.get_exit_err() {
                        ExitFlag::NotExited => (),
                        ExitFlag::TimedOut => {
                            let current_joined = self.read_persons_count().await;

                            if current_joined < min_persons.into() {
                                self.notify(PopulatingTimedOut).await.unwrap();

                                // ❌ Village may gone ...
                                self.cleanup_steps().await;
                                return;
                            } else {
                                self.notify(PeriodReady(self.current_period_raw.cross().unwrap()))
                                    .await
                                    .unwrap();
                                break;
                            }
                        }
                        ExitFlag::VillageDead => {
                            return;
                        }
                    };
                }
                Period::Assignments(_) => loop {
                    let roles = self.assign_roles().await;
                    self.notify(RawString(format!("Roles assigned: {:#?}.", roles)))
                        .await
                        .unwrap();
                    break;
                },
                Period::FirstNight(dur) => {
                    self.notify(RawString("Wolves may know each other now ...".to_string()))
                        .await
                        .unwrap();

                    if self.get_streamer(dur).timeout_or_die().await {
                        return;
                    }
                }
                Period::DaytimeCycle(get_len) => {
                    let mut current_daytime = Daytime::MidNight;
                    loop {
                        current_daytime = current_daytime.cross();
                        let timeout = get_len(current_daytime);
                        self.notify(DaytimeCycled(current_daytime, timeout))
                            .await
                            .unwrap();

                        match current_daytime {
                            Daytime::MidNight => {
                                // Ask for roles to execute night action ...
                                // 1. Wolves may decide to eat. 30s
                                if let Ok(wolves_target) =
                                    self.notify_wolves_and_get_choice(timeout).await
                                {
                                    // Do something with target id ...
                                }
                                // Village dead ☠️
                                else {
                                    return;
                                }
                                // 2. Doctor may save. 20s
                                if let Ok(doctor_target) =
                                    self.notify_doctor_and_get_choice(timeout).await
                                {
                                    // Do something with target id ...
                                }
                                // Village dead ☠️
                                else {
                                    return;
                                }

                                // 3. Detective or Seer may scan roles. 20s
                                if let Ok(seer_target) =
                                    self.notify_seer_and_get_choice(timeout).await
                                {
                                    // Do something with target id ...
                                }
                                // Village dead ☠️
                                else {
                                    return;
                                }
                            }
                            Daytime::SunRaise => {
                                // Check game status ...
                                if self.get_streamer(timeout).timeout_or_die().await {
                                    return;
                                }
                            }
                            Daytime::LynchTime => {
                                // Request alive players to vote ...
                            }
                        }
                    }
                }
                Period::Ending => todo!(),
            }
        }
    }
}
