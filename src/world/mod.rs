mod defaults;
pub mod person;
pub mod village;

use std::{collections::HashMap, future::Future, thread, time::Duration};

use mongodb::{options::ClientOptions, Client};
use rand::prelude::*;
use tokio::sync::mpsc::{channel, Receiver, Sender};

use crate::console_answer;

use self::village::{
    outlet_data::VillageOutlet,
    periods::{Period, RawPeriod},
    simplified_village::SimplifiedVillage,
    Village,
};

#[derive(Debug)]
pub enum WorldInlet {
    None,
    VillageDisposed(String),
    NewPeriod {
        village_id: String,
        period: Period,
    },
    RequestPerson {
        village_id: String,
        person_name: String,
    },
    FillPersons {
        village_id: String,
        count: u8,
    },
    KillVillage {
        village_id: String,
    },
    ListVillages,
    NewVillage,
}

pub trait QuickResolver<T, F> {
    fn resolve_world_inlet(self, on_some: F, msg: &str) -> WorldInlet
    where
        F: FnOnce(T) -> WorldInlet;
}

impl<T, F> QuickResolver<T, F> for Option<T> {
    fn resolve_world_inlet(self, on_some: F, msg: &str) -> WorldInlet
    where
        F: FnOnce(T) -> WorldInlet,
    {
        match self {
            Some(target) => on_some(target),
            None => {
                console_answer!("{}", msg);
                WorldInlet::None
            }
        }
    }
}

impl<T, F, U> QuickResolver<T, F> for Result<T, U> {
    fn resolve_world_inlet(self, on_some: F, msg: &str) -> WorldInlet
    where
        F: FnOnce(T) -> WorldInlet,
    {
        match self {
            Ok(target) => on_some(target),
            Err(_) => {
                console_answer!("{}", msg);
                WorldInlet::None
            }
        }
    }
}

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

    pub fn create_village<G, Gut>(
        &mut self,
        village_name: Option<&str>,
        callback: Option<G>,
        period_maker: fn(&RawPeriod) -> Period,
    ) -> &SimplifiedVillage
    where
        G: Fn(VillageOutlet, SimplifiedVillage) -> Gut + Sync + Send + Copy + 'static,
        Gut: Future<Output = ()> + Sync + Send + 'static,
    {
        let mut rng = thread_rng();
        let sv = Village::new(
            self.client.clone(),
            match village_name {
                Some(name) => name,
                None => self.sample_village_names.choose(&mut rng).unwrap(),
            },
            callback,
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
        self.create_village(
            village_name,
            Some(defaults::received_from_village),
            period_maker,
        )
    }

    pub async fn idle(&mut self, mut rx: Receiver<WorldInlet>) -> () {
        loop {
            let received = tokio::select! {
                v = self.receiver.recv() => v,
                v = rx.recv() => v,
            };

            match received {
                Some(received) => match received {
                    WorldInlet::VillageDisposed(village_id) => {
                        self.villages.remove(&village_id);
                        println!("[🍨 World] Village {} disposed", village_id);
                    }
                    WorldInlet::NewPeriod { village_id, period } => {
                        let village = self.villages.get_mut(&village_id).unwrap();
                        village.village.set_current_period(period);
                        println!(
                            "[🍨 World] Village {}, entered new period: {:?}",
                            village.get_village_name(),
                            period
                        );
                    }
                    WorldInlet::RequestPerson {
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
                    WorldInlet::None => (),
                    WorldInlet::KillVillage { village_id } => {
                        let village = self.villages.get(&village_id).unwrap();
                        village.die().await.unwrap();
                        println!(
                            "[🍨 World] Sent die order to {}.",
                            village.get_village_name()
                        );
                    }
                    WorldInlet::ListVillages => {
                        println!("[🍨 World] Listing villages:");
                        for village in self.villages.values() {
                            println!(
                                "- 🌴 {} ( {} )",
                                village.get_village_name(),
                                village.get_village_id()
                            );
                        }
                    }
                    WorldInlet::NewVillage => {
                        self.create_village_default_receiver(None, defaults::default_period_maker);
                    }
                    WorldInlet::FillPersons { village_id, count } => {
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
                },
                None => (),
            }
        }
    }
}

pub fn idle_for(dur: Duration) {
    thread::sleep(dur)
}
