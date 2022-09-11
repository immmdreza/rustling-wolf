mod defaults;
pub mod person;
pub mod village;
pub mod world_antenna;
pub mod world_inlet;
pub mod world_outlet;

pub use world_antenna::{AskWorld, WorldAnswered, WorldAntenna};

use std::{collections::HashMap, error::Error, thread, time::Duration};

use mongodb::{options::ClientOptions, Client};
use rand::prelude::*;
use tokio::sync::{
    mpsc::{self, channel, Receiver, Sender},
    oneshot,
};

use crate::{
    mongo_fns::world::person::{
        count_village_persons, get_all_alive_persons, get_eatable_alive_persons,
    },
    tower::{Request, Tower},
    world::world_outlet::NightTurn,
};

use self::{
    village::{
        periods::{Period, RawPeriod},
        simplified_village::SimplifiedVillage,
        Village,
    },
    world_antenna::ToWorldAntenna,
    world_inlet::{AddPersonResult, FromHeaven, FromVillage, WorldInlet},
    world_outlet::{SendWorldOutletContext, WorldOutlet},
};

#[derive(Debug)]
enum ReceivedKind {
    WorldInlet(Option<WorldInlet>),
    AskedWorld(Option<Request<AskWorld, WorldAnswered>>),
}

pub struct World {
    client: Client,
    villages: HashMap<String, SimplifiedVillage>,
    receiver: Receiver<WorldInlet>,
    to_heaven_tx: Sender<WorldOutlet>,
    antenna: WorldAntenna,
    antenna_rx: Receiver<Request<AskWorld, WorldAnswered>>,
    pub(crate) to_world_sender: Sender<WorldInlet>,
}

impl World {
    const VILLAGE_NAME_SAMPLES: [&'static str; 10] = [
        "Piodao",
        "Civita di Bagnoregio",
        "Alberobello",
        "Songzanlin",
        "Trakai Island",
        "Bergamo",
        "Ronda",
        "Jodhpur",
        "Marburg",
        "Fenghuang",
    ];

    pub async fn new() -> (World, Receiver<WorldOutlet>) {
        // Parse a connection string into an options struct.
        let mut client_options = ClientOptions::parse("mongodb://localhost:27017")
            .await
            .unwrap();
        // Manually set an option.
        client_options.app_name = Some("My App".to_string());
        // Get a handle to the deployment.
        let client = Client::with_options(client_options).unwrap();

        let (to_world_sender, receiver) = channel::<WorldInlet>(1024);
        let (to_heaven_tx, to_heaven_rx) = channel(1024);

        let tower = Tower::<AskWorld, WorldAnswered>::template(1024);
        let (antenna, antenna_rx) = tower.create_antenna_manual();

        (
            World {
                client,
                villages: HashMap::new(),
                receiver,
                to_heaven_tx,
                antenna: antenna.to_world_antenna(),
                antenna_rx,
                to_world_sender,
            },
            to_heaven_rx,
        )
    }

    fn create_village(
        &mut self,
        village_name: Option<&str>,
        period_maker: fn(&RawPeriod) -> Period,
    ) -> &SimplifiedVillage {
        let mut rng = thread_rng();
        let sv = Village::new(
            self.client.clone(),
            match village_name {
                Some(name) => name,
                None => Self::VILLAGE_NAME_SAMPLES.choose(&mut rng).unwrap(),
            },
            period_maker,
            self.to_world_sender.clone(),
        )
        .simplify();
        let village_id = sv.get_village_id().to_string();

        self.villages.insert(village_id.clone(), sv);
        self.villages.get(&village_id).unwrap()
    }

    fn create_village_default_receiver(
        &mut self,
        village_name: Option<&str>,
        period_maker: fn(&RawPeriod) -> Period,
    ) -> &SimplifiedVillage {
        self.create_village(village_name, period_maker)
    }

    fn kill_village(&mut self, village_id: &str) {
        self.villages.remove(village_id);
    }

    fn get_mut_village(&mut self, village_id: &str) -> &mut SimplifiedVillage {
        self.villages.get_mut(village_id).unwrap()
    }

    async fn get_village_or_notify(&self, village_id: &str) -> Option<&SimplifiedVillage> {
        match self.villages.get(village_id) {
            Some(village) => Some(village),
            None => {
                self.send_raw_string(&format!("Village with id {village_id} not found!"))
                    .await
                    .unwrap_or_default();
                None
            }
        }
    }

    fn send_out(&self) -> SendWorldOutletContext {
        WorldOutlet::send_ctx(&self.to_heaven_tx)
    }

    async fn send_raw_string(&self, raw: &str) -> Result<(), mpsc::error::SendError<WorldOutlet>> {
        self.send_out()
            .send(WorldOutlet::RawStringResult(Err(raw.to_string())))
            .await
    }

    pub fn antenna(&self) -> &WorldAntenna {
        &self.antenna
    }

    pub fn sender(&self) -> &Sender<WorldInlet> {
        &self.to_world_sender
    }

    async fn handle_from_heaven(&mut self, from_heaven: FromHeaven) -> Result<(), Box<dyn Error>> {
        use FromHeaven::*;

        match from_heaven {
            RequestPerson {
                village_id,
                person_name,
            } => {
                if let Some(village) = self.get_village_or_notify(&village_id).await {
                    match village.village.get_current_period() {
                        Period::Populating {
                            min_persons: _,
                            max_persons: _,
                            max_dur: _,
                        } => {
                            village.add_player(&person_name).await?;
                            village
                                .extend_population_dur(Duration::from_secs(10))
                                .await?;
                            Ok(())
                        }
                        _ => Ok(()),
                    }
                } else {
                    Ok(())
                }
            }
            FillPersons { village_id, count } => {
                if let Some(village) = self.get_village_or_notify(&village_id).await {
                    for i in 0..count {
                        match village.village.get_current_period() {
                            Period::Populating {
                                min_persons: _,
                                max_persons: _,
                                max_dur: _,
                            } => {
                                village.add_player(&format!("Player {}", i)).await?;
                            }
                            _ => break,
                        }
                    }
                }

                Ok(())
            }
            KillVillage { village_id } => {
                if let Some(village) = self.get_village_or_notify(&village_id).await {
                    village.die().await?;
                }

                Ok(())
            }
            ListVillages => {
                let mut villages = vec![];
                for village in self.villages.values() {
                    villages.push(village.get_info())
                }

                self.send_out().village_list(villages).await?;

                Ok(())
            }
            NewVillage => {
                self.create_village_default_receiver(None, defaults::default_period_maker);
                Ok(())
            }
            Nothing => Ok(()),
        }
    }

    async fn handle_from_village(
        &mut self,
        village_id: String,
        from_village: FromVillage,
    ) -> Result<(), Box<dyn Error>> {
        use FromVillage::*;

        match from_village {
            RawString(text) => {
                self.send_out()
                    .with_village(&village_id)
                    .raw_string(text)
                    .await?;
                Ok(())
            }

            VillageDisposed => {
                self.kill_village(&village_id);

                self.send_out().with_village(&village_id).disposed().await?;
                Ok(())
            }
            PeriodReady(period) => {
                self.send_out()
                    .with_village(&village_id)
                    .period_ready(period)
                    .await?;

                match period {
                    RawPeriod::Assignments => {
                        let joined_persons = count_village_persons(&self.client, &village_id).await;
                        self.send_out()
                            .with_village(&village_id)
                            .populated(joined_persons)
                            .await?;
                    }
                    RawPeriod::None => todo!(),
                    RawPeriod::Populating => todo!(),
                    RawPeriod::FirstNight => todo!(),
                    RawPeriod::DaytimeCycle => todo!(),
                    RawPeriod::Ending => todo!(),
                };

                Ok(())
            }
            NewPeriod(period) => {
                let village = self.get_mut_village(&village_id);
                village.village.set_current_period(period);

                self.send_out()
                    .with_village(&village_id)
                    .new_period(period)
                    .await?;

                Ok(())
            }
            PopulatingTimedOut => {
                self.kill_village(&village_id);
                self.send_out()
                    .with_village(&village_id)
                    .population_timed_out()
                    .await?;

                Ok(())
            }
            DaytimeCycled(daytime, dur) => {
                self.send_out()
                    .with_village(&village_id)
                    .send(world_outlet::WithVillage::DaytimeCycled(daytime, dur))
                    .await?;

                Ok(())
            }
            AddPerson(result) => {
                self.send_out()
                    .with_village(&village_id)
                    .send(world_outlet::WithVillage::AddPersonResult(result))
                    .await?;

                Ok(())
            }
            WolvesTurn => {
                let eatable_persons = get_eatable_alive_persons(&self.client, &village_id).await;
                self.send_out()
                    .with_village(&village_id)
                    .send(world_outlet::WithVillage::NightTurn {
                        turn: NightTurn::Wolf,
                        available_persons: eatable_persons,
                    })
                    .await?;

                Ok(())
            }
            DoctorTurn => {
                let all_persons = get_all_alive_persons(&self.client, &village_id).await;
                self.send_out()
                    .with_village(&village_id)
                    .send(world_outlet::WithVillage::NightTurn {
                        turn: NightTurn::Doctor,
                        available_persons: all_persons,
                    })
                    .await?;

                Ok(())
            }
            SeerTurn => {
                let all_persons = get_all_alive_persons(&self.client, &village_id).await;
                self.send_out()
                    .with_village(&village_id)
                    .send(world_outlet::WithVillage::NightTurn {
                        turn: NightTurn::Seer,
                        available_persons: all_persons,
                    })
                    .await?;

                Ok(())
            }
            ReportNightActionResult(report) => {
                self.send_out()
                    .with_village(&village_id)
                    .send(world_outlet::WithVillage::NightActionResultReport(report))
                    .await?;

                Ok(())
            }
        }
    }

    async fn answer_whats_asked(
        &self,
        asked: AskWorld,
        sender: oneshot::Sender<WorldAnswered>,
    ) -> Result<(), Box<dyn Error>> {
        use WorldAnswered::*;
        let answer = |answer: WorldAnswered| sender.send(answer);

        match asked {
            AskWorld::RawString(text) => {
                answer(RawString(format!("Echoed {}", text))).unwrap_or_default();

                Ok(())
            }
            AskWorld::AskVillageName(village_id) => {
                answer(match self.villages.get(&village_id) {
                    Some(village) => VillageName(Some(village.get_village_name().to_string())),
                    None => VillageName(None),
                })
                .unwrap_or_default();

                Ok(())
            }
        }
    }

    pub fn live(mut self) {
        tokio::spawn(async move {
            loop {
                let received = tokio::select! {
                    v = self.receiver.recv() => ReceivedKind::WorldInlet(v),
                    v = self.antenna_rx.recv() => ReceivedKind::AskedWorld(v)
                };

                match received {
                    ReceivedKind::WorldInlet(inlet) => match inlet {
                        Some(inlet) => match inlet {
                            WorldInlet::FromHeaven(data) => {
                                if (self.handle_from_heaven(data).await).is_ok() {}
                            }
                            WorldInlet::FromVillage { village_id, data } => {
                                self.handle_from_village(village_id, data).await.unwrap()
                            }
                        },

                        None => (),
                    },
                    ReceivedKind::AskedWorld(request) => match request {
                        Some(request) => {
                            let (asked, sender) = request.extract();
                            self.answer_whats_asked(asked, sender).await.unwrap();
                        }
                        None => (),
                    },
                }
            }
        });
    }
}

pub fn idle_for(dur: Duration) {
    thread::sleep(dur)
}
