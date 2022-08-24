mod defaults;
pub mod person;
pub mod village;
pub mod world_inlet;

use std::{collections::HashMap, thread, time::Duration};

use mongodb::{options::ClientOptions, Client};
use rand::prelude::*;
use tokio::sync::mpsc::{channel, Receiver, Sender};

use crate::mongo_fns::world::person::count_village_persons;

use self::{
    village::{
        periods::{Period, RawPeriod},
        simplified_village::SimplifiedVillage,
        Village,
    },
    world_inlet::{AddPersonResult, FromHeaven, FromVillage, WorldInlet},
};

pub struct World {
    client: Client,
    villages: HashMap<String, SimplifiedVillage>,
    receiver: Receiver<WorldInlet>,
    pub(crate) to_world_sender: Sender<WorldInlet>,
    sample_village_names: [&'static str; 10],
}

impl World {
    pub async fn new() -> Self {
        // Parse a connection string into an options struct.
        let mut client_options = ClientOptions::parse("mongodb://localhost:27017")
            .await
            .unwrap();
        // Manually set an option.
        client_options.app_name = Some("My App".to_string());
        // Get a handle to the deployment.
        let client = Client::with_options(client_options).unwrap();

        let (tx, rx) = channel::<WorldInlet>(1024);

        World {
            client,
            villages: HashMap::new(),
            receiver: rx,
            to_world_sender: tx,
            sample_village_names: [
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
            ],
        }
    }

    pub fn create_village(
        &mut self,
        village_name: Option<&str>,
        period_maker: fn(&RawPeriod) -> Period,
    ) -> &SimplifiedVillage {
        let mut rng = thread_rng();
        let sv = Village::new(
            self.client.clone(),
            match village_name {
                Some(name) => name,
                None => self.sample_village_names.choose(&mut rng).unwrap(),
            },
            period_maker,
            self.to_world_sender.clone(),
        )
        .simplify();
        let village_id = sv.get_village_id().to_string();

        self.villages.insert(village_id.clone(), sv);
        self.villages.get(&village_id).unwrap()
    }

    pub fn create_village_default_receiver(
        &mut self,
        village_name: Option<&str>,
        period_maker: fn(&RawPeriod) -> Period,
    ) -> &SimplifiedVillage {
        self.create_village(village_name, period_maker)
    }

    fn kill_village(&mut self, village_id: &str) {
        self.villages.remove(village_id);
    }

    fn get_village(&self, village_id: &str) -> &SimplifiedVillage {
        self.villages.get(village_id).unwrap()
    }

    fn get_mut_village(&mut self, village_id: &str) -> &mut SimplifiedVillage {
        self.villages.get_mut(village_id).unwrap()
    }

    async fn handle_from_heaven(&mut self, from_heaven: FromHeaven) {
        use FromHeaven::*;

        match from_heaven {
            RawString(s) => println!("[🦀 World]: {}", s),
            RequestPerson {
                village_id,
                person_name,
            } => {
                let village = self.villages.get(&village_id).unwrap();
                match village.village.get_current_period() {
                    Period::Populating {
                        min_persons: _,
                        max_persons: _,
                        max_dur: _,
                    } => {
                        village.add_player(&person_name).await.unwrap();
                        println!(
                            "[🍨 World] Sent player request to {}, with name {}.",
                            village.get_village_name(),
                            person_name
                        );
                        village
                            .extend_population_dur(Duration::from_secs(10))
                            .await
                            .unwrap();
                    }
                    _ => println!(
                        "[🍨 World] Village {}, not populating, no person requested",
                        village.get_village_name(),
                    ),
                }
            }
            FillPersons { village_id, count } => {
                let village = self.villages.get(&village_id).unwrap();
                for i in 0..count {
                    match village.village.get_current_period() {
                        Period::Populating {
                            min_persons: _,
                            max_persons: _,
                            max_dur: _,
                        } => {
                            village.add_player(&format!("Player {}", i)).await.unwrap();
                            // tokio::time::sleep(Duration::from_secs(1)).await;
                        }
                        _ => break,
                    }
                }
            }
            KillVillage { village_id } => {
                let village = self.villages.get(&village_id).unwrap();
                village.die().await.unwrap();
                println!(
                    "[🍨 World] Sent die order to {}.",
                    village.get_village_name()
                );
            }
            ListVillages => {
                println!("[🍨 World] Listing villages:");
                for village in self.villages.values() {
                    println!(
                        "- 🌴 {} ( {} )",
                        village.get_village_name(),
                        village.get_village_id()
                    );
                }
            }
            NewVillage => {
                self.create_village_default_receiver(None, defaults::default_period_maker);
            }
            Nothing => (),
        }
    }

    async fn handle_from_village(&mut self, village_id: String, from_village: FromVillage) {
        use FromVillage::*;

        match from_village {
            RawString(text) => {
                println!("[🌴 {}]: {}", village_id, text);
            }
            VillageDisposed => {
                self.kill_village(&village_id);
                println!("[🍨 World] Village {} disposed", village_id);
            }
            PeriodReady(period) => {
                let village = self.get_village(&village_id);
                println!(
                    "[🌴 {}]: Village is ready to merge to the next period: {:?}",
                    village.get_village_name(),
                    period
                );
                match period {
                    RawPeriod::Assignments => {
                        let joined_persons = count_village_persons(&self.client, &village_id).await;
                        println!(
                            "[🌴 {}]: Village populated with {} persons",
                            village.get_village_name(),
                            joined_persons
                        );
                    }
                    _ => (),
                }
            }
            NewPeriod(period) => {
                let village = self.get_mut_village(&village_id);
                village.village.set_current_period(period);
                println!(
                    "[🍨 World] Village {}, entered new period: {:?}",
                    village.get_village_name(),
                    period
                );
            }
            PopulatingTimedOut => {
                self.kill_village(&village_id);
                println!("[🍨 World] Village {} disposed", village_id);
            }
            DaytimeCycled(daytime, dur) => {
                let village = self.get_village(&village_id);
                println!(
                    "[🌴 {}]: A new daytime {} ( {:#?} ),",
                    village.get_village_name(),
                    daytime,
                    dur
                );
            }
            AddPerson(result) => {
                let village = self.get_village(&village_id);
                match result {
                    AddPersonResult::Added {
                        person_id,
                        current_count,
                    } => println!(
                        "[🌴 {}]: Created person with id {} ({} persons in village).",
                        village.get_village_name(),
                        person_id,
                        current_count
                    ),
                    AddPersonResult::Failed(err) => println!(
                        "[🌴 {}]: Failed creating person: {}.",
                        village.get_village_name(),
                        err
                    ),
                }
            }
            WolvesTurn => todo!(),
            DoctorTurn => todo!(),
            SeerTurn => todo!(),
        }
    }

    pub async fn idle(&mut self, mut rx: Receiver<WorldInlet>) -> () {
        loop {
            let received = tokio::select! {
                v = self.receiver.recv() => v,
                v = rx.recv() => v,
            };

            match received {
                Some(received) => match received {
                    WorldInlet::FromHeaven(data) => self.handle_from_heaven(data).await,
                    WorldInlet::FromVillage { village_id, data } => {
                        self.handle_from_village(village_id, data).await
                    }
                },
                None => (),
            }
        }
    }
}

pub fn idle_for(dur: Duration) {
    thread::sleep(dur)
}
