mod default_receiver;
pub mod person;
pub mod village;

use std::{collections::HashMap, future::Future, thread, time::Duration};

use mongodb::{options::ClientOptions, Client};
use rand::prelude::*;
use tokio::sync::mpsc::{channel, Receiver, Sender};

use self::village::{
    periods::{Period, RawPeriod},
    Village,
};
use crate::prelude::{SimplifiedVillage, VillageOutlet};

#[derive(Debug)]
pub enum WorldInlet {
    VillageDisposed(String),
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
            Some(default_receiver::received_from_village),
            period_maker,
        )
    }

    pub async fn idle(&mut self) -> () {
        loop {
            let received = self.receiver.recv().await;
            match received {
                Some(received) => match received {
                    WorldInlet::VillageDisposed(village_id) => {
                        self.villages.remove(&village_id);
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
