pub mod inlet_data;
pub mod outlet_data;
pub mod periods;
pub mod simplified_village;
pub mod transports;
mod village_main;

use std::{future::Future, time::Duration};

use mongodb::Client;
use tokio::sync::mpsc::{channel, error::SendError, Receiver, Sender};

use self::{
    inlet_data::VillageInlet,
    outlet_data::VillageOutlet,
    periods::{Period, RawPeriod},
    simplified_village::SimplifiedVillage,
    transports::Transporter,
    village_main::village_main,
};

use super::WorldInlet;

pub(super) struct VillageInfo {
    pub(super) client: Client,
    village_id: String,
    village_name: String,
    receiver: Receiver<VillageInlet>,
    sender: Sender<VillageOutlet>,
    period_maker: fn(&RawPeriod) -> Period,
}

#[derive(Debug)]
pub(super) enum VillageInternal {
    PersonsFilled,
    ExtendPopulationTime(Duration),
}

#[derive(Clone)]
pub struct Village {
    village_id: String,
    village_name: String,
    pub(crate) sender: Sender<VillageInlet>,
    pub(crate) to_world_sender: Sender<WorldInlet>,
    pub(crate) client: Client,
}

impl Village {
    pub fn new<G, Gut>(
        client: Client,
        village_name: &str,
        callback: Option<G>,
        period_maker: fn(&RawPeriod) -> Period,
        to_world_sender: Sender<WorldInlet>,
    ) -> Self
    where
        G: Fn(VillageOutlet, SimplifiedVillage) -> Gut + Sync + Send + Copy + 'static,
        Gut: Future<Output = ()> + Sync + Send + 'static,
    {
        let village_id: String = uuid::Uuid::new_v4().to_string();
        let (inlet_tx, inlet_rx) = channel::<VillageInlet>(1024);
        let (outlet_tx, outlet_rx) = channel::<VillageOutlet>(1024);

        let village = Village {
            village_id: village_id.clone(),
            village_name: village_name.to_string(),
            sender: inlet_tx,
            to_world_sender,
            client: client.clone(),
        };

        let _ = match callback {
            Some(callback) => Some(Transporter::spawn(
                "Village to World".to_string(),
                outlet_rx,
                callback,
                village.clone().simplify(),
            )),
            None => None,
        };

        let info = VillageInfo {
            client,
            village_id,
            village_name: village_name.to_string(),
            receiver: inlet_rx,
            sender: outlet_tx,
            period_maker,
        };
        tokio::spawn(async move {
            village_main(info).await;
        });

        village
    }

    pub async fn transmit(&self, inlet_data: VillageInlet) -> Result<(), SendError<VillageInlet>> {
        self.sender.send(inlet_data).await
    }

    pub fn simplify(self) -> SimplifiedVillage {
        SimplifiedVillage::new(self)
    }
}
